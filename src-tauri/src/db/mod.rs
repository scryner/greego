use crate::db::schema::{Derives, Node, Sequences};
use std::sync::Arc;
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use tokio::sync::OnceCell;

pub mod schema;

static DB: OnceCell<Arc<Database>> = OnceCell::const_new();

#[derive(Clone)]
pub struct Database {
    pub client: Arc<Surreal<Db>>,
}

impl Database {
    pub async fn init() -> Result<Arc<Self>, surrealdb::Error> {
        let client = Surreal::new::<RocksDb>("greego.db").await?;
        client.use_ns("greego").use_db("main").await?;

        let db = Arc::new(Self {
            client: Arc::new(client),
        });

        // Initialize if not already set
        let _ = DB.set(db);

        Ok(DB.get().cloned().expect("DB should be initialized"))
    }

    pub fn get() -> Option<Arc<Self>> {
        DB.get().cloned()
    }

    pub async fn load_canvas(
        &self,
        canvas_id: Thing,
    ) -> Result<(Vec<Node>, Vec<Derives>, Vec<Sequences>), surrealdb::Error> {
        let sql = r#"
            SELECT * FROM node WHERE id IN (SELECT VALUE out FROM holds WHERE in = $canvas_id);
            SELECT * FROM derives WHERE canvas = $canvas_id;
            SELECT * FROM sequences WHERE canvas = $canvas_id;
        "#;
        let mut responses = self
            .client
            .query(sql)
            .bind(("canvas_id", canvas_id))
            .await?;

        let nodes: Vec<Node> = responses.take(0)?;
        let derives: Vec<Derives> = responses.take(1)?;
        let sequences: Vec<Sequences> = responses.take(2)?;

        Ok((nodes, derives, sequences))
    }

    pub async fn delete_node(&self, node_id: Thing) -> Result<(), surrealdb::Error> {
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

        self.client
            .query(sql)
            .bind(("node_id", node_id))
            .await?
            .check()?;

        Ok(())
    }

    pub async fn add_node(&self, canvas_id: Thing, node: Node) -> Result<Node, surrealdb::Error> {
        let sql = r#"
            let $node = (CREATE node CONTENT $node_data);
            let $node_id = $node[0].id;
            RELATE $canvas_id -> holds -> $node_id;
            RETURN $node[0];
        "#;

        let mut response = self
            .client
            .query(sql)
            .bind(("canvas_id", canvas_id))
            .bind(("node_data", node))
            .await?;

        let created: Option<Node> = response.take(3)?;
        Ok(created.expect("Failed to create node"))
    }

    pub async fn add_derived_node(
        &self,
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

        let mut response = self
            .client
            .query(sql)
            .bind(("canvas_id", canvas_id))
            .bind(("from", from))
            .bind(("node_data", to))
            .await?;

        let created: Option<Node> = response.take(4)?;
        Ok(created.expect("Failed to create derived node"))
    }

    pub async fn add_sequenced_node(
        &self,
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

        let mut response = self
            .client
            .query(sql)
            .bind(("canvas_id", canvas_id))
            .bind(("from", from))
            .bind(("node_data", to))
            .await?;

        let created: Option<Node> = response.take(4)?;
        Ok(created.expect("Failed to create sequenced node"))
    }

