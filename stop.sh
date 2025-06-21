#!/bin/bash

# Stop development server script for pitch-toy

DEV_PORT=8080

echo "🛑 Stopping pitch-toy development server on port ${DEV_PORT}..."

# Find and kill processes on the port
PIDS=$(lsof -ti:${DEV_PORT} 2>/dev/null || echo "")

if [ -n "$PIDS" ]; then
    echo "📝 Found processes: $PIDS"
    for PID in $PIDS; do
        echo "🔄 Killing process $PID..."
        kill -TERM $PID 2>/dev/null || kill -9 $PID 2>/dev/null
    done
    echo "✅ Server stopped!"
else
    echo "ℹ️  No server running on port ${DEV_PORT}"
fi 