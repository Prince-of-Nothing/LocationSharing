# 6. Runtime View

## 6.1 Location sharing (core MVP scenario)

See [Location Sharing Lifecycle](./location-sharing-lifecycle.md) for the full flow:
select recipient/duration → recipient accepts/declines → active sharing displayed
with freshness/accuracy/expiry → either side can stop/revoke/block/report → expiry
triggers deletion or coarse retention.

## 6.2 Control sequence for a location read

The canonical runtime scenario for authorization-sensitive reads (applies equally
to statuses, caches, friend-map layers, media, reports, and emergency context):

1. Authenticate the caller and validate the token/session.
2. Resolve the immutable caller and target IDs.
3. Check blocks and account state.
4. Check that the caller is the current authorized recipient.
5. Check directional consent, purpose, precision, expiry, and stale status.
6. Apply coarse/approximate transformation if required.
7. Redact fields not necessary for the requested view.
8. Record a privacy-preserving access event.
9. Return only the authorized representation.

See [Security Architecture and Data Protection](./security-architecture-and-data-protection.md)
for the boundary-by-boundary detail (mobile, API, realtime, database, object
storage, operator).

## 6.3 Missing-person / search-corridor activation (deferred feature)

See [Movement Trails and Search Corridors §"Missing-person workflow"](./movement-trails-and-search-corridors.md#missing-person-workflow)
for the full sequence: safety plan creation → pre-authorization → missed check-in
or trusted-person concern → search view activation → last verified point + trail +
corridor + freshness/uncertainty displayed → automatic expiry and access logging.

## 6.4 Check-in lifecycle

Manual, scheduled, and trip check-ins send reminders before any configured
escalation; states are pending, reminder_sent, grace_period, completed, cancelled,
missed. See [API Contract §"Check-Ins"](../reference/api-contract.md) for the
endpoint-level detail.

---
*Part of the [architecture documentation](./README.md) (arc42 §6).*
