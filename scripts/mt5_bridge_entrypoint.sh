#!/bin/bash
set -e

echo "Starting Xvfb..."
# Start Xvfb on display :99
export DISPLAY=:99
Xvfb :99 -screen 0 1024x768x16 &
XVFB_PID=$!

echo "Waiting for Xvfb to be ready..."
sleep 2

# Path to terminal64.exe. Depending on the installation it is usually in Program Files
MT5_PATH="$WINEPREFIX/drive_c/Program Files/MetaTrader 5/terminal64.exe"

if [ -f "$MT5_PATH" ]; then
    echo "Starting MetaTrader 5..."
    wine "$MT5_PATH" /portable &
    MT5_PID=$!
    echo "Waiting for MT5 to initialize..."
    # MT5 might take some time to download updates/connect on first boot
    sleep 15
else
    echo "WARNING: terminal64.exe not found at $MT5_PATH"
    echo "The mt5setup.exe might have failed to download payload. Please install manually if needed."
fi

echo "Starting FastAPI mt5_bridge.py..."
# Start the Python bridge
wine python mt5_bridge.py

# Cleanup if python exits
kill $MT5_PID || true
kill $XVFB_PID || true
