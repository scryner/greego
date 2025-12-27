use crate::db::schema::{Canvas, Derives, Node, Sequences};
use crate::db::schema::{ChatNodeEmbedding, NodeType};
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

async fn fetch_canvas(client: &Surreal<Db>, canvas_id: &Thing) -> anyhow::Result<Canvas> {
    let sql = "SELECT * FROM $id";
    let mut response = client.query(sql).bind(("id", canvas_id.clone())).await?;
    let canvas: Option<Canvas> = response.take(0)?;
    canvas.ok_or_else(|| anyhow::anyhow!("Canvas not found"))
}

async fn enrich_node_with_embedding(
    node: &mut Node,
    embedding_service: Option<&dyn EmbeddingService>,
    embedding_id: Option<&str>,
) -> anyhow::Result<()> {
    if let NodeType::Chat { data, embedding } = &mut node.type_ {
        // Only generate embedding if it doesn't exist yet
        if embedding.is_none() {
            if let Some(emb_id) = embedding_id {
                let service = embedding_service.ok_or_else(|| {
                    anyhow::anyhow!("Embedding service not provided but canvas requires embedding")
                })?;

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
                                if !chunks.is_empty() {
                                    match service.embed("default", chunks.clone()).await {
                                        Ok(embeddings) => {
                                            *embedding = Some(ChatNodeEmbedding {
                                                embedding_id: emb_id.to_string(),
                                                chunks: embeddings,
                                            });
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
    }
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

    created.ok_or_else(|| {
        log::error!("{}: query succeeded but returned no created node", op_name);
        anyhow::anyhow!("Failed to create node")
    })
}

async fn create_node_common(
    client: &Surreal<Db>,
    embedding_service: Option<&dyn EmbeddingService>,
    canvas_id: Thing,
    mut node: Node,
    relation: Option<(Thing, &'static str)>,
    op_name: &str,
) -> anyhow::Result<Node> {
    log::debug!("{}: canvas_id={}, node={:?}", op_name, canvas_id, node);

    // 1. Fetch Canvas to check embedding_id
    let canvas = fetch_canvas(client, &canvas_id).await?;

    // 2. Enrich node with embedding if applicable
    enrich_node_with_embedding(&mut node, embedding_service, canvas.embedding_id.as_deref())
        .await?;

    // 3. Store node in DB
    let result = store_node(client, canvas_id, node, relation, op_name).await?;

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
) -> anyhow::Result<(Vec<Node>, Vec<Derives>, Vec<Sequences>)> {
    log::debug!("load_canvas: canvas_id={}", canvas_id);

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
    Ok((nodes, derives, sequences))
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
        }
    }

    fn create_dummy_embedding_canvas() -> Canvas {
        Canvas {
            id: None,
            title: "Embedding Canvas".to_string(),
            created_at: chrono::Utc::now(),
            embedding_id: Some("local".to_string()),
            reranker_id: None,
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
                embedding: None,
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
        let (nodes, _, _) = load_canvas(&db, canvas_id).await.unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].id, created_node.id);
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
        let (nodes, derives, sequences) = load_canvas(&db, canvas_id).await.unwrap();
        assert_eq!(nodes.len(), 3);
        assert_eq!(derives.len(), 1);
        assert_eq!(sequences.len(), 1);

        assert_eq!(derives[0].from, node1_id);
        assert_eq!(derives[0].to, node2_id);

        assert_eq!(sequences[0].from, node1_id);
        assert_eq!(sequences[0].to, node3_id);
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
                embedding: None,
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
            NodeType::Chat { data, .. } => {
                assert!(data.output.is_some());
                let out = data.output.unwrap();
                match out.content[0] {
                    ContentPart::Text(ref t) => assert_eq!(t, "Response"),
                    _ => panic!("Unexpected content type"),
                }
            }
            _ => panic!("Unexpected node type"),
        }
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

        // 4. Verify embedding is present
        match created_node.type_ {
            NodeType::Chat { embedding, .. } => {
                assert!(embedding.is_some());
                let emb = embedding.unwrap();
                assert_eq!(emb.embedding_id, "local");
                assert!(!emb.chunks.is_empty());
                assert!(emb.chunks[0].len() > 0);
                println!("Embedding verification successful.");
                println!("Embedding ID: {}", emb.embedding_id);
                println!("Chunks Count: {}", emb.chunks.len());
            }
            _ => panic!("Expected Chat node"),
        }
    }
}
