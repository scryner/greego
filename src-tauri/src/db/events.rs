use crate::db::schema::Node;
use surrealdb::sql::Thing;
use tokio::sync::oneshot;

pub enum DbEvent {
    AddNode {
        canvas_id: Thing,
        node: Node,
        response: oneshot::Sender<Result<Node, surrealdb::Error>>,
    },
    AddDerivedNode {
        canvas_id: Thing,
        from: Thing,
        to: Node,
        response: oneshot::Sender<Result<Node, surrealdb::Error>>,
    },
    AddSequencedNode {
        canvas_id: Thing,
        from: Thing,
        to: Node,
        response: oneshot::Sender<Result<Node, surrealdb::Error>>,
    },
    MoveNode {
        node_id: Thing,
        x: f64,
        y: f64,
        response: Option<oneshot::Sender<Result<Node, surrealdb::Error>>>,
    },
    DeleteNode {
        node_id: Thing,
        response: oneshot::Sender<Result<(), surrealdb::Error>>,
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
