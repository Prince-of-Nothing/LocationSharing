# pinpoint mock API — minimal Flask stand-in for offline client development.
#
# Usage with docker compose:
#   docker compose up mock-api
#   docker compose run --rm pinpoint --backend http://mock-api:8000
#
# The endpoints below mirror the wire contract that `src/api.rs` expects
# (session cookie + X-CSRF-Token double-submit, error envelopes, numeric IDs).

from flask import Flask, jsonify, request, make_response

app = Flask(__name__)

SESSION_COOKIE = "session_id=mock-session-123; Path=/; HttpOnly; SameSite=Lax"
CSRF_HEADER = "X-CSRF-Token"
CSRF_VALUE = "mock-csrf-token"

@app.before_request
def capture_session_cookie():
    # Echo a session cookie on every response so the client stores it.
    pass

def _auth_response():
    resp = make_response(jsonify({
        "access_token": "mock-access-token",
        "token_type": "bearer",
        "expires_in": 900,
        "csrf_token": CSRF_VALUE,
        "user": {
            "id": 7,
            "email": "alice@example.com",
            "full_name": "Alice",
            "role": "resident",
            "is_active": True,
            "must_change_password": False,
            "created_at": "2026-01-01T00:00:00Z",
            "last_login_at": None
        }
    }))
    resp.headers.set("Set-Cookie", SESSION_COOKIE)
    resp.headers.set(CSRF_HEADER, CSRF_VALUE)
    return resp

@app.route("/health", methods=["GET"])
def health():
    return jsonify({"status": "ok", "version": "mock-1.0.0"})

@app.route("/api/v1/auth/login", methods=["POST"])
def login():
    return _auth_response(), 200

@app.route("/api/v1/auth/register", methods=["POST"])
def register():
    return _auth_response(), 201

@app.route("/api/v1/auth/logout", methods=["POST"])
def logout():
    resp = make_response(jsonify({"message": "Successfully logged out"}), 200)
    resp.headers.set("Set-Cookie", SESSION_COOKIE)
    return resp

@app.route("/api/v1/auth/session", methods=["POST"])
def session():
    resp = make_response(jsonify({"user": {
        "id": 7,
        "email": "alice@example.com",
        "full_name": "Alice",
        "role": "resident",
        "is_active": True,
        "must_change_password": False
    }}), 200)
    resp.headers.set("Set-Cookie", SESSION_COOKIE)
    resp.headers.set(CSRF_HEADER, CSRF_VALUE)
    return resp

@app.route("/api/v1/friendships/", methods=["GET", "POST"])
def friendships():
    # Enforce CSRF on POST (unsafe method).
    if request.method == "POST":
        if request.headers.get(CSRF_HEADER) != CSRF_VALUE:
            return jsonify({"error": {"code": "CSRF_FAILED", "message": "Missing or invalid CSRF token."}}), 403
    resp = make_response(jsonify([]), 200)
    resp.headers.set("Set-Cookie", SESSION_COOKIE)
    return resp

@app.route("/api/v1/locations/shares", methods=["GET", "POST"])
def shares():
    if request.method == "POST":
        if request.headers.get(CSRF_HEADER) != CSRF_VALUE:
            return jsonify({"error": {"code": "CSRF_FAILED", "message": "Missing or invalid CSRF token."}}), 403
        body = request.get_json(silent=True) or {}
        return jsonify({
            "id": 42,
            "friendship_id": body.get("friendship_id"),
            "owner_id": 7,
            "direction": body.get("direction", "to_friend"),
            "precision_level": body.get("precision_level", "exact"),
            "is_active": True,
            "last_location_update_at": None
        }), 201
    resp = make_response(jsonify([]), 200)
    resp.headers.set("Set-Cookie", SESSION_COOKIE)
    return resp

@app.route("/api/v1/locations/me", methods=["POST"])
def publish_location():
    if request.headers.get(CSRF_HEADER) != CSRF_VALUE:
        return jsonify({"error": {"code": "CSRF_FAILED", "message": "Missing or invalid CSRF token."}}), 403
    return jsonify({"message": "ok", "shares_updated": 1}), 200

@app.route("/api/v1/locations/friends", methods=["GET"])
def friends_locations():
    resp = make_response(jsonify([]), 200)
    resp.headers.set("Set-Cookie", SESSION_COOKIE)
    return resp

@app.route("/api/v1/users/", methods=["GET"])
def users():
    resp = make_response(jsonify([]), 200)
    resp.headers.set("Set-Cookie", SESSION_COOKIE)
    return resp

@app.errorhandler(404)
def not_found(e):
    return jsonify({"error": {"code": "NOT_FOUND", "message": "Resource not found."}}), 404

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=8000)
