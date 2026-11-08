# docker/mock-api — Flask stand-in for the Pinpoint Go/Python API.
#
# Used with docker compose:
#   docker compose up mock-api
#   docker compose run --rm pinpoint --backend http://mock-api:8000
#
# Mirrors the wire contract in `src/api.rs`:
#   - session cookie + X-CSRF-Token double-submit on unsafe methods
#   - error envelopes { error: { code, message } }
#   - numeric user/friendship/share IDs
