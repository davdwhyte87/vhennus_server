#!/bin/bash

set -e

PROJECT_DIR="/root/vhennus"
ENV_FILE="$PROJECT_DIR/.env"

echo "Starting deployment..."

# --------------------------------------------------
# Load environment variables
# --------------------------------------------------

if [ ! -f "$ENV_FILE" ]; then
    echo "ERROR: $ENV_FILE not found!"
    exit 1
fi

set -a
source "$ENV_FILE"
set +a

if [ -z "$DATABASE_URL" ]; then
    echo "ERROR: DATABASE_URL is not set in .env"
    exit 1
fi

echo "Environment loaded successfully."

# --------------------------------------------------
# Environment configuration
# --------------------------------------------------

if [ "$APP_ENV" = "test" ]; then
    echo "Deploying TEST environment..."

    PROJECT_DIR="/root/test_vhennus"
    SERVICE_NAME="test.vhennus.service"
    BRANCH="develop"

elif [ "$APP_ENV" = "prod" ]; then
    echo "Deploying PRODUCTION environment..."

    PROJECT_DIR="/root/vhennus"
    SERVICE_NAME="vhennus.service"
    BRANCH="main"

else
    echo "ERROR: APP_ENV must be 'test' or 'prod'"
    exit 1
fi

cd "$PROJECT_DIR"

# --------------------------------------------------
# Make sure Rust/Cargo is available
# --------------------------------------------------

if ! command -v cargo >/dev/null 2>&1; then
    echo "Cargo not found. Loading Rust environment..."

    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    else
        echo "ERROR: Rust/Cargo is not installed."
        exit 1
    fi
fi

echo "Cargo: $(cargo --version)"

# --------------------------------------------------
# Check database connection
# --------------------------------------------------

echo "Checking PostgreSQL connection..."

if ! pg_isready >/dev/null 2>&1; then
    echo "ERROR: PostgreSQL is not running."
    exit 1
fi

if ! psql "$DATABASE_URL" -c "SELECT 1;" >/dev/null 2>&1; then
    echo "ERROR: Cannot connect to PostgreSQL."
    exit 1
fi

echo "Database connection successful."

# --------------------------------------------------
# Pull latest code
# --------------------------------------------------

echo "Updating source code..."

git fetch origin "$BRANCH"

git reset --hard "origin/$BRANCH"

# --------------------------------------------------
# Build
# --------------------------------------------------

echo "Building Rust application..."

cargo build --release

echo "Build successful."

# --------------------------------------------------
# Stop service
# --------------------------------------------------

echo "Stopping $SERVICE_NAME..."

sudo systemctl stop "$SERVICE_NAME" || true

# --------------------------------------------------
# Copy executable
# --------------------------------------------------

BUILD_DIR="$PROJECT_DIR/target/release"
EXECUTABLE_NAME="vhennus_server"

if [ ! -f "$BUILD_DIR/$EXECUTABLE_NAME" ]; then
    echo "ERROR: Build output not found:"
    echo "$BUILD_DIR/$EXECUTABLE_NAME"
    exit 1
fi

cp "$BUILD_DIR/$EXECUTABLE_NAME" "$PROJECT_DIR/build/"

echo "Executable copied."

# --------------------------------------------------
# Templates
# --------------------------------------------------

if [ -d "$PROJECT_DIR/templates" ] && [ -d "$PROJECT_DIR/build/templates" ]; then
    cp "$PROJECT_DIR/templates/"*.hbs "$PROJECT_DIR/build/templates/" 2>/dev/null || true
fi

# --------------------------------------------------
# Restart service
# --------------------------------------------------

echo "Starting $SERVICE_NAME..."

sudo systemctl start "$SERVICE_NAME"

echo "Deployment complete."

sudo systemctl status "$SERVICE_NAME" --no-pager