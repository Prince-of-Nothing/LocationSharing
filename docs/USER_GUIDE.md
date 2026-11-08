# Pinpoint — User Guide

Pinpoint is a privacy-first, consent-based location-sharing app. The map is
your home screen — it works immediately, offline, without signing in. Signing
in unlocks additional features: friends, location sharing, and admin tools.

This guide walks you through every feature, from opening the map for the first
time to navigating to a friend's location.

---

## 1. First Launch — The Map

When you start Pinpoint, you see the **map** immediately. No login required.

### What you see on the map

- **Demo friend locations** — Three example friends (Alice, Bob, Carol) are
  shown around Chicago with their precision levels (exact, approximate, city
  only). These are demonstration data only; no real user data is loaded.
- **Publish my location** — Enter latitude/longitude and click Publish to
  share your current location (requires sign-in).
- **Navigate to a friend** — Click "Navigate" next to any friend to open
  OpenStreetMap directions in your default browser.
- **OSM Map** — Click "OSM Map" to open the location on OpenStreetMap.org.
- **Search places** — Search for cafés, pharmacies, parks, or any place using
  OpenStreetMap's Nominatim service. Click "Navigate" on any result to open
  directions.

### Map controls

| Button | What it does |
|--------|--------------|
| Navigate | Opens `https://www.openstreetmap.org/directions?to=…` in your browser |
| OSM Map | Opens `https://www.openstreetmap.org/#map=…` centered on the location |
| Search | Queries Nominatim for places matching your text |
| Publish | Sends your lat/lon to the server (sign-in required) |

---

## 2. Sign In — Unlock Additional Features

The map works fully offline without an account. When you're ready to share
your location with friends, click **Sign in** in the top-right corner.

### What signing in gives you

| Feature | Without sign-in | With sign-in |
|---------|----------------|--------------|
| View map & demo locations | ✅ | ✅ |
| Search places | ✅ | ✅ |
| Navigate to any location | ✅ | ✅ |
| Publish your location | ❌ | ✅ |
| Send/accept friend requests | ❌ | ✅ |
| Create location shares | ❌ | ✅ |
| Admin panel (admin role) | ❌ | ✅ |
| Change password | ❌ | ✅ |

### How to sign in

1. Click **Sign in** in the header (top-right).
2. Enter your email and password.
3. Click **Sign in**.

If you don't have an account, click **Register** on the sign-in screen and
create one.

---

## 3. Publishing Your Location

Once signed in, you can publish your location from the map screen:

1. Enter your latitude (e.g. `41.8902`) in the "Latitude" field.
2. Enter your longitude (e.g. `-87.6323`) in the "Longitude" field.
3. Click **Publish**.

Your location is shared with friends who have an active location share with
you. Precision (exact/approximate/city_only) is controlled by the share
settings — you don't choose precision when publishing; the server applies the
share's precision setting automatically.

---

## 4. Friends

After signing in, click **Friends** in the bottom navigation bar.

### Sending a friend request

1. Enter your friend's email in the "Add Friend" field.
2. Click **Send Request**.
3. Your friend receives a pending request they can accept or reject.

### Responding to requests

If someone sends you a request, it appears under **Received Requests** with
**Accept** and **Reject** buttons.

### Sent requests

Requests you've sent appear under **Sent Requests** with status
"awaiting response" until the other person acts.

---

## 5. Location Sharing

After signing in and having at least one accepted friendship, click **Sharing**.

### Creating a share

1. Select a friend from the "Share with" dropdown (only accepted
   friendships appear).
2. Choose direction:
   - **to_friend** — They can see your location.
   - **from_friend** — You can see their location.
   - **mutual** — Both can see each other's location.
3. Choose precision:
   - **Exact** — GPS-level coordinates.
   - **Approximate** — Neighborhood-level (~250m).
   - **City only** — City-level (~11km).
4. Click **Create Share**.

### Managing shares

Each active share shows **Pause** or **Resume**. Pausing a share stops
location updates without deleting the share. Resuming re-enables updates.

---

## 6. Admin Panel

If your account has the `administrator` role, click **Admin** in the
navigation bar. The admin panel shows admin-only actions. User management is
served by the admin API on the server.

---

## 7. Change Password

Click **Password** in the navigation bar to change your password. If your
account has a temporary password (set by an admin), you'll be prompted to
change it on first sign-in.

---

## 8. Offline Mode

Pinpoint works without a server connection:

- The map displays demo friend locations.
- Search queries to Nominatim require an internet connection.
- Navigation links open in your browser (requires internet for the maps
  site).
- Publishing, friend requests, and shares require a server connection.

When the backend is unreachable, a banner says **"Offline mode"** in the
header. The app will retry server requests automatically when you perform
actions that need the server.

---

## 9. Privacy & Security

- **No data scraping**: Pinpoint never scrapes or collects real user data
  without consent. Demo locations are example data only.
- **Consent-based**: Location sharing requires explicit, revocable consent
  per friend. You control precision and can pause/resume at any time.
- **Server-side enforcement**: All authorization decisions are made on the
  server. The client cannot bypass consent checks.
- **Session security**: Session cookies are HttpOnly and SameSite=Lax. CSRF
  double-submit protects against cross-site request forgery.
- **Plain HTTP only for loopback**: Development HTTP (`http://localhost`,
  `http://127.0.0.1`) is allowed. Production requires HTTPS.

---

## 10. Keyboard & Accessibility

- Tab through fields and buttons.
- Enter submits the focused form (sign in, register, publish, search).
- All buttons have clear labels and tooltips.
- Colors are supplemented with text labels (not color-only).

---

## 11. Quick Reference

```
Map first page        → Open app, see map immediately
Sign in               → Click "Sign in" in header
Publish location      → Map screen → enter lat/lon → Publish
Add friend            → Friends screen → enter email → Send Request
Create share          → Sharing screen → select friend → Create Share
Navigate to friend    → Map screen → click "Navigate" next to friend
Search places         → Map screen → enter query → Search → click result
Change password       → Password screen → enter new password
Sign out              → Click "Sign out" in header
```

---

## 12. Troubleshooting (Quick)

| Problem | Fix |
|---------|-----|
| Map won't load | Check your internet connection; demo locations render offline |
| Sign in fails | Verify email/password; ensure backend is running |
| Publish fails | Sign in first; check backend URL |
| Search returns nothing | Check internet; try a simpler query (e.g. "café") |
| Navigate button does nothing | Your browser may be blocking pop-ups; allow pop-ups for this app |
| "Offline mode" banner | Backend unreachable; map still works, publishing/sharing disabled |

For detailed troubleshooting, see `docs/TROUBLESHOOTING.md`.

---
