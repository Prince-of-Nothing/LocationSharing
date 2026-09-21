# Location Sharing Lifecycle

1. A user selects people, a group, or a sharing duration.
2. The recipient accepts or declines; no silent enrollment is allowed.
3. The app displays active sharing, last update, accuracy, and expiry.
4. Either side can stop, revoke, block, or report the relationship.
5. Expired sessions stop collection and move to deletion or coarse history
   according to retention rules.

Background location requires progressive Android/iPhone permission requests and
must degrade clearly when the OS pauses updates.

Advanced sharing can include a movement trail and estimated travel corridor for
selected trusted people. This is opt-in, separately permissioned, and never visible
to ordinary friends or non-friends. See
[Movement Trails and Search Corridors](./movement-trails-and-search-corridors.md).

---
*Part of [§5 Building Block View](./05-building-block-view.md) / [§6 Runtime View](./06-runtime-view.md).*
