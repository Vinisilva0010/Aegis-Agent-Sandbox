# Aegis-Agent-Sandbox

**Middleware de Segurança para Agentes Autônomos**  
**Autor:** Vinicius Pontual  
**Status:** Em Revisão (Draft)  
**Data:** Março, 2026  
**Tecnologias Core:** Rust, Tokio, Axum, Prism-MCP, Browser-Use (CDP)

## 1. Contexto e Objetivo

Modelos de fronteira (LLMs) são probabilísticos e propensos a alucinações. Dar acesso irrestrito ao Chrome DevTools Protocol (CDP) para navegação web ou ações no sistema representa risco crítico de segurança.

O **Aegis-Agent-Sandbox** atua como proxy determinístico de segurança, escrito em Rust para performance e *memory safety*. Ele intercepta intenções do LLM via Model Context Protocol (MCP), valida contra políticas *Zero Trust*, executa ações aprovadas em navegador *headless* e retorna o DOM comprimido.

## 2. Escopo

### Goals (O que o sistema FAZ)
- Recebe payloads de intenção via MCP (ex: click, type, Maps).
- Valida ações contra `policies.yaml` (Domain/Action Whitelisting).
- Executa em Chromium *headless* isolado via CDP.
- Retorna erros semânticos claros para o LLM replanejar.
- Coleta logs (Aprovadas vs. Bloqueadas) para análise.

### Non-Goals (O que o sistema NÃO FAZ)
- Não hospeda/executa LLMs (apenas middleware).
- Não faz *scraping* em massa (foco em interações pontuais).
- Sem UI complexa; infraestrutura *headless*.

## 3. Arquitetura

Sistema em **três camadas desacopladas**:

| Camada | Responsabilidade | Tecnologias |
|--------|------------------|-------------|
| **Transport Layer** | MCP Server (HTTP/WebSockets) | Axum |
| **Policy Engine** | Validação de Guardrails | Rust puro + Serde |
| **Execution Env** | Chromium lifecycle + DOM injection | CDP Worker |

## 4. Fluxo de Dados

LLM → MCP Payload → Aegis Ingest → Policy Check → CDP Exec → DOM Snapshot → LLM

text

**Exemplo real:** LLM quer comprar item → `{"tool": "web_action", "params": {"action": "click", "selector": "#buy", "url": "amazon.com"}}`

1. **Ingestão:** Axum → Rust structs (serde).
2. **Validação:** `amazon.com` whitelisted? `click` permitido?
3. **Execução:** CDP Worker clica `#buy`.
4. **Observability:** Extrai *Accessibility Tree* compactada.
5. **Retorno:** Sucesso + novo estado para LLM.

## 5. Segurança

### Prompt Injection
Validação rigorosa de tipos em Rust bloqueia payloads maliciosos na deserialização (ex: `<script>alert(1)</script>` em campo URL).

### Timeouts
`tokio::time::timeout(5s)` em todas requisições CDP. Sites lentos = abort + erro pro agente.

### Logs
Ações aprovadas/bloqueadas auditadas para otimização posterior.

---

*Repositório em desenvolvimento. Contribuições via PR bem-vindas após revisão.*




graph TD
    LLM[Modelo IA Claude ou GPT] -->|Intencao de Acao via MCP| AEGIS_SERVER

    subgraph Aegis Sandbox Rust
        AEGIS_SERVER[Axum Server Ingestao] -->|Payload JSON| VALIDATION
        VALIDATION[Policy Engine Zero Trust] -->|Checa Whitelist| RULES[Manifesto de Regras]
    end

    VALIDATION -->|Acao Bloqueada| ERROR_SEMANTICO[Erro Semantico]
    ERROR_SEMANTICO -->|Recalcular Rota| LLM

    VALIDATION -->|Acao Aprovada| CDP_WORKER[Chromium CDP Worker]

    subgraph Mundo Real
        CDP_WORKER -->|Injeta Evento| BROWSER[Navegador Headless]
        BROWSER -->|Retorna DOM Atualizado| CDP_WORKER
    end

    CDP_WORKER -->|Sucesso| LLM
