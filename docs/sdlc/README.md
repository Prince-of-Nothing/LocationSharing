# SDLC Documentation — Pinpoint (OSM Map Parity Release)

This folder is the lifecycle documentation for the OpenStreetMap map-parity
release (web + Android + Rust desktop): interactive Slippy map, Nominatim
search, OSRM routing, Overpass POI browse, bookmarks, offline packs, and
the underlying Pinpoint consent-based location-sharing platform.

**Product references (OSM ecosystem):**
- OSM project home: https://wiki.openstreetmap.org/wiki/Main_Page
- Magic Earth (OSM-data consumer): https://wiki.openstreetmap.org/wiki/Magic_Earth
- Organic Maps (offline-first OSM client): https://organicmaps.app/

Pinpoint does **not** copy those products; it adopts the same OSM data
(ODbL), the same offline-first UX patterns, and the same usage policies.

## Index

| # | Document | Purpose |
|---|----------|---------|
| 00 | [00-overview.md](00-overview.md) | Release scope, principles, how to read this set |
| 01 | [01-business-requirements.md](01-business-requirements.md) | BRD: problem, goals, stakeholders, KPIs |
| 02 | [02-scope-and-assumptions.md](02-scope-and-assumptions.md) | In/out scope, assumptions, constraints |
| 03 | [03-functional-requirements.md](03-functional-requirements.md) | FR-01… — verifiable functional requirements |
| 04 | [04-non-functional-requirements.md](04-non-functional-requirements.md) | NFR-01… — performance, security, privacy, offline |
| 05 | [05-user-stories-and-use-cases.md](05-user-stories-and-use-cases.md) | User stories, use cases, acceptance criteria |
| 06 | [06-technical-requirements.md](06-technical-requirements.md) | TRS per client (web / Android / Rust / backend) |
| 07 | [07-openstreetmap-integration.md](07-openstreetmap-integration.md) | Tiles, Nominatim, OSRM, Overpass, ODbL, policies |
| 08 | [08-architecture.md](08-architecture.md) | C4, module boundaries, data flow, diagrams |
| 09 | [09-data-model-and-api.md](09-data-model-and-api.md) | Entities, API contract, precision & freshness |
| 10 | [10-security-and-privacy.md](10-security-and-privacy.md) | Threat model, controls, privacy by design |
| 11 | [11-qa-and-verification.md](11-qa-and-verification.md) | Test levels, automatic QA pipelines, exit criteria |
| 12 | [12-deployment-and-operations.md](12-deployment-and-operations.md) | Environments, CI/CD, rollback, monitoring |
| 13 | [13-maintenance-and-sustainability.md](13-maintenance-and-sustainability.md) | Change, versioning, tech-debt, support |
| 14 | [14-risk-register-and-assumptions.md](14-risk-register-and-assumptions.md) | Risks, issues, assumptions, dependencies |
| 15 | [15-traceability-matrix.md](15-traceability-matrix.md) | Requirement → design → code → evidence |
| 16 | [16-glossary-and-references.md](16-glossary-and-references.md) | Glossary, OSM references, ADRs |

## Conventions

- Requirement IDs: `FR-*` (functional), `NFR-*` (non-functional), `TR-*`
  (technical). Every FR/NFR/TR maps to at least one traceability row.
- Diagrams are Mermaid inside Markdown (never binary). See `docs/uml/`.
- Architecture-significant changes need an ADR (see `docs/adr/`).
- Security changes need a regression test **and** a fail-closed test.
- New dependencies update `THIRD\_PARTY\_NOTICES.md` in the same PR.

## How to use this set

1. Start at 00-overview, then 01 → 05 for *what* is built.
2. 06 → 09 for *how* it is built (including all OSM integration).
3. 10 → 12 for *how it stays safe and shipped*.
4. 13 → 16 for operations, risks, traceability and terms.
