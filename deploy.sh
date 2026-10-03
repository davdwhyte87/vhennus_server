#!/bin/bash

set -e

echo "======================================"
echo "      Vhennus Deployment"
echo "======================================"

# --------------------------------------------------
# Configuration
# --------------------------------------------------

PROJECT_DIR="/root/vhennus"
BUILD_DIR="$PROJECT_DIR/build"
ENV_FILE="$BUILD_DIR/.env"

EXECUTABLE_NAME="vhennus_server"
SERVICE_NAME="vhennus.service"
BRANCH="main"

# --------------------------------------------------
# Check directories
# --------------------------------------------------

if [ ! -d "$PROJECT_DIR" ]; then
    echo "ERROR: Project directory does not exist:"
    echo "$PROJECT_DIR"
    exit 1
fi

mkdir -p "$BUILD_DIR"

# --------------------------------------------------
# Load environment
# --------------------------------------------------

# The server .env is kept in build/
if [ ! -f "$ENV_FILE" ]; then
    echo "ERROR: Environment file not found:"
    echo "$ENV_FILE"
    exit 1
fi

set -a
source "$ENV_FILE"
set +a

if [ -z "$DATABASE_URL" ]; then
    echo "ERROR: DATABASE_URL is not set."
    exit 1
fi

echo "Environment loaded successfully."

# --------------------------------------------------
# Load Rust environment if necessary
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
# Check PostgreSQL
# --------------------------------------------------

echo "Checking PostgreSQL..."

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
# Update source code
# --------------------------------------------------

cd "$PROJECT_DIR"

echo "Updating source code..."

git fetch origin "$BRANCH"
git reset --hard "origin/$BRANCH"

echo "Source code updated."

# --------------------------------------------------
# Run database migrations
# --------------------------------------------------

if command -v sqlx >/dev/null 2>&1; then
    echo "Running database migrations..."
    sqlx migrate run
    echo "Migrations complete."
else
    echo "WARNING: sqlx CLI is not installed."
    echo "Skipping migrations."
fi

# --------------------------------------------------
# Build application
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

if [ ! -f "$PROJECT_DIR/target/release/$EXECUTABLE_NAME" ]; then
    echo "ERROR: Compiled executable not found:"
    echo "$PROJECT_DIR/target/release/$EXECUTABLE_NAME"
    exit 1
fi

cp \
    "$PROJECT_DIR/target/release/$EXECUTABLE_NAME" \
    "$BUILD_DIR/$EXECUTABLE_NAME"

echo "Executable copied."

# --------------------------------------------------
# Restore .env into build/
# --------------------------------------------------

# IMPORTANT:
# .env is server-only and must NOT come from GitHub.
#
# If your .env already exists in build/, it remains there.
# This section simply verifies it exists.

if [ ! -f "$BUILD_DIR/.env" ]; then
    echo "ERROR: $BUILD_DIR/.env disappeared."
    echo "Deployment stopped to prevent starting without secrets."
    exit 1
fi

echo "Environment file verified."

# --------------------------------------------------
# Copy templates
# --------------------------------------------------

if [ -d "$PROJECT_DIR/templates" ]; then

    mkdir -p "$BUILD_DIR/templates"

    cp "$PROJECT_DIR/templates/"*.hbs \
       "$BUILD_DIR/templates/" 2>/dev/null || true

    echo "Templates copied."
fi

# --------------------------------------------------
# Start service
# --------------------------------------------------

echo "Starting $SERVICE_NAME..."

sudo systemctl start "$SERVICE_NAME"

sleep 2

# --------------------------------------------------
# Check service
# --------------------------------------------------

if systemctl is-active --quiet "$SERVICE_NAME"; then

    echo ""
    echo "======================================"
    echo "       Deployment successful"
    echo "======================================"
    echo ""

    sudo systemctl status "$SERVICE_NAME" --no-pager

else

    echo ""
    echo "======================================"
    echo "       Deployment FAILED"
    echo "======================================"
    echo ""

    sudo systemctl status "$SERVICE_NAME" --no-pager

    echo ""
    echo "Recent application logs:"
    sudo journalctl -u "$SERVICE_NAME" -n 50 --no-pager

    exit 1
fi