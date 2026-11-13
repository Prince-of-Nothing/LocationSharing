# 07 — Competitive & Similar-App Analysis

> **Status:** living analysis. Sources are public product/documentation pages
> (see §8). Details may change; verify before citing in the thesis.

## 1. Purpose

Pinpoint is not just "another map app" — it is a **consent-based location-sharing
platform**. This document analyses similar apps, extracts what they do well and
badly, and turns that into concrete documentation + roadmap improvements.

## 2. Apps analysed

| Category | Apps |
|---|---|
| Location sharing | Google Maps Location Sharing, Apple Find My, Life360, Glympse, WhatsApp Live Location, Telegram Live Location, Strava Beacon, GeoZilla, Zenly (defunct) |
| Offline / OSM maps | Organic Maps, OsmAnd, Magic Earth |
| Reference privacy models | Apple iCloud E2E / Advanced Data Protection |

## 3. Feature comparison matrix

| App | Consent model | Precision control | Expiry | E2E location | Offline maps | Trackers / ads | Account required | Open source |
|---|---|---|---|---|---|---|---|---|
| **Pinpoint** | **Directional, per-recipient** | **exact / approximate / city_only** | **Yes (per share)** | **Yes (AES-256-GCM + OS keychain)** | **Yes (tile packs)** | **None (0 trackers, no ads)** | Yes | **Yes (Rust + Kotlin)** |
| Google Maps LS | Symmetric (share with people) | No | "Until you turn it off" | No | No (cached areas only) | Yes (ads/tracking) | Yes (Google) | No |
| Apple Find My | Symmetric (contacts) | No | Until stopped | Yes (Find My / ADP) | No | No (Apple) | Yes (Apple) | No |
| Life360 | Circle (always-on) | No | Until stopped | No | No | **Yes (ads + partner data sharing)** | Yes | No |
| Glympse | Temporary link | No | **Yes (timer)** | No | No | Enterprise SDK | No (recipients) | No |
| WhatsApp Live Location | Temporary broadcast | No | 15m / 1h / 8h | Yes (message E2E) | No | Meta ecosystem | Yes | No |
| Telegram Live Location | Temporary broadcast | No | Timer | Only in Secret Chats | No | Telegram ecosystem | Yes | Client only |
| Strava Beacon | Activity-scoped | No | During activity | No | No | Yes (fitness social) | Yes | No |
| Organic Maps | n/a (maps only) | n/a | n/a | n/a | **Yes (full offline)** | **None** | **No** | **Yes** |
| OsmAnd | n/a (maps only) | n/a | n/a | n/a | **Yes (full offline)** | **None** | **No** | **Yes** |

## 4. Consent & privacy models

### 4.1 Consent models

- **Circle / always-on (Life360, Find My, Google Maps):** joining a circle or
  sharing "until stopped" grants persistent access. Simple, but weak consent —
  no per-recipient precision, no automatic expiry.
- **Temporary broadcast (Glympse, WhatsApp, Telegram, Strava Beacon):**
  time-boxed sharing, but usually **symmetric** and **all-or-nothing** (one
  precision for everyone).
- **Directional / per-recipient (Pinpoint):** each (sharer → recipient) edge
  has its own direction, precision, purpose, expiry, pause/revoke. This is the
  strongest consent model and Pinpoint's core differentiator.

### 4.2 Data practices (the honest comparison)

- **Life360** collects precise geolocation, driving events, motion activity,
  device/advertising identifiers and tracking technologies; it discloses
  precise geolocation and movement data to **business partners for their own
  monetization** (with opt-outs) and uses data for ad targeting. It is the
  canonical example of the data-harvesting model Pinpoint rejects.
- **Apple** keeps keys for standard iCloud data in its data centres; **Advanced
  Data Protection** moves most categories (including Maps pins/guides/search
  history) to **end-to-end encryption with keys on trusted devices** — the model
  Pinpoint's E2E + OS keychain follows.
