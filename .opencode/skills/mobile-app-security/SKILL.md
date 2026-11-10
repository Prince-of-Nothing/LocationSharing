---
name: mobile-app-security
description: "Use when hardening an iOS, Android, React Native, Expo or Flutter app against OWASP MASVS and MASTG findings — secure storage with Keychain accessibility classes, Android Keystore and expo-secure-store instead of AsyncStorage or MMKV for tokens, short-lived access tokens with rotating refresh tokens and PKCE, TLS enforcement and SPKI certificate pinning (and when pinning causes outages), jailbreak, root and emulator detection trade-offs, R8/ProGuard and Hermes bytecode obfuscation and what it does not hide, keeping secrets out of the bundle with a backend plus App Attest and Play Integrity attestation, Universal Links, App Links and intent hijacking, WebView hardening, clipboard and screenshot protection, biometric gating bound to Keystore or Keychain keys, log scrubbing, dependency scanning, and release checklists with MobSF."
license: MIT
---

# Mobile App Security

Makes the agent harden mobile apps along the OWASP MASVS control groups instead of a random
checklist: secrets never ship in the binary, tokens live in hardware-backed storage with the
right accessibility class, transport is pinned only where it will not brick the app, platform
entry points (links, intents, WebViews) are allowlisted, and the release build is scanned before
it reaches a store.

## When to Use This Skill
- A security review or pentest is scheduled for a mobile app
- Deciding where to store tokens, keys and user data on device
- Adding OAuth, biometrics or certificate pinning to a React Native, Expo, Swift or Kotlin app
- A secret was found in a decompiled APK or extracted IPA
- Deep links or WebViews drive authentication or payments
- Preparing a store release and needing a go/no-go list

Face ID and fingerprint flows: `biometric-auth`; link handling and routing: `deep-links`; token
format and validation: `jwt-handling`; server-side sessions and refresh: `session-management`;
web-side counterparts: `owasp-top-10`.

## Workflow
1. Classify the app against MASVS: L1 (every app), L2 (sensitive data, payments, health), R (resilience for apps that must resist tampering).
2. Inventory data at rest: every token, key, PII field and cache; move each to the storage class in the table below.
3. Inventory data in transit: enforce TLS, remove cleartext exceptions, decide pinning per host.
4. Inventory entry points: deep links, exported activities, WebViews, clipboard, screenshots; allowlist and validate each.
5. Remove every secret from the bundle; route privileged calls through a backend guarded by attestation.
6. Scan the release artifact (MobSF, dependency scanners), fix findings, and record the checklist result with the build number.

## MASVS Map
| Group | Covers | First actions |
|---|---|---|
| MASVS-STORAGE | Data at rest, backups, logs, clipboard | Keychain/Keystore for secrets, `allowBackup="false"`, scrub logs |
| MASVS-CRYPTO | Key management, algorithms | Platform crypto only, keys generated in Keystore/Secure Enclave |
| MASVS-AUTH | Local and remote auth, session | PKCE, short tokens, biometric bound to keys |
| MASVS-NETWORK | TLS, pinning | ATS and Network Security Config, SPKI pins with backups |
| MASVS-PLATFORM | IPC, WebViews, links, UI | Non-exported components, verified links, hardened WebView |
| MASVS-CODE | Updates, dependencies, debug | R8, no `debuggable`, dependency scanning, signed OTA updates |
| MASVS-RESILIENCE | Tampering, reversing, hooking | Attestation, integrity checks as signals, obfuscation |
| MASVS-PRIVACY | Data minimization, consent, tracking | Collect less, honor ATT and permissions, delete on logout |

## Secure Storage
| Store | Protection | Use for |
|---|---|---|
| iOS Keychain, `kSecAttrAccessibleWhenUnlockedThisDeviceOnly` | Secure Enclave backed, not in backups, not restorable to another device | Refresh tokens, keys |
| iOS Keychain, `kSecAttrAccessibleAfterFirstUnlock` | Available to background tasks after first unlock | Push-related keys needed in background |
| Android Keystore key + AES-GCM over a file or DataStore | Hardware-backed key, non-exportable | Refresh tokens, keys (Jetpack `EncryptedSharedPreferences` is deprecated; Tink or your own GCM wrapper over a Keystore key replaces it) |
| `expo-secure-store` / `react-native-keychain` | Wraps the two above | React Native and Expo apps; `expo-secure-store` caps values at 2048 bytes |
| `AsyncStorage`, plain MMKV, SQLite, files | None beyond app sandbox; readable on rooted devices and in backups | Preferences, caches, nothing secret |

