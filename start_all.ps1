$ScriptDir = $PSScriptRoot

Write-Host "Setting Environment Variables..."
$env:DATABASE_URL = "postgres://apex:apex@localhost:5432/apex_v3"
$env:REDIS_URL = "redis://localhost:6379"
$env:EVENT_BUS_URL = "http://localhost:50050"

if (Test-Path (Join-Path $ScriptDir ".env")) {
    Get-Content (Join-Path $ScriptDir ".env") | ForEach-Object {
        if ($_ -match "^\s*([A-Za-z0-9_]+)=(.*)$") {
            Set-Item -Path "Env:$($matches[1])" -Value $matches[2]
        }
    }
}

Write-Host "Starting Apex V3 System on Windows natively..."

# Start Postgres and Redis in Docker
Write-Host "Ensuring PostgreSQL and Redis are running in Docker..."
docker-compose -f (Join-Path $ScriptDir "infrastructure\docker\docker-compose.yml") up -d postgres redis

# Wait for databases to be online
Write-Host "Waiting for PostgreSQL database (port 5432) to accept connections..."
while ($true) {
    $tcp = New-Object System.Net.Sockets.TcpClient
    try {
        $tcp.Connect("127.0.0.1", 5432)
        $tcp.Close()
        Write-Host "PostgreSQL is online!"
        break
    } catch {
        Start-Sleep -Seconds 1
    }
}

Write-Host "Waiting for Redis database (port 6379) to accept connections..."
while ($true) {
    $tcp = New-Object System.Net.Sockets.TcpClient
    try {
        $tcp.Connect("127.0.0.1", 6379)
        $tcp.Close()
        Write-Host "Redis is online!"
        break
    } catch {
        Start-Sleep -Seconds 1
    }
}

# Start Python MT5 Bridge
Write-Host "Starting MT5 Bridge..."
Start-Process -FilePath "powershell.exe" -ArgumentList "-NoExit", "-Command", "python scripts/mt5_bridge.py" -WorkingDirectory $ScriptDir -WindowStyle Normal

# Start TS/Node apps using Turbo
Write-Host "Starting TypeScript Services..."
Start-Process -FilePath "powershell.exe" -ArgumentList "-NoExit", "-Command", "pnpm dev" -WorkingDirectory $ScriptDir -WindowStyle Normal

# List of Rust services to start
$rustServices = @(
    "event-bus",
    "execution-engine",
    "risk-engine",
    "signal-engine",
    "position-engine",
    "portfolio-engine",
    "analytics-engine",
    "learning-engine",
    "backtester",
    "performance-engine",
    "validator"
)

# Start each Rust service in a new window
foreach ($service in $rustServices) {
    Write-Host "Starting Rust service: $service"
    $exePath = Join-Path $ScriptDir "target\debug\$service.exe"
    
    if (Test-Path $exePath) {
        # Keep window open if it crashes so we can see the error
        Start-Process -FilePath "powershell.exe" -ArgumentList "-NoExit", "-Command", "& '$exePath'" -WorkingDirectory $ScriptDir -WindowStyle Normal
    } else {
        Write-Warning "Could not find executable for $service at $exePath."
    }
}

Write-Host "All services have been triggered!"
