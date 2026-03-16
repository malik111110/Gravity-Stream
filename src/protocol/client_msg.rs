use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
    Chat { room_id: Uuid, text: String },
    JoinRoom { room_id: Uuid },
    LeaveRoom { room_id: Uuid },
    Ping,
}
