use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::protocol::ServerMsg;
use crate::sessions::Session;

#[derive(Default)]
pub struct SessionManager {
    sessions: RwLock<HashMap<Uuid, Session>>,
    rooms: RwLock<HashMap<Uuid, HashSet<Uuid>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn add(&self, session: Session) {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id, session);
    }

    pub async fn remove(&self, id: &Uuid) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(id);

        let mut rooms = self.rooms.write().await;
        for session_ids in rooms.values_mut() {
            session_ids.remove(id);
        }
    }

    pub async fn join_room(&self, session_id: Uuid, room_id: Uuid) {
        let mut rooms = self.rooms.write().await;
        rooms.entry(room_id).or_default().insert(session_id);
    }

    pub async fn leave_room(&self, session_id: Uuid, room_id: Uuid) {
        let mut rooms = self.rooms.write().await;
        if let Some(session_ids) = rooms.get_mut(&room_id) {
            session_ids.remove(&session_id);
        }
    }

    pub async fn send_to(&self, id: &Uuid, msg: ServerMsg) {
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(id) {
            let _ = session.tx.send(msg).await;
        }
    }

    pub async fn broadcast(&self, msg: ServerMsg) {
        let sessions = self.sessions.read().await;
        for session in sessions.values() {
            let _ = session.tx.send(msg.clone()).await;
        }
    }

    pub async fn broadcast_to_room(&self, room_id: &Uuid, msg: ServerMsg) {
        let rooms = self.rooms.read().await;
        if let Some(session_ids) = rooms.get(room_id) {
            let sessions = self.sessions.read().await;
            for session_id in session_ids {
                if let Some(session) = sessions.get(session_id) {
                    let _ = session.tx.send(msg.clone()).await;
                }
            }
        }
    }
}
