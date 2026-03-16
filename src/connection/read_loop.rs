use fastwebsockets::{OpCode, WebSocket};
use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;
use tokio::sync::mpsc;
use uuid::Uuid;
use crate::protocol::ClientMsg;

pub async fn read_loop(
    mut ws: WebSocket<TokioIo<Upgraded>>,
    router_tx: mpsc::Sender<(Uuid, ClientMsg)>,
    session_id: Uuid,
) -> anyhow::Result<()> {
    loop {
        let frame = ws.read_frame().await.map_err(|e| anyhow::anyhow!("Read error: {:?}", e))?;

        match frame.opcode {
            OpCode::Close => break,
            OpCode::Text | OpCode::Binary => {
                if let Ok(msg) = serde_json::from_slice::<ClientMsg>(&frame.payload) {
                    if router_tx.send((session_id, msg)).await.is_err() {
                        break;
                    }
                }
            }
            OpCode::Ping => {
                // Ping handling if necessary
            }
            _ => {}
        }
    }
    Ok(())
}
