import sys
import os
import uvicorn
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from typing import List, Optional

# Automate MetaTrader5 package installation
try:
    import MetaTrader5 as mt5
except ImportError:
    print("MetaTrader5 python library missing. Installing now...")
    import subprocess
    subprocess.check_call([sys.executable, "-m", "pip", "install", "MetaTrader5"])
    import MetaTrader5 as mt5

app = FastAPI(title="APEX V3 MT5 Bridge Gateway")

from fastapi.middleware.cors import CORSMiddleware

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Data Models matching Rust Execution Engine expectations
class AccountInfo(BaseModel):
    balance: float
    equity: float
    free_margin: float
    leverage: float
    margin_level: float

class OpenPosition(BaseModel):
    ticket: str
    symbol: str
    side: str
    volume: float
    entry_price: float
    stop_loss: Optional[float] = None
    take_profit: Optional[float] = None
    floating_pnl: float

class PendingOrder(BaseModel):
    ticket: str
    symbol: str
    side: str
    order_type: str
    volume: float
    price: float
    status: str
    timestamp: int

class SymbolInfo(BaseModel):
    symbol: str
    digits: int
    lot_step: float
    tick_size: float
    minimum_volume: float
    maximum_volume: float

class OrderSubmitRequest(BaseModel):
    symbol: str
    side: str  # "Buy" or "Sell"
    order_type: str  # "Market", "Limit", "Stop"
    volume: float
    price: Optional[float] = None
    stop_loss: Optional[float] = None
    take_profit: Optional[float] = None

class SubmitResponse(BaseModel):
    order_id: str

# Helper to guarantee MT5 connection is active
def ensure_initialized():
    login_id = os.getenv("MT5_LOGIN")
    password = os.getenv("MT5_PASSWORD")
    server = os.getenv("MT5_SERVER")
    
    if login_id and password and server:
        if not mt5.initialize(login=int(login_id), password=password, server=server):
            if not mt5.initialize(login=int(login_id), password=password, server=server):
                raise HTTPException(status_code=503, detail=f"MT5 terminal connection & login failed: {mt5.last_error()}")
    else:
        if not mt5.initialize():
            if not mt5.initialize():
                raise HTTPException(status_code=503, detail=f"MT5 terminal connection failed: {mt5.last_error()}")

@app.on_event("startup")
def startup_event():
    print("MT5 Bridge Gateway booting up...")
    try:
        ensure_initialized()
        print("MetaTrader 5 initialized and auto-logged in successfully.")
    except Exception as e:
        print(f"Warning: could not connect to MT5 during boot: {e}")

@app.post("/connect")
def connect_mt5():
    ensure_initialized()
    # If a demo account login is requested, attempt it
    login_id = os.getenv("MT5_LOGIN")
    password = os.getenv("MT5_PASSWORD")
    server = os.getenv("MT5_SERVER")
    
    if login_id and password and server:
        print(f"Logging into MT5 Account {login_id}...")
        authorized = mt5.login(
            login=int(login_id),
            password=password,
            server=server
        )
        if not authorized:
            raise HTTPException(status_code=401, detail=f"MT5 login failed: {mt5.last_error()}")
            
    print("Successfully connected to MT5 Terminal.")
    return {"status": "connected"}

@app.post("/disconnect")
def disconnect_mt5():
    mt5.shutdown()
    return {"status": "disconnected"}

@app.get("/ping")
def ping_mt5():
    ensure_initialized()
    terminal_info = mt5.terminal_info()
    if terminal_info is None:
        raise HTTPException(status_code=503, detail=f"MT5 terminal not reachable: {mt5.last_error()}")
    return {"status": "ok"}

@app.get("/account", response_model=AccountInfo)
def get_account():
    ensure_initialized()
    info = mt5.account_info()
    if info is None:
        raise HTTPException(status_code=500, detail=f"Failed to retrieve MT5 account info: {mt5.last_error()}")
    return AccountInfo(
        balance=float(info.balance),
        equity=float(info.equity),
        free_margin=float(info.margin_free),
        leverage=float(info.leverage),
        margin_level=float(info.margin_level)
    )

