#!/usr/bin/env python3
"""
APEX V3 — MT5 Tick Poller (HTTP Sidecar)
Polls mt5-bridge HTTP API and writes ticks to PostgreSQL ticks table.
Runs as a background process on the GCP instance.
"""

import asyncio
import time
import json
import urllib.request
import urllib.error
import psycopg2
import psycopg2.extras
import sys
import os
import logging

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s [%(levelname)s] %(message)s',
    stream=sys.stdout
)
log = logging.getLogger("tick-poller")

# ── Config ────────────────────────────────────────────────────────────────────

MT5_BRIDGE   = os.getenv("MT5_BRIDGE_URL", "http://localhost:8000")
DATABASE_URL = os.getenv("DATABASE_URL",   "postgres://apex:apex@localhost:5432/apex_v3")
POLL_MS      = int(os.getenv("POLL_MS",    "500"))   # poll every 500ms per symbol

# All symbols — MT5 handles forex/metals/indices; crypto skipped (no MT5 feed)
SYMBOLS = [
    "EURUSD","USDJPY","GBPUSD","AUDUSD","USDCAD",
    "USDCHF","NZDUSD","EURGBP","EURJPY","GBPJPY",
    "XAUUSD","US30","BTCUSD",
]

# ── DB helpers ────────────────────────────────────────────────────────────────

def parse_dsn(url: str) -> dict:
    """Parse postgres://user:pass@host:port/db into psycopg2 kwargs."""
    url = url.replace("postgres://", "").replace("postgresql://", "")
    creds, rest = url.split("@", 1)
    user, password = creds.split(":", 1)
    hostport, dbname = rest.split("/", 1)
    dbname = dbname.split("?")[0]
    if ":" in hostport:
        host, port = hostport.split(":", 1)
    else:
        host, port = hostport, "5432"
    return dict(host=host, port=int(port), dbname=dbname, user=user, password=password)

def get_conn():
    return psycopg2.connect(**parse_dsn(DATABASE_URL))

INSERT_SQL = """
INSERT INTO ticks (symbol, bid, ask, last, volume, spread, timestamp_ms)
VALUES (%(symbol)s, %(bid)s, %(ask)s, %(last)s, %(volume)s, %(spread)s, %(timestamp_ms)s)
ON CONFLICT (symbol, timestamp_ms) DO NOTHING
"""

# ── HTTP helpers ──────────────────────────────────────────────────────────────

def fetch_tick(symbol: str) -> dict | None:
    url = f"{MT5_BRIDGE}/symbols/{symbol}/tick"
    try:
        with urllib.request.urlopen(url, timeout=2) as r:
            data = json.loads(r.read())
            if "bid" not in data:
                return None
            return data
    except Exception:
        return None

# ── Main loop ────────────────────────────────────────────────────────────────

def main():
    log.info("APEX V3 Tick Poller starting...")
    log.info(f"  Bridge:   {MT5_BRIDGE}")
    log.info(f"  DB:       {DATABASE_URL.split('@')[1] if '@' in DATABASE_URL else DATABASE_URL}")
    log.info(f"  Symbols:  {SYMBOLS}")
    log.info(f"  Interval: {POLL_MS}ms")

    conn = get_conn()
    conn.autocommit = False
    log.info("PostgreSQL connected.")

    # Tracking last timestamp per symbol to avoid duplicates
    last_ts: dict[str, int] = {}
    total_inserted = 0
    errors = 0

    while True:
        loop_start = time.time()

        batch = []
        for symbol in SYMBOLS:
            tick = fetch_tick(symbol)
            if not tick:
                continue

            ts_ms = int(tick.get("time", 0)) * 1000
            if ts_ms == 0:
                ts_ms = int(time.time() * 1000)

            # Skip duplicate timestamps
            if last_ts.get(symbol) == ts_ms:
                continue
            last_ts[symbol] = ts_ms

            bid    = float(tick.get("bid", 0))
            ask    = float(tick.get("ask", 0))
            last   = float(tick.get("last", 0))
            spread = round(ask - bid, 8)

            if bid <= 0 or ask <= 0:
                continue

            batch.append({
                "symbol":       symbol,
                "bid":          bid,
                "ask":          ask,
                "last":         last,
                "volume":       0,
                "spread":       spread,
                "timestamp_ms": ts_ms,
            })

        if batch:
            try:
                with conn.cursor() as cur:
                    psycopg2.extras.execute_batch(cur, INSERT_SQL, batch)
                conn.commit()
                total_inserted += len(batch)
                syms = [b["symbol"] for b in batch]
                log.info(f"Inserted {len(batch)} ticks: {syms} | Total: {total_inserted}")
            except Exception as e:
                conn.rollback()
                errors += 1
                log.error(f"DB insert error: {e}")
                if errors > 10:
                    log.warning("Reconnecting to DB...")
                    try:
                        conn.close()
                    except Exception:
                        pass
                    conn = get_conn()
                    conn.autocommit = False
                    errors = 0

        elapsed_ms = (time.time() - loop_start) * 1000
        sleep_ms = max(0, POLL_MS - elapsed_ms)
        time.sleep(sleep_ms / 1000)

if __name__ == "__main__":
    main()
