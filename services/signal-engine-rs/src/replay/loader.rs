//! Dataset Loader - Load and manage golden datasets
//!
//! Handles loading from various formats including JSON, CSV, and custom formats.

use super::*;
use std::fs;
use std::path::Path;

/// Loader for historical datasets
pub struct DatasetLoader;

impl DatasetLoader {
    /// Create a new dataset loader
    pub fn new() -> Self {
        Self
    }

    /// Load a dataset from a directory
    pub fn load_from_directory(&self, path: &Path) -> Result<GoldenDataset> {
        if !path.exists() {
            return Err(ReplayError::DatasetNotFound(path.to_string_lossy().to_string()));
        }

        // Look for metadata file
        let metadata_path = path.join("metadata.json");
        let metadata = if metadata_path.exists() {
            self.load_metadata(&metadata_path)?
        } else {
            DatasetMetadata {
                name: path.file_name().map_or_else(
                    || "unknown".to_string(),
                    |n| n.to_string_lossy().to_string(),
                ),
                version: "1.0.0".to_string(),
                created_at: Utc::now(),
                total_scenarios: 0,
                categories: Vec::new(),
                symbols: Vec::new(),
                timeframes: Vec::new(),
                total_candles: 0,
                total_expected_signals: 0,
            }
        };

        let mut dataset = GoldenDataset {
            metadata,
            scenarios: Vec::new(),
            by_category: HashMap::new(),
            by_symbol: HashMap::new(),
        };

        // Load scenarios from scenarios directory
        let scenarios_path = path.join("scenarios");
        if scenarios_path.exists() && scenarios_path.is_dir() {
            for entry in fs::read_dir(&scenarios_path)? {
                let entry = entry?;
                let path = entry.path();

                if path.extension().map_or(false, |e| e == "json") {
                    match self.load_scenario(&path) {
                        Ok(scenario) => dataset.add_scenario(scenario),
                        Err(e) => eprintln!("Failed to load scenario from {:?}: {}", path, e),
                    }
                }
            }
        }

        // Load from CSV files if available
        let csv_path = path.join("candles.csv");
        if csv_path.exists() {
            self.load_candles_csv(&csv_path, &mut dataset)?;
        }

        Ok(dataset)
    }

    /// Load metadata from JSON
    fn load_metadata(&self, path: &Path) -> Result<DatasetMetadata> {
        let content = fs::read_to_string(path)?;
        let metadata: DatasetMetadata =
            serde_json::from_str(&content).map_err(|e| ReplayError::InvalidFormat(e.to_string()))?;
        Ok(metadata)
    }

    /// Load a single scenario from JSON
    fn load_scenario(&self, path: &Path) -> Result<TestScenario> {
        let content = fs::read_to_string(path)?;
        let scenario: TestScenario =
            serde_json::from_str(&content).map_err(|e| ReplayError::InvalidFormat(e.to_string()))?;
        Ok(scenario)
    }

    /// Load candles from CSV format
    fn load_candles_csv(&self, path: &Path, dataset: &mut GoldenDataset) -> Result<()> {
        let mut reader = csv::Reader::from_path(path)?;
        let mut current_scenario: Option<TestScenario> = None;
        let mut candles: Vec<Candle> = Vec::new();

        for result in reader.deserialize() {
            let record: CandleCsvRecord = result?;

            // Group by (symbol, timeframe, run_id)
            let run_id = format!("{}_{}", record.symbol, record.timeframe);

            if let Some(ref mut scenario) = current_scenario {
                let current_id = format!("{}_{}", scenario.symbol, scenario.timeframe);
                if current_id != run_id {
                    // Save previous scenario
                    let mut completed = scenario.clone();
                    completed.candles = candles.clone();
                    dataset.add_scenario(completed);

                    // Start new scenario
                    candles.clear();
                    current_scenario = Some(self.create_scenario_from_record(&record)?);
                }
            } else {
                current_scenario = Some(self.create_scenario_from_record(&record)?);
            }

            candles.push(record.to_candle()?);
        }

        // Don't forget the last scenario
        if let Some(mut scenario) = current_scenario {
            scenario.candles = candles;
            dataset.add_scenario(scenario);
        }

        Ok(())
    }

    fn create_scenario_from_record(&self, record: &CandleCsvRecord) -> Result<TestScenario> {
        Ok(TestScenario {
            scenario_id: format!("{}_{}_{}", record.symbol, record.timeframe, record.timestamp),
            name: format!("{} {} Scenario", record.symbol, record.timeframe),
            description: "Loaded from CSV".to_string(),
            category: ScenarioCategory::Complex, // Default, would need analysis to determine
            symbol: record.symbol.clone(),
            timeframe: record.timeframe.clone(),
            start_time: record.timestamp,
            end_time: record.timestamp, // Will be updated
            candles: Vec::new(),
            expected_signals: Vec::new(),
            metadata: HashMap::new(),
        })
    }

