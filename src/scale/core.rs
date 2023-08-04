use super::reading::Reading;

#[derive(Debug, Clone)]
pub struct ScaleCoreState {
    last: Reading,
}

impl ScaleCoreState {
    pub fn new(default_unit: &str) -> Self {
        Self {
            last: Reading::from_source("core", "", 0, &normalize_unit(default_unit)),
        }
    }

    pub fn apply_reading(&mut self, mut reading: Reading) -> &Reading {
        if reading.unit.trim().is_empty() {
            reading.unit = self.last.unit.clone();
        }
        self.last = reading;
        &self.last
    }

    pub fn last(&self) -> &Reading {
        &self.last
    }
}

fn normalize_unit(unit: &str) -> String {
    let unit = unit.trim().to_ascii_lowercase();
    if unit.is_empty() {
        "kg".to_string()
    } else {
        unit
    }
}

#[cfg(test)]
mod tests {
    use super::ScaleCoreState;
    use crate::scale::Reading;

    #[test]
    fn core_keeps_last_unit_when_driver_omits_unit_like_go() {
        let mut core = ScaleCoreState::new("kg");
        core.apply_reading(Reading::from_source("wifi", "scale.local", 0, "g"));

        let applied = core.apply_reading(Reading::from_source("wifi", "scale.local", 0, ""));

        assert_eq!(applied.unit, "g");
    }

    #[test]
    fn core_defaults_unit_to_kg() {
        let core = ScaleCoreState::new("");

        assert_eq!(core.last().unit, "kg");
    }
}