```ts
import * as SecureStore from "expo-secure-store";

await SecureStore.setItemAsync("refresh_token", token, {
  keychainAccessible: SecureStore.WHEN_UNLOCKED_THIS_DEVICE_ONLY,
  requireAuthentication: false,            // true forces biometric/passcode on read (see Biometric Gating)
});
const token = await SecureStore.getItemAsync("refresh_token");
await SecureStore.deleteItemAsync("refresh_token");     // on logout and on 401 from the refresh endpoint
```
Encrypted MMKV or SQLCipher are fine for bulk data when the key itself lives in the Keychain or
Keystore. Set `android:allowBackup="false"` (or a `fullBackupContent` rule excluding secrets).

## Tokens and Sessions
- Use the OAuth authorization code flow with PKCE through the system browser (`ASWebAuthenticationSession`, Custom Tabs; `expo-auth-session` or AppAuth). No implicit flow, no password grant, no embedded WebView login.
- Access tokens live in memory only and expire in minutes; the refresh token goes to secure storage, rotates on every use, and the server revokes the whole family on reuse.
- Redirect URIs must be claimed `https` links (Universal Links / App Links); a custom scheme redirect can be claimed by another app.

## Transport Security and Pinning
Enforce TLS first: iOS App Transport Security stays on with no `NSAllowsArbitraryLoads`; Android
`android:usesCleartextTraffic="false"` plus a Network Security Config. Pin only hosts you operate:
```xml
<!-- res/xml/network_security_config.xml -->
<network-security-config>
  <domain-config cleartextTrafficPermitted="false">
    <domain includeSubdomains="true">api.example.com</domain>
    <pin-set expiration="2027-06-01">
      <pin digest="SHA-256">base64-spki-hash-of-current-intermediate</pin>
      <pin digest="SHA-256">base64-spki-hash-of-backup-key</pin>
    </pin-set>
  </domain-config>
</network-security-config>
```
iOS: `NSPinnedDomains` in `Info.plist` with `NSPinnedCAIdentities` or `NSPinnedLeafIdentities`
(SPKI SHA-256), or TrustKit; React Native: `react-native-ssl-pinning` or configure OkHttp's
`CertificatePinner` and the iOS plist in native code.

When pinning hurts: certificates rotate (90-day ACME certs make leaf pins unusable), CDNs and
third-party APIs change chains without notice, and a bad pin set is an outage you can only fix
through a store release. Pin the intermediate or your own CA's SPKI, always ship a backup pin
for a key you hold offline, set `expiration` so pins fail open instead of bricking old builds,
and keep a remote kill switch. Skip pinning for hosts you do not control.

## Device Integrity: Detection and Attestation
- Root/jailbreak and emulator checks (`jail-monkey`, RootBeer, Cydia path and `cydia://` probes, Frida port scans) are bypassable in minutes with Frida or Magisk hiding, and they produce false positives on custom ROMs and developer devices. Use them as a risk signal sent to the server, not as a hard block, unless regulation demands it.
- Attestation moves the decision server-side. iOS App Attest (`DCAppAttestService`): `attestKey` with a server challenge, then `generateAssertion` on sensitive requests. Android Play Integrity (`IntegrityManager`): request a token with a server nonce, decrypt the verdict server-side, require `MEETS_DEVICE_INTEGRITY`. Firebase App Check wraps both.

## Secrets, Obfuscation and the Bundle
Everything in the app is readable: `apktool d app.apk`, `unzip app.ipa`, `strings` on the
Hermes bundle, and `hermes-dec` for bytecode. `EXPO_PUBLIC_*`, `react-native-config`, `.env`
files bundled at build time, and `Info.plist` values are all public. Consequences:
- Third-party API keys that bill you or grant access (payments, LLMs, maps with billing) belong on a backend endpoint that authenticates the user and, for abuse-prone calls, checks attestation. Keys that are meant to be public (Maps browser key with referrer restrictions, Firebase config) can ship, restricted by bundle id and SHA-256 fingerprint.
- R8 (`minifyEnabled true`, `shrinkResources true`) and ProGuard rules shrink and rename Java/Kotlin symbols; upload `mapping.txt` to your crash reporter. Hermes compiles JS to bytecode, which hides source but not string constants or control flow. Neither stops a determined reverser; they raise effort and cost, which is their only promise.