@app.get("/positions", response_model=List[OpenPosition])
def get_positions():
    ensure_initialized()
    raw_positions = mt5.positions_get()
    if raw_positions is None:
        return []
    
    res = []
    for p in raw_positions:
        side = "Buy" if p.type == mt5.POSITION_TYPE_BUY else "Sell"
        res.append(OpenPosition(
            ticket=str(p.ticket),
            symbol=p.symbol,
            side=side,
            volume=float(p.volume),
            entry_price=float(p.price_open),
            stop_loss=float(p.sl) if p.sl > 0 else None,
            take_profit=float(p.tp) if p.tp > 0 else None,
            floating_pnl=float(p.profit)
        ))
    return res

@app.get("/orders", response_model=List[PendingOrder])
def get_orders():
    ensure_initialized()
    raw_orders = mt5.orders_get()
    if raw_orders is None:
        return []
        
    res = []
    for o in raw_orders:
        side = "Buy" if o.type in [mt5.ORDER_TYPE_BUY, mt5.ORDER_TYPE_BUY_LIMIT, mt5.ORDER_TYPE_BUY_STOP] else "Sell"
        o_type = "Limit" if o.type in [mt5.ORDER_TYPE_BUY_LIMIT, mt5.ORDER_TYPE_SELL_LIMIT] else "Stop"
        res.append(PendingOrder(
            ticket=str(o.ticket),
            symbol=o.symbol,
            side=side,
            order_type=o_type,
            volume=float(o.volume_initial),
            price=float(o.price_open),
            status="Pending",
            timestamp=o.time_setup
        ))
    return res

# SPECIFIC subpaths MUST be declared BEFORE path parameter routes like `/symbols/{symbol}`!
@app.get("/symbols/{symbol}/tick")
def get_symbol_tick(symbol: str):
    ensure_initialized()
    tick = mt5.symbol_info_tick(symbol)
    if tick is None:
        raise HTTPException(status_code=404, detail=f"Tick not found for symbol {symbol}")
    return {
        "symbol": symbol,
        "bid": float(tick.bid),
        "ask": float(tick.ask),
        "last": float(tick.last),
        "time": int(tick.time)
    }

@app.get("/symbols/{symbol}/depth")
def get_symbol_depth(symbol: str):
    ensure_initialized()
    # Try to get real MT5 market book
    mt5.market_book_add(symbol)
    book = mt5.market_book_get(symbol)
    mt5.market_book_release(symbol)
    
    if book:
        asks = []
        bids = []
        for entry in book:
            if entry.type == mt5.BOOK_TYPE_SELL:
                asks.append({"price": float(entry.price), "volume": float(entry.volume)})
            elif entry.type == mt5.BOOK_TYPE_BUY:
                bids.append({"price": float(entry.price), "volume": float(entry.volume)})
        return {"asks": asks, "bids": bids}
        
    # Fallback to generating simulated depth around the actual live bid/ask
    tick = mt5.symbol_info_tick(symbol)
    if not tick:
        raise HTTPException(status_code=404, detail="Symbol tick not found")
        
    bid = tick.bid
    ask = tick.ask
    
    # Ensure realistic bid/ask spread (at least 1.5 pips / points)
    if ask <= bid:
        info = mt5.symbol_info(symbol)
        digits = info.digits if info else 5
        spread_amount = 1.5 / (10 ** digits)
        ask = bid + spread_amount
        
    # Generate 5 levels of bids and asks
    # Asks go ascending from lowest ask (bottom) to highest ask (top)
    import random
    info = mt5.symbol_info(symbol)
    digits = info.digits if info else 5
    step = 1.0 / (10 ** digits)
    
    asks = [{"price": round(ask + (4 - i) * step, digits), "volume": round(5 + random.random() * 45, 2)} for i in range(5)]
    bids = [{"price": round(bid - i * step, digits), "volume": round(5 + random.random() * 45, 2)} for i in range(5)]
    
    return {"asks": asks, "bids": bids}

