# Pinpoint — Software Design Patterns

**Status:** Implemented (Rust client, `src/patterns/`)  
**Audience:** Developers, reviewers, students, future contributors  
**Related:** `docs/sdlc/06-technical-requirements.md`, `docs/sdlc/08-architecture.md`, `docs/uml/`

This document explains every design pattern used in Pinpoint, why it was chosen, how it is implemented in Rust, where it is used, its benefits, its trade-offs, and its test coverage. It is intended to be read cover-to-cover by a developer joining the project.

---

## 1. Pattern Overview

Pinpoint is a **Rust multiplatform client** (`eframe`/`egui` desktop + Android/iOS via `cargo-apk`/`cargo-xcodebuild`). Long-term direction is Rust-first for performance close to C/C++, memory safety, and a single codebase across desktop, Android, iOS, and wasm. Non-Rust stacks (Go/Python backend, Kotlin, vanilla-JS web) are archived except the Go backend + web client restored at root for live friend use — see `docs/adr/0007-rust-multiplatform.md` and `docs/adr/0008-backend-restore-and-rust-development.md`.

Patterns implemented:

| Pattern | Module | Purpose |
|---|---|---|
| Repository | `patterns/repository.rs` | Abstract data access (users/friendships/shares/locations) |
| Strategy | `patterns/strategy.rs` | Pluggable precision handling (exact/approximate/city_only) |
| Command | `patterns/command.rs` | Encapsulate user actions, enable undo/history |
| State | `patterns/state.rs` | Screen/flow state (Map/Login/Register/Friends/Sharing/Admin/ChangePassword) |
| Observer | `patterns/observer.rs` | Publish/subscribe for state changes, audit |
| Factory | `patterns/factory.rs` | Create API/OSM clients and precision strategies |
| Decorator | `patterns/decorator.rs` | Add logging, retry, timing around API trait |
| Proxy | `patterns/proxy.rs` | Offline/online proxy for network services |

Security measures are documented separately in `docs/security-architecture.md`. This document focuses on structure and behavior.

---

## 2. Repository Pattern

### Problem

The application needs to access domain objects (users, friendships, location shares, friend locations) from multiple sources: remote API, in-memory cache, on-device storage, and offline demo data. Business logic should not depend on storage details. Without an abstraction, every caller would need to know how to fetch data, handle network errors, and manage caching.

### Solution

Define **trait-based repository interfaces** (`UserRepository`, `FriendshipRepository`, `LocationShareRepository`, `FriendLocationRepository`, combined as `Repository`). Implementations hide persistence details.

**In this codebase:** `src/patterns/repository.rs`

Key traits:

```rust
pub trait UserRepository: Send + Sync {
    fn get_current_user(&self) -> RepositoryResult<Option<User>>;
    fn set_current_user(&self, user: Option<User>) -> RepositoryResult<()>;
    fn update_user(&self, user: &User) -> RepositoryResult<()>;
}

pub trait FriendshipRepository: Send + Sync {
    fn list_friendships(&self) -> RepositoryResult<Vec<Friendship>>;
    fn get_friendship(&self, id: i64) -> RepositoryResult<Option<Friendship>>;
    fn create_friendship(&self, friendship: &Friendship) -> RepositoryResult<Friendship>;
    fn update_friendship(&self, friendship: &Friendship) -> RepositoryResult<()>;
    fn delete_friendship(&self, id: i64) -> RepositoryResult<()>;
}
```

`InMemoryRepository` provides a `Mutex`-protected in-memory implementation suitable for offline mode and testing. Data persists only for the process lifetime unless paired with a file-backed store.

### Benefits

1. **Testability** — Mock or fake repositories in unit tests without a backend.
2. **Swappability** — Swap `ApiClient`-backed repository for `InMemoryRepository` in offline mode.
3. **Single error type** — `RepositoryError::{NotFound, AlreadyExists, InvalidInput, NetworkError, Unauthorized, StorageError}` normalizes all failure modes.
4. **No data leakage** — Repositories operate on domain models, never on transport DTOs.

### Trade-offs

- **Indirection** — An extra layer between UI and data. Justified because storage backends vary per platform.
- **Boilerplate** — Each entity needs a trait + implementation. Acceptable for a small domain (4 entities).

### Test Coverage

```text
patterns::repository::tests::friendship_crud ... ok
patterns::repository::tests::location_upsert ... ok
```

Tests cover create/read/update/delete, upsert semantics, and not-found errors.

---

## 3. Strategy Pattern

### Problem

Location precision (exact/approximate/city-only) changes rounding behavior. The same three rules must be applied consistently for display, publishing, and tests, and must be swappable when a share's precision changes.

### Solution

