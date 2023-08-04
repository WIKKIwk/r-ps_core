use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq)]
pub struct Reading {
    pub source: String,
    pub port: String,
    pub baud: u32,
    pub weight: Option<f64>,
    pub unit: String,
    pub stable: Option<bool>,
    pub raw: String,
    pub error: String,
    pub updated_at: SystemTime,
}

impl Reading {
    pub fn from_source(source: &str, port: &str, baud: u32, unit: &str) -> Self {
        Self {
            source: source.trim().to_string(),
            port: port.trim().to_string(),
            baud,
            weight: None,
            unit: unit.to_string(),
            stable: None,
            raw: String::new(),
            error: String::new(),
            updated_at: SystemTime::now(),
        }
    }

    pub fn serial(port: &str, baud: u32, unit: &str) -> Self {
        Self::from_source("serial", port, baud, unit)
    }

    pub fn with_weight(mut self, weight: f64, stable: Option<bool>, raw: &str) -> Self {
        self.weight = Some(weight);
        self.stable = stable;
        self.raw = raw.to_string();
        self.updated_at = SystemTime::now();
        self
    }

    pub fn with_raw(mut self, raw: &str) -> Self {
        self.raw = raw.to_string();
        self.updated_at = SystemTime::now();
        self
    }

    pub fn with_error(mut self, error: &str) -> Self {
        self.error = error.to_string();
        self.updated_at = SystemTime::now();
        self
    }
}
