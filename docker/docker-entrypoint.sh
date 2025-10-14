#!/bin/sh
set -e

# Default values if environment variables are not set
API_URL="${API_URL:-http://localhost:5002/api/v1}"
RELAY_URL="${RELAY_URL:-ws://localhost:5002/api/v1/relay/ws}"

echo "🔧 Configuring Flagship with runtime settings..."
echo "   API_URL:   $API_URL"
echo "   RELAY_URL: $RELAY_URL"

# Substitute environment variables in config template
# Note: We export the variables for envsubst
export API_URL
export RELAY_URL

# Generate runtime config.js from template
envsubst '${API_URL} ${RELAY_URL}' < /etc/nginx/config.template.js > /usr/share/nginx/html/config.js

echo "✅ Runtime configuration generated at /usr/share/nginx/html/config.js"

# Inject config.js script tag into index.html
echo "🔧 Injecting config.js script tag into index.html..."
sed -i 's|<title>Riff.CC</title>|<title>Riff.CC</title>\n  <!-- Runtime configuration (injected by Docker at container startup) -->\n  <script src="/config.js"></script>|' /usr/share/nginx/html/index.html

echo "✅ Runtime configuration script tag injected"

# Start nginx in foreground
echo "🚀 Starting nginx..."
exec nginx -g 'daemon off;'
