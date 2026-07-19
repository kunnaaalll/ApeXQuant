use market_data_engine_rs::aggregation::MultiTimeframeAggregator;
use market_data_engine_rs::binance_stream::BinanceTickStream;
use market_data_engine_rs::candle::Timeframe;
use market_data_engine_rs::event_bus::EventBusPublisher;
use market_data_engine_rs::mt5_stream::{Mt5Config, Mt5TickStream};
use market_data_engine_rs::process::TickProcessor;
use market_data_engine_rs::storage::{CandleRecord as DbCandleRecord, MarketDataStore, TickRecord};
use market_data_engine_rs::streaming::TickStream;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::sync::Arc;
use tokio::time::Duration;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting APEX V3 Market Data Engine...");

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let event_bus_url =
        env::var("EVENT_BUS_URL").unwrap_or_else(|_| "http://localhost:50050".to_string());
    let allowed_symbols = env::var("ALLOWED_SYMBOLS").unwrap_or_else(|_| "BTCUSDT".to_string());
    let symbols: Vec<String> = allowed_symbols
        .split(',')
        .map(|s| s.trim().to_uppercase())
        .collect();

    info!("Connecting to PostgreSQL...");
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&db_url)
        .await?;

    info!("Initializing Event Bus Publisher...");
    let event_bus_publisher = EventBusPublisher::connect(event_bus_url).await?;
    let event_bus = Arc::new(event_bus_publisher);

    info!("Initializing Market Data Store...");
    let store = Arc::new(MarketDataStore::new(pool, Some(event_bus.clone())));

    for symbol in symbols {
        let store = store.clone();

        tokio::spawn(async move {
            info!("Starting pipeline for symbol: {}", symbol);

            // For cryptos like BTCUSDT, use Binance. For forex like EURUSD, use MT5.
            let mut stream: Box<dyn TickStream> =
                if symbol.contains("BTC") || symbol.contains("ETH") || symbol.contains("USDT") {
                    Box::new(BinanceTickStream::new_spot(symbol.clone()))
                } else {
                    let config = Mt5Config {
                        endpoint: env::var("MT5_BRIDGE_ENDPOINT")
                            .unwrap_or_else(|_| "127.0.0.1:5555".to_string()),
                        ..Default::default()
                    };
                    Box::new(Mt5TickStream::new(symbol.clone(), config))
                };

            if let Err(e) = stream.connect().await {
                error!("Failed to connect stream for {}: {}", symbol, e);
                return;
            }

            let processor = TickProcessor::new();
            let mut aggregator = MultiTimeframeAggregator::new(vec![
                Timeframe::M1,
                Timeframe::M5,
                Timeframe::M15,
                Timeframe::H1,
                Timeframe::H4,
                Timeframe::D1,
            ]);
            let mut gap_detector = market_data_engine_rs::gaps::GapDetector::new();
            let mut expected_sequence = 1;

            while let Some(raw_tick) = stream.next_tick().await {
                // Gap Detection
                let is_duplicate = raw_tick.sequence < expected_sequence;
                let missing = raw_tick.sequence.saturating_sub(expected_sequence);

                if let Some(gap_event) =
                    gap_detector.process_tick(raw_tick.timestamp, missing, is_duplicate)
                {
                    tracing::warn!("Gap detected for {}: {:?}", symbol, gap_event);
                }
                if !is_duplicate {
                    expected_sequence = raw_tick.sequence + 1;
                }

                // Validation & Normalization
                match processor.process(raw_tick) {
                    Ok(tick) => {
                        // Persist Tick
                        let tick_record = TickRecord {
                            symbol: tick.symbol.clone(),
                            sequence: tick.sequence as i64,
                            bid: tick.bid,
                            ask: tick.ask,
                            spread: tick.spread,
                            timestamp: tick.timestamp,
                        };

                        if let Err(e) = store.ticks.save_tick(&tick_record).await {
                            error!("Failed to save tick for {}: {}", symbol, e);
                        }

                        // Aggregation
                        let closed_candles = aggregator.process_tick(&tick);
                        for (tf, candle) in closed_candles {
                            let candle_record = DbCandleRecord {
                                symbol: symbol.clone(),
                                timeframe: format!("{:?}", tf),
                                open: candle.open,
                                high: candle.high,
                                low: candle.low,
                                close: candle.close,
                                volume: candle.volume,
                                start_time: candle.start_time,
                                end_time: candle.end_time,
                            };

                            if let Err(e) = store.candles.save_candle(&candle_record).await {
                                error!("Failed to save candle for {}: {}", symbol, e);
                            }
                        }
                    }
                    Err(e) => {
                        // Drop invalid ticks
                        tracing::debug!("Dropped invalid tick for {}: {:?}", symbol, e);
                    }
                }
            }

            error!("Stream ended unexpectedly for symbol: {}", symbol);
        });
    }

    // Keep main thread alive
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
