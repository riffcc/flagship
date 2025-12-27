# Flagship Production Image
# Lightweight Alpine + lighttpd serving static SPA bundle

FROM node:22-alpine AS builder

# Accept build-time env vars for Vite
ARG VITE_API_URL
ARG VITE_ARCHIVIST_GATEWAY
ARG VITE_ARCHIVIST_API_URL
ARG VITE_LIBRARIAN_API_URL
ENV VITE_API_URL=$VITE_API_URL
ENV VITE_ARCHIVIST_GATEWAY=$VITE_ARCHIVIST_GATEWAY
ENV VITE_ARCHIVIST_API_URL=$VITE_ARCHIVIST_API_URL
ENV VITE_LIBRARIAN_API_URL=$VITE_LIBRARIAN_API_URL

# Install build dependencies for native modules
RUN apk add --no-cache python3 py3-setuptools make g++ linux-headers

WORKDIR /app

# Enable corepack for pnpm
RUN corepack enable pnpm

# Copy package files first for caching
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml .electron-vendors.cache.json ./

# Copy workspace packages and version plugin
COPY packages/ ./packages/
COPY version/ ./version/

# Install dependencies
RUN pnpm install --frozen-lockfile

# Build production bundle
RUN pnpm build

# Production image
FROM alpine:latest

RUN apk add --no-cache lighttpd

# Copy lighttpd config
COPY <<'EOF' /etc/lighttpd/lighttpd.conf
server.document-root = "/var/www/html"
server.port = 80

mimetype.assign = (
  ".html" => "text/html",
  ".css" => "text/css",
  ".js" => "text/javascript",
  ".json" => "application/json",
  ".png" => "image/png",
  ".jpg" => "image/jpeg",
  ".jpeg" => "image/jpeg",
  ".gif" => "image/gif",
  ".svg" => "image/svg+xml",
  ".ico" => "image/x-icon",
  ".woff" => "font/woff",
  ".woff2" => "font/woff2",
  ".ttf" => "font/ttf",
  ".wasm" => "application/wasm"
)

# SPA fallback - serve index.html for 404s
server.error-handler-404 = "/index.html"

index-file.names = ( "index.html" )

server.modules += ( "mod_deflate" )
deflate.mimetypes = ( "text/html", "text/css", "text/javascript", "application/json", "application/javascript" )
EOF

# Copy built assets
COPY --from=builder /app/packages/renderer/dist/web /var/www/html

EXPOSE 80

CMD ["lighttpd", "-D", "-f", "/etc/lighttpd/lighttpd.conf"]
