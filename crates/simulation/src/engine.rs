//! Основной движок симуляции термокарстовых процессов

use thermokarst_core::{
    EnvironmentParams, Result, SimulationResult, ThermokarstError, ThermokarstLens,
};
use thermokarst_geology::{LateralExpansionCalculator, StabilityAnalyzer};
use thermokarst_physics::{SubsidenceCalculator, ThawDepthCalculator};

/// Конфигурация симуляции
#[derive(Debug, Clone)]
pub struct SimulationConfig {
    /// Количество лет для симуляции
    pub years: u32,

    /// Шаг симуляции (годы)
    pub time_step: u32,

    /// Сохранять ли промежуточные результаты
    pub save_intermediate: bool,

    /// Интервал сохранения (каждые N лет)
    pub save_interval: u32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            years: 50,
            time_step: 1,
            save_intermediate: true,
            save_interval: 1,
        }
    }
}

/// Движок симуляции
pub struct SimulationEngine {
    params: EnvironmentParams,
    config: SimulationConfig,
    thaw_calc: ThawDepthCalculator,
    subsidence_calc: SubsidenceCalculator,
    expansion_calc: LateralExpansionCalculator,
}

impl SimulationEngine {
    /// Создать новый движок симуляции
    pub fn new(params: EnvironmentParams, config: SimulationConfig) -> Self {
        let thaw_calc = ThawDepthCalculator::new(params.clone());
        let subsidence_calc = SubsidenceCalculator::new(params.clone());
        let expansion_calc = LateralExpansionCalculator::new(params.clone());

        Self {
            params,
            config,
            thaw_calc,
            subsidence_calc,
            expansion_calc,
        }
    }

    /// Запустить симуляцию
    pub fn run(&self) -> Result<SimulationResult> {
        let mut lenses = Vec::new();
        let mut previous_volume = 0.0;

        for year in 1..=self.config.years {
            // Расчет глубины протаивания
            let thaw_depth = self.thaw_calc.calculate(year)?;

            // Расчет просадки
            let depth = self.subsidence_calc.calculate_subsidence(thaw_depth);

            // Расчет диаметра
            let diameter = self.expansion_calc.calculate_diameter(depth, year);

            // Создание линзы
            let mut lens = ThermokarstLens::new(depth, diameter, year);

            // Расчет скорости роста
            lens.growth_rate = lens.volume - previous_volume;
            previous_volume = lens.volume;

            // Сохранение результата
            if self.config.save_intermediate && year % self.config.save_interval == 0 {
                lenses.push(lens);
            } else if year == self.config.years {
                lenses.push(lens);
            }
        }

        // Определение финальной стадии
        let final_lens = lenses.last().ok_or_else(|| {
            ThermokarstError::SimulationError("Нет результатов симуляции".to_string())
        })?;

        let stage = StabilityAnalyzer::determine_stage(final_lens);

        Ok(SimulationResult {
            lenses,
            environment: self.params.clone(),
            stage,
            total_years: self.config.years,
        })
    }

    /// Запустить симуляцию с детальным выводом
    pub fn run_with_details(&self) -> Result<DetailedSimulationResult> {
        let mut lenses = Vec::new();
        let mut thaw_depths = Vec::new();
        let mut subsidence_values = Vec::new();
        let mut stability_scores = Vec::new();

        let mut previous_volume = 0.0;

        for year in 1..=self.config.years {
            let thaw_depth = self.thaw_calc.calculate(year)?;
            let depth = self.subsidence_calc.calculate_subsidence(thaw_depth);
            let diameter = self.expansion_calc.calculate_diameter(depth, year);

            let mut lens = ThermokarstLens::new(depth, diameter, year);
            lens.growth_rate = lens.volume - previous_volume;
            previous_volume = lens.volume;

            let stability = StabilityAnalyzer::long_term_stability_score(&lens);

            if year % self.config.save_interval == 0 {
                lenses.push(lens);
                thaw_depths.push(thaw_depth);
                subsidence_values.push(depth);
                stability_scores.push(stability);
            }
        }

        let final_lens = lenses.last().ok_or_else(|| {
            ThermokarstError::SimulationError("Нет результатов".to_string())
        })?;

        let stage = StabilityAnalyzer::determine_stage(final_lens);

        Ok(DetailedSimulationResult {
            basic: SimulationResult {
                lenses,
                environment: self.params.clone(),
                stage,
                total_years: self.config.years,
            },
            thaw_depths,
            subsidence_values,
            stability_scores,
        })
    }
}

/// Детальный результат симуляции
#[derive(Debug, Clone)]
pub struct DetailedSimulationResult {
    pub basic: SimulationResult,
    pub thaw_depths: Vec<f64>,
    pub subsidence_values: Vec<f64>,
    pub stability_scores: Vec<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_runs() {
        let params = EnvironmentParams::default();
        let config = SimulationConfig {
            years: 10,
            time_step: 1,
            save_intermediate: true,
            save_interval: 2,
        };

        let engine = SimulationEngine::new(params, config);
        let result = engine.run();

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.lenses.is_empty());
    }

    #[test]
    fn test_lens_grows_over_time() {
        let params = EnvironmentParams::default();
        let config = SimulationConfig {
            years: 20,
            ..Default::default()
        };

        let engine = SimulationEngine::new(params, config);
        let result = engine.run().unwrap();

        let first = &result.lenses[0];
        let last = result.lenses.last().unwrap();

        assert!(last.volume > first.volume);
        assert!(last.diameter > first.diameter);
    }
}
