# Incident Response Plan

## Scope

This plan covers suspected account takeover, unauthorized location access, data
exposure, abusive group activity, malicious media, provider outage, notification
failure, and compromised moderation access.

## Severity

- **Critical:** confirmed or likely unauthorized precise location disclosure,
  active stalking risk, widespread account takeover, or infrastructure
  compromise.
- **High:** unauthorized access attempt, compromised moderator account, serious
  abuse report, or prolonged location-service failure.
- **Medium:** isolated policy violation, repeated rate-limit bypass,
  notification delay, or suspicious account behavior.
- **Low:** cosmetic issue, non-sensitive content problem, or isolated false
  positive.

## Response stages

1. **Detect:** receive alert, report, audit anomaly, user complaint, or mentor
   escalation.
2. **Triage:** confirm scope, affected accounts/data, severity, confidence, and
   immediate physical-safety risk.
3. **Contain:** revoke sessions/tokens, disable affected sharing, freeze
   compromised accounts, isolate services, block abusive content, and preserve
   only necessary evidence.
4. **Eradicate:** patch the vulnerability, remove malicious access, rotate
   keys/secrets, correct permissions, and validate dependencies.
5. **Recover:** restore from verified backups if needed, re-enable services
   gradually, notify affected users, and monitor for recurrence.
6. **Review:** record timeline, root cause, controls that failed, user impact,
   decisions, and improvements.

## Location-specific procedure

If unauthorized location access is suspected, revoke all affected shares,
invalidate sessions, preserve relevant access logs, contact the affected user
through a safe channel, and explain what is known without creating false
certainty. Do not automatically contact emergency services. Provide practical
guidance and an exportable summary where appropriate.

## Evidence and privacy

Evidence access is restricted by role. Store audit events and explicitly
reported content separately from normal content. Do not preserve all messages
or location history indefinitely for hypothetical investigations. Define
retention, legal holds, deletion, and user-notification rules before public
launch.

## Communication

The incident owner maintains a timeline and assigns technical, safety,
moderation, and communication responsibilities. User-facing messages should
state known facts, uncertainty, actions taken, and recommended user actions.
Never expose another person's private information while explaining an
incident.

## Practice

Run tabletop exercises for account takeover, location breach, provider outage,
and abusive group escalation. Record the result as project evidence and update
the threat model and control register.

---
*Part of [§7 Deployment View](./07-deployment-view.md) and
[§8 Crosscutting Concepts](./08-crosscutting-concepts.md).*
