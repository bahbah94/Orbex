# Database configuration
export DB_HOST := "localhost"
export DB_PORT := "5432"
export DB_NAME := "polkadot_clob"
export DB_USER := "postgres"
export DB_PASSWORD := "password"

# Default recipe - show available commands
default:
    @just --list

# Start the TimescaleDB container
db-start:
    docker compose up -d timescaledb
    @echo "Waiting for database to be ready..."
    @sleep 3
    @just db-wait

# Wait for database to be ready
db-wait:
    @echo "Checking database connection..."
    @until docker compose exec -T timescaledb pg_isready -U {{DB_USER}} -d {{DB_NAME}} > /dev/null 2>&1; do \
        echo "Waiting for database..."; \
        sleep 2; \
    done
    @echo "Database is ready!"

# Run database migrations
db-migrate:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Running migrations..."
    for file in indexer/db/migrations/*.sql; do
        echo "Applying migration: $(basename $file)"
        docker compose exec -T timescaledb psql -U {{DB_USER}} -d {{DB_NAME}} -f /dev/stdin < $file
    done
    echo "Migrations completed!"

# Setup TimescaleDB hyperfunctions and continuous aggregates
db-setup-timescale:
    @echo "Setting up TimescaleDB hyperfunctions..."
    docker compose exec -T timescaledb psql -U {{DB_USER}} -d {{DB_NAME}} -f /mnt/timescale.sql
    @echo "TimescaleDB setup completed!"

# Full database setup (start, migrate, setup timescale)
db-setup: db-start db-migrate db-setup-timescale
    @echo "Database setup completed successfully!"

# Reset database (drop and recreate)
db-reset:
    @echo "Resetting database..."
    docker compose exec -T timescaledb psql -U {{DB_USER}} -d postgres -c "DROP DATABASE IF EXISTS {{DB_NAME}};"
    docker compose exec -T timescaledb psql -U {{DB_USER}} -d postgres -c "CREATE DATABASE {{DB_NAME}};"
    @echo "Database reset completed!"
    @just db-migrate
    @just db-setup-timescale

# Connect to database with psql
db-shell:
    docker compose exec timescaledb psql -U {{DB_USER}} -d {{DB_NAME}}

# View database logs
db-logs:
    docker compose logs -f timescaledb

# Stop the database container
db-stop:
    docker compose stop timescaledb

# Stop and remove the database container
db-down:
    docker compose down timescaledb

# Full teardown (stop and remove volumes)
db-teardown:
    docker compose down -v
    @echo "Database and volumes removed!"

# Check database status
db-status:
    @docker compose ps timescaledb
    @echo ""
    @echo "Database connection test:"
    @docker compose exec -T timescaledb pg_isready -U {{DB_USER}} -d {{DB_NAME}} || echo "Database not ready"
