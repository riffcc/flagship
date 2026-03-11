#!/bin/bash
# Generate self-signed TLS certificates for syslog-ng
# For production, use Let's Encrypt or your own CA

set -e

CERT_DIR="./certs"
DOMAIN="relay.global.riff.cc"
DAYS_VALID=3650  # 10 years

echo "🔐 Generating TLS certificates for syslog-ng..."

# Create certs directory
mkdir -p "$CERT_DIR"
cd "$CERT_DIR"

# Generate CA private key
if [ ! -f ca-key.pem ]; then
    echo "📝 Generating CA private key..."
    openssl genrsa -out ca-key.pem 4096
fi

# Generate CA certificate
if [ ! -f ca-cert.pem ]; then
    echo "📝 Generating CA certificate..."
    openssl req -new -x509 -days $DAYS_VALID -key ca-key.pem -out ca-cert.pem \
        -subj "/C=US/ST=Internet/L=Cloud/O=Riff.CC/OU=Infrastructure/CN=Riff.CC Syslog CA"
fi

# Generate server private key
if [ ! -f server-key.pem ]; then
    echo "📝 Generating server private key..."
    openssl genrsa -out server-key.pem 4096
fi

# Generate server certificate signing request
if [ ! -f server-cert.csr ]; then
    echo "📝 Generating server CSR..."
    openssl req -new -key server-key.pem -out server-cert.csr \
        -subj "/C=US/ST=Internet/L=Cloud/O=Riff.CC/OU=Infrastructure/CN=$DOMAIN"
fi

# Create SAN extension file
cat > server-san.ext <<EOF
subjectAltName = @alt_names
[alt_names]
DNS.1 = $DOMAIN
DNS.2 = relay.riff.cc
DNS.3 = localhost
IP.1 = 127.0.0.1
EOF

# Sign server certificate with CA
if [ ! -f server-cert.pem ]; then
    echo "📝 Signing server certificate..."
    openssl x509 -req -days $DAYS_VALID -in server-cert.csr \
        -CA ca-cert.pem -CAkey ca-key.pem -CAcreateserial \
        -out server-cert.pem -extfile server-san.ext
fi

# Set proper permissions
chmod 600 server-key.pem ca-key.pem
chmod 644 server-cert.pem ca-cert.pem

# Clean up CSR and SAN files
rm -f server-cert.csr server-san.ext

echo ""
echo "✅ Certificates generated successfully!"
echo ""
echo "📁 Certificate files:"
echo "   CA Certificate:     $CERT_DIR/ca-cert.pem"
echo "   Server Certificate: $CERT_DIR/server-cert.pem"
echo "   Server Private Key: $CERT_DIR/server-key.pem"
echo ""
echo "🔧 Configuration for Bunny.net:"
echo "   Protocol: TLS"
echo "   Host: $DOMAIN"
echo "   Port: 6514"
echo "   Format: RFC5424 or BSD"
echo ""
echo "📤 Upload CA certificate to Bunny.net if they require it:"
echo "   cat $CERT_DIR/ca-cert.pem"
echo ""
echo "🚀 Start syslog-ng with: docker-compose up -d"
