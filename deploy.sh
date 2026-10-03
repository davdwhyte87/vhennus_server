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
# Check project directory
# --------------------------------------------------

if [ ! -d "$PROJECT_DIR" ]; then
    echo "ERROR: Project directory does not exist:"
    echo "$PROJECT_DIR"
    exit 1
fi

mkdir -p "$BUILD_DIR"

# --------------------------------------------------
# Check .env
# --------------------------------------------------

if [ ! -f "$ENV_FILE" ]; then
    echo "ERROR: Environment file not found:"
    echo "$ENV_FILE"
    exit 1
fi

# Load environment variables for SQLx,
# migrations, and the build.
set -a
source "$ENV_FILE"
set +a

if [ -z "$DATABASE_URL" ]; then
    echo "ERROR: DATABASE_URL is not set in:"
    echo "$ENV_FILE"
    exit 1
fi

echo "Environment loaded successfully."

# --------------------------------------------------
# Load Rust / Cargo
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

echo ""
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

echo ""
echo "Updating source code..."

git fetch origin "$BRANCH"

git reset --hard "origin/$BRANCH"

echo "Source code updated."

# --------------------------------------------------
# Run database migrations
# --------------------------------------------------

echo ""
echo "Running database migrations..."

if ! command -v sqlx >/dev/null 2>&1; then
    echo "ERROR: sqlx CLI is not installed."
    echo "Install it with:"
    echo ""
    echo "cargo install sqlx-cli --no-default-features --features postgres"
    exit 1
fi

sqlx migrate run

echo "Database migrations complete."

# --------------------------------------------------
# Build application
# --------------------------------------------------

echo ""
echo "Building Rust application..."

cargo build --release

echo ""
echo "Build successful."

# --------------------------------------------------
# Verify executable
# --------------------------------------------------

BUILD_BINARY="$PROJECT_DIR/target/release/$EXECUTABLE_NAME"

if [ ! -f "$BUILD_BINARY" ]; then
    echo "ERROR: Compiled executable not found:"
    echo "$BUILD_BINARY"
    exit 1
fi

# --------------------------------------------------
# Stop service
# --------------------------------------------------

echo ""
echo "Stopping $SERVICE_NAME..."

sudo systemctl stop "$SERVICE_NAME" || true

# --------------------------------------------------
# Copy executable
# --------------------------------------------------

cp "$BUILD_BINARY" "$BUILD_DIR/$EXECUTABLE_NAME"

echo "Executable copied."

# --------------------------------------------------
# Verify .env
# --------------------------------------------------

if [ ! -f "$BUILD_DIR/.env" ]; then
    echo "ERROR: .env is missing from build directory."
    exit 1
fi

echo "Environment file verified."

# --------------------------------------------------
# Copy log4rs configuration
# --------------------------------------------------

if [ ! -f "$PROJECT_DIR/log4rs.yaml" ]; then
    echo "ERROR: log4rs.yaml not found:"
    echo "$PROJECT_DIR/log4rs.yaml"
    exit 1
fi

cp "$PROJECT_DIR/log4rs.yaml" "$BUILD_DIR/log4rs.yaml"

echo "Logging configuration copied."

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
# Show build directory
# --------------------------------------------------

echo ""
echo "Build directory:"

ls -la "$BUILD_DIR"

# --------------------------------------------------
# Start service
# --------------------------------------------------

echo ""
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