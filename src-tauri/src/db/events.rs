use crate::db::schema::{Canvas, Node};
use surrealdb::sql::Thing;
use tokio::sync::oneshot;

pub enum DbEvent {
    AddCanvas {
        canvas: Canvas,
        response: oneshot::Sender<anyhow::Result<Canvas>>,
    },
    ListCanvas {
        limit: usize,
        offset: usize,
        response: oneshot::Sender<anyhow::Result<Vec<Canvas>>>,
    },
    AddNode {
        canvas_id: Thing,
        node: Node,
        response: oneshot::Sender<anyhow::Result<Node>>,
    },
    AddDerivedNode {
        canvas_id: Thing,
        from: Thing,
        to: Node,
        response: oneshot::Sender<anyhow::Result<Node>>,
    },
    AddSequencedNode {
        canvas_id: Thing,
        from: Thing,
        to: Node,
        response: oneshot::Sender<anyhow::Result<Node>>,
    },
    MoveNode {
        node_id: Thing,
        x: f64,
        y: f64,
        response: Option<oneshot::Sender<anyhow::Result<Node>>>,
    },
    DeleteNode {
        node_id: Thing,
        response: oneshot::Sender<anyhow::Result<()>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventPriority {
    Immediate,
    Coalescable,
}

impl DbEvent {
    pub fn priority(&self) -> EventPriority {
        match self {
            DbEvent::MoveNode { .. } => EventPriority::Coalescable,
            _ => EventPriority::Immediate,
        }
    }
}
