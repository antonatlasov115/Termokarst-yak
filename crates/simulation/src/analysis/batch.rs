//! Параллельная симуляция множества сценариев

use rayon::prelude::*;
use thermokarst_core::{EnvironmentParams, Result, SimulationResult};

use crate::engine::{SimulationConfig, SimulationEngine};

/// Сценарий симуляции
#[derive(Debug, Clone)]
pub struct Scenario {
    pub name: String,
    pub params: EnvironmentParams,
}

impl Scenario {
    pub fn new(name: impl Into<String>, params: EnvironmentParams) -> Self {
        Self {
            name: name.into(),
            params,
        }
    }
}

/// Результат батч-симуляции
#[derive(Debug)]
pub struct BatchResult {
    pub scenario_name: String,
    pub result: Result<SimulationResult>,
}

/// Батч-симулятор для параллельного запуска
pub struct BatchSimulator {
    scenarios: Vec<Scenario>,
    config: SimulationConfig,
}

impl BatchSimulator {
    pub fn new(config: SimulationConfig) -> Self {
        Self {
            scenarios: Vec::new(),
            config,
        }
    }

    /// Добавить сценарий
    pub fn add_scenario(&mut self, scenario: Scenario) {
        self.scenarios.push(scenario);
    }

    /// Добавить стандартные сценарии для Якутии
    pub fn add_yakutia_scenarios(&mut self) {
        self.add_scenario(Scenario::new(
            "Северная Якутия",
            EnvironmentParams::northern_yakutia(),
        ));

        self.add_scenario(Scenario::new(
            "Центральная Якутия",
            EnvironmentParams::central_yakutia(),
        ));

        self.add_scenario(Scenario::new(
            "Южная Якутия",
            EnvironmentParams::southern_yakutia(),
        ));
    }

    /// Запустить все сценарии параллельно
    pub fn run_parallel(&self) -> Vec<BatchResult> {
        self.scenarios
            .par_iter()
            .map(|scenario| {
                let engine = SimulationEngine::new(scenario.params.clone(), self.config.clone());
                let result = engine.run();

                BatchResult {
                    scenario_name: scenario.name.clone(),
                    result,
                }
            })
            .collect()
    }

    /// Запустить все сценарии последовательно
    pub fn run_sequential(&self) -> Vec<BatchResult> {
        self.scenarios
            .iter()
            .map(|scenario| {
                let engine = SimulationEngine::new(scenario.params.clone(), self.config.clone());
                let result = engine.run();

                BatchResult {
                    scenario_name: scenario.name.clone(),
                    result,
                }
            })
            .collect()
    }

    /// Количество сценариев
    pub fn scenario_count(&self) -> usize {
        self.scenarios.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_simulation() {
        let config = SimulationConfig {
            years: 10,
            ..Default::default()
        };

        let mut batch = BatchSimulator::new(config);
        batch.add_yakutia_scenarios();

        assert_eq!(batch.scenario_count(), 3);

        let results = batch.run_parallel();
        assert_eq!(results.len(), 3);

        for result in results {
            assert!(result.result.is_ok());
        }
    }
}
