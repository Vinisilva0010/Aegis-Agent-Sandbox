use axum::{
    routing::post,
    Router,
    Json,
    extract::State,
    response::IntoResponse,
    http::StatusCode,
};
use tokio::sync::{mpsc, oneshot};
use std::sync::Arc;

use crate::domain::models::{ActionPayload, SecurityPolicy};
use crate::application::actor_worker::WorkerCommand;
use crate::domain::errors::AegisError;

/// O Estado Compartilhado do nosso servidor Web.
/// Tudo aqui precisa ser thread-safe (por isso usamos mpsc::Sender e Arc).
#[derive(Clone)]
pub struct AppState {
    pub tx: mpsc::Sender<WorkerCommand>,
    pub policy: Arc<SecurityPolicy>,
}

/// Cria o roteador do Axum e injeta o nosso estado nele.
pub fn app_router(state: AppState) -> Router {
    Router::new()
        // Define a rota exata que o LLM vai chamar via POST
        .route("/mcp/action", post(handle_action))
        .with_state(state)
}

/// O Handler HTTP. Aqui o Axum já converteu o JSON pra `ActionPayload` magicamente.
/// Se o LLM mandar um JSON zoado, o Axum devolve um erro 400 antes mesmo de rodar isso aqui.
async fn handle_action(
    State(state): State<AppState>,
    Json(payload): Json<ActionPayload>,
) -> impl IntoResponse {
    tracing::info!("Recebida requisicao HTTP via MCP: {:?}", payload.action);

    // 1. Cria o walkie-talkie (oneshot) exclusivo para essa requisição HTTP
    let (reply_tx, reply_rx) = oneshot::channel();

    // 2. Monta o pacote pro Worker
    let cmd = WorkerCommand {
        payload,
        policy: (*state.policy).clone(),
        reply_to: reply_tx,
    };

    // 3. Joga na fila (mpsc). Se a fila estiver cheia (mais de 100 reqs), o .await
    // faz o Axum esperar educadamente sem gastar CPU (Backpressure puro!).
    if state.tx.send(cmd).await.is_err() {
        tracing::error!("Falha crítica: Canal do Worker fechado!");
        return (StatusCode::INTERNAL_SERVER_ERROR, "Erro interno do Aegis".to_string());
    }

    // 4. Espera a resposta do Worker e traduz o nosso AegisError para um HTTP Status Code correto.
    match reply_rx.await {
        Ok(Ok(_)) => {
            let success_msg = r#"{"status": "success", "message": "Acao executada com sucesso no DOM."}"#;
            (StatusCode::OK, success_msg.to_string())
        },
        Ok(Err(AegisError::PolicyViolation { action, target, allowed })) => {
            // Esse é o payload que vai forçar o Claude a pensar numa rota alternativa!
            let error_json = format!(
                r#"{{"status": "blocked", "error": "Zero Trust Violation", "action": "{}", "target": "{}", "allowed_targets": {:?}}}"#,
                action, target, allowed
            );
            (StatusCode::FORBIDDEN, error_json)
        },
        Ok(Err(AegisError::UntrustedDomain(d))) => {
            let error_json = format!(r#"{{"status": "blocked", "error": "Untrusted Domain", "domain": "{}"}}"#, d);
            (StatusCode::FORBIDDEN, error_json)
        },
        Ok(Err(e)) => (StatusCode::BAD_REQUEST, format!(r#"{{"error": "{}"}}"#, e)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Worker falhou ao responder".to_string()),
    }
}