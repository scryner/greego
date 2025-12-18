use crate::db::events::{DbEvent, EventPriority};
use crate::db::operation;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use tokio::sync::mpsc;
use tokio::time::{interval, Interval};

pub struct EventLoop {
    client: Arc<Surreal<Db>>,
    queue: VecDeque<DbEvent>,
    receiver: mpsc::Receiver<DbEvent>,
    error_sender: mpsc::Sender<String>,
    interval: Interval,
    batch_size: usize,
    max_queue_len: usize,
}

impl EventLoop {
    pub fn new(
        client: Arc<Surreal<Db>>,
        receiver: mpsc::Receiver<DbEvent>,
        error_sender: mpsc::Sender<String>,
        check_interval: Duration,
        batch_size: usize,
        max_queue_len: usize,
    ) -> Self {
        let mut interval = interval(check_interval);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        Self {
            client,
            queue: VecDeque::with_capacity(max_queue_len),
            receiver,
            error_sender,
            interval,
            batch_size,
            max_queue_len,
        }
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                // Receive new event
                Some(event) = self.receiver.recv() => {
                   self.handle_incoming_event(event).await;
                }
                // Interval tick
                _ = self.interval.tick() => {
                    self.process_queue().await;
                }
            }
        }
    }

    async fn handle_incoming_event(&mut self, event: DbEvent) {
        let priority = event.priority();

        if self.queue.len() >= self.max_queue_len {
            self.process_queue().await;
        }

        self.queue.push_back(event);

        if priority == EventPriority::Immediate {
            self.process_queue().await;
        }
    }

    async fn process_queue(&mut self) {
        if self.queue.is_empty() {
            return;
        }

        self.optimize_queue();

        let count = self.queue.len().min(self.batch_size);
        for _ in 0..count {
            if let Some(event) = self.queue.pop_front() {
                self.execute_event(event).await;
            }
        }
    }

    fn optimize_queue(&mut self) {
        let mut seen_moves = std::collections::HashSet::new();
        let mut indices_to_remove = Vec::new();

        for i in (0..self.queue.len()).rev() {
            if let Some(DbEvent::MoveNode { node_id, .. }) = self.queue.get(i) {
                if seen_moves.contains(node_id) {
                    indices_to_remove.push(i);
                } else {
                    seen_moves.insert(node_id.clone());
                }
            }
        }

        for i in indices_to_remove {
            self.queue.remove(i);
        }
    }

    async fn report_error_if_any<T>(&self, result: &Result<T, surrealdb::Error>) {
        if let Err(e) = result {
            let _ = self.error_sender.send(e.to_string()).await;
        }
    }

    async fn execute_event(&self, event: DbEvent) {
        match event {
            DbEvent::AddNode {
                canvas_id,
                node,
                response,
            } => {
                let res = operation::add_node(&self.client, canvas_id, node).await;
                self.report_error_if_any(&res).await;
                let _ = response.send(res);
            }
            DbEvent::AddDerivedNode {
                canvas_id,
                from,
                to,
                response,
            } => {
                let res = operation::add_derived_node(&self.client, canvas_id, from, to).await;
                self.report_error_if_any(&res).await;
                let _ = response.send(res);
            }
            DbEvent::AddSequencedNode {
                canvas_id,
                from,
                to,
                response,
            } => {
                let res = operation::add_sequenced_node(&self.client, canvas_id, from, to).await;
                self.report_error_if_any(&res).await;
                let _ = response.send(res);
            }
            DbEvent::MoveNode {
                node_id,
                x,
                y,
                response,
            } => {
                let res = operation::move_node_position(&self.client, node_id, x, y).await;
                self.report_error_if_any(&res).await;
                if let Some(tx) = response {
                    let _ = tx.send(res);
                }
            }
            DbEvent::DeleteNode { node_id, response } => {
                let res = operation::delete_node(&self.client, node_id).await;
                self.report_error_if_any(&res).await;
                let _ = response.send(res);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::{Node, NodePosition, NodeType};
    use surrealdb::engine::local::Mem;
    use surrealdb::sql::Thing;
    use tokio::sync::oneshot;

    async fn setup_test_context() -> (
        Arc<Surreal<Db>>,
        mpsc::Sender<DbEvent>,
        mpsc::Receiver<String>,
        tokio::task::JoinHandle<()>,
    ) {
        let client = Surreal::new::<Mem>(()).await.unwrap();
        client.use_ns("test").use_db("test").await.unwrap();
        let client = Arc::new(client);

        let (err_tx, err_rx) = mpsc::channel(100);
        let (sender, receiver) = mpsc::channel(100);

        // Use a short interval for testing
        let event_loop = EventLoop::new(
            client.clone(),
            receiver,
            err_tx,
            Duration::from_millis(10),
            50,
            1000,
        );

        let handle = tokio::spawn(async move {
            event_loop.run().await;
        });

        (client, sender, err_rx, handle)
    }

    fn mock_node() -> Node {
        Node {
            id: None,
            position: NodePosition { x: 0.0, y: 0.0 },
            type_: NodeType::Chat {
                value: serde_json::json!({"text": "test"}),
            },
        }
    }

    #[tokio::test]
    async fn test_event_loop_add_node() {
        let (client, sender, _err_rx, _handle) = setup_test_context().await;
        let canvas_id = Thing::from(("canvas", "test"));
        let (tx, rx) = oneshot::channel();

        sender
            .send(DbEvent::AddNode {
                canvas_id: canvas_id.clone(),
                node: mock_node(),
                response: tx,
            })
            .await
            .unwrap();

        let res = rx.await.unwrap();
        assert!(res.is_ok());
        let node = res.unwrap();
        assert!(node.id.is_some());

        // Verify DB state
        let nodes: Vec<Node> = client.select("node").await.unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].id, node.id);
    }

    #[tokio::test]
    async fn test_event_loop_add_derived_node() {
        let (client, sender, _err_rx, _handle) = setup_test_context().await;
        let canvas_id = Thing::from(("canvas", "test"));

        // Add root node
        let (tx1, rx1) = oneshot::channel();
        sender
            .send(DbEvent::AddNode {
                canvas_id: canvas_id.clone(),
                node: mock_node(),
                response: tx1,
            })
            .await
            .unwrap();
        let root = rx1.await.unwrap().unwrap();

        // Add derived node
        let (tx2, rx2) = oneshot::channel();
        sender
            .send(DbEvent::AddDerivedNode {
                canvas_id: canvas_id.clone(),
                from: root.id.clone().unwrap(),
                to: mock_node(),
                response: tx2,
            })
            .await
            .unwrap();

        let derived = rx2.await.unwrap().unwrap();
        assert!(derived.id.is_some());

        // Verify relation in DB using load_canvas
        let (_, derives, _) = operation::load_canvas(&client, canvas_id).await.unwrap();
        assert_eq!(derives.len(), 1);
        assert_eq!(derives[0].from, root.id.unwrap());
        assert_eq!(derives[0].to, derived.id.unwrap());
    }

    #[tokio::test]
    async fn test_event_loop_add_sequenced_node() {
        let (client, sender, _err_rx, _handle) = setup_test_context().await;
        let canvas_id = Thing::from(("canvas", "test"));

        // Add root node
        let (tx1, rx1) = oneshot::channel();
        sender
            .send(DbEvent::AddNode {
                canvas_id: canvas_id.clone(),
                node: mock_node(),
                response: tx1,
            })
            .await
            .unwrap();
        let root = rx1.await.unwrap().unwrap();

        // Add sequenced node
        let (tx2, rx2) = oneshot::channel();
        sender
            .send(DbEvent::AddSequencedNode {
                canvas_id: canvas_id.clone(),
                from: root.id.clone().unwrap(),
                to: mock_node(),
                response: tx2,
            })
            .await
            .unwrap();

        let sequenced = rx2.await.unwrap().unwrap();
        assert!(sequenced.id.is_some());

        // Verify relation in DB using load_canvas
        let (_, _, sequences) = operation::load_canvas(&client, canvas_id).await.unwrap();
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].from, root.id.unwrap());
        assert_eq!(sequences[0].to, sequenced.id.unwrap());
    }

    #[tokio::test]
    async fn test_event_loop_move_node_coalescing() {
        let (client, sender, _err_rx, _handle) = setup_test_context().await;
        let canvas_id = Thing::from(("canvas", "test"));

        // Add node
        let (tx, rx) = oneshot::channel();
        sender
            .send(DbEvent::AddNode {
                canvas_id,
                node: mock_node(),
                response: tx,
            })
            .await
            .unwrap();
        let node = rx.await.unwrap().unwrap();
        let node_id = node.id.unwrap();

        // Send multiple moves quickly.
        let (_tx_q, rx_q) = mpsc::channel(100);
        let (tx_e, _rx_e) = mpsc::channel(100);
        let mut el = EventLoop::new(client.clone(), rx_q, tx_e, Duration::from_secs(1), 50, 1000);

        el.queue.push_back(DbEvent::MoveNode {
            node_id: node_id.clone(),
            x: 10.0,
            y: 10.0,
            response: None,
        });
        el.queue.push_back(DbEvent::MoveNode {
            node_id: node_id.clone(),
            x: 20.0,
            y: 20.0,
            response: None,
        });
        el.queue.push_back(DbEvent::MoveNode {
            node_id: node_id.clone(),
            x: 30.0,
            y: 30.0,
            response: None,
        });

        assert_eq!(el.queue.len(), 3);
        el.optimize_queue();
        assert_eq!(el.queue.len(), 1);

        if let Some(DbEvent::MoveNode { x, y, .. }) = el.queue.pop_front() {
            assert_eq!(x, 30.0);
            assert_eq!(y, 30.0);
        } else {
            panic!("Expected MoveNode event");
        }
    }

    #[tokio::test]
    async fn test_event_loop_delete_node() {
        let (client, sender, _err_rx, _handle) = setup_test_context().await;
        let canvas_id = Thing::from(("canvas", "test"));

        // Add node
        let (tx1, rx1) = oneshot::channel();
        sender
            .send(DbEvent::AddNode {
                canvas_id,
                node: mock_node(),
                response: tx1,
            })
            .await
            .unwrap();
        let node = rx1.await.unwrap().unwrap();

        // Delete node
        let (tx2, rx2) = oneshot::channel();
        sender
            .send(DbEvent::DeleteNode {
                node_id: node.id.clone().unwrap(),
                response: tx2,
            })
            .await
            .unwrap();
        rx2.await.unwrap().unwrap();

        // Verify DB empty
        let nodes: Vec<Node> = client.select("node").await.unwrap();
        assert_eq!(nodes.len(), 0);
    }

    #[tokio::test]
    async fn test_macro_event_flow() {
        let (_client, sender, _err_rx, _handle) = setup_test_context().await;
        let canvas_id = Thing::from(("canvas", "macro_test"));

        // 1. Add root
        let (tx, rx) = oneshot::channel();
        sender
            .send(DbEvent::AddNode {
                canvas_id: canvas_id.clone(),
                node: mock_node(),
                response: tx,
            })
            .await
            .unwrap();
        let root = rx.await.unwrap().unwrap();
        let root_id = root.id.as_ref().unwrap();

        // 2. Add derived and sequenced nodes
        let (tx1, rx1) = oneshot::channel();
        sender
            .send(DbEvent::AddDerivedNode {
                canvas_id: canvas_id.clone(),
                from: root_id.clone(),
                to: mock_node(),
                response: tx1,
            })
            .await
            .unwrap();

        let (tx2, rx2) = oneshot::channel();
        sender
            .send(DbEvent::AddSequencedNode {
                canvas_id: canvas_id.clone(),
                from: root_id.clone(),
                to: mock_node(),
                response: tx2,
            })
            .await
            .unwrap();

        let d1 = rx1.await.unwrap().unwrap();
        let s1 = rx2.await.unwrap().unwrap();

        // 3. Move them
        let (tx3, rx3) = oneshot::channel();
        sender
            .send(DbEvent::MoveNode {
                node_id: d1.id.clone().unwrap(),
                x: 100.0,
                y: 100.0,
                response: Some(tx3),
            })
            .await
            .unwrap();
        rx3.await.unwrap().unwrap();

        // 4. Delete leaf
        let (tx4, rx4) = oneshot::channel();
        sender
            .send(DbEvent::DeleteNode {
                node_id: s1.id.unwrap(),
                response: tx4,
            })
            .await
            .unwrap();
        rx4.await.unwrap().unwrap();

        // 5. Attempt to delete root (should fail because d1 exists)
        let (tx5, rx5) = oneshot::channel();
        sender
            .send(DbEvent::DeleteNode {
                node_id: root_id.clone(),
                response: tx5,
            })
            .await
            .unwrap();
        let res = rx5.await.unwrap();
        assert!(res.is_err());
    }
}
