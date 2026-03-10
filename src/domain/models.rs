use serde::{Deserialize, Serialize};

/// Define estritamente quais ações o LLM tem permissão de tentar.
/// Se o modelo alucinar e enviar um "action": "delete_database",
/// o `serde` vai recusar o parse imediatamente. Não chega nem na lógica de validação.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Navigate,
    Click,
    Type,
    ExtractDom,
    // Se no futuro você quiser deixar ele dar scroll, a gente adiciona aqui.
}

/// O payload exato que esperamos receber do servidor MCP quando o LLM invoca a Tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPayload {
    pub action: ActionType,
    // Usamos Option porque uma ação de "Navigate" precisa de URL, 
    // mas um "Click" precisa de um selector. O Rust obriga a gente a tratar o que é nulo.
    pub target_selector: Option<String>,
    pub value: Option<String>, // Usado para a ação 'Type' (ex: texto a ser digitado)
    pub target_url: Option<String>,
}

/// A nossa Política Zero Trust (A regra do jogo).
/// Isso aqui vai ser carregado do seu policies.yaml depois.
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    pub allowed_domains: Vec<String>,
    pub allowed_selectors: Vec<String>,
}