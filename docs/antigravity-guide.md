# Pinpoint — Google Antigravity Efficiency Guide

How to use Google Antigravity to build this app with the fewest tokens and
least wasted effort. Pinpoint is a large Rust multiplatform crate with a
modular server; Antigravity works best when you give it a **narrow, well-
bounded task** plus the **right context**, instead of the whole repo.

## 1. Ground rules (read once)

- **One task per session.** Don't ask for "build the mobile app" — ask for
  "add `android_main` wiring in `src/mobile/android.rs`". Smaller asks =
  fewer tokens, fewer mistakes.
- **Always point at files.** Paste the exact path (`src/security/keychain.rs`)
  or a short snippet. Antigravity searches poorly; explicit paths save tokens.
- **Run the gate locally first.** `cargo fmt --all -- --check`,
  `cargo clippy --all-targets -- -D warnings`, `cargo build --locked`,
  `cargo test --locked`. Paste only the failing output, not the whole log.
- **Never paste full files.** Paste the function or the diff. The agent
  re-reads files itself; pasting 500 lines wastes tokens and confuses it.
- **Prefer edits over rewrites.** "Change X in file Y" beats "rewrite file Y".
- **Commit often, small.** Each green gate = a commit. Small commits keep
  context windows small.

## 2. Setup (one time)

1. Open the repo root (`C:\Users\Dinusauuuur\Desktop\git\Pinpoint`) as the
   workspace — not `archive/`.
2. Tell Antigravity the build env (Windows MSVC from `archive/msvc`):
   ```
   PATH += archive/msvc/VC/Tools/MSVC/14.51.36231/bin/Hostx64/x64
   INCLUDE = archive/msvc/VC/Tools/MSVC/14.51.36231/include;...
   LIB     = archive/msvc/VC/Tools/MSVC/14.51.36231/lib/x64;...
   ```
   (See `docs/docker-podman.md` §7.2.) Or just run cargo via the helper
   in `scripts/` if you add one.
