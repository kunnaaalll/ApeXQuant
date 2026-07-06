#!/bin/bash
set -e

# Official MetaQuotes Linux prefix (matches Dockerfile WINEPREFIX)
export WINEPREFIX="${WINEPREFIX:-/root/.mt5}"
export DISPLAY=:99

echo "Starting Xvfb..."
Xvfb :99 -screen 0 1024x768x16 &
XVFB_PID=$!

echo "Waiting for Xvfb to be ready..."
sleep 2

# MT5 terminal path inside the official ~/.mt5 Wine prefix
MT5_PATH="$WINEPREFIX/drive_c/Program Files/MetaTrader 5/terminal64.exe"

if [ -f "$MT5_PATH" ]; then
    echo "Starting MetaTrader 5..."
    WINEPREFIX="$WINEPREFIX" wine "$MT5_PATH" /portable &
    MT5_PID=$!
    echo "Waiting for MT5 to initialize (60s)..."
    # MT5 downloads updates and opens a socket on first boot — give it time
    sleep 60
else
    echo "WARNING: terminal64.exe not found at $MT5_PATH"
    echo "mt5setup.exe may have failed or the terminal needs a first-run GUI login."
    echo "Mount a pre-configured Wine prefix volume to skip this."
fi

echo "Starting FastAPI bridge (native Linux Python3)..."
# The MetaTrader5 Python package connects to the MT5 terminal via localhost socket.
# This runs as a native Linux process — no Wine needed for the bridge itself.
exec python3 /app/mt5_bridge.py
