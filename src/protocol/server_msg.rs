use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMsg {
    ChatMessage {
        from: Uuid,
        room_id: Uuid,
        text: String,
    },
    Chunk {
        stream_id: Uuid,
        text: String,
    },
    StreamEnd {
        stream_id: Uuid,
    },
    Pong,
    Error {
        code: u16,
        message: String,
    },
}
