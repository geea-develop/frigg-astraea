# Brainstorming

## What

A Rust library that brings transparency to AI operations.

- High performance
- Needs Python interop (approach TBD)

## Pillars

A layered approach:

1. **Security** — the foundation. Defines what an AI agent is allowed to do (boundaries, permissions, rules).
2. **Visibility** — observes what is actually happening within those boundaries.
3. **Mitigation** — empowers the user/actor to intervene based on what visibility reveals.

```
Security → Visibility → Mitigation → Trust
(boundaries)  (awareness)  (action)     (outcome)
```

## Outcome

**Trust** — enabling safe AI usage by managing risk rather than eliminating it. The library makes risk manageable and transparent so users can trust the system.

## Open Questions

- Project name
- Concrete scope of each pillar
- Python interop strategy (PyO3, FFI, sidecar?)
- Target audience / who the "user/actor" is
