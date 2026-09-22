# Reference

*Information-oriented*: precise, factual, and complete descriptions of the
system — consulted while working, not read start-to-end. No opinions or
tutorials here, only accurate facts.

| Document | Contents |
|---|---|
| [Actors and Accounts](./actors-and-accounts.md) | Roles, identity/visibility fields, account model |
| [Profile and Visibility Model](./profile-and-visibility-model.md) | Default-empty non-friend profile rule, relationship-based views |
| [Domain Data Model](./domain-data-model.md) | Core entities (User, LocationShare, MovementTrail, Cache, Challenge, etc.) |
| [API Contract](./api-contract.md) | Versioned `api/v1` HTTPS + WebSocket endpoints, RFC 7807 errors |
| [API Security Implementation](./api-security-implementation.md) | Detailed security implementation for all endpoints: OAuth 2.0/PKCE flow, authorization sequences, rate limiting, input validation, error handling, audit logging |
| [Technology Stack and Infrastructure](./technology-stack-and-infrastructure.md) | Frozen MVP stack (Flutter/FastAPI/PostgreSQL/Redis/etc.) and rationale |
| [Security Control Register](./security-control-register.md) | SC-01 through SC-09: problem/relevance/implementation/test/limitation |
| [MVP Requirements and Acceptance Criteria](./mvp-requirements-and-acceptance-criteria.md) | REQ-01 through REQ-09 with acceptance criteria and explicit non-goals |
| [Requirements Traceability Matrix](./requirements-traceability-matrix.md) | REQ → UI → API → DB → security control → test → evidence |
| [UI Navigation and Wireframes](./ui-navigation-and-wireframes.md) | Primary navigation and key screens |

See the [Diátaxis reference guide](https://diataxis.fr/reference/) —
reference material should mirror the structure of the system itself, and
stay strictly descriptive. For architecture rationale, see
[docs/architecture/](../architecture); for deferred-feature discussion, see
[docs/explanation/](../explanation).
