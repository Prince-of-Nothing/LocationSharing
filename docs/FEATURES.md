# Pinpoint — Feature Reference (v2.0, 2026-11-08)

Complete reference of all Pinpoint features, organized by access level
(signed-out vs. signed-in) and by area (map, friends, sharing, places, trips, emergency, admin).

> v2.0 adds: share expiry + delete, unfriend, user search, my-location endpoint,
> check-ins (trips), emergency panic alerts, profile/units, geo helpers
> (haversine + bearing + formatting), OSRM routing/Overpass POI/reverse-geocode
> wiring, on-device bookmarks UI, map distance/compass display.

---

## Map Features (Available Without Sign-In)

The map is the landing screen and works immediately when you open the app.
No account is required.

### Interactive Map

- **Friend location pins** — Demo locations are shown on the map with
  precision indicators (exact/approximate/city_only).
- **Precision color coding** — Green (exact), yellow (approximate), red
  (city_only), gray (unknown).
- **Freshness indicators** — Each location shows how recently it was
  updated.
- **Location details** — Latitude, longitude, and accuracy are shown for
  each friend.

### Navigation

- **Navigate to friend** — Open OpenStreetMap directions to any friend's
  location in your default browser.
- **Open on OSM** — Open any location on OpenStreetMap.org at zoom level 14.
- **Directions links** — `https://www.openstreetmap.org/directions?to=…`
- **Place links** — `https://www.openstreetmap.org/#map=14/lat/lon`

### Search (OpenStreetMap / Nominatim)

- **Search by name** — Type any place name (café, pharmacy, park, city).
- **Debounced input** — Search triggers after you stop typing (350ms).
- **Rate-limited** — Maximum 1 request per second (Nominatim policy).
- **Results list** — Shows place name and coordinates.
- **Navigate from results** — Click "Navigate" or "OSM Map" on any result.

### Location Publish (Input Only)

- **Latitude/Longitude inputs** — Enter coordinates manually.
- **Publish button** — Sends location to the server (sign-in required;
  shows error if not signed in).

### Offline Mode

- **Demo locations** — Three example friends are always visible offline.
- **Offline banner** — Shown when the backend is unreachable.
- **Graceful degradation** — Map and search work; publishing/sharing
  requires server.

---

## Friends Features (Requires Sign-In)

After signing in, click **Friends** in the navigation bar.

### Friend Requests

- **Send request** — Enter a friend's email and send a friendship request.
- **Receive requests** — Incoming requests appear under "Received Requests".
- **Accept/Reject** — Accept or reject incoming requests individually.
- **Sent requests** — View requests you've sent (status: awaiting response).
- **Sent requests list** — Shows friend's name and "awaiting response" status.

### Friend List

- **Accepted friends** — Shown with green status indicator.
- **Pending friends** — Shown with yellow status indicator.
- **Rejected friends** — Shown with gray status indicator.

---

## Location Sharing Features (Requires Sign-In)

After signing in and having at least one accepted friendship, click **Sharing**.

### Creating a Share

- **Select friend** — Choose from accepted friendships.
- **Direction:**
  - `to_friend` — They see your location.
  - `from_friend` — You see their location.
  - `mutual` — Both see each other's location.
- **Precision:**
  - `exact` — Full GPS coordinates.
  - `approximate` — Rounded to ~1.1 km (2 decimal places).
  - `city_only` — Rounded to ~11 km (1 decimal place).
- **Create share** — Activates the share immediately.

### Managing Shares

- **Pause** — Temporarily stop sharing without deleting the share.
- **Resume** — Re-enable a paused share.
- **Status indicator** — Shows ACTIVE or PAUSED for each share.
- **Precision badge** — Color-coded precision level (green/yellow/red).
- **Last update** — Shows when the location was last updated.

### Precision Enforcement

All precision downgrades happen **server-side**. The client cannot bypass
precision settings. Even if you modify the client, the server enforces the
share's precision level on every read.

---

## Admin Features (Requires Admin Role)

If your account has the `administrator` role, click **Admin** in the
navigation bar.

- **Admin actions** — Placeholder for admin panel (server-side implementation).
- **User management** — Served by the admin API (not implemented in client).
- **Access control** — Non-admin users see "Admin access required".

---

## Password Features (Requires Sign-In)

Click **Password** in the navigation bar.

### Changing Your Password

- **Current password** — Required for regular password changes.
- **New password** — Minimum 8 characters.
- **Confirm password** — Must match the new password.
- **Update** — Changes password and revokes all other sessions.

### Forced Password Change

- **Temporary passwords** — Admin-provided temporary passwords require
  change on first sign-in.
- **Forced change dialog** — Appears automatically; current password field
  is hidden.
- **Session revocation** — All other sessions are revoked after a regular
  password change.

---

## Security Features

### Transport Security

- **HTTPS required** for production backends.
- **Plain HTTP only for loopback** (`localhost`, `127.0.0.1`, `::1`).
- **ADR-0006** — Fail-closed transport policy.

### Session Security

- **HttpOnly cookies** — Session tokens are not accessible via JavaScript.
- **SameSite=Lax** — Cookies are not sent on cross-site requests.
- **CSRF double-submit** — Token in cookie + header on unsafe methods.
- **Session rotation** — New CSRF token on every authenticated response.

### Authorization

- **Server-side enforcement** — All consent decisions made on the server.
- **No client bypass** — The client cannot access data without server
  authorization.
- **IDOR protection** — Accessing another user's data returns 403/404.

### Privacy

- **0 trackers** — No analytics, crash reporting, or advertising SDKs.
- **No data scraping** — Demo locations are example data only.
- **Consent-based** — Location sharing requires explicit, revocable consent.
- **Latest-only storage** — Server stores only the most recent location,
  not history.

---

## Offline Features

| Feature | Works Offline |
|---------|---------------|
| Map view (demo locations) | ✅ |
| Map navigation links | ⚠️ Requires browser + internet |
| Place search | ❌ Requires Nominatim |
| Location publishing | ❌ Requires server |
| Friend management | ❌ Requires server |
| Location sharing | ❌ Requires server |
| Password change | ❌ Requires server |

---

## What's NOT in Pinpoint

The following features are explicitly **not** part of Pinpoint:

- **No real-time tracking** — Locations are published on-demand, not
  streamed continuously.
- **No location history** — Only the latest location is stored.
- **No messaging/calls** — Pinpoint is for location sharing only.
- **No public discovery** — You can only see friends who have consented.
- **No advertising** — Pinpoint does not show ads or sell data.
- **No tracking SDKs** — No Firebase, FCM, Sentry, or analytics.
- **No bulk data scraping** — OSM services are queried sparingly and only
  with user input.

---

## Roadmap (Planned Features)

The following features are planned for future releases:

- **Realtime updates** — WebSocket-based location updates.
- **Check-ins** — Scheduled and trip check-ins with missed-check-in alerts.
- **Emergency mode** — Panic share to preselected contacts.
- **Offline map packs** — Download map regions for offline use.
- **Offline routing** — GraphHopper/Valhalla integration for offline
  turn-by-turn navigation.
- **E2E encryption** — Optional end-to-end encryption for location payloads.

Each planned feature includes its own threat analysis before
implementation.

---
