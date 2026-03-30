---
title: Architecture
description: Architecture
---

# Current State

As of March 9, 2026, the active runtime baseline in this repository is:

- `ecs` as the ECS foundation on the runtime path
- `engine::App` as the runtime entry surface
- `engine_sim` for shared simulation contracts
- `engine_history` for checkpoints, journals, archives, and replay validation
- `engine_net` + `engine_net_quic` for dedicated-authority networking
- `grotto_online` for Axiom handoff/control-plane integration
- `grotto_client` and `grotto_server` as dedicated client/server binaries

## Canonical References

- Workspace roadmap: [docs/roadmaps/ROADMAP.md](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/docs/roadmaps/ROADMAP.md)
- Architecture guidelines: [docs/guidelines/ARCHITECTURE.md](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/docs/guidelines/ARCHITECTURE.md)
- Domain map: [docs/guidelines/DOMAIN_MAP.md](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/docs/guidelines/DOMAIN_MAP.md)
- Net architecture (current): [net/architecture.puml](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/net/architecture.puml)
