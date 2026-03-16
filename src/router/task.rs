use crate::router::routes;

use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;
use crate::protocol::ClientMsg;
use crate::AppState;

pub async fn router_task(
    mut rx: mpsc::Receiver<(Uuid, ClientMsg)>,
    state: Arc<AppState>,
) {
    while let Some((session_id, msg)) = rx.recv().await {
        routes::handle_message(session_id, msg, state.clone()).await;
    }
}
