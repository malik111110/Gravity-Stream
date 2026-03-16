pub mod routes;
pub mod task;

use bytes::Bytes;
use hyper::{Request, Response, StatusCode};
use hyper::body::Incoming;
use http_body_util::{Full, BodyExt, combinators::BoxBody};
use fastwebsockets::upgrade::{upgrade, is_upgrade_request};
use crate::connection::handle_connection;
use std::sync::Arc;
use crate::AppState;
use tokio::sync::mpsc;
use uuid::Uuid;
use crate::protocol::ClientMsg;

pub async fn handle(
    mut req: Request<Incoming>,
    state: Arc<AppState>,
    router_tx: mpsc::Sender<(Uuid, ClientMsg)>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    if is_upgrade_request(&req) {
        // Simple auth check for ShopyLink ecosystem
        let token = req.headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "));

        if let Some(token) = token {
            if let Err(e) = state.auth_service.verify_token(token) {
                tracing::warn!("Auth failed: {}", e);
                return Ok(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(Full::new(Bytes::from("Unauthorized")).map_err(|_| unreachable!()).boxed())
                    .unwrap());
            }
        } else {
            // For now, let's allow it but log a warning (in production we'd enforce)
            tracing::warn!("No auth token provided for upgrade");
        }

        let (response, fut) = match upgrade(&mut req) {
            Ok(res) => res,
            Err(_) => {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Full::new(Bytes::from("Upgrade failed")).map_err(|_| unreachable!()).boxed())
                    .unwrap());
            }
        };

        tokio::spawn(async move {
            match fut.await {
                Ok(upgraded) => {
                    handle_connection(upgraded, state, router_tx).await;
                }
                Err(e) => tracing::error!("Upgrade failed: {:?}", e),
            }
        });

        return Ok(response.map(|b| b.map_err(|_| unreachable!()).boxed()));
    }

    Ok(Response::new(
        Full::from(Bytes::from_static(b"ShopyLink WebSocket Server"))
            .map_err(|_| unreachable!())
            .boxed()
    ))
}
