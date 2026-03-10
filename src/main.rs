mod domain;
mod application;
mod interfaces;

use std::sync::Arc;
use tokio::sync::mpsc;
use application::actor_worker::{AegisWorker, WorkerCommand};
use domain::models::SecurityPolicy;
use interfaces::mcp_server::{app_router, AppState};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("Iniciando Aegis Core v2026 - Booting HTTP Server...");

    // 1. Cria a fila de comunicação (Backpressure de 100 requisições)
    let (tx, rx) = mpsc::channel::<WorkerCommand>(100);

    // 2. Sobe o Worker do Chromium no background
    let worker = AegisWorker::new(rx);
    tokio::spawn(async move {
        worker.run().await;
    });

    // 3. Define as regras de segurança na memória
    let policy = Arc::new(SecurityPolicy {
        allowed_domains: vec!["amazon.com".to_string(), "github.com".to_string()],
        allowed_selectors: vec!["#buy-btn".to_string(), "#search-bar".to_string()],
    });

    // 4. Monta o estado da Aplicação
    let app_state = AppState { tx, policy };

    // 5. Configura e sobe o Servidor Axum
    let app = app_router(app_state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    
    tracing::info!("🚀 Aegis API Gateway rodando na porta 3000. Aguardando instrucoes MCP...");
    
    axum::serve(listener, app).await.unwrap();
}