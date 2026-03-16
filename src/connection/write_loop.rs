use fastwebsockets::{Frame, OpCode, WebSocket};
use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;
use tokio::sync::mpsc;
use crate::protocol::ServerMsg;

pub async fn write_loop(
    mut ws: WebSocket<TokioIo<Upgraded>>,
    mut rx: mpsc::Receiver<ServerMsg>,
) -> anyhow::Result<()> {
    while let Some(msg) = rx.recv().await {
        let payload = serde_json::to_vec(&msg)?;
        let frame = Frame::text(payload.into());
        ws.write_frame(frame).await.map_err(|e| anyhow::anyhow!("Write error: {:?}", e))?;
    }
    Ok(())
}
