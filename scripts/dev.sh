#!/bin/bash
set -e

# Load environment variables
set -a
source .env
set +a

# Function to check if postgres is ready
wait_for_postgres() {
    echo "Waiting for PostgreSQL to be ready..."
    until pg_isready -h localhost -p 5432 -U postgres
    do
        echo "PostgreSQL is unavailable - sleeping"
        sleep 1
    done
    echo "PostgreSQL is up - executing command"
}

# Ensure database exists
create_database() {
    echo "Creating database if it doesn't exist..."
    psql -h localhost -U postgres -tc "SELECT 1 FROM pg_database WHERE datname = 'podcast_crawler_dev'" | grep -q 1 || \
    psql -h localhost -U postgres -c "CREATE DATABASE podcast_crawler_dev"
}

# Run migrations
run_migrations() {
    echo "Running database migrations..."
    diesel migration run
}

# Start development server with auto-reload
start_dev_server() {
    echo "Starting development server..."
    cargo watch -x run
}

# Main execution
wait_for_postgres
create_database
run_migrations
start_dev_server
