use crate::db::schema::{Canvas, Derives, Node, Sequences};
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

pub async fn add_node(client: &Surreal<Db>, canvas_id: Thing, node: Node) -> anyhow::Result<Node> {
    log::debug!("add_node: canvas_id={}, node={:?}", canvas_id, node);

    let sql = r#"
        IF array::len((SELECT * FROM $canvas_id)) == 0 {
            THROW "Canvas not found";
        };
        let $node = (CREATE node CONTENT $node_data);
        let $node_id = $node[0].id;
        RELATE $canvas_id -> holds -> $node_id;
        RETURN $node[0];
    "#;

    let mut response = client
        .query(sql)
        .bind(("canvas_id", canvas_id))
        .bind(("node_data", node))
        .await
        .inspect_err(|e| log::error!("add_node: query failed: {}", e))?;

    let created: Option<Node> = response.take(4).inspect_err(|e| {
        log::error!(
            "add_node: failed to retrieve created node from response: {}",
            e
        )
    })?;
    let result = created.ok_or_else(|| {
        log::error!("add_node: query succeeded but returned no created node");
        anyhow::anyhow!("Failed to create node")
    })?;
    log::debug!("add_node: success, created={:?}", result);
    Ok(result)
}

pub async fn add_derived_node(
    client: &Surreal<Db>,
    canvas_id: Thing,
    from: Thing,
    to: Node,
) -> anyhow::Result<Node> {
    log::debug!(
        "add_derived_node: canvas_id={}, from={}, to={:?}",
        canvas_id,
        from,
        to
    );

    let sql = r#"
        IF array::len((SELECT * FROM $canvas_id)) == 0 {
            THROW "Canvas not found";
        };
        let $node = (CREATE node CONTENT $node_data);
        let $node_id = $node[0].id;
        RELATE $canvas_id -> holds -> $node_id;
        RELATE $from -> derives -> $node_id SET canvas = $canvas_id;
        RETURN $node[0];
    "#;

    let mut response = client
        .query(sql)
        .bind(("canvas_id", canvas_id))
        .bind(("from", from))
        .bind(("node_data", to))
        .await
        .inspect_err(|e| log::error!("add_derived_node: query failed: {}", e))?;

    let created: Option<Node> = response
        .take(5)
        .inspect_err(|e| log::error!("add_derived_node: failed to retrieve created node: {}", e))?;
    let result = created.ok_or_else(|| {
        log::error!("add_derived_node: query succeeded but returned no created node");
        anyhow::anyhow!("Failed to create derived node")
    })?;
    log::debug!("add_derived_node: success, created={:?}", result);
    Ok(result)
}

pub async fn add_sequenced_node(
    client: &Surreal<Db>,
    canvas_id: Thing,
    from: Thing,
    to: Node,
) -> anyhow::Result<Node> {
    log::debug!(
        "add_sequenced_node: canvas_id={}, from={}, to={:?}",
        canvas_id,
        from,
        to
    );

    let sql = r#"
        IF array::len((SELECT * FROM $canvas_id)) == 0 {
            THROW "Canvas not found";
        };
        let $node = (CREATE node CONTENT $node_data);
        let $node_id = $node[0].id;
        RELATE $canvas_id -> holds -> $node_id;
        RELATE $from -> sequences -> $node_id SET canvas = $canvas_id;
        RETURN $node[0];
    "#;

    let mut response = client
        .query(sql)
        .bind(("canvas_id", canvas_id))
        .bind(("from", from))
        .bind(("node_data", to))
        .await
        .inspect_err(|e| log::error!("add_sequenced_node: query failed: {}", e))?;

    let created: Option<Node> = response.take(5).inspect_err(|e| {
        log::error!("add_sequenced_node: failed to retrieve created node: {}", e)
    })?;
    let result = created.ok_or_else(|| {
        log::error!("add_sequenced_node: query succeeded but returned no created node");
        anyhow::anyhow!("Failed to create sequenced node")
    })?;
    log::debug!("add_sequenced_node: success, created={:?}", result);
    Ok(result)
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
