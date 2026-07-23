#!/bin/bash
set -e

# Official MetaQuotes Linux prefix (matches Dockerfile WINEPREFIX)
export WINEPREFIX="${WINEPREFIX:-/root/.mt5}"
export DISPLAY=:99

echo "Starting Xvfb..."
rm -f /tmp/.X99-lock
Xvfb :99 -screen 0 1024x768x16 &
XVFB_PID=$!

echo "Waiting for Xvfb to be ready..."
sleep 2

# MT5 terminal path inside the official ~/.mt5 Wine prefix
MT5_PATH="$WINEPREFIX/drive_c/Program Files/MetaTrader 5/terminal64.exe"

if [ -f "$MT5_PATH" ]; then
    echo "Creating MT5 startup configuration with AutoTrading enabled..."
    mkdir -p /root/.mt5/drive_c/app
    cat <<EOF > /root/.mt5/drive_c/app/mt5_config.ini
[Common]
Login=${MT5_LOGIN}
Password=${MT5_PASSWORD}
Server=${MT5_SERVER}
AutoConfiguration=1

[Experts]
AllowLiveTrading=1
Enabled=1
Account=1
Profile=1
DLLImport=1
EOF

    echo "Starting MetaTrader 5..."
    WINEPREFIX="$WINEPREFIX" wine "$MT5_PATH" /portable /config:"C:\\app\\mt5_config.ini" &
    MT5_PID=$!
    echo "Waiting for MT5 to initialize (60s)..."
    # MT5 downloads updates and opens a socket on first boot — give it time
    sleep 60
else
    echo "WARNING: terminal64.exe not found at $MT5_PATH"
    echo "mt5setup.exe may have failed or the terminal needs a first-run GUI login."
    echo "Mount a pre-configured Wine prefix volume to skip this."
fi

echo "Starting FastAPI bridge (Wine python.exe — MetaTrader5 requires Windows IPC)..."
# MetaTrader5 is a Windows-only C extension — must run inside Wine alongside terminal64.exe
exec wine 'C:\Python310\python.exe' /app/mt5_bridge.py
