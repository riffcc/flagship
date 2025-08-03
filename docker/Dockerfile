# Build stage
FROM node:22-alpine AS builder

# Install build dependencies
RUN apk add --no-cache python3 py3-setuptools make g++ git

# Install pnpm
RUN corepack enable && corepack prepare pnpm@latest --activate

# Set working directory
WORKDIR /app

# Copy package files and lens-sdk for local dependency
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY lens-sdk/ ./lens-sdk/

# Install dependencies
# Using --no-frozen-lockfile temporarily due to lens-sdk path mismatch
RUN pnpm install --no-frozen-lockfile --ignore-scripts --unsafe-perm

# Copy remaining source code
COPY . .

# Generate Electron vendors file
RUN node scripts/update-electron-vendors.mjs

# Build the web version
RUN pnpm build

# Production stage
FROM nginx:alpine

# Copy built files from builder stage
COPY --from=builder /app/packages/renderer/dist/web /usr/share/nginx/html

# Copy custom nginx config for SPA routing and proper MIME types
COPY docker/nginx.conf /etc/nginx/conf.d/default.conf

# Expose port 80
EXPOSE 80

# Start nginx
CMD ["nginx", "-g", "daemon off;"]
