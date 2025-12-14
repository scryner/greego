use crate::db::schema::{ChatNode, FlowEdge};
use std::sync::Arc;
use surrealdb::engine::local::{Db, RocksDb};
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

    pub async fn upsert_node(&self, node: ChatNode) -> Result<ChatNode, surrealdb::Error> {
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
        let created: Option<ChatNode> = result.take(0)?;
        Ok(created.expect("Database returned no result for node upsert"))
    }

    pub async fn connect_nodes(
        &self,
        _edge_id: Option<String>,
        source: String,
        target: String,
    ) -> Result<FlowEdge, surrealdb::Error> {
        // RELATE source->connected_to->target
        // We are ignoring edge_id for now as RELATE handles ID generation,
        // unless we want to force specific logic which is complex with RELATE in this context without specific ID format.

        let mut response = self
            .client
            .query("RELATE type::thing($source)->connected_to->type::thing($target) RETURN AFTER")
            .bind(("source", source))
            .bind(("target", target))
            .await?;

        let created_edge: Option<FlowEdge> = response.take(0)?;
        Ok(created_edge.expect("Database returned no result for edge creation"))
    }

    pub async fn load_graph(&self) -> Result<(Vec<ChatNode>, Vec<FlowEdge>), surrealdb::Error> {
        let mut nodes_res = self.client.query("SELECT * FROM node").await?;
        let nodes: Vec<ChatNode> = nodes_res.take(0)?;

        let mut edges_res = self.client.query("SELECT * FROM connected_to").await?;
        let edges: Vec<FlowEdge> = edges_res.take(0)?;

        Ok((nodes, edges))
    }
}
