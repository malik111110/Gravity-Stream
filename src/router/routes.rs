use std::sync::Arc;
use uuid::Uuid;
use crate::AppState;
use crate::protocol::{ClientMsg, ServerMsg};

pub async fn handle_message(session_id: Uuid, msg: ClientMsg, state: Arc<AppState>) {
    match msg {
        ClientMsg::Ping => {
            state.session_manager.send_to(&session_id, ServerMsg::Pong).await;
        }
        ClientMsg::Chat { room_id, text } => {
            tracing::info!("Chat from {} in room {}: {}", session_id, room_id, text);
            state.session_manager.broadcast_to_room(&room_id, ServerMsg::ChatMessage {
                from: session_id,
                room_id,
                text,
            }).await;
        }
        ClientMsg::JoinRoom { room_id } => {
            tracing::info!("Session {} joining room {}", session_id, room_id);
            state.session_manager.join_room(session_id, room_id).await;
            
            state.session_manager.broadcast_to_room(&room_id, ServerMsg::ChatMessage {
                from: session_id,
                room_id,
                text: format!("User {} joined the room", session_id),
            }).await;
        }
        ClientMsg::LeaveRoom { room_id } => {
            tracing::info!("Session {} leaving room {}", session_id, room_id);
            state.session_manager.leave_room(session_id, room_id).await;

            state.session_manager.broadcast_to_room(&room_id, ServerMsg::ChatMessage {
                from: session_id,
                room_id,
                text: format!("User {} left the room", session_id),
            }).await;
        }
    }
}