3. Pin the toolchain: Rust 1.99.1, edition 2021. Don't let it "upgrade".
4. Ignore `archive/`, `target/`, `node_modules/` in the agent's context
   (they're huge and not part of the active crate).

## 3. Token-saving playbook

| Do | Don't |
|---|---|
| Ask for one function/screen/module | Ask for "the whole security module" |
| Paste the failing compiler error only | Paste 200 lines of cargo output |
| Reference `AGENTS.md` sections by name | Re-explain the repo layout |
| Use `scripts/commit.ps1 -Message "..."` | Hand-write commit + trailers |
| Let the agent grep (`rg`) for symbols | Paste file contents |
| Scope to one `cfg(target_os)` at a time | Ask for "all platforms at once" |
| Reuse `src/patterns/*` and `src/security/*` | Reimplement crypto/validation |
| Ask "is X already done?" before coding | Assume and duplicate |

## 4. Context budget per task type

- **Bug fix:** ~2 files (the file + its test). ~5–10k tokens.
- **New screen:** `src/ui/<name>.rs` + `src/app.rs` hook + `CurrentScreen`
  variant. ~15–25k tokens.
- **New backend endpoint:** `src/server/*_handlers.rs` + `src/backend/rest.rs`
  + `src/models.rs`. ~20–35k tokens.
- **New platform feature (mobile):** `src/platform.rs` + `src/mobile/` +
  manifest. ~25–40k tokens (and needs NDK/Xcode to verify — do locally).
- **Security change:** read `docs/security-architecture.md` + the specific
  `src/security/*.rs`. ~20–30k tokens. Always run `cargo audit`.

If a task exceeds ~40k tokens, split it.

## 5. Recommended task order (cheapest first)

1. **Desktop polish** (no toolchain deps): UI screens, i18n strings,
   bookmarks, tilepacks. Verifiable with `cargo run`.
2. **Server handlers** (in-process tests): `router_probe.rs` pattern.
   Verifiable with `cargo test` on this host.
3. **Security modules** (unit tests): `src/security/*`. Verifiable here.
4. **Mobile scaffolding** (code only): compiles on host via cfg-gating.
   Real builds need NDK/Xcode — defer to CI.
5. **Docs** (no build): always cheap.

## 6. Mobile apps (Kotlin + Rust) — dual strategy

- **Kotlin/Android** (`mobile/`): builds with JDK 17 + Android SDK on
  this host. Start here for fast iteration.
- **Rust mobile** (`android/`, `ios/`): needs NDK/Xcode; only edit code
  here, build in CI (`android`/`macos` jobs).

### "Does the Rust one work on Windows?"

Short answer: **the Rust *source* works on Windows; the Rust *mobile
binary* does not.**

- The crate compiles on Windows as the **desktop** app (`cargo build`).
  So you can develop and test all the shared logic (models, API, security,
  UI) on Windows — that's the whole point of the multiplatform crate.
- The **Android/iOS binaries** are cross-compiled targets
  (`aarch64-linux-android`, `aarch64-apple-ios`). They need the NDK/Xcode
  and cannot be produced on Windows. There is no Windows Phone target.
- Practical workflow: develop & unit-test on Windows (desktop target),
  then build the mobile binaries on a Linux/macOS host or in CI.
- When asking Antigravity to touch mobile code, specify the target
  (`cfg(target_os = "android")`) and whether it must compile on host.

- Both share `src/backend/` + the REST v1 contract. Don't fork backend
  logic per platform — add an adapter in `src/backend/` instead.

## 7. Workflow template (copy/paste)

```
Task: <one sentence>
Files: <exact paths>
Constraint: must pass `cargo fmt/clippy/build/test --locked`
Context: <paste only the relevant function or error>
Do not: touch archive/, rewrite whole files, add deps without justification
```

Then: agent edits → you run the gate → if green, `scripts/commit.ps1 -Message "..."`.

## 8. Anti-patterns (waste tokens)

- "Make the app better" — no scope, huge context.
- Pasting entire `src/app.rs` (1100+ lines) — agent re-reads it anyway.
- Asking for mobile builds on Windows — impossible; defer to CI.
- Re-implementing crypto/validation — reuse `src/security/*`.
- Letting the agent add `Co-authored-by` trailers — now disabled
  (see `AGENTS.md`).
- Editing `archive/` — never built, never needed.

## 9. Design the app: UI, FRs, and NFRs (make it "true")

The app is only "true" (production-ready) when the UI, functional
requirements (FRs), and non-functional requirements (NFRs) are all
explicit. Tell Antigravity to design these **before** coding a feature,
and to keep them in sync. Use `docs/BEST_APP_CRITERIA.md` and
`docs/sdlc/` as the canonical checklists.

### 9.1 Functional requirements (FRs) — what the app does

For every feature, write a one-line FR with an ID and a test:

```
FR-XX  <actor> can <action> <object> under <condition>.
       Test: <unit/e2e check that proves it>
```

Examples already in this repo:
- FR-AUTH-1: A user can sign in with email+password and receive a
  session cookie. Test: `tests/contract.rs::login_parses_user...`
- FR-SHARE-2: A user can create a directional share with expiry.
  Test: `scripts/e2e.ps1` checks 11–14.
- FR-MAP-3: A user can save a place on-device (bookmarks). Test:
  `bookmarks::tests::file_store_round_trip`.

**Rule:** no FR ships without a test. If Antigravity proposes a feature
without a test, reject it.

### 9.2 Non-functional requirements (NFRs) — how the app behaves

Pinpoint's NFRs (from `docs/sdlc/03-non-functional-requirements.md` and
`docs/SECURITY.md`):

| NFR | Target | Where enforced |
|---|---|---|
| Performance | Map renders < 16 ms/frame; API p95 < 200 ms | `cargo bench` (none yet) / load test |
| Offline | All 10 screens render without network | `PinpointApp` offline mode + `PendingMutation` queue |
| Security | Argon2id (memory-hard); CSRF double-submit; Secure cookies | `src/security/*`, `src/api.rs` |
| Privacy | Friend coords never sent to OSM/Nominatim/OSRM | `src/osm.rs` (no third-party coords) |
| Portability | One Rust crate → desktop/Android/iOS/Web | `src/platform.rs`, `cfg(target_os)` |
| i18n | 79 languages, auto-detect | `src/i18n.rs` |
| Accessibility | Keyboard nav, screen-reader labels | `src/ui/*` |
| Storage | Bookmarks encrypted at rest (OS keychain) | `src/security/keychain.rs` |
| Audit | All auth/share/publish logged | `src/security/audit.rs` |
| Zero trackers | No Firebase/FCM/analytics SDKs | `mobile/`, `web/` |