Define a `PrecisionStrategy` trait with `ExactStrategy`, `ApproximateStrategy`, and `CityOnlyStrategy` implementations, plus a `PrecisionContext` holder that delegates to the current strategy.

**In this codebase:** `src/patterns/strategy.rs`

- **Exact** — Coordinates unchanged, accuracy preserved.
- **Approximate** — Rounds to 2 decimals (~1.1 km), accuracy set to 250 m.
- **City-only** — Rounds to 1 decimal (~11 km), accuracy set to 11,000 m.

`PrecisionLevel::from_str` parses the API's string values via `std::str::FromStr` (trait-correct, avoids clippy `should_implement_trait` warning).

### Benefits

1. **Open/closed** — New precision levels can be added without modifying callers.
2. **Single source of truth** — Rounding rules live in one place.
3. **Context object** — `PrecisionContext` allows runtime strategy switching.

### Trade-offs

- **Overkill for 3 values** — A match expression would also work. The pattern pays off when precision rules grow (e.g., geofence zones).

### Test Coverage

```text
patterns::strategy::tests::exact_preserves_coordinates ... ok
patterns::strategy::tests::approximate_rounds_to_2_decimals ... ok
patterns::strategy::tests::city_only_rounds_to_1_decimal ... ok
patterns::strategy::tests::strategy_switching ... ok
```

---

## 4. Command Pattern

### Problem

User actions (login, publish location, create share, toggle share) need consistent execution semantics: success/failure reporting, ordering, potential undo, and audit logging. Ad-hoc `async fn` calls scattered through the UI make this fragile.

### Solution

Encapsulate each action as a `Command` object with `name()`, `execute()`, `undo()`, `is_undoable()`, and `to_json()` (for audit). `CommandExecutor` maintains a bounded history with undo/redo support.

**In this codebase:** `src/patterns/command.rs`

Commands implemented:

| Command | Undoable | Payload |
|---|---|---|
| `LoginCommand` | No | email, password |
| `PublishLocationCommand` | No | lat, lon, accuracy |
| `CreateShareCommand` | No | friendship_id, direction, precision |
| `ToggleShareCommand` | Yes | share_id, is_active |
| `SendFriendRequestCommand` | No | email |

`CommandExecutor` (default history 100):

- `execute()` — Truncates redo branch, enforces max history, runs command, records success.
- `undo()` / `redo()` — Move through history.
- `can_undo()` / `can_redo()` — Availability checks.
- `history()` — Ordered command names (for audit/UI).

**Important design decision:** Only **undoable** commands enter history. Non-undoable actions (login, publish) execute but are not stored, preventing false undo affordances. The `non_undoable_command` test pins this.

### Benefits

1. **Auditability** — `to_json()` serializes commands for audit logs.
2. **Undo/redo** — Share toggling is reversible without re-fetching.
3. **Testability** — Commands can be tested without the full app.

### Trade-offs

- **Async complexity** — Commands here are sync (`execute` returns `CommandResult`). Real API commands are async in `PinpointApp`; the Command pattern is currently used for queue/history semantics, not as a replacement for the app's async methods. Full async Command + `tokio` is a documented extension point.

### Test Coverage

```text
patterns::command::tests::execute_and_undo ... ok
patterns::command::tests::non_undoable_command ... ok
```

---

## 5. State Pattern

### Problem

The UI has seven screens (Map, Register, Login, Friends, Sharing, Admin, ChangePassword) with different transition rules and auth requirements. Scattered `if` statements for navigation become error-prone as screens grow.

### Solution

Each screen is a `ScreenState` object: `MapState`, `LoginState`, `RegisterState`, `FriendsState`, `SharingState`, `AdminState`, `ChangePasswordState`. `StateContext` holds the current state and a map of available states.

**In this codebase:** `src/patterns/state.rs`

Key behaviors:

- `screen_id()` — Returns the `CurrentScreen` enum.
- `requires_auth()` — `true` for Friends/Sharing/Admin/ChangePassword.
- `title()` / `description()` — UI strings.
- `clone_box()` — Enables cloning `Box<dyn ScreenState>` (required for `StateContext` to store trait objects).

`PinpointApp` owns a `StateContext` (behind `Arc<Mutex<...>>`) and synchronizes it with `current_screen` on every `set_screen()` call. `requires_sign_in()` delegates to the State pattern instead of a hardcoded `matches!` (with a fallback for safety).

**Current default:** `Map` — the map-first UX. Login/Register are reachable via header buttons, not as the landing page.

### Benefits

1. **Auth rules centralized** — `requires_auth()` is the single source of truth.
2. **Testable transitions** — `default_state_is_map`, `friends_requires_auth`, `state_transition` tests.
3. **Extensible** — Adding a screen means adding one state struct + one map entry.

