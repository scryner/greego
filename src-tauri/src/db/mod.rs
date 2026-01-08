use crate::db::event_loop::EventLoop;
use crate::db::events::DbEvent;
use crate::db::schema::{Canvas, CanvasData, Node};
use crate::embedding::EmbeddingServiceManager;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use surrealdb::engine::local::{Db, Mem, RocksDb};
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, oneshot, OnceCell, RwLock};

pub mod event_loop;
pub mod events;
pub mod operation;
pub mod schema;

static DB: OnceCell<Arc<Database>> = OnceCell::const_new();

#[derive(Clone)]
pub struct Database {
    pub client: Arc<Surreal<Db>>,
    sender: mpsc::Sender<DbEvent>,
    config: DatabaseConfig,
}

#[derive(Clone)]
pub struct DatabaseConfig {
    pub connection: DatabaseConnectionConfig,
    pub default_first_canvas_title: String,
}

#[derive(Clone)]
pub enum DatabaseConnectionConfig {
    InMemory,
    Persistent(PathBuf),
}

impl Database {
    pub async fn init(
        app_handle: AppHandle,
        config: DatabaseConfig,
        embedding_manager: Arc<RwLock<EmbeddingServiceManager>>,
    ) -> anyhow::Result<Arc<Self>> {
        let client = match config.connection.clone() {
            DatabaseConnectionConfig::InMemory => {
                let client = Surreal::new::<Mem>(()).await?;
                client
            }
            DatabaseConnectionConfig::Persistent(path) => {
                let client = Surreal::new::<RocksDb>(path).await?;
                client
            }
        };
        client.use_ns("greego").use_db("main").await?;
        let client = Arc::new(client);

        // Error feedback channel
        let (err_tx, mut err_rx) = mpsc::channel(100);

        // Spawn error listener
        let app_handle_clone = app_handle.clone();
        tokio::spawn(async move {
            while let Some(msg) = err_rx.recv().await {
                // Emit event to frontend
                let _ = app_handle_clone.emit("db-error", msg);
            }
        });

        // Event Loop Configuration
        let (sender, receiver) = mpsc::channel(100);
        let event_loop = EventLoop::new(
            client.clone(),
            embedding_manager,
            receiver,
            err_tx,
            Duration::from_millis(500),
            50,
            1000,
        );

        // Spawn event loop
        tokio::spawn(async move {
            event_loop.run().await;
        });

        let db = Arc::new(Self {
            client,
            sender,
            config,
        });

        // Initialize if not already set
        let _ = DB.set(db);

        Ok(DB.get().cloned().expect("DB should be initialized"))
    }

    pub fn get() -> Option<Arc<Self>> {
        DB.get().cloned()
    }

