# Pinpoint — Service discovery (gateway as the central directory)

## One place knows where everything runs

`docker/nginx-gateway.conf` (and the HTTPS profile) is the single,
human-readable routing table:

| Service | Docker DNS upstream | Public path (gateway) |
|---|---|---|
| auth | `auth:8001` | `/api/v1/auth/` |
| users | `users:8002` | `/api/v1/users/`, `/api/v1/friendships/` |
| locations | `locations:8003` | `/api/v1/locations/` |
| map | `map:8004` | `/map/` |

Clients talk only to the gateway (`:8000`/`:8443`). The gateway picks
**any** healthy replica — no client ever learns a replica's IP.

```bash
Invoke-RestMethod http://127.0.0.1:8000/registry
# {"gateway":"pinpoint-gateway","services":[{"name":"auth","upstream":"auth:8001",
#  "path":"/api/v1/auth/"}, ...]}
```

## How "takes any" works (no static upstream)

Each location uses `set $x_upstream http://<dns>:<port>; proxy_pass $x_upstream`
+ `resolver 127.0.0.11 valid=30s`. That forces nginx to re-resolve Docker
DNS on *every* request and round-robins the returned A-records, so
scaling (`--scale auth=3`) or killing a replica is picked up without a
gateway reload. `proxy_next_upstream ... http_502/503/504` retries the
next replica when the picked one is down.

Static `upstream { server auth:8001; }` was tried first: it resolves
DNS at config-load and fails `host not found in upstream` when services
aren't up yet, and it never sees new replicas without a reload. The
per-request-variable pattern is the Docker-correct form.

## Observing it

Every proxied response carries `X-Upstream: <ip>:<port>` of the replica
that served it. Hit an endpoint twice and you should see two different
IPs when 2 replicas run:

```powershell
curl.exe -sI http://127.0.0.1:8000/api/v1/auth/me | Select-String X-Upstream
curl.exe -sI http://127.0.0.1:8000/api/v1/auth/me | Select-String X-Upstream
```

## Adding a service

1. Add the `set $x_upstream` + `location` pair in **both** gateway
   configs (replace, don't duplicate).
2. Add the entry to the `/registry` JSON in the same commit.
3. The gateway's `depends_on` in `docker-compose.yml` already waits for
   healthy services; nothing else needs to know the new address.

See `docs/redundancy.md` for the failover drill and `docker-compose.yml`
for the per-service replica counts.
