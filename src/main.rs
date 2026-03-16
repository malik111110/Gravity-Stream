mod connection;
mod protocol;
mod router;
mod services;
mod sessions;

use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::sessions::SessionManager;
use crate::services::auth_service::AuthService;
use crate::services::agent_service::AgentService;
use crate::services::metrics_service::MetricsService;
use crate::router::task::router_task;
use crate::router::handle;

pub struct AppState {
    pub session_manager: SessionManager,
    pub auth_service: AuthService,
    pub agent_service: AgentService,
    pub metrics_service: MetricsService,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    // In production, load from env
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "shopy_secret_key_123".to_string());
    
    let session_manager = SessionManager::new();
    let auth_service = AuthService::new(jwt_secret.as_bytes());
    let agent_service = AgentService::new();
    let metrics_service = MetricsService::new();
    
    let state = Arc::new(AppState { 
        session_manager,
        auth_service,
        agent_service,
        metrics_service,
    });

    let (router_tx, router_rx) = mpsc::channel(100);
    
    // Start central router task
    let router_state = state.clone();
    tokio::spawn(async move {
        router_task(router_rx, router_state).await;
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let state = state.clone();
        let router_tx = router_tx.clone();

        tokio::spawn(async move {
            let service = service_fn(move |req| {
                handle(req, state.clone(), router_tx.clone())
            });

            if let Err(err) = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, service)
                .with_upgrades()
                .await
            {
                tracing::error!("Error serving connection: {:?}", err);
            }
        });
    }
}
