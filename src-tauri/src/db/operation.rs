use crate::db::schema::NodeType;
use crate::db::schema::{Canvas, CanvasData, Chunk, Derives, Node, Sequences};
use crate::embedding::EmbeddingService; // Changed from EmbeddingServiceManager
use crate::llm::ContentPart;
use surrealdb::engine::local::Db;
use surrealdb::sql::Thing;
use surrealdb::Surreal;

pub async fn add_canvas(client: &Surreal<Db>, canvas: Canvas) -> anyhow::Result<Canvas> {
    log::debug!("add_canvas: canvas={:?}", canvas);

    let sql = r#"
        CREATE canvas CONTENT $canvas_data;
    "#;

    let mut response = client
        .query(sql)
        .bind(("canvas_data", canvas))
        .await
        .inspect_err(|e| log::error!("add_canvas: query failed: {}", e))?;

    let created: Option<Canvas> = response.take(0).inspect_err(|e| {
        log::error!(
            "add_canvas: failed to retrieve created canvas from response: {}",
            e
        )
    })?;
    let result = created.ok_or_else(|| {
        log::error!("add_canvas: query succeeded but returned no created canvas");
        anyhow::anyhow!("Failed to create canvas")
    })?;
    log::debug!("add_canvas: success, created={:?}", result);
    Ok(result)
}

pub async fn list_canvas(
    client: &Surreal<Db>,
    limit: usize,
    offset: usize,
) -> anyhow::Result<Vec<Canvas>> {
    log::debug!("list_canvas: limit={}, offset={}", limit, offset);

    let sql = r#"
        SELECT * FROM canvas ORDER BY created_at DESC LIMIT $limit START $offset;
    "#;

    let mut response = client
        .query(sql)
        .bind(("limit", limit))
        .bind(("offset", offset))
        .await
        .inspect_err(|e| log::error!("list_canvas: query failed: {}", e))?;

    let canvases: Vec<Canvas> = response.take(0).inspect_err(|e| {
        log::error!(
            "list_canvas: failed to retrieve canvases from response: {}",
            e
        )
    })?;

    log::debug!("list_canvas: success, count={}", canvases.len());
    Ok(canvases)
}

pub async fn is_canvas_empty(client: &Surreal<Db>) -> anyhow::Result<bool> {
    let sql = "SELECT count() FROM canvas GROUP ALL";
    let mut response = client.query(sql).await?;
    let result: Option<usize> = response.take("count")?;
    Ok(result.unwrap_or(0) == 0)
}

async fn fetch_canvas(client: &Surreal<Db>, canvas_id: &Thing) -> anyhow::Result<Canvas> {
    let sql = "SELECT * FROM $id";
    let mut response = client.query(sql).bind(("id", canvas_id.clone())).await?;
    let canvas: Option<Canvas> = response.take(0)?;
    canvas.ok_or_else(|| anyhow::anyhow!("Canvas not found"))
}

