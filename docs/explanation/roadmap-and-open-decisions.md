# Roadmap and Open Decisions

## Suggested build order

1. Trusted location sharing, maps, navigation, and safety controls.
2. Location caches, trusted messaging, optional movement trails, and a
   private visited-regions map.
3. Friend mosaic maps, friend challenges, creator groups, and moderated
   random group chats.
4. Daily prompts, media, ratings, discovery, and calls.
5. Additional creative challenges, calls, and community experiments.

## Frozen decisions (MVP technology stack)

The following stack decisions are **frozen** for the MVP (see
[Technology Stack and Infrastructure](../reference/technology-stack-and-infrastructure.md)):

- **Mobile framework:** Flutter + native location modules (Android/iOS
  platform channels)
- **Backend API:** FastAPI + SQLAlchemy + Alembic
- **Database:** PostgreSQL
- **Queue/cache:** Redis + Celery
- **Object storage:** S3-compatible encrypted storage
- **Identity/authorization:** OIDC/OAuth 2.0 with PKCE S256
- **Realtime transport:** Authenticated WebSocket gateway
- **Map provider:** OSM-derived provider behind adapter + external
  navigation deep links
- **Operations:** Docker + GitHub Actions; managed container hosting with
  managed PostgreSQL

## Open decisions (feature-level, not foundational)

These remain active and pertain to feature-level choices, not the frozen
stack above. Each should become an ADR once decided:

- How long should precise location history be retained?
- What age and verification rules apply to random groups?
- Which media formats and moderation workflow support prompts?
- Which features belong in Lite versus Full at launch?
- What sandbox and resource limits will protect experimental bot services?
- What check-in, recipient, retention, and emergency-service rules should
  govern search corridors?
- Should public caches reveal an exact point, a larger area, or only a map
  hint before a verified visit?
- Should historical region overlap unlock content, or only a visit after
  cache creation?
- What minimum region size and sensitive-place suppression rules prevent
  friend-map inference?
- Should challenges default to cooperative play, friendly competition, or
  let the creator choose?

---
*See also [§11 Risks and Technical Debt](../architecture/11-risks-and-technical-debt.md)
and the [ADR index](../adr) for how these decisions get recorded once made.*