**Tell Antigravity:** "Every new feature must state which NFRs it
affects and not regress them." (e.g., a new screen must render offline.)

### 9.3 UI design — screens and flows

Pinpoint has 10 screens (`CurrentScreen` in `src/app.rs`):
Map, Register, Login, Friends, Sharing, Places, Bookmarks, Trips,
Emergency, Profile, Admin, ChangePassword.

For each new screen, Antigravity must deliver:
1. A `src/ui/<name>.rs` module with `draw_<name>_screen(ui, ctx)`.
2. A `CurrentScreen::<Name>` variant + `requires_sign_in` entry.
3. A `match` arm in `src/ui.rs` (the shell).
4. Offline-safe rendering (no network in `draw_*`).
5. i18n strings via `crate::i18n::t(...)` — never hardcode English.

**UI rules for Antigravity:**
- Use egui primitives (`ui.label`, `ui.button`, `ui.group`, `ui.horizontal`).
- No platform-specific code in `src/ui.rs` (AGENTS.md rule).
- Reuse `self.draw_main_toolbar(ui)` for consistency.
- Show loading spinners via `is_loading()`; surface errors via `set_error`.
- Every interactive element needs a test in `tests/` or a manual checklist
  entry in `docs/client-platforms.md`.

### 9.4 "Make it true" checklist (production-ready)

Before calling the app "done", all must hold (from `docs/BEST_APP_CRITERIA.md`):

- [ ] A. Security: password hashing, CSRF, MFA, HTTPS, normalized DB,
      typed migrations, store convergence, tile proxy, E2E + keychain.
- [ ] B. Features: all 10 screens render offline-safe; 36/36 e2e checks.
- [ ] C. Quality: fmt 0, clippy 0, test 104+ lib + 10 contract + 4 router,
      compose healthy, e2e 38/38, no `tile.openstreetmap.org` URLs.
- [ ] D. Docs: threat model, feature inventory, final LaTeX report,
      README run/test guide.

**Tell Antigravity:** "Check off the relevant `BEST_APP_CRITERIA.md`
items in the same PR as the code change. Never ship a feature without
updating the criteria."

### 9.5 Prompt template for design work

```
Task: Design + implement <feature>.
FR: FR-XX  <one-line requirement>. Test: <check>.
NFRs affected: <list from table above>.
UI: new screen? which existing screen? wireframe in 3 bullets.
Files: <exact paths>.
Constraint: must pass cargo fmt/clippy/build/test --locked; update
  BEST_APP_CRITERIA.md checkboxes in same PR.
Do not: touch archive/, add deps without justification, hardcode English.
```

## 10. Read `AGENTS.md` first (source of truth)

Before any task, point Antigravity at `AGENTS.md` and say "treat this as
the spec". It already contains:

- The module map (where everything lives).
- The commands that must pass (`cargo fmt/clippy/build/test`).
- The gotchas (no secrets in tests, contract tests for wire changes, ADRs
  for semantic changes).
- The commit rules (owner-only, no trailers).

This single file replaces ~80% of the context you'd otherwise paste.
Antigravity should **not** re-derive the repo layout — it's already there.

## 11. Example prompts (good vs. bad)

**Bad (vague, huge scope):**
> "Make the mobile app work on Android."

**Good (scoped, paths, verifiable):**
> "In `src/mobile/android.rs`, implement `android_main` so it calls
> `launch_android_ui()`. Gate behind `cfg(target_os = "android")`. It
> won't build on Windows (no NDK) — just make sure `cargo check` on the
> host still passes. Don't add new deps."