### Test Coverage

```text
patterns::state::tests::default_state_is_map ... ok
patterns::state::tests::friends_requires_auth ... ok
patterns::state::tests::map_does_not_require_auth ... ok
patterns::state::tests::state_transition ... ok
patterns::state::tests::list_available_states ... ok
```

---

## 6. Observer Pattern

### Problem

Multiple components need to react to the same event (user signed in, location updated, offline mode changed) without tight coupling.

### Solution

`EventSubject` maintains a `HashMap<AppEvent, Vec<Observer>>`. Observers implement `on_event()` and `handles_event()`. `LoggingObserver` provides a console/audit sink.

**In this codebase:** `src/patterns/observer.rs`

Events:

| Event | Trigger |
|---|---|
| `UserSignedIn` / `UserSignedOut` | Auth lifecycle |
| `SessionExpired` | Session timeout |
| `LocationUpdated` | After publish or refresh |
| `FriendshipChanged` | After request/response |
| `ShareCreated` / `ShareUpdated` / `ShareRevoked` | Share lifecycle |
| `OfflineModeChanged` | Connectivity change |
| `ErrorOccurred` | Any API failure |

### Benefits

1. **Decoupling** — Publishers don't know subscribers.
2. **Audit hook** — `LoggingObserver` formats events for logs.
3. **Testable** — `TestObserver` records received events.

### Trade-offs

- **Unordered delivery** — Observers run in registration order; no priority.
- **Sync only** — Observers are called synchronously. Async observers need `tokio::spawn`.

### Test Coverage

```text
patterns::observer::tests::observer_receives_matching_events ... ok
patterns::observer::tests::multiple_observers ... ok
```

---

## 7. Factory Pattern

### Problem

API clients, OSM clients, and precision strategies have construction logic that should not be scattered through the codebase.

### Solution

Static factories with `create()` / `create_with_timeout()` / `create_offline()` methods.

**In this codebase:** `src/patterns/factory.rs`

- `ApiClientFactory` — Creates `ApiClient` from URL (validates via `ApiClient::new`).
- `OsmClientFactory` — Creates `OsmClient` for Nominatim/OSRM/Overpass.
- `PrecisionStrategyFactory` — Creates the correct precision strategy for a `PrecisionLevel`.

### Benefits

1. **Encapsulation** — Construction details (timeouts, User-Agent, validation) live in one place.
2. **Consistency** — All call sites use the same creation path.

### Test Coverage

```text
patterns::factory::tests::api_client_factory_creates_client ... ok
patterns::factory::tests::osm_client_factory_creates_client ... ok
patterns::factory::tests::precision_strategy_factory_creates_strategies ... ok
```

---

## 8. Decorator Pattern

### Problem

Cross-cutting concerns (logging, retry, timing) should be applied to clients without modifying the client itself.

### Solution

`ApiClientTrait` defines a minimal client interface. Decorators wrap it:

| Decorator | Adds |
|---|---|
| `LoggingApiClient` | Request/response logging with duration |
| `RetryApiClient` | Retry with configurable attempts + delay |
| `TimingApiClient` | Tracks call count and average latency |

All decorators implement `ApiClientTrait` and delegate to the inner client, so they compose: `RetryApiClient<LoggingApiClient<ApiClientWrapper>>`.

**In this codebase:** `src/patterns/decorator.rs`

Note: This module defines its own `ApiClientTrait` (not `crate::api::ApiClient`, which is a concrete async client). The trait mirrors login/logout/get_friend_locations/publish_location for decorator use with sync test doubles. Async adaptation is an extension point.

### Benefits

1. **Separation of concerns** — Logging/retry are not mixed into client logic.
2. **Composability** — Stack decorators as needed.
3. **Testability** — `MockClient` validates decorator behavior without network.

### Test Coverage

```text
patterns::decorator::tests::timing_client_tracks_calls ... ok
```

---

## 9. Proxy Pattern

### Problem

The app must keep working when the backend is unreachable (server halted, offline mode). Direct API calls would fail and the UI would be empty.

### Solution

`ProxyLocationService` wraps `RealLocationService` (backed by `ApiClient`) with an in-memory cache. When online, it delegates to the real service and refreshes the cache. When offline or on error, it returns cached locations.

**In this codebase:** `src/patterns/proxy.rs`

- `set_offline()` / `set_online()` — Force mode.
- `is_online()` — Current connectivity.
- `get_friend_locations()` — Online: fetch + cache. Offline: return cache.
- `publish_location()` — Fails fast when offline (cannot publish without server).
- `refresh_cache()` — Explicit cache refresh.

Combined with `PinpointApp::demo_locations()` (Alice/Bob/Carol around Chicago), the map is **fully functional offline** from first launch.

