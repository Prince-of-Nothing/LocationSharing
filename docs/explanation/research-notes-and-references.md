# Research Notes and References

## Research areas resolved/tracked before implementation

- OpenStreetMap attribution, tile usage, geocoding, and routing-provider
  licenses.
- Android and iPhone background-location policies and review requirements.
- Privacy law, data deletion, consent, children's safety, and
  emergency-service disclaimers.
- Geofencing accuracy, battery behavior, indoor positioning, and offline
  maps.
- Moderation design for random groups, creator groups, media, and calls.
- Secure messaging, notification privacy, and metadata minimization.
- Coding-prompt safety, sandboxed previews, and media/file handling if user
  submissions include executable or interactive content.
- Accessibility standards and device/network coverage.

## Sources consulted

- OpenStreetMap Foundation, Tile Usage Policy:
  <https://operations.osmfoundation.org/policies/tiles/>
- OpenStreetMap Foundation, Nominatim Usage Policy:
  <https://operations.osmfoundation.org/policies/nominatim/>
- Android Developers, Request location permissions:
  <https://developer.android.com/develop/sensors-and-location/location/permissions>
- Android Developers, Access location in the background:
  <https://developer.android.com/develop/sensors-and-location/location/background>
- Apple Developer, Requesting authorization to use location services:
  <https://developer.apple.com/documentation/corelocation/requesting-authorization-to-use-location-services>
- Apple Developer, Handling location updates in the background:
  <https://developer.apple.com/documentation/corelocation/handling-location-updates-in-the-background>
- IETF RFC 6749, OAuth 2.0 Authorization Framework:
  <https://www.rfc-editor.org/rfc/rfc6749>
- IETF RFC 7636, Proof Key for Code Exchange:
  <https://www.rfc-editor.org/rfc/rfc7636>
- MDN, Content-Security-Policy:
  <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Security-Policy>
- MDN, Set-Cookie:
  <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Set-Cookie>
- OWASP Application Security Verification Standard:
  <https://owasp.org/www-project-application-security-verification-standard/>
- OWASP API Security Top 10: <https://owasp.org/API-Security/>

## Research conclusions

- The public OSM tile and Nominatim services are suitable for careful
  prototypes, not an assumed production SLA or offline-map backend.
- Android and iOS background location require explicit user-facing
  justification, platform-specific permissions, and battery-aware behavior.
- Mobile clients are public OAuth clients; authorization code plus PKCE is
  the appropriate baseline.
- A strict web security policy matters for any web companion, but native
  mobile authorization and secure storage remain separate concerns.

Record decisions, sources, assumptions, and unresolved risks here as
research progresses.

---
*See also [§2 Constraints](../architecture/02-constraints.md) and
[Technology Stack and Infrastructure](../reference/technology-stack-and-infrastructure.md)
for how this research shaped the frozen stack.*