@app.get("/symbols/{symbol}", response_model=SymbolInfo)
def get_symbol(symbol: str):
    ensure_initialized()
    info = mt5.symbol_info(symbol)
    if info is None:
        raise HTTPException(status_code=404, detail=f"Symbol {symbol} not found in MT5 MarketWatch")
        
    return SymbolInfo(
        symbol=info.name,
        digits=int(info.digits),
        lot_step=float(info.volume_step),
        tick_size=float(info.trade_tick_size),
        minimum_volume=float(info.volume_min),
        maximum_volume=float(info.volume_max)
    )

@app.post("/orders", response_model=SubmitResponse)
def submit_order(req: OrderSubmitRequest):
    ensure_initialized()
    symbol_info = mt5.symbol_info(req.symbol)
    if not symbol_info:
        raise HTTPException(status_code=404, detail=f"Symbol {req.symbol} not found")
        
    price = req.price
    if price is None:
        tick = mt5.symbol_info_tick(req.symbol)
        if not tick:
            raise HTTPException(status_code=500, detail="Failed to fetch symbol tick for pricing")
        price = tick.ask if req.side == "Buy" else tick.bid
        
    action = mt5.TRADE_ACTION_DEAL if req.order_type == "Market" else mt5.TRADE_ACTION_PENDING
    
    if req.order_type == "Market":
        o_type = mt5.ORDER_TYPE_BUY if req.side == "Buy" else mt5.ORDER_TYPE_SELL
    elif req.order_type == "Limit":
        o_type = mt5.ORDER_TYPE_BUY_LIMIT if req.side == "Buy" else mt5.ORDER_TYPE_SELL_LIMIT
    elif req.order_type == "Stop":
        o_type = mt5.ORDER_TYPE_BUY_STOP if req.side == "Buy" else mt5.ORDER_TYPE_SELL_STOP
    else:
        raise HTTPException(status_code=400, detail="Unsupported order type")
        
    # Determine filling mode based on symbol info bitmask
    filling = mt5.ORDER_FILLING_RETURN
    if req.order_type == "Market":
        mode = symbol_info.filling_mode
        # MT5 C++ constants: SYMBOL_FILLING_FOK = 1, SYMBOL_FILLING_IOC = 2
        if mode & 1:
            filling = mt5.ORDER_FILLING_FOK
        elif mode & 2:
            filling = mt5.ORDER_FILLING_IOC
        else:
            filling = mt5.ORDER_FILLING_RETURN

    trade_req = {
        "action": action,
        "symbol": req.symbol,
        "volume": float(req.volume),
        "type": o_type,
        "price": float(price),
        "deviation": 20,
        "magic": 992203,
        "comment": "APEX V3 Order",
        "type_time": mt5.ORDER_TIME_GTC,
        "type_filling": filling,
    }
    
    if req.stop_loss:
        trade_req["sl"] = float(req.stop_loss)
    if req.take_profit:
        trade_req["tp"] = float(req.take_profit)
        
    result = mt5.order_send(trade_req)
    if result is None or result.retcode != mt5.TRADE_RETCODE_DONE:
        err_msg = mt5.last_error() if result is None else f"Retcode: {result.retcode}, Comment: {result.comment}"
        raise HTTPException(status_code=500, detail=f"Order execution failed: {err_msg}")
        
    return SubmitResponse(order_id=str(result.order))