### Benefits

1. **Resilience** — Server downtime degrades gracefully.
2. **Speed** — Cached data renders instantly.
3. **Explicit modes** — Offline is a first-class state, not an error path.

### Test Coverage

```text
patterns::proxy::tests::proxy_starts_online ... ok
patterns::proxy::tests::proxy_can_toggle_offline ... ok
```

---

## 10. Pattern Layering (How Patterns Compose)

```
┌─────────────────────────────────────────────────────────────────┐
│                        PinpointApp (app.rs)                      │
│  Holds: ApiClient, StateContext, RateLimiter, AuditLogger,       │
│         KeyManager, CommandExecutor, Repository                  │
└──────────────┬──────────────────────────────────────────────────┘
               │
       ┌───────┴───────┐
       │     app.rs    │  Validation gate (validate_email, validate_location)
       └───────┬───────┘  Rate limit gate (check "api_login"/"api_general")
               │          Audit log (log_event)
               ▼
   ┌───────────────────────┐
   │   ApiClient (api.rs)  │  Session cookie + CSRF, fail-closed, ADR-0006
   └───────────────────────┘
               │
   ┌───────────────────────┐
   │  Patterns (patterns/) │  Repository, Strategy, Command, State,
   │                       │  Observer, Factory, Decorator, Proxy
   └───────────────────────┘
               │
   ┌───────────────────────┐
   │  Security (security/) │  Validation, RateLimiter, Encryption,
   │                       │  Audit, Session, Signing
   └───────────────────────┘
               │
   ┌───────────────────────┐
   │  OS Services (osm.rs) │  Nominatim/OSRM/Overpass (reqwest+serde)
   └───────────────────────┘
```

Tagline: **Repository abstracts data. Strategy varies precision. Command captures actions. State drives UI. Observer notifies. Factory creates. Decorator enhances. Proxy enables offline.**

---

## 11. Pattern Selection Rationale

Why these patterns and not others?

| Considered pattern | Decision | Reason |
|---|---|---|
| Singleton | Rejected | Global state is hard to test; `Arc<Mutex<...>>` + DI preferred. |
| Builder | Partial | `PinpointApp::new()` is the builder; separate Builder pattern unnecessary. |
| Adapter | Rejected | `ApiClient` already adapts the Go contract; no legacy adapter needed. |
| Chain of Responsibility | Rejected | Validation → rate-limit → audit is linear; `PinpointApp` methods suffice. |
| Mediator | Rejected | Eg. UI talks directly to `PinpointApp`; no mediator needed at this scale. |
| Flyweight | Rejected | No heavyweight shared objects (no huge lookup tables). |

The implemented set covers **data access (Repository), algorithm variation (Strategy), action capture (Command), UI state (State), event decoupling (Observer), creation (Factory), cross-cutting augmentation (Decorator), and offline resilience (Proxy)** — the eight patterns that map directly to Pinpoint's risks.

---

## 12. Extension Points

| Extension | How |
|---|---|
| File-backed Repository | Implement `Repository` traits over `BookmarkStore` JSON or SQLite. |
| Async Command | Add `async fn execute_async()` using `tokio`; keep sync `execute()` for sync callers. |
| Realtime Observer | Emit `Observer` events over WebSocket when the backend adds realtime location pushes. |
| Offline Tile Packs | Add `OfflineTileStore` repository + Proxy cache for raster tiles. |
| E2E Encryption for shares | Add `EncryptionKey` exchange key to `KeyManager`; encrypt `FriendLocation` payloads before publish. |
| GraphHopper/Valhalla routing | Add `RoutingStrategy` implementing OSRM-compatible `Route` construction. |

---

## 13. Conventions

- **Naming:** Patterns live in `src/patterns/<pattern>.rs` and are re-exported via `src/patterns/mod.rs`.
- **Traits are object-safe** where instantiation requires it (`Send + Sync`).
- **Errors** surface as `String` (`CommandResult::Failure(String)`, `RepositoryResult<T>`) to avoid a megacrate error type across pattern boundaries.
- **Tests:** Every pattern has at least one unit test in its file. Run `cargo test --locked` to execute all 69 lib tests (12 pre-existing + 57 new pattern/security tests).
- **Clippy:** All pattern code passes `cargo clippy --all-targets -- -D warnings`.

---

## 14. References

- Pressman, R. *Software Engineering: A Practitioner's Approach* (pattern catalog).
- Gamma et al., *Design Patterns* (original GoF).
- Rust API Guidelines (trait naming, `FromStr`, `Default`).
- `docs/sdlc/06-technical-requirements.md` (TR layering).
- `docs/security-architecture.md` (security measures that complement patterns).
