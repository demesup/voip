#!/bin/bash

# Start frontend server

echo ""
echo "============================================"
echo "   VoIP Application - Frontend Server"
echo "============================================"
echo ""

cd "$(dirname "$0")/frontend"

echo "Starting frontend on http://localhost:3000"
echo "Close this window to stop the server."
echo ""

# Try Python 3 first, then Python 2
if command -v python3 &> /dev/null; then
    python3 -m http.server 3000
elif command -v python &> /dev/null; then
    python -m http.server 3000
else
    echo "Error: Python not found!"
    echo "Please ensure Python 3 is installed."
    exit 1
fi
