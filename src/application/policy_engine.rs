use crate::domain::errors::AegisError;
use crate::domain::models::{ActionPayload, ActionType, SecurityPolicy};
use tracing::{info, warn};

/// O motor de regras do Aegis. 
/// Ele recebe o estado desejado (payload) e cruza com a política de segurança (Zero Trust).
pub struct PolicyEngine;

impl PolicyEngine {
    /// O método principal de validação. Retorna () se estiver tudo ok, ou um AegisError detalhado.
    pub fn validate_action(payload: &ActionPayload, policy: &SecurityPolicy) -> Result<(), AegisError> {
        info!("Avaliando intencao do LLM: {:?}", payload.action);

        match payload.action {
            ActionType::Navigate => {
                let target_url = payload.target_url.as_ref().ok_or(AegisError::InvalidPayload)?;
                
                // Checagem de Domínio: O LLM só vai onde a gente deixa.
                let is_allowed = policy.allowed_domains.iter().any(|domain| target_url.contains(domain));
                if !is_allowed {
                    warn!("Bloqueio critico: LLM tentou navegar para dominio nao confiavel: {}", target_url);
                    return Err(AegisError::UntrustedDomain(target_url.clone()));
                }
            }
            ActionType::Click | ActionType::Type => {
                let selector = payload.target_selector.as_ref().ok_or(AegisError::InvalidPayload)?;
                
                // Checagem Semântica: O elemento alvo está na whitelist?
                if !policy.allowed_selectors.contains(selector) {
                    warn!("Bloqueio de DOM: LLM tentou interagir com seletor nao autorizado: {}", selector);
                    return Err(AegisError::PolicyViolation {
                        action: format!("{:?}", payload.action),
                        target: selector.clone(),
                        allowed: policy.allowed_selectors.clone(),
                    });
                }
            }
            ActionType::ExtractDom => {
                // Extração de DOM geralmente é segura, mas podemos adicionar limites no futuro.
                info!("Acao de leitura de DOM autorizada.");
            }
        }

        info!("Acao validada com sucesso pelo Aegis.");
        Ok(())
    }
}