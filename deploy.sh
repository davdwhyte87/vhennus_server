#!/bin/bash

# Load environment variables from .env file
if [ -f build/.env ]; then
    export $(grep -v '^#' .env | xargs)
else
    echo ".env file not found!"
    exit 1
fi

if [ "$APP_ENV" = "test" ]; then
    echo "Deploying to TEST environment..."
    # Test deployment commands
    PROJECT_DIR="/root/test_vhennus"
    SERVICE_NAME="test.vhennus.service"
    BRANCH = "develop"
    export DATABASE_URL="postgres://postgres:admin05501@127.0.0.1:5432/vhennus_test"
elif [ "$APP_ENV" = "prod" ]; then
    echo "Deploying to PRODUCTION environment..."
    # Production deployment commands
     PROJECT_DIR="/root/vhennus"
     SERVICE_NAME="vhennus.service"
     BRANCH = "main"
     export DATABASE_URL="postgres://postgres:admin05501@127.0.0.1:5432/vhennus"
else
    echo "Unknown environment. Check your .env file."
    exit 1
fi
# Set variables
BUILD_DIR="$PROJECT_DIR/target/release"
EXECUTABLE_NAME="vhennus_server"



# Navigate to project directory
cd "$PROJECT_DIR" || { echo "Failed to change directory to $PROJECT_DIR"; exit 1; }

# Force pull latest changes
git reset --hard origin/$BRANCH
if ! git pull origin $BRANCH; then
    echo "Failed to pull latest changes from GitHub"
    exit 1
fi

# Build the Rust project
if ! cargo build --release; then
    echo "Build failed"
    exit 1
fi


# Copy the built executable to the project root
if [ -f "$BUILD_DIR/$EXECUTABLE_NAME" ]; then
    cp "$BUILD_DIR/$EXECUTABLE_NAME" "$PROJECT_DIR/build/"
else
    echo "Build output not found"
    exit 1
fi

# Move all .hbs template files
mv "$PROJECT_DIR/templates"/*.hbs "$PROJECT_DIR/build/templates"/

# Check if move was successful
if [ $? -eq 0 ]; then
    echo "Templates moved successfully to $DEST_DIR"
else
    echo "Failed to move templates"
    exit 1
fi

# Restart the systemd service
if ! sudo systemctl restart "$SERVICE_NAME"; then
    echo "Failed to restart service $SERVICE_NAME"
    exit 1
fi

echo "Deployment done"

systemctl status "$SERVICE_NAME"
