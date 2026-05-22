#!/bin/bash
# Article Blog - Database Startup Script
# This script sets up the SQLite database and runs the server

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "========================================"
echo "  The Inkwell - Article Blog"
echo "  Database Startup Script"
echo "========================================"
echo ""

# Check for required tools
if ! command -v sqlite3 &> /dev/null; then
    echo "Error: sqlite3 is required but not installed."
    echo "Install with: brew install sqlite3"
    exit 1
fi

# Create/reset database
DB_FILE="blog.db"
echo "Setting up database: $DB_FILE"

if [ -f "$DB_FILE" ]; then
    echo "  Removing existing database..."
    rm "$DB_FILE"
fi

echo "  Creating schema and inserting sample data..."
sqlite3 "$DB_FILE" < schema.sql

echo "  Database created successfully!"
echo ""

# Verify data
ARTICLE_COUNT=$(sqlite3 "$DB_FILE" "SELECT COUNT(*) FROM articles")
echo "  Articles in database: $ARTICLE_COUNT"
echo ""

# Compile the app if needed
WASM_FILE="app-db.wasm"
CLN_FILE="app-db.cln"

# Find the compiler
COMPILER=""
if [ -f "$HOME/.cleen/bin/cln" ]; then
    COMPILER="$HOME/.cleen/bin/cln"
elif command -v cln &> /dev/null; then
    COMPILER="cln"
else
    echo "Error: Clean Language compiler (cln) not found."
    echo "Install with: cleen install latest"
    exit 1
fi

echo "Compiling $CLN_FILE..."
$COMPILER compile "$CLN_FILE" -o "$WASM_FILE" --plugins
echo "  Compiled to $WASM_FILE"
echo ""

# Find the server
SERVER=""
if [ -f "$HOME/.cleen/server/1.5.0/clean-server" ]; then
    SERVER="$HOME/.cleen/server/1.5.0/clean-server"
elif [ -f "$HOME/.cleen/server/1.2.0/clean-server" ]; then
    SERVER="$HOME/.cleen/server/1.2.0/clean-server"
elif command -v clean-server &> /dev/null; then
    SERVER="clean-server"
else
    echo "Error: clean-server not found."
    echo "Install with: cleen server install"
    exit 1
fi

# Start server with database
echo "Starting server..."
echo "  Database: sqlite://$SCRIPT_DIR/$DB_FILE"
echo ""

export DATABASE_URL="sqlite://$SCRIPT_DIR/$DB_FILE"
$SERVER "$WASM_FILE" --port 3000
