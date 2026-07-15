// Configuration for healthpulse analyzer

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub max_complexity: u32,
    pub max_function_length: u32,
    pub max_file_length: u32,
    pub min_test_ratio: f64,
    pub ignore: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_complexity: 10,
            max_function_length: 50,
            max_file_length: 500,
            min_test_ratio: 0.3,
            ignore: vec![],
        }
    }
}

impl Config {
    pub fn load(&mut self, path: &str) -> Result<(), String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {e}"))?;

        let cfg: Config = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {e}"))?;

        self.max_complexity = cfg.max_complexity;
        self.max_function_length = cfg.max_function_length;
        self.max_file_length = cfg.max_file_length;
        self.min_test_ratio = cfg.min_test_ratio;
        self.ignore.extend(cfg.ignore);
        Ok(())
    }

    pub fn add_ignore(&mut self, pattern: &str) {
        self.ignore.push(pattern.to_string());
    }

    pub fn should_ignore(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        for pattern in &self.ignore {
            if path_str.contains(pattern) {
                return true;
            }
        }
        false
    }
}