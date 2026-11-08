#!/bin/sh
# Generate a self-signed cert for local HTTPS testing (dev only, not production).
set -eu
mkdir -p "$(dirname "$0")/certs"
openssl req -x509 -newkey rsa:2048 -nodes \
  -keyout "$(dirname "$0")/certs/key.pem" \
  -out "$(dirname "$0")/certs/cert.pem" \
  -days 365 -subj "/CN=localhost" \
  -addext "subjectAltName=DNS:localhost,IP:127.0.0.1"
echo "certs written to $(dirname "$0")/certs/"
