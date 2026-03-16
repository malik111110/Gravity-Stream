use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;
use fastwebsockets::{OpCode, WebSocket};
use hyper_util::rt::TokioIo;
use hyper::upgrade::Upgraded;
use crate::AppState;
use crate::protocol::{ClientMsg, ServerMsg};
use crate::sessions::Session;

pub async fn handle_connection(
    mut ws: WebSocket<TokioIo<Upgraded>>,
    state: Arc<AppState>,
    router_tx: mpsc::Sender<(Uuid, ClientMsg)>,
) {
    ws.set_writev(false);

    let session_id = Uuid::new_v4();
    let (tx, mut rx) = mpsc::channel::<ServerMsg>(32);

    let session = Session::new(session_id, tx);
    state.session_manager.add(session).await;

    loop {
        tokio::select! {
            frame_res = ws.read_frame() => {
                let frame = match frame_res {
                    Ok(f) => f,
                    Err(e) => {
                        tracing::error!("Read error: {:?}", e);
                        break;
                    }
                };

                match frame.opcode {
                    OpCode::Close => break,
                    OpCode::Text | OpCode::Binary => {
                        if let Ok(msg) = serde_json::from_slice::<ClientMsg>(&frame.payload) {
                            let _ = router_tx.send((session_id, msg)).await;
                        }
                    }
                    _ => {}
                }
            }
            Some(msg) = rx.recv() => {
                if let Ok(payload) = serde_json::to_vec(&msg) {
                    if let Err(e) = ws.write_frame(fastwebsockets::Frame::binary(payload.into())).await {
                        tracing::error!("Write error: {:?}", e);
                        break;
                    }
                }
            }
        }
    }

    state.session_manager.remove(&session_id).await;
}
