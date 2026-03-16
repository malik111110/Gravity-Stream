use tokio::sync::mpsc;
use uuid::Uuid;
use crate::protocol::ServerMsg;

#[derive(Debug, Clone)]
pub struct Session {
    pub id: Uuid,
    pub tx: mpsc::Sender<ServerMsg>,
}

impl Session {
    pub fn new(id: Uuid, tx: mpsc::Sender<ServerMsg>) -> Self {
        Self { id, tx }
    }
}