**Bad (full file dump):**
> "Here's all of `src/app.rs` [1100 lines]. Fix the keychain wiring."

**Good (targeted):**
> "In `src/app.rs`, `PinpointApp::new` (around the `key_manager` lines),
> replace the in-memory `KeyManager::new()` with
> `keychain::open_default(app_data_dir())` + `load_or_generate()`,
> falling back to in-memory. Keep the rest unchanged. Run `cargo test`."

**Bad (no verification):**
> "Add OS keychain support."

**Good (verification included):**
> "Implement `OsKeychain` in `src/security/keychain.rs` using the
> `keyring` crate. Add a unit test that skips on platforms without a
> native store. Must pass `cargo clippy --all-targets -D warnings` and
> `cargo test`. Don't edit other files."

**Bad (cross-platform everything at once):**
> "Make GPS work on Android, iOS, and desktop."

**Good (one target at a time):**
> "Add a `GpsProvider` trait to `src/platform.rs` with a `NoFixGps`
> desktop fallback returning `None`. Mobile impls come later. Must
> compile on Windows (`cargo check`)."

## 12. Context packs (what to feed the agent)

Give Antigravity only the files relevant to the task. Suggested packs:

| Task type | Files to include |
|---|---|
| Security change | `src/security/<module>.rs`, `src/security/mod.rs`, `docs/security-architecture.md`, `docs/security/e2e-mfa-design.md` |
| New screen | `src/ui/<screen>.rs`, `src/app.rs` (state + screen enum), `src/ui.rs` (module decl), `src/models.rs` |
| New backend endpoint | `src/server/*_handlers.rs`, `src/backend/rest.rs`, `src/backend/mod.rs`, `src/models.rs`, `tests/router_probe.rs` |
| Mobile change | `src/mobile/<os>.rs`, `src/mobile/mod.rs`, `src/platform.rs`, `android/AndroidManifest.xml` or `ios/Info.plist` |
| Wire contract change | `src/api.rs`, `tests/contract.rs`, `docs/adr/` (new ADR required) |
| i18n | `src/i18n.rs` (the `Lang` enum + the matching block) |
| Docs only | the specific doc + `AGENTS.md` (for conventions) |

Never include `archive/`, `target/`, or `Cargo.lock` unless the task is
dependency-related.

## 13. Failure-recovery recipes

| Symptom | Fix |
|---|---|
| `cargo fmt --check` fails | Run `cargo fmt --all`, re-check. Don't hand-format. |
| `clippy -D warnings` (dead_code) | Remove or `#[allow(dead_code)]` with a reason. |
| `clippy` (unused import) | Remove the import; don't `#[allow]`. |
| `build --locked` fails (lock out of sync) | Run `cargo update` (updates lock), then `--locked`. |
| `cl.exe not found` (Windows) | Set `PATH`/`INCLUDE`/`LIB` per `docs/docker-podman.md` §7.2. |
| `NoDefaultStore` (keyring, Linux CI) | Fall back to `FileKeychain`; the `open_default` helper does this. |
| Mobile build fails (NDK/Xcode) | Not fixable on Windows; defer to CI or a Mac/Linux host. |
| Test flake (DB) | Run at `scale: 1`; HA tests are eventually-consistent. |
| `cargo audit` advisory | Check `docs/security-architecture.md` §12; bump or accept with note. |

## 14. When to stop and ask a human

- Anything needing NDK/Xcode/macOS (mobile builds, iOS).
- Production secrets, real user data, or DB migrations on a live DB.
- Changing the wire contract (`src/api.rs` + `tests/contract.rs`) — needs
  an ADR (`docs/adr/`).
- Anything the gate can't verify locally.

## 15. Quick reference

| Need | Command |
|---|---|
| Format | `cargo fmt --all` |
| Lint | `cargo clippy --all-targets -- -D warnings` |
| Build | `cargo build --locked` |
| Test | `cargo test --locked` |
| Commit | `scripts/commit.ps1 -Message "feat(x): ..."` |
| Desktop run | `cargo run -- --backend http://localhost:8000` |

Keep tasks small, context tight, and the gate green. That's the whole game.
