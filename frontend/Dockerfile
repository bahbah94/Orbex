# ----- base -----
FROM node:20-alpine AS base
WORKDIR /app
RUN apk add --no-cache libc6-compat && corepack enable

# Only lockfile + manifest first for better caching
COPY package.json pnpm-lock.yaml ./
RUN corepack prepare pnpm@9 --activate && pnpm fetch

# ----- build -----
FROM base AS build
COPY . .
# If you removed the workspace file, this installs just this app
RUN pnpm install --no-frozen-lockfile
RUN pnpm build

# ----- runtime -----
FROM node:20-alpine AS runtime
WORKDIR /app
RUN addgroup -S nextjs && adduser -S nextjs -G nextjs

# Copy minimal runtime artifacts
COPY --from=build /app/.next ./.next
COPY --from=build /app/public ./public
COPY --from=build /app/package.json .
COPY --from=build /app/node_modules ./node_modules
COPY --from=build /app/next.config.mjs ./next.config.mjs

EXPOSE 3000
USER nextjs
CMD ["node", "node_modules/next/dist/bin/next", "start", "-p", "3000"]
