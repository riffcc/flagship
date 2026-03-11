#!/bin/bash
# Quick setup script for syslog-ng server

set -e

echo "🏰 Setting up Syslog-ng for Bunny.net CDN logs..."
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "⚠️  Warning: Not running as root. You may need sudo for firewall rules."
fi

# Generate TLS certificates
if [ ! -f certs/server-cert.pem ]; then
    echo "📝 Generating TLS certificates..."
    ./generate-certs.sh
else
    echo "✅ TLS certificates already exist"
fi

# Create logs directory
if [ ! -d logs ]; then
    echo "📁 Creating logs directory..."
    mkdir -p logs/{bunny,raw}
fi

# Set random password for Dozzle
if [ -f docker-compose.yml ]; then
    RANDOM_PASS=$(openssl rand -hex 16)
    echo ""
    echo "🔑 Dozzle Password: $RANDOM_PASS"
    echo "   (Save this! It's not stored anywhere else)"
    echo ""

    # Replace placeholder in docker-compose.yml
    sed -i "s/changeme_\$(openssl rand -hex 16)/changeme_${RANDOM_PASS}/" docker-compose.yml || true
fi

# Start services
echo "🚀 Starting services..."
docker-compose up -d

# Wait for services to start
echo "⏳ Waiting for services to start..."
sleep 5

# Check service status
echo ""
echo "📊 Service Status:"
docker-compose ps

# Get external IP
EXTERNAL_IP=$(curl -s ifconfig.me || echo "your-server-ip")

echo ""
echo "✅ Setup complete!"
echo ""
echo "📡 Syslog Endpoints:"
echo "   TLS: relay.global.riff.cc:6514"
echo "   UDP: relay.global.riff.cc:514 (testing only)"
echo ""
echo "🌐 Web Interfaces:"
echo "   Dozzle (live logs): http://${EXTERNAL_IP}:8888"
echo "   Log Viewer:         http://${EXTERNAL_IP}:8889"
echo ""
echo "🔧 Configure Bunny.net:"
echo "   1. Go to Pull Zone → Logging"
echo "   2. Set Host: relay.global.riff.cc"
echo "   3. Set Port: 6514"
echo "   4. Set Protocol: TLS/SSL"
echo "   5. Set Format: RFC5424"
echo "   6. Set Identifier: bunnycdn"
echo "   7. Save and test"
echo ""
echo "🔍 View logs:"
echo "   docker-compose logs -f syslog-ng"
echo "   tail -f logs/bunny/all.log"
echo ""
echo "🛡️  Don't forget to configure firewall rules!"
echo "   ufw allow 6514/tcp comment 'Syslog TLS'"
echo ""
echo "📚 Full documentation: README.md"
