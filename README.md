# Frigg-Astraea

**Rust-powered AI agent governance engine with Python bindings.**

> *"Impossible is a word to be found only in the dictionary of fools."* — Napoleon

Frigg enforces rules on AI agent actions — blocking, warning, or logging tool calls before they execute. Drop it into Claude Code, LangChain, or any agent framework as a policy layer.

## Overview

| Component | Role |
|-----------|------|
| **Frigg** | Engine — rules evaluation, action logging, decision enforcement |
| **Astraea** | Platform (planned) — weighting, correlation, trust synthesis |

Three pillars: **Boundaries → Visibility → Mitigation → Trust**

## Who This Is For

Individuals who want to work safely and freely with AI.

You use AI agents daily — they write code, call APIs, manage files. You embrace that fully. But you also want to know what's happening, set boundaries you're comfortable with, and be able to step in when something matters. Frigg gives you that without slowing you down.

**This is for you if:**
- You run AI agents locally or in your own workflows and want a lightweight safety net
- You're a developer or operator who wants boundaries defined upfront, not discovered after something goes wrong
- You want an audit trail you can actually review in under a minute
- You value human authority — the AI works for you, not the other way around

**This is NOT for:**
- Fraud prevention, malicious exploitation detection, or virus/malware scanning — Frigg is a personal governance tool, not a security product
- Enterprise compliance teams looking for a platform (Frigg is a library, not a product — yet)
- People who want to block everything by default (Frigg assumes freedom first, constraints where needed)
- Teams looking for an AI framework or orchestrator (Frigg governs, it doesn't run agents)

## Install

```bash
cd frigg-py
python -m venv .venv && source .venv/bin/activate
pip install maturin
maturin develop
```

This builds `frigg-core` (Rust) and installs the `frigg` Python package locally.

## Usage

### Rust

```rust
use frigg_core::Frigg;

let frigg = Frigg::from_config("rules.yaml", "frigg.log");
let result = frigg.check(&action);
```

### Python

```python
from frigg import Frigg

frigg = Frigg.from_config("rules.yaml", "frigg.log")
result = frigg.check({"name": "file_write", "params": {"path": "/etc/passwd"}})
# result: {"decision": "blocked", "rule_id": "no-etc-write", "reason": "..."}
```

Possible decisions: `allowed`, `blocked`, `warned`.

### Claude Code Hook

Register as a `PreToolUse` hook — reads JSON from stdin, outputs a `permissionDecision`:

```bash
python -m frigg.hook --config rules.yaml --log frigg.log
```

### LangChain Middleware

```python
from frigg.middleware import FriggMiddleware

agent = create_agent(middleware=[FriggMiddleware("rules.yaml")])
```

### Generic Decorator

```python
from frigg import Frigg
from frigg.decorator import guard

frigg = Frigg.from_config("rules.yaml", "frigg.log")

@guard(frigg, action_name="send_email")
def send_email(to, body):
    ...
```

## Rules YAML Format

```yaml
rules:
  - id: no-etc-write
    description: Block writes to /etc
    pattern: "file_write:/etc/**"
    severity: block
    enabled: true

  - id: warn-shell
    description: Warn on shell commands
    pattern: "shell:*"
    severity: warn
    enabled: true
```

| Field | Description |
|-------|-------------|
| `id` | Unique rule identifier |
| `description` | Human-readable explanation |
| `pattern` | Glob pattern matching `action_name:params` |
| `severity` | `block` · `warn` · `log` · `ask_human` |
| `enabled` | Toggle rule on/off |

## Architecture

```
frigg-astraea/
├── frigg-core/       # Rust library — rule engine, logger, decision logic
├── frigg-py/         # PyO3 bindings (built with maturin)
│   ├── src/          # Rust FFI layer
│   └── frigg/        # Python package
│       ├── hook.py       # Claude Code PreToolUse hook
│       ├── middleware.py # LangChain middleware
│       └── decorator.py  # Generic @guard decorator
├── rules.yaml        # Example rules
└── README.md
```

**Cargo workspace** with two crates: `frigg-core` (lib) and `frigg-py` (cdylib via PyO3).

## Tests

```bash
cargo test          # 18 Rust tests
cd frigg-py && python -m pytest
```

## TODO

### Phase 1 — Core hardening
- [ ] **Log encryption** — encrypt audit logs at rest (logs contain sensitive action data — paths, params, decisions)
- [ ] **Audit log** — structured JSON logs, log rotation, summary/digest command
- [ ] **Rule engine** — regex patterns, condition expressions, rule chaining
- [ ] **Config validation** — warn on overlapping/conflicting rules, dry-run mode

### Phase 2 — Ship it
- [ ] **Tests & usability** — expand test coverage, clean API surface, clear error messages, sensible defaults
- [ ] **Distribution** — publish to PyPI / crates.io

### Phase 3 — Evolve
- [ ] **Human-in-the-loop** — timeout defaults, remember decisions for session, trust escalation
- [ ] **More integrations** — OpenAI function calling, CrewAI, other agent frameworks (including error-handling patterns)
- [ ] **Astraea layer** — trust scoring, action correlation, behavioral baselines

## License

MIT
