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

    pub async fn upsert_node(&self, node: Node) -> Result<Node, surrealdb::Error> {
        // We clone needed parts to satisfy 'static requirement or move them
        let mut result = if let Some(id) = &node.id {
            self.client
                .query("UPDATE $id CONTENT $data RETURN AFTER")
                .bind(("id", id.clone())) // Clone Thing
                .bind(("data", node.clone())) // Clone Node
                .await?
        } else {
            self.client
                .query("CREATE node CONTENT $data RETURN AFTER")
                .bind(("data", node.clone()))
                .await?
        };

        // The result is usually a list of results, we take the first one
        let created: Option<Node> = result.take(0)?;
        Ok(created.expect("Database returned no result for node upsert"))
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
}