## Links, Intents and WebViews
- Register Universal Links (`apple-app-site-association`, Associated Domains) and Android App Links (`android:autoVerify="true"`, `/.well-known/assetlinks.json`); a bare custom scheme can be registered by any app and will receive your OAuth code or password-reset token.
- Validate every deep link parameter as untrusted input; never accept a session token, redirect URL or price from a link. Open the screen, then fetch state from the API.
- Android: every `Activity`, `Service`, `BroadcastReceiver` is `android:exported="false"` unless it must be an entry point; use explicit intents for internal navigation; `PendingIntent` with `FLAG_IMMUTABLE`; no sensitive data in implicit intents or sticky broadcasts.
- WebView: load only your own origins (React Native WebView `originWhitelist={["https://app.example.com"]}` plus `onShouldStartLoadWithRequest`), `allowFileAccess={false}`, `allowUniversalAccessFromFileURLs={false}`, `mixedContentMode="never"`, no `injectedJavaScript` that forwards secrets, `addJavascriptInterface` only for https pages you serve, and `setWebContentsDebuggingEnabled(false)` in release.

## Clipboard, Screenshots and Biometric Gating
- Mark sensitive text fields `secureTextEntry` / `inputType="textPassword"`; set `ClipDescription.EXTRA_IS_SENSITIVE` when copying codes on Android, `UIPasteboard.setItems(..., options: [.expirationDate: ..., .localOnly: true])` on iOS, and clear the clipboard after an OTP is used.
- Block screenshots and screen recording on sensitive screens with `FLAG_SECURE` (`expo-screen-capture` `preventScreenCaptureAsync`); iOS cannot block them, so overlay a blur on `applicationWillResignActive` for the app switcher and log `userDidTakeScreenshotNotification` if policy requires.
- Biometric prompts that return a boolean are bypassable by hooking. Bind the check to cryptography: Android Keystore key with `setUserAuthenticationRequired(true)` and a `CryptoObject` in `BiometricPrompt`; iOS Keychain item with `kSecAccessControlBiometryCurrentSet` so a new enrolled fingerprint invalidates it. `react-native-keychain` `accessControl: BIOMETRY_CURRENT_SET` maps to these.

## Logging, Dependencies and Release Checklist
- No tokens, PII or request bodies in `console.log`, `Log.d` or `os_log`; strip console calls in release (`babel-plugin-transform-remove-console`) and scrub crash-reporter breadcrumbs (`beforeSend`/`beforeBreadcrumb`).
- Scan dependencies every build: `npm audit --omit=dev`, `osv-scanner --lockfile package-lock.json`, Gradle `dependencyCheckAnalyze`; pin lockfiles; Renovate or Dependabot for updates.
- Static scan the artifact: `docker run -p 8000:8000 opensecurity/mobile-security-framework-mobsf`, upload the release APK/IPA, triage Manifest, Permissions, Hardcoded Secrets and Network Security sections. `apkleaks` is a quick secrets pass for Android.
```text
Release gate (fail any = no ship)
[ ] android:debuggable false, allowBackup false or scoped, R8 on, mapping.txt uploaded
[ ] No NSAllowsArbitraryLoads, no cleartext traffic, pins have backup + expiration
[ ] grep of bundle for sk_live, AKIA, -----BEGIN, Bearer, api_key returns nothing
[ ] All components exported=false except launcher/link entry points; links autoVerify/AASA pass
[ ] Tokens only in Keychain/Keystore; logout clears them; refresh rotation verified
[ ] WebView allowlist, no file access, debugging off
[ ] MobSF high findings resolved; dependency scan clean or risk-accepted with owner
[ ] OTA updates code-signed; dev client and debug menus absent from the build
```

## Common Mistakes
- **Tokens in `AsyncStorage` or unencrypted MMKV**: readable on rooted devices and sometimes in backups; use the Keychain or Keystore.
- **Shipping secrets as `EXPO_PUBLIC_*` or `.env` and relying on obfuscation**: the string is one `strings` command away.
- **Pinning the leaf certificate with no backup or expiry**: the next rotation locks every user out until a store review passes.
- **Custom scheme redirects for OAuth**: another app registers the scheme and receives the code; use claimed https links.
- **Exported activities with default settings**: any app on the device can launch internal screens with crafted extras.
- **Biometric check that only returns a boolean**: a Frida hook flips it; bind the check to a key.

## Verify
- `strings` on the extracted bundle and `apkleaks` on the APK find no live keys, private keys or bearer tokens.
- On a rooted emulator, the app's data directory contains no readable token; `SecureStore` items are absent from `adb backup` output.
- Charles or mitmproxy with a user-installed CA cannot read pinned traffic in release; the same build still works after rotating to the backup key.
- An OAuth redirect to a custom scheme is rejected; the Universal Link / App Link verification endpoints return valid JSON and the OS treats the link as verified.
- `adb shell dumpsys package <pkg>` shows only the launcher and link activities as exported; a crafted intent to an internal activity is refused.
- MobSF reports no high findings; `npm audit --omit=dev` and `osv-scanner` return no critical vulnerabilities.
