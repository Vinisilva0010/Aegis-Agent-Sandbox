Aegis-Agent-Sandbox (Middleware de Segurança para Agentes Autônomos)
Autor: Vinicius Pontual
Status: Em Revisão (Draft)
Data: Março, 2026
Tecnologias Core: Rust, Tokio, Axum, Prism-MCP, Browser-Use (CDP)

1. Contexto e Objetivo
Atualmente, modelos de fronteira (LLMs) são probabilísticos e propensos a alucinações. Dar a um LLM acesso irrestrito ao Chrome DevTools Protocol (CDP) para navegar na web ou executar ações no sistema do usuário é um risco de segurança crítico.

O objetivo do Aegis-Agent-Sandbox é atuar como um proxy determinístico de segurança (escrito em Rust para performance e memory safety). Ele intercepta intenções de ação do LLM via Model Context Protocol (MCP), valida essas intenções contra políticas estritas (Zero Trust) e, somente se aprovadas, executa a ação no navegador web (headless), retornando o novo estado do DOM comprimido para o modelo.

2. Escopo (Goals & Non-Goals)
Goals (O que o sistema FAZ):

Receber payloads de intenção de ação (ex: click, type, Maps) via MCP.

Validar as ações contra um arquivo de manifesto policies.yaml (Domain Whitelisting, Action Whitelisting).

Executar as ações em um ambiente Chromium Headless isolado via CDP.

Retornar erros semânticos claros para o LLM caso a ação seja negada, forçando o modelo a replanejar.

Coletar logs brutos de ações (Aprovadas vs. Bloqueadas) para posterior análise de dados e otimização.

Non-Goals (O que o sistema NÃO FAZ):

Não hospeda nem executa o LLM em si (somos apenas o middleware).

Não é um scraper de dados em massa (foco é em interações agentísticas pontuais).

Não possui interface gráfica (UI) complexa para usuários finais; é uma ferramenta de infraestrutura "headless".

3. Arquitetura do Sistema
Para visualizarmos como os componentes se integram, observe o fluxo da arquitetura:

O sistema é dividido em três camadas desacopladas:

Transport Layer (MCP Server): Implementado com axum. Escuta conexões via HTTP/WebSockets.

Policy Engine (O Cérebro): O motor em Rust puro que aplica os "Guardrails". Deserializa o payload e checa as permissões.

Execution Environment (CDP Worker): Gerencia o ciclo de vida da instância do Chromium e executa a injeção no DOM.

4. Fluxo de Dados (Data Flow)
O caminho de uma requisição ("The Life of a Request") segue a seguinte ordem cronológica:

Intenção: O LLM decide comprar um item. Ele envia via MCP o JSON: {"tool": "web_action", "params": {"action": "click", "selector": "#buy", "url": "amazon.com"}}.

Ingestão: O servidor axum recebe o payload e converte para structs Rust usando serde.

Validação (Aegis Core): * A Policy Engine verifica: "amazon.com" está na whitelist? (Sim).

A ação "click" é permitida nesta URL? (Sim).

Execução: O CDP Worker assume a conexão ativa com o Chromium, localiza o seletor #buy e despacha o evento de clique.

Observabilidade: O Worker extrai a "Accessibility Tree" (o estado da tela atualizada após o clique) e compacta o dado.

Retorno: O Aegis responde ao LLM via MCP com sucesso e o novo estado da página.

5. Segurança e Tratamento de Erros (Mitigação de Riscos)
Prompt Injection / Payload Manipulation: Como usamos validação rigorosa de tipos no Rust, se o LLM tentar enviar um script malicioso (ex: <script>alert(1)</script>) em um campo que espera uma URL, o sistema falha na hora da deserialização, bloqueando a requisição antes mesmo da lógica de negócio.

Timeouts e Recursos: O LLM não pode travar o worker. Toda requisição CDP é envelopada com tokio::time::timeout. Se o site demorar mais de 5 segundos para responder, a operação é abortada e o erro é repassado ao agente.