async fn generate_and_store_embeddings(
    client: &Surreal<Db>,
    node: &Node,
    node_id: &Thing,
    canvas_id: &Thing,
    embedding_service: Option<&dyn EmbeddingService>,
    embedding_id: Option<&str>,
) -> anyhow::Result<()> {
    let mut chunk_count = 0;
    if let NodeType::Chat { data } = &node.type_ {
        if embedding_id.is_some() {
            let service = match embedding_service {
                Some(s) => s,
                None => {
                    log::warn!("Embedding service not provided but canvas requires embedding. Skipping embedding generation.");
                    return Ok(());
                }
            };

            if let Some(output) = &data.output {
                let mut text = String::new();
                for part in &output.content {
                    if let ContentPart::Text(t) = part {
                        text.push_str(t);
                    }
                }

                if !text.is_empty() {
                    use crate::embedding::chunking::SemanticChunking;
                    let chunker = SemanticChunking::new("default".to_string(), 0.8);

                    match chunker.chunk(&text, service).await {
                        Ok(chunks) => {
                            chunk_count = chunks.len();
                            if !chunks.is_empty() {
                                match service.embed("default", chunks.clone()).await {
                                    Ok(embeddings) => {
                                        // Delete existing chunks for this node to avoid duplicates via relation
                                        let delete_sql = r#"
                                            DELETE chunk WHERE id IN (SELECT VALUE out FROM has_chunk WHERE in = $node_id);
                                            DELETE has_chunk WHERE in = $node_id;
                                        "#;
                                        client
                                            .query(delete_sql)
                                            .bind(("node_id", node_id.clone()))
                                            .await
                                            .inspect_err(|e| {
                                                log::error!(
                                                    "Failed to delete existing chunks and relations: {}",
                                                    e
                                                )
                                            })?;

                                        // Store new chunks and create relations
                                        let insert_sql = r#"
                                            let $chunk = (CREATE chunk CONTENT $chunk_data);
                                            let $cid = $chunk[0].id;
                                            RELATE $node_id -> has_chunk -> $cid;
                                        "#;
                                        for (i, content) in chunks.iter().enumerate() {
                                            if let Some(embedding) = embeddings.get(i) {
                                                let chunk = Chunk {
                                                    id: None,
                                                    content: content.clone(),
                                                    embedding: embedding.clone(),
                                                    node: node_id.clone(),
                                                    canvas: canvas_id.clone(),
                                                    created_at: chrono::Utc::now(),
                                                };
                                                client
                                                    .query(insert_sql)
                                                    .bind(("chunk_data", chunk))
                                                    .bind(("node_id", node_id.clone()))
                                                    .await
                                                    .inspect_err(|e| {
                                                        log::error!("Failed to save chunk: {}", e)
                                                    })?;
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("Failed to generate embeddings: {}", e)
                                    }
                                }
                            }
                        }
                        Err(e) => log::error!("Failed to chunk text: {}", e),
                    }
                }
            }
        }
    }
    log::debug!(
        "generate_and_store_embeddings: success, node_id={}, chunks={}, embedding_id={:?}",
        node_id,
        chunk_count,
        embedding_id
    );
    Ok(())
}

async fn store_node(
    client: &Surreal<Db>,
    canvas_id: Thing,
    data: Node,
    relation: Option<(Thing, &'static str)>,
    op_name: &str,
) -> anyhow::Result<Node> {
    let (extra_relate_sql, from_id) = match relation {
        Some((from, edge)) => (
            format!(
                "RELATE $from -> {} -> $node_id SET canvas = $canvas_id;",
                edge
            ),
            Some(from),
        ),
        None => (String::new(), None),
    };

    let sql = format!(
        r#"
        let $node = (CREATE node CONTENT $node_data);
        let $node_id = $node[0].id;
        RELATE $canvas_id -> holds -> $node_id;
        {}
        RETURN $node[0];
    "#,
        extra_relate_sql
    );

    let mut query = client
        .query(sql)
        .bind(("canvas_id", canvas_id))
        .bind(("node_data", data));

    if let Some(from) = from_id {
        query = query.bind(("from", from));
    }

    let mut response = query
        .await
        .inspect_err(|e| log::error!("{}: query failed: {}", op_name, e))?
        .check()
        .inspect_err(|e| log::error!("{}: check failed: {}", op_name, e))?;

    let return_idx = if !extra_relate_sql.is_empty() { 4 } else { 3 };

    let created: Option<Node> = response
        .take(return_idx)
        .inspect_err(|e| log::error!("{}: failed to retrieve created node: {}", op_name, e))?;

    let result = created.ok_or_else(|| {
        log::error!("{}: query succeeded but returned no created node", op_name);
        anyhow::anyhow!("Failed to create node")
    })?;
    Ok(result)
}

async fn create_node_common(
    client: &Surreal<Db>,
    embedding_service: Option<&dyn EmbeddingService>,
    canvas_id: Thing,
    node: Node,
    relation: Option<(Thing, &'static str)>,
    op_name: &str,
) -> anyhow::Result<Node> {
    log::debug!("{}: canvas_id={}, node={:?}", op_name, canvas_id, node);

    // 1. Fetch Canvas to check embedding_id
    let canvas = fetch_canvas(client, &canvas_id).await?;

    // 2. Store node in DB
    let result = store_node(client, canvas_id.clone(), node.clone(), relation, op_name).await?;

    // 3. Generate and store embeddings if applicable (now using the created node's ID)
    if let Some(node_id) = &result.id {
        generate_and_store_embeddings(
            client,
            &node, // Use original node logic or result? Result has ID, but type is same.
            node_id,
            &canvas_id,
            embedding_service,
            canvas.embedding_id.as_deref(),
        )
        .await?;
    }

    log::debug!("{}: success, created={:?}", op_name, result);
    Ok(result)
}

pub async fn add_node(
    client: &Surreal<Db>,
    embedding_service: Option<&dyn EmbeddingService>,
    canvas_id: Thing,
    node: Node,
) -> anyhow::Result<Node> {
    create_node_common(client, embedding_service, canvas_id, node, None, "add_node").await
}

pub async fn add_derived_node(
    client: &Surreal<Db>,
    embedding_service: Option<&dyn EmbeddingService>,
    canvas_id: Thing,
    from: Thing,
    to: Node,
) -> anyhow::Result<Node> {
    create_node_common(
        client,
        embedding_service,
        canvas_id,
        to,
        Some((from, "derives")),
        "add_derived_node",
    )
    .await
}

pub async fn add_sequenced_node(
    client: &Surreal<Db>,
    embedding_service: Option<&dyn EmbeddingService>,
    canvas_id: Thing,
    from: Thing,
    to: Node,
) -> anyhow::Result<Node> {
    create_node_common(
        client,
        embedding_service,
        canvas_id,
        to,
        Some((from, "sequences")),
        "add_sequenced_node",
    )
    .await
}

pub async fn move_node_position(
    client: &Surreal<Db>,
    node_id: Thing,
    x: f64,
    y: f64,
) -> anyhow::Result<Node> {
    log::debug!("move_node_position: node_id={}, x={}, y={}", node_id, x, y);

    let sql = r#"
        UPDATE $node_id
        MERGE {position: { x: $x, y: $y }}
        RETURN AFTER
    "#;

    let mut response = client
        .query(sql)
        .bind(("node_id", node_id))
        .bind(("x", x))
        .bind(("y", y))
        .await
        .inspect_err(|e| log::error!("move_node_position: query failed: {}", e))?;

    let updated: Option<Node> = response.take(0).inspect_err(|e| {
        log::error!("move_node_position: failed to retrieve updated node: {}", e)
    })?;
    let result = updated.ok_or_else(|| {
        log::error!("move_node_position: query succeeded but returned no updated node (maybe node_id not found?)");
        anyhow::anyhow!("Failed to move node")
    })?;
    log::debug!("move_node_position: success, updated={:?}", result);
    Ok(result)
}

pub async fn delete_node(client: &Surreal<Db>, node_id: Thing) -> anyhow::Result<()> {
    log::debug!("delete_node: node_id={}", node_id);

    // Check if the node is referenced by any 'derives' or 'sequences' edges as the source ('in')
    let sql = r#"
        let $derives_count = (SELECT count() FROM derives WHERE in = $node_id GROUP ALL);
        let $sequences_count = (SELECT count() FROM sequences WHERE in = $node_id GROUP ALL);
        
        IF $derives_count[0].count > 0 OR $sequences_count[0].count > 0 {
            THROW "Cannot delete node: it is referenced by other nodes.";
        };

        // Delete chunks associated via has_chunk
        DELETE chunk WHERE id IN (SELECT VALUE out FROM has_chunk WHERE in = $node_id);
        DELETE has_chunk WHERE in = $node_id;

        // Delete 'holds' relations associated with this node
        DELETE holds WHERE out = $node_id;

        // Delete the node itself
        DELETE $node_id;
    "#;

    client
        .query(sql)
        .bind(("node_id", node_id))
        .await
        .inspect_err(|e| {
            log::error!(
                "delete_node: query failed to execute deletion checks: {}",
                e
            )
        })?
        .check()
        .inspect_err(|e| log::error!("delete_node: sanity check failed: {}", e))?;

    log::debug!("delete_node: success");
    Ok(())
}

pub async fn load_canvas(
    client: &Surreal<Db>,
    canvas_id: Thing,
) -> anyhow::Result<Option<CanvasData>> {
    log::debug!("load_canvas: canvas_id={}", canvas_id);

    // 1. Check if canvas exists
    let sql = "SELECT * FROM $id";
    let mut response = client.query(sql).bind(("id", canvas_id.clone())).await?;
    let canvas_result: Option<Canvas> = response.take(0)?;
    let canvas = match canvas_result {
        Some(c) => c,
        None => {
            log::debug!("load_canvas: canvas not found");
            return Ok(None);
        }
    };

    let sql = r#"
        SELECT * FROM node WHERE id IN (SELECT VALUE out FROM holds WHERE in = $canvas_id);
        SELECT * FROM derives WHERE canvas = $canvas_id;
        SELECT * FROM sequences WHERE canvas = $canvas_id;
    "#;
    let mut responses = client
        .query(sql)
        .bind(("canvas_id", canvas_id))
        .await
        .inspect_err(|e| log::error!("load_canvas: query failed: {}", e))?;

    let nodes: Vec<Node> = responses
        .take(0)
        .inspect_err(|e| log::error!("load_canvas: failed to take nodes: {}", e))?;
    let derives: Vec<Derives> = responses
        .take(1)
        .inspect_err(|e| log::error!("load_canvas: failed to take derives: {}", e))?;
    let sequences: Vec<Sequences> = responses
        .take(2)
        .inspect_err(|e| log::error!("load_canvas: failed to take sequences: {}", e))?;

    log::debug!(
        "load_canvas: success, nodes={}, derives={}, sequences={}",
        nodes.len(),
        derives.len(),
        sequences.len()
    );

    Ok(Some(CanvasData {
        canvas,
        nodes,
        derives,
        sequences,
    }))
}

pub async fn update_canvas_chat_model_id(
    client: &Surreal<Db>,
    canvas_id: Thing,
    model_id: Option<String>,
) -> anyhow::Result<Canvas> {
    log::debug!(
        "update_canvas_chat_model_id: canvas_id={}, model_id={:?}",
        canvas_id,
        model_id
    );

    let sql = r#"
        UPDATE $canvas_id
        SET chat_model_id = $model_id
        RETURN AFTER
    "#;

    let mut response = client
        .query(sql)
        .bind(("canvas_id", canvas_id))
        .bind(("model_id", model_id))
        .await
        .inspect_err(|e| log::error!("update_canvas_chat_model_id: query failed: {}", e))?;

    let updated: Option<Canvas> = response.take(0).inspect_err(|e| {
        log::error!(
            "update_canvas_chat_model_id: failed to retrieve updated canvas: {}",
            e
        )
    })?;

    let result = updated.ok_or_else(|| {
        log::error!("update_canvas_chat_model_id: query succeeded but returned no updated canvas");
        anyhow::anyhow!("Failed to update canvas chat model id")
    })?;

    log::debug!("update_canvas_chat_model_id: success, updated={:?}", result);
    Ok(result)
}

pub async fn update_chat_node_output(
    client: &Surreal<Db>,
    node_id: Thing,
    output: crate::llm::LlmOutput,
) -> anyhow::Result<Node> {
    log::debug!("update_chat_node_output: node_id={}", node_id);

    let sql = r#"
        UPDATE $node_id
        SET type.data.data.output = $output
        RETURN AFTER
    "#;

    let mut response = client
        .query(sql)
        .bind(("node_id", node_id))
        .bind(("output", output))
        .await
        .inspect_err(|e| log::error!("update_chat_node_output: query failed: {}", e))?;

    let updated: Option<Node> = response.take(0).inspect_err(|e| {
        log::error!(
            "update_chat_node_output: failed to retrieve updated node: {}",
            e
        )
    })?;
    let result = updated.ok_or_else(|| {
        log::error!("update_chat_node_output: query succeeded but returned no updated node");
        anyhow::anyhow!("Failed to update chat node output")
    })?;
    log::debug!("update_chat_node_output: success, updated={:?}", result);
    Ok(result)
}

pub async fn get_node(client: &Surreal<Db>, node_id: Thing) -> anyhow::Result<Node> {
    log::debug!("get_node: node_id={}", node_id);

    let sql = r#"
        SELECT * FROM $node_id;
    "#;

    let mut response = client
        .query(sql)
        .bind(("node_id", node_id))
        .await
        .inspect_err(|e| log::error!("get_node: query failed: {}", e))?;

    let node: Option<Node> = response
        .take(0)
        .inspect_err(|e| log::error!("get_node: failed to retrieve node from response: {}", e))?;

    let result = node.ok_or_else(|| {
        log::error!("get_node: query succeeded but returned no node");
        anyhow::anyhow!("Node not found")
    })?;

    log::debug!("get_node: success, result={:?}", result);
    Ok(result)
}

pub async fn search_chunks(
    client: &Surreal<Db>,
    canvas_id: Thing,
    query_embedding: Vec<f32>,
    limit: usize,
    threshold: f32,
) -> anyhow::Result<Vec<Chunk>> {
    log::debug!(
        "search_chunks: canvas_id={}, limit={}, threshold={}",
        canvas_id,
        limit,
        threshold
    );

    let sql = r#"
        SELECT *, vector::similarity::cosine(embedding, $query_embedding) AS score
        FROM chunk
        WHERE canvas = $canvas_id
          AND vector::similarity::cosine(embedding, $query_embedding) > $threshold
        ORDER BY score DESC
        LIMIT $limit;
    "#;

    let mut response = client
        .query(sql)
        .bind(("canvas_id", canvas_id))
        .bind(("query_embedding", query_embedding))
        .bind(("threshold", threshold))
        .bind(("limit", limit))
        .await
        .inspect_err(|e| log::error!("search_chunks: query failed: {}", e))?;

    let chunks: Vec<Chunk> = response.take(0).inspect_err(|e| {
        log::error!(
            "search_chunks: failed to retrieve chunks from response: {}",
            e
        )
    })?;

    log::debug!("search_chunks: success, count={}", chunks.len());
    Ok(chunks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::{ChatNodeData, NodePosition, NodeType};
    use crate::embedding::EmbeddingServiceManager;
    use crate::llm::{ContentPart, LlmInput, LlmOutput, Message, Role};
    use std::sync::Arc;
    use surrealdb::engine::local::Mem;
    use tauri::Url;
    use tokio::sync::RwLock;

    async fn setup_db() -> Surreal<Db> {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();
        db
    }

    fn create_dummy_canvas() -> Canvas {
        Canvas {
            id: None,
            title: "Test Canvas".to_string(),
            created_at: chrono::Utc::now(),
            embedding_id: None,
            reranker_id: None,
            chat_model_id: None,
        }
    }

    fn create_dummy_embedding_canvas() -> Canvas {
        Canvas {
            id: None,
            title: "Embedding Canvas".to_string(),
            created_at: chrono::Utc::now(),
            embedding_id: Some("local".to_string()),
            reranker_id: None,
            chat_model_id: None,
        }
    }

    fn create_dummy_node() -> Node {
        Node {
            id: None,
            position: NodePosition { x: 100.0, y: 200.0 },
            type_: NodeType::Link {
                url: Url::parse("https://example.com").unwrap(),
            },
        }
    }

    fn create_dummy_chat_node() -> Node {
        Node {
            id: None,
            position: NodePosition { x: 100.0, y: 200.0 },
            type_: NodeType::Chat {
                data: ChatNodeData {
                    input: LlmInput {
                        system_prompt: None,
                        history: vec![],
                        user_input: Message::new_text(Role::User, "Hello world"),
                    },
                    output: None,
                    model_id: None,
                },
            },
        }
    }

    #[tokio::test]
    async fn test_canvas_lifecycle() {
        let db = setup_db().await;

        // Test add_canvas
        let canvas = create_dummy_canvas();
        let created_canvas = add_canvas(&db, canvas.clone()).await.unwrap();
        assert!(created_canvas.id.is_some());
        assert_eq!(created_canvas.title, canvas.title);

        // Test list_canvas
        let canvases = list_canvas(&db, 10, 0).await.unwrap();
        assert_eq!(canvases.len(), 1);
        assert_eq!(canvases[0].id, created_canvas.id);

        let canvas2 = create_dummy_canvas();
        add_canvas(&db, canvas2).await.unwrap();

        let canvases = list_canvas(&db, 10, 0).await.unwrap();
        assert_eq!(canvases.len(), 2);
    }

    #[tokio::test]
    async fn test_add_node() {
        let db = setup_db().await;
        let canvas = add_canvas(&db, create_dummy_canvas()).await.unwrap();
        let canvas_id = canvas.id.unwrap();

        let node = create_dummy_node();
        let created_node = add_node(&db, None, canvas_id.clone(), node.clone())
            .await
            .unwrap();

        assert!(created_node.id.is_some());
        assert_eq!(created_node.position.x, node.position.x);

        // Verify node is in canvas via load_canvas
        let canvas_data = load_canvas(&db, canvas_id).await.unwrap().unwrap();
        assert_eq!(canvas_data.nodes.len(), 1);
        assert_eq!(canvas_data.nodes[0].id, created_node.id);
    }

    #[tokio::test]
    async fn test_add_node_invalid_canvas() {
        let db = setup_db().await;
        let fake_canvas_id = Thing::from(("canvas", "nonexistent"));
        let node = create_dummy_node();

        let result = add_node(&db, None, fake_canvas_id, node).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Canvas not found"));
    }

    #[tokio::test]
    async fn test_load_canvas_none() {
        let db = setup_db().await;
        let fake_canvas_id = Thing::from(("canvas", "nonexistent"));

        let result = load_canvas(&db, fake_canvas_id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_node_relations() {
        let db = setup_db().await;
        let canvas = add_canvas(&db, create_dummy_canvas()).await.unwrap();
        let canvas_id = canvas.id.unwrap();

        let node1 = add_node(&db, None, canvas_id.clone(), create_dummy_node())
            .await
            .unwrap();
        let node1_id = node1.id.unwrap();

        // Test add_derived_node
        let node2_data = create_dummy_node();
        let node2 = add_derived_node(&db, None, canvas_id.clone(), node1_id.clone(), node2_data)
            .await
            .unwrap();
        let node2_id = node2.id.clone().unwrap();

        // Test add_sequenced_node
        let node3_data = create_dummy_node();
        let node3 = add_sequenced_node(&db, None, canvas_id.clone(), node1_id.clone(), node3_data)
            .await
            .unwrap();
        let node3_id = node3.id.unwrap();

        // Verify relations
        let canvas_data = load_canvas(&db, canvas_id).await.unwrap().unwrap();
        assert_eq!(canvas_data.nodes.len(), 3);
        assert_eq!(canvas_data.derives.len(), 1);
        assert_eq!(canvas_data.sequences.len(), 1);

        assert_eq!(canvas_data.derives[0].from, node1_id);
        assert_eq!(canvas_data.derives[0].to, node2_id);

        assert_eq!(canvas_data.sequences[0].from, node1_id);
        assert_eq!(canvas_data.sequences[0].to, node3_id);
    }

    #[tokio::test]
    async fn test_move_node() {
        let db = setup_db().await;
        let canvas = add_canvas(&db, create_dummy_canvas()).await.unwrap();
        let node = add_node(&db, None, canvas.id.unwrap(), create_dummy_node())
            .await
            .unwrap();
        let node_id = node.id.unwrap();

        let updated_node = move_node_position(&db, node_id.clone(), 500.0, 600.0)
            .await
            .unwrap();
        assert_eq!(updated_node.position.x, 500.0);
        assert_eq!(updated_node.position.y, 600.0);

        let fetched_node = get_node(&db, node_id).await.unwrap();
        assert_eq!(fetched_node.position.x, 500.0);
        assert_eq!(fetched_node.position.y, 600.0);
    }

    #[tokio::test]
    async fn test_delete_node() {
        let db = setup_db().await;
        let canvas = add_canvas(&db, create_dummy_canvas()).await.unwrap();
        let canvas_id = canvas.id.unwrap();

        let node1 = add_node(&db, None, canvas_id.clone(), create_dummy_node())
            .await
            .unwrap();
        let node1_id = node1.id.clone().unwrap();

        let node2 = add_derived_node(&db, None, canvas_id, node1_id.clone(), create_dummy_node())
            .await
            .unwrap();
        let node2_id = node2.id.unwrap();

        // Try deleting node1 (parent) - should fail because of reference
        let result = delete_node(&db, node1_id.clone()).await;
        assert!(result.is_err());

        // Delete node2 (child) - should succeed
        let result = delete_node(&db, node2_id.clone()).await;
        assert!(result.is_ok());

        // Now delete node1 - should succeed
        let result = delete_node(&db, node1_id.clone()).await;
        assert!(result.is_ok());

        let fetched = get_node(&db, node1_id).await;
        assert!(fetched.is_err());
    }

    #[tokio::test]
    async fn test_update_chat_node_output() {
        let db = setup_db().await;
        let canvas = add_canvas(&db, create_dummy_canvas()).await.unwrap();

        let chat_node_data = ChatNodeData {
            input: LlmInput {
                system_prompt: None,
                history: vec![],
                user_input: Message::new_text(Role::User, "Hello"),
            },
            output: None,
            model_id: None,
        };

        let node_data = Node {
            id: None,
            position: NodePosition { x: 0.0, y: 0.0 },
            type_: NodeType::Chat {
                data: chat_node_data,
            },
        };

        let node = add_node(&db, None, canvas.id.unwrap(), node_data)
            .await
            .unwrap();
        let node_id = node.id.unwrap();

        let output = LlmOutput {
            content: vec![ContentPart::Text("Response".to_string())],
            usage: None,
            raw: None,
        };

        let updated_node = update_chat_node_output(&db, node_id.clone(), output.clone())
            .await
            .unwrap();

        match updated_node.type_ {
            NodeType::Chat { data } => {
                assert!(data.output.is_some());
                let out = data.output.unwrap();
                assert_eq!(out.content.len(), 1);
                if let ContentPart::Text(t) = &out.content[0] {
                    assert_eq!(t, "Response");
                } else {
                    panic!("Unexpected content part type");
                }
            }
            _ => panic!("Unexpected node type"),
        }
    }

    #[tokio::test]
    async fn test_update_canvas_chat_model_id() {
        let db = setup_db().await;
        let canvas = add_canvas(&db, create_dummy_canvas()).await.unwrap();
        let canvas_id = canvas.id.unwrap();

        // Initial state: None
        assert!(canvas.chat_model_id.is_none());

        // Update to "model-v1"
        let updated =
            update_canvas_chat_model_id(&db, canvas_id.clone(), Some("model-v1".to_string()))
                .await
                .unwrap();
        assert_eq!(updated.chat_model_id, Some("model-v1".to_string()));

        // Update to None
        let updated = update_canvas_chat_model_id(&db, canvas_id.clone(), None)
            .await
            .unwrap();
        assert!(updated.chat_model_id.is_none());
    }

    #[tokio::test]

    async fn test_add_node_with_embedding() {
        use crate::embedding::provider::local::LocalEmbedding;

        // 1. Setup DB and Manager
        let db = setup_db().await;
        let mut manager = EmbeddingServiceManager::new();
        // Create local service (this might fail if models are not downloaded, but strictly for this test we might mock?
        // But the requirement says "local embedding service를 만들고... tested logic".
        // LocalEmbedding::new() downloads models.
        // Assuming environment allows usage of LocalEmbedding if tests are running.
        let local_service = LocalEmbedding::new()
            .await
            .expect("Failed to create local embedding service");
        manager.add_service("local".to_string(), Box::new(local_service));
        let manager_arc = Arc::new(RwLock::new(manager));
        println!("Test environment setup complete.");

        // 2. Create Canvas with embedding_id="local"
        let canvas = add_canvas(&db, create_dummy_embedding_canvas())
            .await
            .unwrap();
        let canvas_id = canvas.id.unwrap();
        println!("Canvas created with embedding_id='local': {:?}", canvas_id);

        // 3. Add Chat Node
        // Need to provide output for embedding generation now
        let mut node_data = create_dummy_chat_node();
        if let NodeType::Chat { data, .. } = &mut node_data.type_ {
            data.output = Some(LlmOutput {
                content: vec![ContentPart::Text(
                    "This is the response that will be chunked and embedded.".to_string(),
                )],
                usage: None,
                raw: None,
            });
        }

        let mg = manager_arc.read().await;
        let service = mg.get_service("local").unwrap();
        let created_node = add_node(&db, Some(service.as_ref()), canvas_id.clone(), node_data)
            .await
            .unwrap();
        println!("Chat Node created: {:?}", created_node);

        // 4. Verify embedding is present (in chunks table)
        let node_id = created_node.id.unwrap();

        let sql = "SELECT * FROM chunk WHERE node = $node_id";
        let mut response = db.query(sql).bind(("node_id", node_id)).await.unwrap();
        let chunks: Vec<Chunk> = response.take(0).unwrap();

        assert!(!chunks.is_empty());
        assert_eq!(
            chunks[0].content.trim(),
            "This is the response that will be chunked and embedded."
        ); // Semantic chunking might return the whole sentence
        assert!(!chunks[0].embedding.is_empty());
        println!("Chunks verification successful. Count: {}", chunks.len());
    }

    #[tokio::test]
    async fn test_search_chunks() {
        use crate::embedding::provider::local::LocalEmbedding;

        // 1. Setup DB and Manager
        let db = setup_db().await;
        let mut manager = EmbeddingServiceManager::new();
        let local_service = LocalEmbedding::new()
            .await
            .expect("Failed to create local embedding service");
        manager.add_service("local".to_string(), Box::new(local_service));
        let manager_arc = Arc::new(RwLock::new(manager));

        let canvas = add_canvas(&db, create_dummy_embedding_canvas())
            .await
            .unwrap();
        let canvas_id = canvas.id.unwrap();

        // 2. Add Node with content
        let mut node_data = create_dummy_chat_node();
        if let NodeType::Chat { data, .. } = &mut node_data.type_ {
            data.output = Some(LlmOutput {
                content: vec![ContentPart::Text(
                    "The quick brown fox jumps over the lazy dog.".to_string(),
                )],
                usage: None,
                raw: None,
            });
        }

        let mg = manager_arc.read().await;
        let service = mg.get_service("local").unwrap();
        add_node(&db, Some(service.as_ref()), canvas_id.clone(), node_data)
            .await
            .unwrap();

        // 3. Search
        let query_vec = service
            .embed("default", vec!["fox".to_string()])
            .await
            .unwrap()[0]
            .clone();

        let chunks = search_chunks(&db, canvas_id, query_vec, 5, 0.0)
            .await
            .unwrap();

        assert!(!chunks.is_empty());
        assert!(chunks[0].content.contains("fox"));
    }

    #[tokio::test]
    async fn test_delete_node_cascades_chunks() {
        use crate::embedding::provider::local::LocalEmbedding;
        let db = setup_db().await;

        let mut manager = EmbeddingServiceManager::new();
        // Assuming test env has model or we use a mock.
        // For real integration tests we need the model.
        // If this fails due to missing model, we might need a workaround or ensure model is present.
        let local_service = LocalEmbedding::new()
            .await
            .expect("Failed to create local embedding service");
        manager.add_service("local".to_string(), Box::new(local_service));
        let manager_arc = Arc::new(RwLock::new(manager));

        let canvas = add_canvas(&db, create_dummy_embedding_canvas())
            .await
            .unwrap();
        let canvas_id = canvas.id.unwrap();

        // 1. Create Node with Embedding
        let mut node_data = create_dummy_chat_node();
        if let NodeType::Chat { data, .. } = &mut node_data.type_ {
            data.output = Some(LlmOutput {
                content: vec![ContentPart::Text(
                    "Cascading deletion test content.".to_string(),
                )],
                usage: None,
                raw: None,
            });
        }
        let mg = manager_arc.read().await;
        let service = mg.get_service("local").unwrap();
        let node = add_node(&db, Some(service.as_ref()), canvas_id.clone(), node_data)
            .await
            .unwrap();
        let node_id = node.id.unwrap();

        // 2. Verify chunks exist
        let sql = "SELECT * FROM chunk WHERE node = $node_id";
        let mut response = db
            .query(sql)
            .bind(("node_id", node_id.clone()))
            .await
            .unwrap();
        let chunks: Vec<Chunk> = response.take(0).unwrap();
        assert!(!chunks.is_empty(), "Chunks should exist before deletion");

        // 3. Verify relations exist
        let rel_sql = "SELECT value out FROM has_chunk WHERE in = $node_id";
        let mut response = db
            .query(rel_sql)
            .bind(("node_id", node_id.clone()))
            .await
            .unwrap();
        let chunk_ids: Vec<Thing> = response.take(0).unwrap();
        assert!(
            !chunk_ids.is_empty(),
            "Relations should exist before deletion"
        );

        // 4. Delete Node
        delete_node(&db, node_id.clone()).await.unwrap();

        // 5. Verify Chunks are gone
        let mut response = db
            .query(sql)
            .bind(("node_id", node_id.clone()))
            .await
            .unwrap();
        let chunks: Vec<Chunk> = response.take(0).unwrap();
        assert!(chunks.is_empty(), "Chunks should be deleted");

        // 6. Verify Relations are gone
        let mut response = db.query(rel_sql).bind(("node_id", node_id)).await.unwrap();
        let chunk_ids: Vec<Thing> = response.take(0).unwrap();
        assert!(chunk_ids.is_empty(), "Relations should be deleted");
    }
}
