# 0. Architecture Documentation

This is the arc42-structured architecture documentation for **Always
Together** (a consent-based location-sharing safety/social app). Each
numbered file corresponds to a standard arc42 section. Diagrams follow the
C4 model and live in [`diagrams/`](./diagrams).

| # | Section |
|---|---------|
| 1 | [Introduction and Goals](./01-introduction-and-goals.md) |
| — | [Scope and MVP Boundaries](./scope-and-mvp-boundaries.md) |
| 2 | [Constraints](./02-constraints.md) |
| 3 | [Context and Scope](./03-context-and-scope.md) |
| 4 | [Solution Strategy](./04-solution-strategy.md) |
| — | [Architecture Models](./architecture-models.md) |
| 5 | [Building Block View](./05-building-block-view.md) |
| 6 | [Runtime View](./06-runtime-view.md) |
| 7 | [Deployment View](./07-deployment-view.md) |
| 8 | [Crosscutting Concepts](./08-crosscutting-concepts.md) |
| 9 | [Architecture Decisions](./09-architecture-decisions.md) (index into [ADRs](../adr)) |
| 10 | [Quality Requirements](./10-quality-requirements.md) |
| 11 | [Risks and Technical Debt](./11-risks-and-technical-debt.md) |
| 12 | [Glossary](./12-glossary.md) |

## Deep-dive / standalone architecture documents

| Document | Topic |
|---|---|
| [Privacy & Data Handling](./privacy-and-data-handling.md) | Standalone privacy reference — referenced from §8 |
| [Security Architecture and Data Protection](./security-architecture-and-data-protection.md) | Data classification, security boundaries, encryption, authorization control sequence |
| [Security Threats and Controls](./security-threats-and-controls.md) | Threat-first view across location, identity, social/content, sponsorship, infrastructure |
| [Incident Response Plan](./incident-response-plan.md) | Severity levels, response stages, evidence handling |
| [Location Sharing Lifecycle](./location-sharing-lifecycle.md) | Core MVP consent/sharing/expiry flow |
| [Movement Trails and Search Corridors](./movement-trails-and-search-corridors.md) | Deferred: opt-in trails, missing-person search workflow |
| [Friend Mosaic Maps](./friend-mosaic-maps.md) | Deferred: collaborative visited-region maps |
| [Cache and Permission Model](./cache-and-permission-model.md) | Location-bound content and location-locked/public caches |
| [Messaging and Group Lifecycle](./messaging-and-group-lifecycle.md) | Deferred: direct messaging, groups, in-chat challenges |

For the full documentation structure and rationale, see
[docs/README.md](../README.md). For product scope, requirements, data model,
API contract, and security control register, see
[docs/reference/](../reference). For deferred/Full-mode feature rationale,
see [docs/explanation/](../explanation).
