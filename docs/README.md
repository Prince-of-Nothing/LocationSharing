# Documentation

This project's documentation follows two complementary, widely used open-source
frameworks so that content stays organized as the project grows, and so that
architectural knowledge and the *reasoning* behind it is never lost:

- **[Diátaxis](https://diataxis.fr/)** — organizes all user/contributor-facing
  documentation into four distinct modes, each answering a different question
  and serving a specific purpose:
  - [`tutorials/`](./tutorials) — *learning-oriented*: guided lessons for newcomers.
  - [`how-to/`](./how-to) — *task-oriented*: step-by-step recipes for a specific goal.
  - [`reference/`](./reference) — *information-oriented*: precise, factual descriptions
    (APIs, configuration, data schemas, CLI flags).
  - [`explanation/`](./explanation) — *understanding-oriented*: discussion, background,
    and the "why" behind design choices.

- **[arc42](https://arc42.org/)** + **[C4 model](https://c4model.com/)** — organizes the
  **system architecture** itself into a standard 12-section template, with diagrams at
  four levels of zoom (Context → Container → Component → Code). See
  [`architecture/`](./architecture).

- **[Architecture Decision Records (ADRs)](https://adr.github.io/)** — every significant
  architectural decision is captured as a small, immutable, numbered record in
  [`adr/`](./adr), so that *why* a decision was made is preserved even after the
  decision itself is superseded. This is the primary mechanism for making sure no
  architectural knowledge is ever lost, even as the system evolves.

## Why this layered structure?

Documentation should match the stakes and context of the project. We chose Diátaxis + arc42 + ADRs because:

1. **Separation of concerns**: Each layer serves a distinct audience and purpose — tutorials teach, how-tos guide, reference informs, explanation clarifies, architecture structures, and ADRs preserve decisions. This avoids the common failure mode of mixed-purpose documents that satisfy no one well.

2. **Compliance & auditability**: GDPR, child-safety regulations, and platform policies demand traceable decisions, clear data-handling documentation, and defensible architectural choices.

3. **Security-critical features**: Location sharing, consent management, and relationship-based access control require documented rationale that can be reviewed and traced from requirements through implementation.

4. **Knowledge retention**: ADRs ensure architectural reasoning survives team changes, refactors, and rewrites — the single biggest source of "lost architecture knowledge" in most projects.

5. **Scalability**: Clear ownership and separation of concerns means documentation scales with the project rather than becoming an unmaintainable mess.

### Trade-offs we accept

| Aspect | Informal/Mixed Approach | Layered Approach (Our Choice) |
|---|---|---|
| Initial overhead | Low | Higher (must classify, follow templates) |
| Onboarding speed | Fast for simple tasks | Slower start, predictable path to mastery |
| Knowledge retention | Fragile (scattered in issues/chat) | Durable (ADRs preserve rationale) |
| Scalability | Breaks down as project grows | Scales well with clear ownership |
| Auditability | Hard to trace decisions | Full traceability REQ → ARCH → TEST |
| Flexibility | Easy pivot, risks inconsistency | Structured change via ADRs ensures consistency |

**This isn't universal superiority** — it's context matching. For a fast prototype
with no regulatory constraints or sensitive data, a lighter approach might be preferable.
For Always Together's combination of compliance, security, and safety concerns, the
additional documentation overhead is a feature, not a bug.

For comparison: Bluesky's docs mix guides, API reference, and design notes informally
with minimal decision history in-repo. That works for their context (fast-moving protocol,
technical audience, synchronous communication). Our stricter structure responds to
different pressures: sensitive data handling, internship constraints with mentor review,
and the high cost of getting privacy/security wrong.

## Documentation Structure

This documentation tree is organized following **Diátaxis**, **arc42**, and **ADRs** to ensure layered, navigable content for different audiences and purposes. For internship report preparation, we recommend starting with the key reference documents listed below, then diving into architecture and explanation sections as needed.

### Quick Navigation for Report Writing

| Topic | Primary Documents | Supporting Documents |
|---|---|---|
| **Product Vision & Scope** | [§1 Introduction and Goals](./architecture/introduction-and-goals.md), [Scope and MVP Boundaries](./architecture/scope-and-mvp-boundaries.md) | [Social and Community Features](./explanation/social-and-community-features.md), [Roadmap](./explanation/development-roadmap-and-evidence.md) |
| **Requirements** | [MVP Requirements](./reference/mvp-requirements-and-acceptance-criteria.md), [Traceability Matrix](./reference/requirements-traceability-matrix.md) | [Actors and Accounts](./reference/actors-and-accounts.md), [UI Navigation](./reference/ui-navigation-and-wireframes.md) |
| **Architecture** | [Architecture Models](./architecture/architecture-models.md), [Building Block View](./architecture/building-block-view.md), [Deployment View](./architecture/deployment-view.md) | [C4 Diagrams](./architecture/diagrams/README.md), [Technology Stack](./reference/technology-stack-and-infrastructure.md) |
| **Security & Privacy** | [Security Architecture](./architecture/security-architecture-and-data-protection.md), [Security Control Register](./reference/security-control-register.md), [Privacy and Data Handling](./architecture/privacy-and-data-handling.md) | [Threats and Controls](./architecture/security-threats-and-controls.md), [Incident Response Plan](./architecture/incident-response-plan.md) |
| **Data Model** | [Domain Data Model](./reference/domain-data-model.md), [Profile and Visibility Model](./reference/profile-and-visibility-model.md) | [API Contract](./reference/api-contract.md) |
| **Research & Validation** | [Research Notes and References](./explanation/research-notes-and-references.md) | [Constraints](./architecture/constraints.md), [Quality Requirements](./architecture/quality-requirements.md) |
| **Roadmap & Evidence** | [Development Roadmap](./explanation/development-roadmap-and-evidence.md), [DAS Internship Alignment](./explanation/das-internship-alignment.md) | [Risks and Technical Debt](./architecture/risks-and-technical-debt.md), [ADR Index](./architecture/architecture-decisions-link.md) |

See [`CONTRIBUTING.md`](./CONTRIBUTING.md) for how to decide where new content goes,
the docs-as-code review workflow, style conventions, and the doc-audit cadence used
to keep this from rotting.

## Layout

```
docs/
├── README.md               <- this file
├── CONTRIBUTING.md         <- how to write/review docs, style, audit cadence
├── tutorials/              <- Diátaxis: learning-oriented guides
├── how-to/                 <- Diátaxis: task-oriented recipes (testing strategy, ...)
├── reference/              <- Diátaxis: API contract, data model, security control
│                              register, actors/accounts, MVP requirements, RTM, ...
├── explanation/            <- Diátaxis: social/creative features, roadmap, research
│                              notes, internship alignment, ...
├── architecture/           <- arc42-structured architecture documentation
│   ├── 01-introduction-and-goals.md
│   ├── scope-and-mvp-boundaries.md
│   ├── 02-constraints.md
│   ├── 03-context-and-scope.md
│   ├── 04-solution-strategy.md
│   ├── architecture-models.md
│   ├── 05-building-block-view.md
│   ├── 06-runtime-view.md
│   ├── 07-deployment-view.md
│   ├── 08-crosscutting-concepts.md
│   ├── privacy-and-data-handling.md          <- standalone, referenced from §8
│   ├── security-architecture-and-data-protection.md
│   ├── security-threats-and-controls.md
│   ├── incident-response-plan.md
│   ├── location-sharing-lifecycle.md
│   ├── movement-trails-and-search-corridors.md   <- deferred feature
│   ├── friend-mosaic-maps.md                     <- deferred feature
│   ├── cache-and-permission-model.md
│   ├── messaging-and-group-lifecycle.md          <- deferred feature
│   ├── 09-architecture-decisions.md   <- index into adr/
│   ├── 10-quality-requirements.md
│   ├── 11-risks-and-technical-debt.md
│   ├── 12-glossary.md
│   └── diagrams/            <- C4 diagrams (context/container/component/code)
└── adr/                     <- Architecture Decision Records (one file per decision)
    ├── template.md
    ├── 0001-record-architecture-decisions.md
    └── 0002-adopt-always-together-product-direction.md
```

## Keeping architecture knowledge from being lost

1. **Every non-trivial architectural decision gets an ADR** before or as it is made —
   not retroactively. Use [`adr/template.md`](./adr/template.md).
2. **arc42 sections are living documents.** When the system changes shape, update the
   relevant section (building blocks, runtime, deployment, etc.) in the same PR as the
   code change, not "later."
3. **Superseded decisions are marked, not deleted.** ADRs are numbered and immutable;
   a new ADR that changes course references and supersedes the old one.
4. **Diagrams are text-based** (Mermaid/PlantUML) and versioned alongside the docs, so
   diffs are reviewable and diagrams never silently drift from reality.