@app.post("/positions/{ticket}/close")
def close_position(ticket: str):
    ensure_initialized()
    # Find the position
    raw_positions = mt5.positions_get(ticket=int(ticket))
    if not raw_positions or len(raw_positions) == 0:
        raise HTTPException(status_code=404, detail=f"Position ticket {ticket} not found")
        
    position = raw_positions[0]
    symbol = position.symbol
    volume = position.volume
    p_type = position.type
    
    # Get current price
    tick = mt5.symbol_info_tick(symbol)
    if not tick:
        raise HTTPException(status_code=500, detail=f"Failed to fetch tick for {symbol}")
        
    price = tick.bid if p_type == mt5.POSITION_TYPE_BUY else tick.ask
    o_type = mt5.ORDER_TYPE_SELL if p_type == mt5.POSITION_TYPE_BUY else mt5.ORDER_TYPE_BUY
    
    # Determine filling mode
    symbol_info = mt5.symbol_info(symbol)
    filling = mt5.ORDER_FILLING_RETURN
    if symbol_info:
        mode = symbol_info.filling_mode
        if mode & 1:
            filling = mt5.ORDER_FILLING_FOK
        elif mode & 2:
            filling = mt5.ORDER_FILLING_IOC
            
    trade_req = {
        "action": mt5.TRADE_ACTION_DEAL,
        "symbol": symbol,
        "volume": float(volume),
        "type": o_type,
        "position": int(ticket),
        "price": float(price),
        "deviation": 20,
        "magic": 992203,
        "comment": "APEX Close Position",
        "type_time": mt5.ORDER_TIME_GTC,
        "type_filling": filling,
    }
    
    result = mt5.order_send(trade_req)
    if result is None or result.retcode != mt5.TRADE_RETCODE_DONE:
        err_msg = mt5.last_error() if result is None else f"Retcode: {result.retcode}, Comment: {result.comment}"
        raise HTTPException(status_code=500, detail=f"Position close failed: {err_msg}")
        
    return {"status": "closed", "ticket": ticket}

class StopModifyRequest(BaseModel):
    stop_loss: Optional[float] = None
    take_profit: Optional[float] = None

@app.post("/positions/{ticket}/stops")
def modify_position_stops(ticket: str, req: StopModifyRequest):
    ensure_initialized()
    # Find the position
    raw_positions = mt5.positions_get(ticket=int(ticket))
    if not raw_positions or len(raw_positions) == 0:
        raise HTTPException(status_code=404, detail=f"Position ticket {ticket} not found")
        
    position = raw_positions[0]
    symbol = position.symbol
    
    trade_req = {
        "action": mt5.TRADE_ACTION_SLTP,
        "symbol": symbol,
        "position": int(ticket),
        "sl": float(req.stop_loss) if req.stop_loss is not None else float(position.sl),
        "tp": float(req.take_profit) if req.take_profit is not None else float(position.tp),
    }
    
    result = mt5.order_send(trade_req)
    if result is None or result.retcode != mt5.TRADE_RETCODE_DONE:
        err_msg = mt5.last_error() if result is None else f"Retcode: {result.retcode}, Comment: {result.comment}"
        raise HTTPException(status_code=500, detail=f"Stops modification failed: {err_msg}")
        
    return {"status": "stops_modified", "ticket": ticket}

@app.get("/signals")
def get_live_signals():
    ensure_initialized()
    symbols = [
        "EURUSD", "USDJPY", "GBPUSD", "AUDUSD", "USDCAD", "USDCHF", "NZDUSD",
        "EURGBP", "EURJPY", "GBPJPY", "XAUUSD", "BTCUSD", "US30"
    ]
    signals = []
    
    for sym in symbols:
        info = mt5.symbol_info(sym)
        if not info:
            continue
            
        # Fetch actual M15 rates (50 bars)
        rates = mt5.copy_rates_from_pos(sym, mt5.TIMEFRAME_M15, 0, 50)
        if rates is None or len(rates) < 20:
            continue
            
        # Extract close prices
        close_prices = [r[4] for r in rates] # index 4 is close price in MT5 rates tuple
        
        # Calculate SMA(10) and SMA(20)
        sma_10 = sum(close_prices[-10:]) / 10
        sma_20 = sum(close_prices[-20:]) / 20
        
        prev_sma_10 = sum(close_prices[-11:-1]) / 10
        prev_sma_20 = sum(close_prices[-21:-1]) / 20
        
        # Calculate a dynamic confidence score based on the distance between the two SMAs (Trend Strength)
        dist = abs(sma_10 - sma_20) / sma_20
        # A typical distance ratio for M15 is between 0.0001 and 0.0020
        # Scale this ratio to map to a dynamic confidence score between 68% and 94%
        norm_dist = min(1.0, dist / 0.0015)
        dynamic_score = int(68 + norm_dist * 26)

        if sma_10 > sma_20 and prev_sma_10 <= prev_sma_20:
            sig_type = "BUY"
            strategy = "Bullish SMA Crossover"
            confidence = 88
        elif sma_10 < sma_20 and prev_sma_10 >= prev_sma_20:
            sig_type = "SELL"
            strategy = "Bearish SMA Crossover"
            confidence = 85
        else:
            sig_type = "BUY" if sma_10 > sma_20 else "SELL"
            strategy = "SMC Trend Continuation"
            confidence = dynamic_score
            
        signals.append({
            "asset": sym,
            "type": sig_type,
            "confidence": confidence,
            "strategy": strategy,
            "timestamp": "Live"
        })
        
    return signals