    /// Save a dataset to directory
    pub fn save_to_directory(&self, dataset: &GoldenDataset, path: &Path) -> Result<()> {
        fs::create_dir_all(path)?;

        // Save metadata
        let metadata_path = path.join("metadata.json");
        let metadata_json = serde_json::to_string_pretty(&dataset.metadata)?;
        fs::write(&metadata_path, metadata_json)?;

        // Save scenarios
        let scenarios_dir = path.join("scenarios");
        fs::create_dir_all(&scenarios_dir)?;

        for (idx, scenario) in dataset.scenarios.iter().enumerate() {
            let filename = format!("{:04}_{}.json", idx, scenario.scenario_id);
            let scenario_path = scenarios_dir.join(&filename);
            let scenario_json = serde_json::to_string_pretty(scenario)?;
            fs::write(&scenario_path, scenario_json)?;
        }

        Ok(())
    }

    /// Generate a synthetic dataset for testing
    pub fn generate_synthetic_dataset(
        &self,
        name: &str,
        num_scenarios: usize,
        candles_per_scenario: usize,
    ) -> GoldenDataset {
        let mut dataset = GoldenDataset::new(name.to_string());
        let symbols = vec!["EURUSD", "GBPUSD", "USDJPY", "USDCHF"];
        let timeframes = vec!["M15", "H1", "H4"];
        let categories = vec![
            ScenarioCategory::TrendingUp,
            ScenarioCategory::TrendingDown,
            ScenarioCategory::Ranging,
            ScenarioCategory::HighVolatility,
            ScenarioCategory::Breakout,
        ];

        for i in 0..num_scenarios {
            let symbol = symbols[i % symbols.len()].to_string();
            let timeframe = timeframes[i % timeframes.len()].to_string();
            let category = categories[i % categories.len()];

            // Generate random walk candles
            let candles = self.generate_random_walk_candles(candles_per_scenario);

            let start_time = Utc::now() - chrono::Duration::hours(candles_per_scenario as i64);
            let end_time = Utc::now();

            let scenario = TestScenario {
                scenario_id: format!("synthetic_{:04}", i),
                name: format!("Synthetic {} Scenario", category_name(&category)),
                description: format!(
                    "Synthetic {} {} scenario with {} candles",
                    symbol, timeframe, candles_per_scenario
                ),
                category,
                symbol,
                timeframe,
                start_time,
                end_time,
                candles,
                expected_signals: Vec::new(),
                metadata: HashMap::new(),
            };

            dataset.add_scenario(scenario);
        }

        dataset
    }

    fn generate_random_walk_candles(&self, count: usize) -> Vec<Candle> {
        use crate::market_data::Candle;

        let mut candles = Vec::with_capacity(count);
        let mut price = 1.0850;
        let volatility = 0.001;

        for i in 0..count {
            let open = price;
            let change = (rand::random::<f64>() - 0.5) * volatility;
            let close = open + change;
            let high = open.max(close) + rand::random::<f64>() * volatility * 0.5;
            let low = open.min(close) - rand::random::<f64>() * volatility * 0.5;

            let candle = Candle {
                timestamp: Utc::now() + chrono::Duration::minutes(i as i64 * 15),
                open,
                high,
                low,
                close,
                volume: (rand::random::<f64>() * 1000.0 + 100.0) as u64,
            };

            candles.push(candle);
            price = close;
        }

        candles
    }
}

impl Default for DatasetLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// CSV record structure for candles
#[derive(Debug, Deserialize)]
struct CandleCsvRecord {
    timestamp: DateTime<Utc>,
    symbol: String,
    timeframe: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: u64,
    #[serde(default)]
    direction: Option<String>,
    #[serde(default)]
    expected_signal: Option<String>,
}

impl CandleCsvRecord {
    fn to_candle(&self) -> Result<Candle> {
        Ok(Candle {
            timestamp: self.timestamp,
            open: self.open,
            high: self.high,
            low: self.low,
            close: self.close,
            volume: self.volume,
        })
    }
}

fn category_name(category: &ScenarioCategory) -> &'static str {
    match category {
        ScenarioCategory::TrendingUp => "Uptrend",
        ScenarioCategory::TrendingDown => "Downtrend",
        ScenarioCategory::Ranging => "Ranging",
        ScenarioCategory::HighVolatility => "HighVol",
        ScenarioCategory::LowVolatility => "LowVol",
        ScenarioCategory::Breakout => "Breakout",
        ScenarioCategory::LiquiditySweep => "LiqSweep",
        ScenarioCategory::StrongBos => "StrongBOS",
        ScenarioCategory::StrongChoch => "StrongCHOCH",
        ScenarioCategory::GoodOrderBlock => "OrderBlock",
        ScenarioCategory::GoodFvg => "FVG",
        ScenarioCategory::Complex => "Complex",
    }
}

/// Historical dataset handle
pub struct HistoricalDataset {
    pub path: PathBuf,
    pub metadata: DatasetMetadata,
}

impl HistoricalDataset {
    /// Load dataset from path
    pub fn load(path: PathBuf) -> Result<Self> {
        let loader = DatasetLoader::new();
        let golden = loader.load_from_directory(&path)?;

        Ok(Self {
            path,
            metadata: golden.metadata,
        })
    }

    /// Get dataset name
    pub fn name(&self) -> &str {
        &self.metadata.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_synthetic_dataset() {
        let loader = DatasetLoader::new();
        let dataset = loader.generate_synthetic_dataset("test", 10, 100);

        assert_eq!(dataset.metadata.total_scenarios, 10);
        assert!(dataset.scenarios.len() > 0);
        assert_eq!(dataset.scenarios[0].candles.len(), 100);
    }
}
