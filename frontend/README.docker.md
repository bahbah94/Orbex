# Docker Setup for Orbex

## Prerequisites
- Docker installed on your system
- Docker Compose installed

## Environment Variables

The application requires the following environment variables:
- `NODE_ENV`: Set to `prod` for production
- `SERVER_URL`: API server URL (default: http://localhost:3000)
- `NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID`: Your WalletConnect project ID

## Quick Start

### 1. Build and run with Docker Compose

\`\`\`bash
docker-compose up --build
\`\`\`

The application will be available at http://localhost:3000

### 2. Run in detached mode

\`\`\`bash
docker-compose up -d
\`\`\`

### 3. Stop the application

\`\`\`bash
docker-compose down
\`\`\`

## Manual Docker Commands

### Build the image

\`\`\`bash
docker build -t orbex-web-app .
\`\`\`

### Run the container

\`\`\`bash
docker run -p 3000:3000 \
  -e NODE_ENV=prod \
  -e SERVER_URL=http://localhost:3000 \
  -e NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID=your-project-id \
  orbex-web-app
\`\`\`

## Development

To override environment variables, create a `.env` file or modify the `docker-compose.yml` file.

## Troubleshooting

- If port 3000 is already in use, modify the port mapping in `docker-compose.yml`
- Check logs: `docker-compose logs -f`
- Rebuild after code changes: `docker-compose up --build`
