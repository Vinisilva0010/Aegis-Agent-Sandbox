# Aegis-Agent-Sandbox

**Security Middleware for Autonomous Agents**  
**Author:** Vinicius Pontual  
**Status:** In Review (Draft)  
**Date:** March, 2026  
**Core Technologies:** Rust, Tokio, Axum, Prism-MCP, Browser-Use (CDP)

## 1. Context and Objective

Frontier models (LLMs) are probabilistic and prone to hallucinations. Granting unrestricted access to the Chrome DevTools Protocol (CDP) for web navigation or system actions represents a critical security risk.

The **Aegis-Agent-Sandbox** acts as a deterministic security proxy, written in Rust for performance and *memory safety*. It intercepts LLM intentions via the Model Context Protocol (MCP), validates them against **Zero Trust** policies, executes approved actions in a **headless** browser, and returns a compressed DOM.

## 2. Scope

### Goals (What the system DOES)

- Receives intent payloads via MCP (e.g., click, type, Maps)
- Validates actions against `policies.yaml` (**Domain/Action Whitelisting**)
- Executes actions in isolated **headless** Chromium via CDP
- Returns clear semantic errors so the LLM can replan
- Collects logs (**Approved vs Blocked**) for analysis

### Non-Goals (What the system DOES NOT DO)

- Does not host or run LLMs (**middleware only**)
- Does not perform large-scale scraping (**focused on point interactions**)
- No complex UI; **headless infrastructure**

## 3. Architecture

System built in **three decoupled layers**:

### Transport Layer
**Responsibility:** MCP Server (HTTP/WebSockets)  
**Technologies:** Axum  

### Policy Engine
**Responsibility:** Guardrail validation  
**Technologies:** Rust + Serde  

### Execution Environment
**Responsibility:** Chromium lifecycle + DOM injection  
**Technologies:** CDP Worker  

## 4. Data Flow
LLM → MCP Payload → Aegis Ingest → Policy Check → CDP Exec → DOM Snapshot → LLM

text

### Real Example

LLM wants to buy an item:
{
"tool": "web_action",
"params": {
"action": "click",
"selector": "#buy",
"url": "amazon.com"
} }

text

### Flow

1. **Ingestion:** Axum → Rust structs (Serde)  
2. **Validation:** Is `amazon.com` whitelisted? Is `click` allowed?  
3. **Execution:** CDP Worker clicks `#buy`  
4. **Observability:** Extracts compacted **Accessibility Tree**  
5. **Return:** Success + new state for the LLM  

## 5. Security

### Prompt Injection

Strict type validation in Rust blocks malicious payloads during deserialization  
(e.g., `<script>alert(1)</script>` in the URL field).

### Timeouts
tokio::time::timeout(Duration::from_secs(5), task)

text

Slow sites → abort + error returned to the agent.

### Logs

Approved and blocked actions are audited for future optimization.
