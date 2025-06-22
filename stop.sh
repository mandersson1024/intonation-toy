#!/bin/bash

# Stop development server script for pitch-toy
# Handles both Yew (trunk) and legacy Ruby servers

DEV_PORT=8080
WS_PORT=8081

echo "🛑 Stopping pitch-toy development servers..."

# Stop main development server on port 8080
echo "🔄 Checking port ${DEV_PORT}..."
PIDS=$(lsof -ti:${DEV_PORT} 2>/dev/null || echo "")

if [ -n "$PIDS" ]; then
    echo "📝 Found processes on port ${DEV_PORT}: $PIDS"
    for PID in $PIDS; do
        echo "🔄 Killing process $PID..."
        kill -TERM $PID 2>/dev/null || kill -9 $PID 2>/dev/null
    done
    echo "✅ Server on port ${DEV_PORT} stopped!"
else
    echo "ℹ️  No server running on port ${DEV_PORT}"
fi

# Stop Yew hot reload WebSocket server on port 8081
echo "🔄 Checking WebSocket port ${WS_PORT}..."
WS_PIDS=$(lsof -ti:${WS_PORT} 2>/dev/null || echo "")

if [ -n "$WS_PIDS" ]; then
    echo "📝 Found WebSocket processes on port ${WS_PORT}: $WS_PIDS"
    for PID in $WS_PIDS; do
        echo "🔄 Killing WebSocket process $PID..."
        kill -TERM $PID 2>/dev/null || kill -9 $PID 2>/dev/null
    done
    echo "✅ WebSocket server on port ${WS_PORT} stopped!"
else
    echo "ℹ️  No WebSocket server running on port ${WS_PORT}"
fi

# Kill any remaining trunk processes
TRUNK_PIDS=$(pgrep -f "trunk serve" 2>/dev/null || echo "")
if [ -n "$TRUNK_PIDS" ]; then
    echo "🦀 Stopping remaining trunk processes: $TRUNK_PIDS"
    for PID in $TRUNK_PIDS; do
        kill -TERM $PID 2>/dev/null || kill -9 $PID 2>/dev/null
    done
    echo "✅ All trunk processes stopped!"
fi

echo "🏁 All development servers stopped!" 