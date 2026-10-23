# Auth9 Portal - Remix Frontend
# Multi-stage build for minimal image size
# Build context: repo root (for file: dependency on @auth9/core)

# ==================== SDK Build Stage ====================
FROM node:22.23.4-alpine AS sdk-builder

WORKDIR /app/sdk

# Install SDK dependencies and build
COPY sdk/package.json sdk/pnpm-lock.yaml sdk/pnpm-workspace.yaml sdk/tsconfig.base.json ./
COPY sdk/packages/core/package.json packages/core/
RUN corepack enable && corepack prepare pnpm@10.29.2 --activate
RUN --mount=type=cache,target=/root/.local/share/pnpm/store \
    pnpm install --frozen-lockfile --filter @auth9/core
COPY sdk/packages/core/ packages/core/
RUN pnpm --filter @auth9/core build

# ==================== Portal Builder Stage ====================
FROM node:22.23.4-alpine AS builder

WORKDIR /app

# Copy built SDK package
COPY --from=sdk-builder /app/sdk/packages/core/package.json sdk/packages/core/
COPY --from=sdk-builder /app/sdk/packages/core/dist sdk/packages/core/dist

# Copy package files
COPY auth9-portal/package.json auth9-portal/package-lock.json auth9-portal/

# Install dependencies
WORKDIR /app/auth9-portal
RUN --mount=type=cache,target=/root/.npm \
    npm ci

# Copy source code
COPY auth9-portal/ .

# Build the application
RUN npm run build

# ==================== Runtime Stage ====================
FROM node:22.23.4-alpine

# Apply security patches and create non-root user
RUN apk upgrade --no-cache && \
    addgroup -g 1001 -S nodejs && \
    adduser -S remix -u 1001

WORKDIR /app

# Copy built SDK package (needed for production npm ci)
COPY --from=sdk-builder /app/sdk/packages/core/package.json sdk/packages/core/
COPY --from=sdk-builder /app/sdk/packages/core/dist sdk/packages/core/dist

# Copy built assets from builder
COPY --from=builder --chown=remix:nodejs /app/auth9-portal/build ./auth9-portal/build
COPY --from=builder --chown=remix:nodejs /app/auth9-portal/public ./auth9-portal/public
COPY --from=builder --chown=remix:nodejs /app/auth9-portal/package.json ./auth9-portal/package.json
COPY --from=builder --chown=remix:nodejs /app/auth9-portal/package-lock.json ./auth9-portal/package-lock.json

# Install production dependencies only
WORKDIR /app/auth9-portal
RUN npm ci --omit=dev

# Switch to non-root user
USER remix

# Environment
ENV NODE_ENV=production
ENV PORT=3000

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000/ || exit 1

# Run
CMD ["npm", "start"]
