use tokio::sync::{mpsc, oneshot};
use tracing::{info, warn};
use crate::domain::models::{ActionPayload, SecurityPolicy};
use crate::domain::errors::AegisError;
use crate::application::policy_engine::PolicyEngine;

/// O "envelope" que trafega no nosso canal de comunicação.
/// Ele carrega a intenção do LLM, as regras, e um "canal de retorno" (oneshot)
/// para que o Worker possa responder diretamente para a requisição HTTP que chamou.
#[derive(Debug)]
pub struct WorkerCommand {
    pub payload: ActionPayload,
    pub policy: SecurityPolicy,
    pub reply_to: oneshot::Sender<Result<(), AegisError>>,
}

/// O Worker do Aegis. Ele é o dono exclusivo do Chromium (na próxima fase).
/// Nenhuma outra parte do código tem permissão para executar ações além dele.
pub struct AegisWorker {
    // O Worker fica escutando a ponta receptora do canal (Receiver)
    receiver: mpsc::Receiver<WorkerCommand>,
}

impl AegisWorker {
    pub fn new(receiver: mpsc::Receiver<WorkerCommand>) -> Self {
        Self { receiver }
    }

    /// O loop infinito de processamento. 
    /// Ele roda em uma thread isolada do Tokio, processando uma requisição do Claude 4.6 por vez.
    pub async fn run(mut self) {
        info!("Aegis Worker (Actor) iniciado e aguardando intenções...");

        // O while let fica bloqueado (sem consumir CPU) até chegar uma nova mensagem no canal
        while let Some(cmd) = self.receiver.recv().await {
            info!("Worker retirou uma intenção da fila. Processando...");

            // 1. Passa pelo cérebro (Camada B) antes de chegar perto do browser
            let validation_result = PolicyEngine::validate_action(&cmd.payload, &cmd.policy);
            
            if validation_result.is_err() {
                warn!("Ação bloqueada pelo motor de políticas. Rejeitando antes da execução.");
                // Devolve o erro semântico direto pro Axum/Claude e vai pra próxima fila
                let _ = cmd.reply_to.send(validation_result);
                continue;
            }

            // 2. Execução Segura (Camada C)
            // Aqui é onde o browser-use/chromiumoxide vai entrar depois.
            // Por enquanto, simulamos que fomos até o DOM, clicamos e esperamos o carregamento.
            info!("Motor aprovou. Simulando injeção CDP no Chromium (500ms)...");
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            // 3. Sucesso! Devolvemos a liberação pelo canal de retorno.
            info!("Ação executada com sucesso no DOM simulado.");
            let _ = cmd.reply_to.send(Ok(()));
        }
        
        warn!("Canal mpsc foi fechado. Aegis Worker desligando.");
    }
}