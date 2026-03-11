#!/bin/sh
# Docker entrypoint for Flagship containers
# Dynamically generates .env based on which lens node we're connecting to

set -e

echo "Flagship entrypoint starting for PORT=$PORT"

# Check if we can read VITE_SITE_ADDRESS from writer-data config
if [ -f "/app/writer-data/config.json" ]; then
    echo "Found writer-data config, reading site address..."
    WRITER_SITE_ADDRESS=$(jq -r '.address' /app/writer-data/config.json 2>/dev/null || echo "")
    if [ -n "$WRITER_SITE_ADDRESS" ] && [ "$WRITER_SITE_ADDRESS" != "null" ]; then
        echo "Using site address from writer-data: $WRITER_SITE_ADDRESS"
        export VITE_SITE_ADDRESS="$WRITER_SITE_ADDRESS"
    else
        echo "Could not read valid address from writer-data config"
    fi
else
    echo "No writer-data config found at /app/writer-data/config.json"
fi

# Determine which lens node we're connecting to based on PORT
if [ "$PORT" = "5175" ]; then
    echo "Configuring Flagship for Primary lens node..."

    # Wait for ALL required files
    echo "Waiting for primary lens node to be ready..."
    while true; do
        if [ -f /shared/primary-site-address.txt ] && [ -f /shared/primary-multiaddr.txt ]; then
            # Verify files are not empty
            if [ -s /shared/primary-site-address.txt ] && [ -s /shared/primary-multiaddr.txt ]; then
                echo "Primary lens node files found and non-empty"
                break
            fi
        fi
        echo "Still waiting for primary lens node files... (site: $(test -f /shared/primary-site-address.txt && echo 'exists' || echo 'missing'), multiaddr: $(test -f /shared/primary-multiaddr.txt && echo 'exists' || echo 'missing'))"
        sleep 2
    done

    SITE_ADDRESS=$(cat /shared/primary-site-address.txt)
    MULTIADDR=$(cat /shared/primary-multiaddr.txt)
    API_PORT=3001

elif [ "$PORT" = "5176" ]; then
    echo "Configuring Flagship for Light lens node..."

    # Light node uses same site as primary but has its own multiaddr
    echo "Waiting for light lens node to be ready..."
    while true; do
        if [ -f /shared/primary-site-address.txt ] && [ -f /shared/light-multiaddr.txt ]; then
            # Verify files are not empty
            if [ -s /shared/primary-site-address.txt ] && [ -s /shared/light-multiaddr.txt ]; then
                echo "Light lens node files found and non-empty"
                break
            fi
        fi
        echo "Still waiting for light lens node files... (site: $(test -f /shared/primary-site-address.txt && echo 'exists' || echo 'missing'), multiaddr: $(test -f /shared/light-multiaddr.txt && echo 'exists' || echo 'missing'))"
        sleep 2
    done

    SITE_ADDRESS=$(cat /shared/primary-site-address.txt)
    MULTIADDR=$(cat /shared/light-multiaddr.txt)
    API_PORT=3002

elif [ "$PORT" = "5177" ]; then
    echo "Configuring Flagship for Federated lens node..."

    echo "Waiting for federated lens node to be ready..."
    while true; do
        if [ -f /shared/federated-site-address.txt ] && [ -f /shared/federated-multiaddr.txt ]; then
            # Verify files are not empty
            if [ -s /shared/federated-site-address.txt ] && [ -s /shared/federated-multiaddr.txt ]; then
                echo "Federated lens node files found and non-empty"
                break
            fi
        fi
        echo "Still waiting for federated lens node files... (site: $(test -f /shared/federated-site-address.txt && echo 'exists' || echo 'missing'), multiaddr: $(test -f /shared/federated-multiaddr.txt && echo 'exists' || echo 'missing'))"
        sleep 2
    done

    SITE_ADDRESS=$(cat /shared/federated-site-address.txt)
    MULTIADDR=$(cat /shared/federated-multiaddr.txt)
    API_PORT=3003
else
    echo "ERROR: Unknown PORT value: $PORT"
    exit 1
fi

# Wait for relay multiaddr to be available
echo "Waiting for relay to share its multiaddr..."
RELAY_MULTIADDR=""
for i in $(seq 1 30); do
    if [ -f /shared/relay-multiaddr.txt ] && [ -s /shared/relay-multiaddr.txt ]; then
        RELAY_MULTIADDR=$(cat /shared/relay-multiaddr.txt)
        echo "Found relay multiaddr: $RELAY_MULTIADDR"
        break
    fi
    echo "Waiting for relay multiaddr... ($i/30)"
    sleep 1
done

if [ -z "$RELAY_MULTIADDR" ]; then
    echo "WARNING: Relay multiaddr not found after 30 seconds"
    RELAY_MULTIADDR=""
fi

# Generate .env file
# Use writer-data site address if available, otherwise use the one from shared files
FINAL_SITE_ADDRESS=${VITE_SITE_ADDRESS:-$SITE_ADDRESS}

cat > /app/.env << EOF
# Auto-generated environment for Flagship
VITE_SITE_ADDRESS=$FINAL_SITE_ADDRESS
VITE_LENS_NODE=$MULTIADDR
VITE_BOOTSTRAPPERS=$RELAY_MULTIADDR
VITE_API_URL=http://localhost:$API_PORT/api/v1
PORT=$PORT
EOF

echo "Generated .env with:"
echo "  SITE_ADDRESS: $FINAL_SITE_ADDRESS"
echo "  LENS_NODE: $MULTIADDR"
echo "  BOOTSTRAPPERS: $RELAY_MULTIADDR"
echo "  API_PORT: $API_PORT"

# Now run the original command
exec "$@"
