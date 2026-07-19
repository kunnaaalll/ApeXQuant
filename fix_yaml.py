import re

with open('docker-compose.remote.yml', 'r') as f:
    content = f.read()

# For Rust services
rust_services = ['event-bus', 'market-data-engine', 'strategy-engine', 'signal-engine', 'risk-engine', 'execution-engine', 'position-engine', 'portfolio-engine', 'analytics-engine', 'learning-engine', 'backtester']

# For TS services
ts_services = ['api', 'dashboard', 'orchestrator', 'ai-council']

def inject_env(service, envs, content):
    # Find the environment block for the service
    pattern = r'(  ' + service + r':.*?    environment:\n)(.*?(?=    [a-z]+:))'
    match = re.search(pattern, content, re.DOTALL)
    if match:
        existing_envs = match.group(2)
        new_envs = existing_envs
        for env in envs:
            if env.split('=')[0] not in existing_envs:
                new_envs += f"      - {env}\n"
        content = content[:match.start(2)] + new_envs + content[match.end(2):]
    return content

def inject_target(service, target, content):
    pattern = r'(  ' + service + r':.*?    build:\n(?:      .*?\n)+)'
    match = re.search(pattern, content)
    if match:
        build_block = match.group(1)
        if 'target:' not in build_block:
            new_build_block = build_block + f"      target: {target}\n"
            content = content[:match.start(1)] + new_build_block + content[match.end(1):]
    return content

rust_envs = [
    'OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:4317',
    'REDIS_URL=redis://redis:6379',
    'DATABASE_URL=postgres://apex:${DB_PASSWORD}@postgres:5432/apex_v3?sslmode=disable',
    'LOG_FORMAT=json'
]

ts_envs = [
    'NODE_ENV=production',
    'LOG_LEVEL=info',
    'OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:4317',
    'REDIS_URL=redis://redis:6379',
    'DATABASE_URL=postgres://apex:${DB_PASSWORD}@postgres:5432/apex_v3?sslmode=disable'
]

for s in rust_services:
    content = inject_env(s, rust_envs, content)
    content = inject_target(s, 'production', content)

for s in ts_services:
    content = inject_env(s, ts_envs, content)
    content = inject_target(s, 'production', content)

with open('docker-compose.remote.yml', 'w') as f:
    f.write(content)

