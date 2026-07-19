#!/usr/bin/env python3
"""
APEX V3 — MT5 Historical Data Seeder
Fetches M1 historical candles from MT5 Bridge and interpolates them into ticks
for accurate tick-level backtesting across all configured symbols.
"""

import os
import sys
import json
import urllib.request
import logging
import psycopg2
import psycopg2.extras

logging.basicConfig(level=logging.INFO, format='%(asctime)s [%(levelname)s] %(message)s')
log = logging.getLogger("historical-seeder")

MT5_BRIDGE   = os.getenv("MT5_BRIDGE_URL", "http://localhost:8000")
DATABASE_URL = os.getenv("DATABASE_URL",   "postgres://apex:apex@localhost:5432/apex_v3")
CANDLES_COUNT = int(os.getenv("CANDLES_COUNT", "10000")) # ~1 week of M1

# All symbols configured in APEX
SYMBOLS = [
    "EURUSD","USDJPY","GBPUSD","AUDUSD","USDCAD",
    "USDCHF","NZDUSD","EURGBP","EURJPY","GBPJPY",
    "XAUUSD","US30","BTCUSD",
]

def parse_dsn(url: str) -> dict:
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

def fetch_rates(symbol: str, count: int) -> list:
    url = f"{MT5_BRIDGE}/history/rates/{symbol}?count={count}"
    try:
        req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'})
        with urllib.request.urlopen(req, timeout=30) as r:
            return json.loads(r.read())
    except Exception as e:
        log.error(f"Failed to fetch rates for {symbol}: {e}")
        return []

def synthesize_ticks(symbol: str, rates: list) -> list:
    """Interpolates M1 candles into 4 structural ticks (O, H, L, C) per minute."""
    ticks = []
    for r in rates:
        t_base = int(r["time"]) * 1000
        o, h, l, c = r["open"], r["high"], r["low"], r["close"]
        spread = r.get("spread", 1)
        spread_price = spread * 0.00001 # Approximation, will be rounded anyway

        # Structure sequence based on candle direction
        if c >= o:
            # Bullish: Open -> Low -> High -> Close
            seq = [o, l, h, c]
        else:
            # Bearish: Open -> High -> Low -> Close
            seq = [o, h, l, c]

        # Space out by 15 seconds
        for i, price in enumerate(seq):
            bid = price
            ask = price + spread_price
            ticks.append({
                "symbol": symbol,
                "bid": bid,
                "ask": ask,
                "last": price,
                "volume": r.get("tick_volume", 1) / 4.0,
                "spread": spread_price,
                "timestamp_ms": t_base + (i * 15000)
            })
            
    return ticks

def main():
    log.info("Starting Historical Data Seeder...")
    conn = get_conn()
    conn.autocommit = False

    for symbol in SYMBOLS:
        log.info(f"Fetching {CANDLES_COUNT} M1 candles for {symbol}...")
        rates = fetch_rates(symbol, CANDLES_COUNT)
        if not rates:
            continue
            
        ticks = synthesize_ticks(symbol, rates)
        log.info(f"Synthesized {len(ticks)} ticks for {symbol}. Inserting to DB...")
        
        try:
            with conn.cursor() as cur:
                psycopg2.extras.execute_batch(cur, INSERT_SQL, ticks, page_size=5000)
            conn.commit()
            log.info(f"✅ Success: {symbol}")
        except Exception as e:
            conn.rollback()
            log.error(f"❌ DB insert error for {symbol}: {e}")
            
    log.info("Historical data seeding complete.")

if __name__ == "__main__":
    main()
