# Landscape Research

## Name Availability

- **frigg-astraea** — no existing usage found anywhere.
- **Frigg** alone — used by a JS integration framework (friggframework.org), different space.
- **Astraea** alone — used by several unrelated projects (intelligence platform, astronomy package, planet generator). None in AI transparency/governance.

## Existing Projects in the Space

| Project | Language | What it does | Gap |
|---------|----------|-------------|-----|
| Bulwark | Rust | Governance proxy between agents and tools (MCP gateway) | Proxy architecture, not an embedded library |
| bastion-ai | Rust | Consensus gating, verification, audit trails | Deterministic verification only, no layered model |
| guardrails-ai | Python | Input/output validation for LLM calls | Python-only, content filtering only |
| NeMo Guardrails | Python | Programmable guardrails for LLM apps | NVIDIA-tied, conversation-focused |
| Microsoft Agent Governance Toolkit | Multi | Policy engines, trust, SRE for agents | Enterprise/Azure-centric, not a library |
| AgentTrace / AgentSight | Research | Observability/telemetry for agents | Academic, observability-only |

## Where Frigg Fits

No one is doing the full Security → Visibility → Mitigation stack as a single, layered Rust library with Python interop.

The space has:
- Pure guardrails (rules only, no visibility)
- Pure observability (visibility only, no action)
- Heavy enterprise platforms (not libraries)
- Python-only solutions (no performance focus)

Frigg's differentiator: layered approach with human-in-the-loop philosophy (TMO-inspired "human decides, system assists") vs. the common "block everything automatically" guardrails pattern.