    pub async fn move_node_position(
        &self,
        node_id: Thing,
        x: f64,
        y: f64,
    ) -> Result<Node, surrealdb::Error> {
        let sql = "UPDATE $node_id SET position = { x: $x, y: $y } RETURN AFTER";

        let mut response = self
            .client
            .query(sql)
            .bind(("node_id", node_id))
            .bind(("x", x))
            .bind(("y", y))
            .await?;

        let updated: Option<Node> = response.take(0)?;
        Ok(updated.expect("Failed to move node"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::{NodePosition, NodeType};
    use surrealdb::engine::local::Mem;

    async fn setup_test_db() -> Database {
        let client = Surreal::new::<Mem>(()).await.unwrap();
        client.use_ns("test").use_db("test").await.unwrap();
        Database {
            client: Arc::new(client),
        }
    }

    fn mock_node() -> Node {
        Node {
            id: None,
            position: NodePosition { x: 100.0, y: 100.0 },
            type_: NodeType::Chat {
                value: serde_json::json!({"text": "Hello"}),
            },
        }
    }

    #[tokio::test]
    async fn test_add_node() {
        let db = setup_test_db().await;
        let canvas_id = Thing::from(("canvas", "test_canvas"));
        let node = mock_node();

        let created = db.add_node(canvas_id.clone(), node).await.unwrap();
        assert!(created.id.is_some());

        // Verify holds relation
        let (nodes, _, _) = db.load_canvas(canvas_id).await.unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].id, created.id);
    }

    #[tokio::test]
    async fn test_add_derived_node() {
        let db = setup_test_db().await;
        let canvas_id = Thing::from(("canvas", "test_canvas"));
        let root = db.add_node(canvas_id.clone(), mock_node()).await.unwrap();

        let derived = db
            .add_derived_node(
                canvas_id.clone(),
                root.id.as_ref().unwrap().clone(),
                mock_node(),
            )
            .await
            .unwrap();
        assert!(derived.id.is_some());

        let (_, derives, _) = db.load_canvas(canvas_id).await.unwrap();
        assert_eq!(derives.len(), 1);
        assert_eq!(derives[0].from, root.id.as_ref().unwrap().clone());
        assert_eq!(derives[0].to, derived.id.unwrap());
    }

    #[tokio::test]
    async fn test_add_sequenced_node() {
        let db = setup_test_db().await;
        let canvas_id = Thing::from(("canvas", "test_canvas"));
        let root = db.add_node(canvas_id.clone(), mock_node()).await.unwrap();

        let sequenced = db
            .add_sequenced_node(
                canvas_id.clone(),
                root.id.as_ref().unwrap().clone(),
                mock_node(),
            )
            .await
            .unwrap();
        assert!(sequenced.id.is_some());

        let (_, _, sequences) = db.load_canvas(canvas_id).await.unwrap();
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].from, root.id.as_ref().unwrap().clone());
        assert_eq!(sequences[0].to, sequenced.id.unwrap());
    }

    #[tokio::test]
    async fn test_move_node_position() {
        let db = setup_test_db().await;
        let canvas_id = Thing::from(("canvas", "test_canvas"));
        let node = db.add_node(canvas_id, mock_node()).await.unwrap();

        let updated = db
            .move_node_position(node.id.unwrap(), 200.0, 300.0)
            .await
            .unwrap();
        assert_eq!(updated.position.x, 200.0);
        assert_eq!(updated.position.y, 300.0);
    }

    #[tokio::test]
    async fn test_delete_node_success() {
        let db = setup_test_db().await;
        let canvas_id = Thing::from(("canvas", "test_canvas"));
        let node = db.add_node(canvas_id.clone(), mock_node()).await.unwrap();

        db.delete_node(node.id.unwrap()).await.unwrap();

        let (nodes, _, _) = db.load_canvas(canvas_id).await.unwrap();
        assert_eq!(nodes.len(), 0);
    }

    #[tokio::test]
    async fn test_delete_node_failure() {
        let db = setup_test_db().await;
        let canvas_id = Thing::from(("canvas", "test_canvas"));
        let root = db.add_node(canvas_id.clone(), mock_node()).await.unwrap();
        let _derived = db
            .add_derived_node(canvas_id, root.id.clone().unwrap(), mock_node())
            .await
            .unwrap();

        // Should fail because it has a derivation edge
        let result = db.delete_node(root.id.unwrap()).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("referenced by other nodes"));
    }

    #[tokio::test]
    async fn test_macro_flow() {
        let db = setup_test_db().await;
        let canvas_id = Thing::from(("canvas", "macro_test"));

        // 1. Add root node
        let root = db
            .add_node(canvas_id.clone(), mock_node())
            .await
            .expect("Failed to add root");
        let root_id = root.id.clone().unwrap();

        // 2. Add derived node
        let d1 = db
            .add_derived_node(canvas_id.clone(), root_id.clone(), mock_node())
            .await
            .expect("Failed to add d1");
        let d1_id = d1.id.clone().unwrap();

        // 3. Add sequenced node from d1
        let s1 = db
            .add_sequenced_node(canvas_id.clone(), d1_id.clone(), mock_node())
            .await
            .expect("Failed to add s1");
        let s1_id = s1.id.clone().unwrap();

        // 4. Move s1
        let s1_moved = db
            .move_node_position(s1_id.clone(), 500.0, 500.0)
            .await
            .expect("Failed to move s1");
        assert_eq!(s1_moved.position.x, 500.0);

        // 5. Attempt to delete root (should fail)
        let del_root_res = db.delete_node(root_id.clone()).await;
        assert!(del_root_res.is_err());

        // 6. Delete s1 (success)
        db.delete_node(s1_id).await.expect("Failed to delete s1");

        // 7. Delete d1 (success)
        db.delete_node(d1_id).await.expect("Failed to delete d1");

        // 8. Delete root (success)
        db.delete_node(root_id)
            .await
            .expect("Failed to delete root");

        // 9. Verify canvas is empty
        let (nodes, derives, sequences) = db.load_canvas(canvas_id).await.unwrap();
        assert_eq!(nodes.len(), 0);
        assert_eq!(derives.len(), 0);
        assert_eq!(sequences.len(), 0);
    }
}
