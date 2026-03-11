#!/bin/sh
set -e

# Citadel mesh peers (comma-separated host:port)
CITADEL_PEERS="${CITADEL_PEERS:-}"
# Lens node API endpoint (full URL, e.g., https://api.global.riff.cc/api/v1)
LENS_NODE="${LENS_NODE:-http://127.0.0.1:8080/api/v1}"

# Derive WebSocket URL from LENS_NODE
LENS_NODE_WS=$(echo "$LENS_NODE" | sed 's|^http://|ws://|' | sed 's|^https://|wss://|' | sed 's|/api/v1$|/ws|')

echo "🔧 Configuring Flagship with runtime settings..."
echo "   CITADEL_PEERS: $CITADEL_PEERS"
echo "   LENS_NODE:     $LENS_NODE"
echo "   LENS_NODE_WS:  $LENS_NODE_WS"

# Export for envsubst
export CITADEL_PEERS
export LENS_NODE
export LENS_NODE_WS

# Generate runtime config.js from template
envsubst '${CITADEL_PEERS} ${LENS_NODE}' < /etc/nginx/config.template.js > /usr/share/nginx/html/config.js
echo "✅ Runtime config.js generated"

# Generate nginx config from template
envsubst '${LENS_NODE} ${LENS_NODE_WS}' < /etc/nginx/nginx.conf.template > /etc/nginx/conf.d/default.conf
echo "✅ Nginx config generated with API proxy to $LENS_NODE"

# Inject config.js script tag into index.html if not already present
if ! grep -q 'config.js' /usr/share/nginx/html/index.html; then
    echo "🔧 Injecting config.js script tag into index.html..."
    sed -i 's|<title>Riff.CC</title>|<title>Riff.CC</title>\n  <script src="/config.js"></script>|' /usr/share/nginx/html/index.html
    echo "✅ Runtime configuration script tag injected"
fi

# Start nginx in foreground
echo "🚀 Starting nginx..."
exec nginx -g 'daemon off;'