    pub async fn get_map(&self, canvas_id: Thing) -> anyhow::Result<Option<CanvasData>> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.sender
            .send(DbEvent::LoadCanvas(canvas_id, tx))
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send LoadCanvas event: {}", e))?;
        rx.await?
    }

    pub async fn load_canvas(&self, canvas_id: Thing) -> anyhow::Result<Option<CanvasData>> {
        let result = operation::load_canvas(&self.client, canvas_id).await?;
        if result.is_none() {
            // Check if DB is empty
            if operation::is_canvas_empty(&self.client).await? {
                // Create default canvas
                let new_canvas = Canvas {
                    id: None,
                    title: self.config.default_first_canvas_title.clone(),
                    created_at: chrono::Utc::now(),
                    embedding_id: Some("local".to_string()),
                    reranker_id: None,
                };
                let created = self.add_canvas(new_canvas).await?;
                return Ok(Some(CanvasData {
                    canvas: created,
                    nodes: vec![],
                    derives: vec![],
                    sequences: vec![],
                }));
            }
        }
        Ok(result)
    }

    pub async fn add_canvas(&self, canvas: Canvas) -> anyhow::Result<Canvas> {
        let (tx, rx) = oneshot::channel();
        let event = DbEvent::AddCanvas {
            canvas,
            response: tx,
        };
        self.sender.send(event).await.expect("Event loop closed");
        rx.await.expect("Response dropped")
    }

    pub async fn list_canvas(&self, limit: usize, offset: usize) -> anyhow::Result<Vec<Canvas>> {
        let (tx, rx) = oneshot::channel();
        let event = DbEvent::ListCanvas {
            limit,
            offset,
            response: tx,
        };
        self.sender.send(event).await.expect("Event loop closed");
        rx.await.expect("Response dropped")
    }

    pub async fn add_node(&self, canvas_id: Thing, node: Node) -> anyhow::Result<Node> {
        let (tx, rx) = oneshot::channel();
        let event = DbEvent::AddNode {
            canvas_id,
            node,
            response: tx,
        };
        self.sender.send(event).await.expect("Event loop closed");
        rx.await.expect("Response dropped")
    }

    pub async fn add_derived_node(
        &self,
        canvas_id: Thing,
        from: Thing,
        to: Node,
    ) -> anyhow::Result<Node> {
        let (tx, rx) = oneshot::channel();
        let event = DbEvent::AddDerivedNode {
            canvas_id,
            from,
            to,
            response: tx,
        };
        self.sender.send(event).await.expect("Event loop closed");
        rx.await.expect("Response dropped")
    }

    pub async fn add_sequenced_node(
        &self,
        canvas_id: Thing,
        from: Thing,
        to: Node,
    ) -> anyhow::Result<Node> {
        let (tx, rx) = oneshot::channel();
        let event = DbEvent::AddSequencedNode {
            canvas_id,
            from,
            to,
            response: tx,
        };
        self.sender.send(event).await.expect("Event loop closed");
        rx.await.expect("Response dropped")
    }

    pub async fn move_node_position(&self, node_id: Thing, x: f64, y: f64) -> anyhow::Result<Node> {
        let (tx, rx) = oneshot::channel();
        let event = DbEvent::MoveNode {
            node_id,
            x,
            y,
            response: Some(tx),
        };
        self.sender.send(event).await.expect("Event loop closed");

        match rx.await {
            Ok(res) => res,
            Err(_) => {
                // Channel closed (likely coalesced/dropped).
                // Returning a panic for now as this signifies the operation was optimised away.
                // In a real app we might want a specific error variant or to ignore.
                panic!("Event was coalesced/optimised away");
            }
        }
    }

    pub async fn delete_node(&self, node_id: Thing) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        let event = DbEvent::DeleteNode {
            node_id,
            response: tx,
        };
        self.sender.send(event).await.expect("Event loop closed");
        rx.await.expect("Response dropped")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::{Node, NodePosition, NodeType};
    use surrealdb::engine::local::Mem;

    async fn setup_test_db() -> Database {
        let client = Surreal::new::<Mem>(()).await.unwrap();
        client.use_ns("test").use_db("test").await.unwrap();
        let client = Arc::new(client);

        // Dummy error channel for tests
        let (err_tx, _err_rx) = mpsc::channel(100);

        let (sender, receiver) = mpsc::channel(100);
        let dummy_manager = Arc::new(RwLock::new(EmbeddingServiceManager::new()));
        let event_loop = EventLoop::new(
            client.clone(),
            dummy_manager,
            receiver,
            err_tx,
            Duration::from_millis(50),
            10,
            100,
        );

        tokio::spawn(async move {
            event_loop.run().await;
        });

        Database {
            client,
            sender,
            config: DatabaseConfig {
                connection: DatabaseConnectionConfig::InMemory,
                default_first_canvas_title: "New Canvas".to_string(),
            },
        }
    }

    fn mock_node() -> Node {
        use crate::llm::{LlmInput, Message, Role};
        let input = LlmInput {
            system_prompt: None,
            history: vec![],
            user_input: Message::new_text(Role::User, "Hello"),
        };
        Node {
            id: None,
            position: NodePosition { x: 100.0, y: 100.0 },
            type_: NodeType::Chat {
                data: crate::db::schema::ChatNodeData {
                    input,
                    output: None,
                    model_id: None,
                },
            },
        }
    }

    fn mock_canvas(title: &str) -> Canvas {
        Canvas {
            id: None,
            title: title.to_string(),
            created_at: chrono::Utc::now(),
            embedding_id: None,
            reranker_id: None,
        }
    }

    #[tokio::test]
    async fn test_add_node() {
        let db = setup_test_db().await;
        // Create canvas first
        let canvas = db.add_canvas(mock_canvas("test_canvas")).await.unwrap();
        let canvas_id = canvas.id.unwrap();

        let node = mock_node();

        let created = db.add_node(canvas_id.clone(), node).await.unwrap();
        assert!(created.id.is_some());

        let canvas_data = db.load_canvas(canvas_id).await.unwrap().unwrap();
        assert_eq!(canvas_data.nodes.len(), 1);
    }

    #[tokio::test]
    async fn test_move_node_position() {
        let db = setup_test_db().await;
        let canvas = db.add_canvas(mock_canvas("test_canvas")).await.unwrap();
        let canvas_id = canvas.id.unwrap();

        let node = db.add_node(canvas_id, mock_node()).await.unwrap();

        let updated = db
            .move_node_position(node.id.unwrap(), 200.0, 300.0)
            .await
            .unwrap();
        assert_eq!(updated.position.x, 200.0);
    }

    #[tokio::test]
    async fn test_event_optimization() {
        let db = setup_test_db().await;
        let canvas = db.add_canvas(mock_canvas("opt_test")).await.unwrap();
        let canvas_id = canvas.id.unwrap();

        let node = db.add_node(canvas_id, mock_node()).await.unwrap();
        let _node_id = node.id.unwrap();

        // Just run tests to ensure no crash
    }
    #[tokio::test]
    async fn test_load_canvas_after_add_node() {
        let db = setup_test_db().await;
        // Use a clean canvas ID
        let canvas = db.add_canvas(mock_canvas("test_load")).await.unwrap();
        let canvas_id = canvas.id.unwrap();

        let node = mock_node();

        let created_node = db.add_node(canvas_id.clone(), node).await.unwrap();
        let created_id = created_node.id.unwrap();

        // Verify we can load it back via load_canvas (which checks relations)
        let canvas_data = db.load_canvas(canvas_id).await.unwrap().unwrap();
        assert_eq!(
            canvas_data.nodes.len(),
            1,
            "Should find 1 node attached to canvas"
        );
        assert_eq!(canvas_data.nodes[0].id, Some(created_id), "ID should match");
    }

    #[tokio::test]
    async fn test_database_load_canvas_auto_create() {
        // 1. Setup DB with specific config
        let client = Surreal::new::<Mem>(()).await.unwrap();
        client.use_ns("test").use_db("test").await.unwrap();
        let client = Arc::new(client);
        let (sender, receiver) = mpsc::channel(100);
        let (err_tx, _) = mpsc::channel(100);

        let dummy_manager = Arc::new(RwLock::new(EmbeddingServiceManager::new()));
        let event_loop = EventLoop::new(
            client.clone(),
            dummy_manager,
            receiver,
            err_tx,
            Duration::from_millis(50),
            10,
            100,
        );
        tokio::spawn(async move {
            event_loop.run().await;
        });

        // Initialize with "My Default Canvas" title
        let db = Database {
            client,
            sender,
            config: DatabaseConfig {
                connection: DatabaseConnectionConfig::InMemory,
                default_first_canvas_title: "My Default Canvas".to_string(),
            },
        };

        // 2. Database is empty. Request a load.
        let fake_id = Thing::from(("canvas", "nonexistent"));
        let result = db.load_canvas(fake_id.clone()).await.unwrap();

        // 3. Should return a canvas
        assert!(result.is_some(), "Should have auto-created a canvas");
        let data = result.unwrap();
        assert_eq!(data.canvas.title, "My Default Canvas");

        // 4. Verify it was actually saved
        let list = db.list_canvas(10, 0).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].title, "My Default Canvas");
        assert_eq!(list[0].embedding_id, Some("local".to_string()));

        // 5. Subsequent load of nonexistent ID should NOT create another one
        // because DB is no longer empty.
        let fake_id_2 = Thing::from(("canvas", "another_fake"));
        let result_2 = db.load_canvas(fake_id_2).await.unwrap();
        assert!(
            result_2.is_none(),
            "Should NOT create second default canvas"
        );
    }
}