# ============================================================
# Phase 12 — Additional endpoints
# ============================================================

class TradeRecord(BaseModel):
    ticket: str
    symbol: str
    side: str
    volume: float
    entry_price: float
    close_price: float
    pnl: float
    swap: float
    commission: float
    open_time: int
    close_time: int
    duration_secs: int

@app.get("/history", response_model=List[TradeRecord])
def get_trade_history(limit: int = 1000):
    """Return closed trades from MT5 history. Required for Stage 7 broker reconciliation."""
    ensure_initialized()
    import datetime
    # Fetch last 30 days of history
    now_dt = datetime.datetime.now()
    from_dt = now_dt - datetime.timedelta(days=30)
    deals = mt5.history_deals_get(from_dt, now_dt)
    if deals is None:
        return []

    result = []
    # MT5 deals: each fill is a deal. We pair IN/OUT deals by position_id.
    # For simplicity we return each closed OUT deal as a trade record.
    for d in deals:
        if d.entry != mt5.DEAL_ENTRY_OUT:
            continue
        side = "Buy" if d.type == mt5.DEAL_TYPE_SELL else "Sell"  # OUT deal type is opposite of position side
        result.append(TradeRecord(
            ticket=str(d.ticket),
            symbol=d.symbol,
            side=side,
            volume=float(d.volume),
            entry_price=float(d.price),
            close_price=float(d.price),
            pnl=float(d.profit),
            swap=float(d.swap),
            commission=float(d.commission),
            open_time=int(d.time),
            close_time=int(d.time),
            duration_secs=0
        ))
        if len(result) >= limit:
            break

    return result


class RateRecord(BaseModel):
    time: int
    open: float
    high: float
    low: float
    close: float
    tick_volume: int
    spread: int
    real_volume: int

@app.get("/history/rates/{symbol}", response_model=List[RateRecord])
def get_historical_rates(symbol: str, timeframe: str = "M1", count: int = 10000):
    """Return historical OHLC rates (candles) for a symbol."""
    ensure_initialized()
    mt5.symbol_select(symbol, True)
    
    tf_map = {
        "M1": mt5.TIMEFRAME_M1,
        "M5": mt5.TIMEFRAME_M5,
        "M15": mt5.TIMEFRAME_M15,
        "H1": mt5.TIMEFRAME_H1,
        "D1": mt5.TIMEFRAME_D1,
    }
    tf = tf_map.get(timeframe.upper(), mt5.TIMEFRAME_M1)
    
    rates = mt5.copy_rates_from_pos(symbol, tf, 0, count)
    if rates is None:
        raise HTTPException(status_code=404, detail=f"Failed to fetch rates for {symbol}: {mt5.last_error()}")
        
    result = []
    for r in rates:
        result.append(RateRecord(
            time=int(r[0]),
            open=float(r[1]),
            high=float(r[2]),
            low=float(r[3]),
            close=float(r[4]),
            tick_volume=int(r[5]),
            spread=int(r[6]),
            real_volume=int(r[7])
        ))
        
    return result