- **Organic Maps / OsmAnd** are the reference for "privacy-first offline maps":
  no account, no trackers, no ads, data on device, OSM data, open source.

### 4.3 What Pinpoint does better

1. **Directional consent with per-recipient precision** — no mainstream
   location-sharing app offers this.
2. **E2E location encryption + OS keychain** — server stores ciphertext only;
   matches Apple's ADP model, unlike Life360/Google.
3. **0 trackers / 0 ads / OSM-only** — matches Organic Maps/OsmAnd, unlike
   Life360.
4. **Expiry + pause/resume/revoke** as first-class share properties.
5. **Open source** (Rust + Kotlin) — auditable, unlike the mainstream apps.
6. **ODbL-compliant tile policy** — never `tile.openstreetmap.org`.

## 5. Architecture patterns worth borrowing

| Pattern | From | Value for Pinpoint |
|---|---|---|
| Temporary share **link** (no account for recipients) | Glympse | Lets a user share with non-users; big usability win |
| Real-time streaming (WebSocket/SSE) | Life360, Google Maps | Polling (≤30s) feels stale vs. competitors |
| Battery-aware sampling | Organic Maps, OsmAnd | Preserve battery; adapt refresh to battery level |
| Optional E2E-encrypted cloud sync for bookmarks | OsmAnd Cloud | Cross-device bookmarks without server plaintext |
| Offline routing engine | OsmAnd, Organic Maps | Real offline navigation (currently hand-off only) |
| Public-transport routing | OsmAnd | OSRM supports transit in some deployments |
| OSM editing from the app | Organic Maps, OsmAnd | Community contribution; nice-to-have |
| 3D terrain / elevation profiles | OsmAnd | Outdoor use cases |

## 6. Gaps in Pinpoint (honest)

1. **No real-time streaming** — polling only; competitors feel live.
2. **No account-free sharing** — Glympse-style links would broaden reach.
3. **No offline routing engine** — currently hand-off to OsmAnd/Organic Maps.
4. **No battery-aware sampling** — refresh interval is manual only.
5. **No optional cross-device bookmark sync** — bookmarks are device-local.
6. **No public-transport / 3D terrain / OSM editing** — map-depth gap vs. OsmAnd.
7. **No crash detection / safety services** — intentionally out of scope
   (privacy-first), but worth documenting as a deliberate trade-off.

## 7. Recommendations (prioritised)

### P1 — do before the thesis demo / report

1. **Document the competitive positioning** (this doc) and reference it from
   `docs/sdlc/00-overview.md` and `docs/PRIVACY.md`.
2. **Add a "Related work / competitive context" section** to the SDLC set
   (short, pointing here).
3. **Add a threat-model note** contrasting Pinpoint's data minimisation with
   Life360-style harvesting (defence-in-depth narrative).

### P2 — high-value features (post-MVP / stretch)

4. **Temporary share link** (Glympse-style): a revocable, expiring link that
   shows a coarse location without an account. Aligns with the consent model.
5. **Real-time streaming option** (WebSocket/SSE) with the existing poll as
   fallback.

### P3 — nice-to-have

6. Battery-aware sampling (adapt refresh interval to battery level).
7. Optional E2E-encrypted bookmark sync.
8. Public-transport routing; 3D terrain; OSM editing.

## 8. Sources

- Organic Maps — https://organicmaps.app/ (features, privacy, tracker-free
  claims, Exodus Privacy verification).
- OsmAnd — https://osmand.net/ (offline maps/navigation, no account, open
  source, OsmAnd Cloud).
- Apple — https://support.apple.com/en-us/102651 (iCloud data security
  overview; end-to-end vs. standard data protection; Advanced Data Protection).
- Life360 — https://life360.com/privacy/ (data collection, Circle sharing,
  business-partner disclosures, advertising, retention).
- Product documentation for Google Maps Location Sharing, Find My, Glympse,
  WhatsApp/Telegram Live Location, Strava Beacon (public help pages).

> Note: competitor capabilities evolve; re-verify any figure before publishing
> it in the thesis.
