use crate::db::schema::{Derives, Node, Sequences};
use surrealdb::engine::local::Db;
use surrealdb::sql::Thing;
use surrealdb::Surreal;

pub async fn add_node(
    client: &Surreal<Db>,
    canvas_id: Thing,
    node: Node,
) -> Result<Node, surrealdb::Error> {
    let sql = r#"
        let $node = (CREATE node CONTENT $node_data);
        let $node_id = $node[0].id;
        RELATE $canvas_id -> holds -> $node_id;
        RETURN $node[0];
    "#;

    let mut response = client
        .query(sql)
        .bind(("canvas_id", canvas_id))
        .bind(("node_data", node))
        .await?;

    let created: Option<Node> = response.take(3)?;
    Ok(created.expect("Failed to create node"))
}

pub async fn add_derived_node(
    client: &Surreal<Db>,
    canvas_id: Thing,
    from: Thing,
    to: Node,
) -> Result<Node, surrealdb::Error> {
    let sql = r#"
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
        .await?;

    let created: Option<Node> = response.take(4)?;
    Ok(created.expect("Failed to create derived node"))
}

pub async fn add_sequenced_node(
    client: &Surreal<Db>,
    canvas_id: Thing,
    from: Thing,
    to: Node,
) -> Result<Node, surrealdb::Error> {
    let sql = r#"
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
        .await?;

    let created: Option<Node> = response.take(4)?;
    Ok(created.expect("Failed to create sequenced node"))
}

pub async fn move_node_position(
    client: &Surreal<Db>,
    node_id: Thing,
    x: f64,
    y: f64,
) -> Result<Node, surrealdb::Error> {
    let sql = "UPDATE $node_id SET position = { x: $x, y: $y } RETURN AFTER";

    let mut response = client
        .query(sql)
        .bind(("node_id", node_id))
        .bind(("x", x))
        .bind(("y", y))
        .await?;

    let updated: Option<Node> = response.take(0)?;
    Ok(updated.expect("Failed to move node"))
}

pub async fn delete_node(client: &Surreal<Db>, node_id: Thing) -> Result<(), surrealdb::Error> {
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
        .await?
        .check()?;

    Ok(())
}

pub async fn load_canvas(
    client: &Surreal<Db>,
    canvas_id: Thing,
) -> Result<(Vec<Node>, Vec<Derives>, Vec<Sequences>), surrealdb::Error> {
    let sql = r#"
        SELECT * FROM node WHERE id IN (SELECT VALUE out FROM holds WHERE in = $canvas_id);
        SELECT * FROM derives WHERE canvas = $canvas_id;
        SELECT * FROM sequences WHERE canvas = $canvas_id;
    "#;
    let mut responses = client.query(sql).bind(("canvas_id", canvas_id)).await?;

    let nodes: Vec<Node> = responses.take(0)?;
    let derives: Vec<Derives> = responses.take(1)?;
    let sequences: Vec<Sequences> = responses.take(2)?;

    Ok((nodes, derives, sequences))
}
