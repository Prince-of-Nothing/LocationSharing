# Documentation

This project's documentation follows two complementary, widely used open-source
frameworks so that content stays organized as the project grows, and so that
architectural knowledge and the *reasoning* behind it is never lost:

- **[Diátaxis](https://diataxis.fr/)** — organizes all user/contributor-facing
  documentation into four modes, each answering a different question:
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

See [`CONTRIBUTING.md`](./CONTRIBUTING.md) for how to decide where new content goes,
the docs-as-code review workflow, style conventions, and the doc-audit cadence used
to keep this from rotting.

## Why this structure (vs. e.g. Bluesky's docs)

Bluesky's own docs (`bsky-docs` + the `atproto` repo) are a reasonable reference, but
they mix guides, API reference, and design notes fairly informally, and store almost
no architectural rationale or decision history in-repo. For a project like **Always
Together**, Diátaxis + arc42 + ADRs gives us:

- A **stronger separation of concerns** for user-facing docs (Diátaxis), avoiding the
  common failure mode of half-tutorial/half-reference pages that satisfy no one well.
- A **standard, complete architecture template** (arc42) instead of ad-hoc design docs,
  so nothing structural gets forgotten (constraints, quality goals, risks, glossary...).
- **Durable decision history** (ADRs) that survives refactors and rewrites — the
  single biggest source of "lost architecture knowledge" in most projects.

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
