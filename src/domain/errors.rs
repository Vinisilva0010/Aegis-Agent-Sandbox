use thiserror::Error;

/// Centralizamos todos os erros do sistema aqui.
/// Cada erro carrega contexto para devolvermos um feedback rico ao LLM via MCP.
#[derive(Error, Debug)]
pub enum AegisError {
    #[error("Violacao de Politica (Zero Trust): A acao '{action}' no alvo '{target}' foi bloqueada. Alvos permitidos nesta tela: {allowed:?}. Recalcule sua estrategia.")]
    PolicyViolation {
        action: String,
        target: String,
        allowed: Vec<String>,
    },

    #[error("Dominio Nao Autorizado: O agente tentou navegar para '{0}', mas este dominio nao esta na whitelist do Aegis.")]
    UntrustedDomain(String),

    #[error("Falha no Parse da Intencao: O payload enviado e invalido ou incompleto. Certifique-se de seguir o schema da Tool.")]
    InvalidPayload,

    #[error("Timeout da Camada C: A execucao no Chromium demorou mais que o esperado ({0}ms). Abortando para preservar recursos.")]
    ExecutionTimeout(u64),
}