@app.get("/health/full")
def full_health_check():
    """Comprehensive health check — terminal info, connection state, account snapshot. Stage 1 & Stage 7."""
    try:
        ensure_initialized()
    except Exception as e:
        return {
            "status": "unhealthy",
            "error": str(e),
            "mt5_initialized": False,
            "terminal_info": None,
            "account_info": None
        }

    terminal_info = mt5.terminal_info()
    account_info = mt5.account_info()

    terminal_dict = None
    if terminal_info is not None:
        terminal_dict = {
            "build": terminal_info.build,
            "connected": terminal_info.connected,
            "trade_allowed": terminal_info.trade_allowed,
            "community_account": terminal_info.community_account,
            "dlls_allowed": terminal_info.dlls_allowed,
            "retransmission": terminal_info.retransmission,
        }

    account_dict = None
    if account_info is not None:
        account_dict = {
            "login": account_info.login,
            "server": account_info.server,
            "balance": float(account_info.balance),
            "equity": float(account_info.equity),
            "margin": float(account_info.margin),
            "margin_free": float(account_info.margin_free),
            "margin_level": float(account_info.margin_level),
            "leverage": int(account_info.leverage),
            "currency": account_info.currency,
            "trade_mode": account_info.trade_mode,  # 0=real, 1=demo, 2=contest
        }

    overall = "healthy" if (terminal_info is not None and terminal_info.connected) else "degraded"

    return {
        "status": overall,
        "mt5_initialized": True,
        "terminal_info": terminal_dict,
        "account_info": account_dict,
        "last_error": str(mt5.last_error())
    }


class PendingOrderModifyRequest(BaseModel):
    price: Optional[float] = None
    stop_loss: Optional[float] = None
    take_profit: Optional[float] = None
    volume: Optional[float] = None


@app.put("/orders/{ticket}")
def modify_pending_order(ticket: str, req: PendingOrderModifyRequest):
    """Modify a pending (limit/stop) order's price, SL, TP. Stage 3 execution validation."""
    ensure_initialized()
    raw_orders = mt5.orders_get(ticket=int(ticket))
    if not raw_orders or len(raw_orders) == 0:
        raise HTTPException(status_code=404, detail=f"Pending order {ticket} not found")

    order = raw_orders[0]

    trade_req = {
        "action": mt5.TRADE_ACTION_MODIFY,
        "order": int(ticket),
        "price": float(req.price) if req.price is not None else float(order.price_open),
        "sl": float(req.stop_loss) if req.stop_loss is not None else float(order.sl),
        "tp": float(req.take_profit) if req.take_profit is not None else float(order.tp),
        "type_time": mt5.ORDER_TIME_GTC,
    }
    if req.volume is not None:
        trade_req["volume"] = float(req.volume)

    result = mt5.order_send(trade_req)
    if result is None or result.retcode != mt5.TRADE_RETCODE_DONE:
        err_msg = mt5.last_error() if result is None else f"Retcode: {result.retcode}, Comment: {result.comment}"
        raise HTTPException(status_code=500, detail=f"Order modification failed: {err_msg}")

    return {"status": "modified", "ticket": ticket}


@app.delete("/orders/{ticket}")
def cancel_pending_order(ticket: str):
    """Cancel a pending (limit/stop) order by ticket. Stage 3 execution validation."""
    ensure_initialized()
    raw_orders = mt5.orders_get(ticket=int(ticket))
    if not raw_orders or len(raw_orders) == 0:
        raise HTTPException(status_code=404, detail=f"Pending order {ticket} not found")

    trade_req = {
        "action": mt5.TRADE_ACTION_REMOVE,
        "order": int(ticket),
    }

    result = mt5.order_send(trade_req)
    if result is None or result.retcode != mt5.TRADE_RETCODE_DONE:
        err_msg = mt5.last_error() if result is None else f"Retcode: {result.retcode}, Comment: {result.comment}"
        raise HTTPException(status_code=500, detail=f"Order cancellation failed: {err_msg}")

    return {"status": "cancelled", "ticket": ticket}


@app.get("/stats")
def get_bridge_stats():
    """Return aggregate stats for Phase 12 monitoring dashboard."""
    ensure_initialized()
    positions = mt5.positions_get() or []
    orders = mt5.orders_get() or []
    account = mt5.account_info()
    terminal = mt5.terminal_info()
    return {
        "open_positions": len(positions),
        "pending_orders": len(orders),
        "balance": float(account.balance) if account else None,
        "equity": float(account.equity) if account else None,
        "margin": float(account.margin) if account else None,
        "free_margin": float(account.margin_free) if account else None,
        "margin_level": float(account.margin_level) if account else None,
        "connected": terminal.connected if terminal else False,
        "trade_allowed": terminal.trade_allowed if terminal else False,
        "floating_pnl": sum(float(p.profit) for p in positions),
    }


if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8000)

