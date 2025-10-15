#!/bin/bash
# Test script to verify syslog-ng is receiving logs

set -e

echo "🧪 Testing syslog-ng logging..."
echo ""

# Test UDP
echo "📤 Sending UDP test message..."
logger -n localhost -P 514 "TEST-UDP: $(date) - Test message from test-logging.sh"
sleep 1

# Test TCP (non-TLS)
echo "📤 Sending TCP test message..."
echo "<134>TEST-TCP: $(date) - Test message from test-logging.sh" | nc localhost 601 || echo "⚠️  TCP port 601 not available"
sleep 1

# Test TLS (requires openssl)
echo "📤 Sending TLS test message..."
echo "<134>TEST-TLS: $(date) - Test message from test-logging.sh" | openssl s_client -connect localhost:6514 -quiet -ign_eof 2>/dev/null || echo "⚠️  TLS connection failed"
sleep 2

# Check if logs were received
echo ""
echo "🔍 Checking for test messages in logs..."
echo ""

if [ -f logs/raw/all-$(date +%Y%m%d).log ]; then
    echo "📋 Recent logs:"
    tail -n 10 logs/raw/all-$(date +%Y%m%d).log
    echo ""

    # Count test messages
    TEST_COUNT=$(grep -c "TEST-" logs/raw/all-$(date +%Y%m%d).log 2>/dev/null || echo "0")
    echo "✅ Found $TEST_COUNT test message(s)"
else
    echo "❌ No log files found. Check if syslog-ng is running:"
    echo "   docker-compose ps"
    echo "   docker-compose logs syslog-ng"
fi

echo ""
echo "🐳 Docker logs (last 20 lines):"
docker-compose logs --tail=20 syslog-ng

echo ""
echo "📊 Service status:"
docker-compose ps

echo ""
echo "✅ Test complete!"
echo ""
echo "🔧 To view logs continuously:"
echo "   tail -f logs/raw/all-$(date +%Y%m%d).log"
echo "   docker-compose logs -f syslog-ng"
