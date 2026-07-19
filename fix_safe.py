with open("docker-compose.remote.yml", "r") as f:
    lines = f.readlines()

rust_envs = [
    "      - OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:4317\n",
    "      - REDIS_URL=redis://redis:6379\n",
    "      - DATABASE_URL=postgres://apex:${DB_PASSWORD}@postgres:5432/apex_v3?sslmode=disable\n",
    "      - LOG_FORMAT=json\n"
]
ts_envs = [
    "      - NODE_ENV=production\n",
    "      - LOG_LEVEL=info\n",
    "      - OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:4317\n",
    "      - REDIS_URL=redis://redis:6379\n",
    "      - DATABASE_URL=postgres://apex:${DB_PASSWORD}@postgres:5432/apex_v3?sslmode=disable\n"
]

rust_services = ['event-bus:', 'market-data-engine:', 'strategy-engine:', 'signal-engine:', 'risk-engine:', 'execution-engine:', 'position-engine:', 'portfolio-engine:', 'analytics-engine:', 'learning-engine:', 'backtester:']
ts_services = ['api:', 'dashboard:', 'orchestrator:', 'ai-council:']

current_service = None
service_type = None
in_build = False

out_lines = []
for i, line in enumerate(lines):
    if line.startswith("  ") and not line.startswith("   "):
        current_service = line.strip()
        if current_service in rust_services:
            service_type = "rust"
        elif current_service in ts_services:
            service_type = "ts"
        else:
            service_type = None

    if service_type and line.startswith("    build:"):
        out_lines.append(line)
        out_lines.append("      target: production\n")
        continue

    # Note: this injects envs right after "environment:" which works well.
    if service_type and line.startswith("    environment:"):
        out_lines.append(line)
        envs = rust_envs if service_type == "rust" else ts_envs
        out_lines.extend(envs)
        continue

    out_lines.append(line)

with open("docker-compose.remote.yml", "w") as f:
    f.writelines(out_lines)

