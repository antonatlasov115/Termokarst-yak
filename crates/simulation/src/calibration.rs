//! Калибровка модели по реальным данным наблюдений

use crate::{SimulationConfig, SimulationEngine};
use thermokarst_core::{dataset::ObservationDataset, EnvironmentParams, Result, SoilType};

/// Калибратор параметров модели
pub struct ModelCalibrator {
    dataset: ObservationDataset,
}

impl ModelCalibrator {
    pub fn new(dataset: ObservationDataset) -> Self {
        Self { dataset }
    }

    /// Оценить параметры среды на основе наблюдений
    pub fn estimate_environment_params(&self) -> Result<EnvironmentParams> {
        let stats = self.dataset.statistics();

        // Средняя глубина активного слоя
        let avg_alt = stats
            .active_layer_stats
            .as_ref()
            .map(|s| s.mean)
            .unwrap_or(1.5);

        // Средняя температура грунта
        let avg_temp = stats
            .temperature_stats
            .as_ref()
            .map(|s| s.mean)
            .unwrap_or(-2.0);

        // Оценка температуры воздуха (эмпирическая связь)
        let air_temp = estimate_air_temp_from_ground(avg_temp, avg_alt);

        // Оценка льдистости по просадке
        let ice_content = stats
            .subsidence_stats
            .as_ref()
            .map(|s| estimate_ice_content(s.mean, avg_alt))
            .unwrap_or(0.6);

        // Определение типа грунта по метаданным
        let soil_type = self.estimate_soil_type();

        Ok(EnvironmentParams {
            air_temp,
            permafrost_temp: avg_temp,
            ice_content,
            soil_type,
            vegetation_cover: 0.4, // По умолчанию
            water_availability: 0.6,
            permafrost_depth: avg_alt,
            warm_season_days: 120,
        })
    }

    /// Определить тип грунта из метаданных
    fn estimate_soil_type(&self) -> SoilType {
        let mut soil_counts = std::collections::HashMap::new();

        for obs in &self.dataset.observations {
            if let Some(soil) = obs.metadata.get("soil_type") {
                *soil_counts.entry(soil.clone()).or_insert(0) += 1;
            }
        }

        // Наиболее частый тип
        let most_common = soil_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(soil, _)| soil.as_str());

        match most_common {
            Some("clay") => SoilType::Clay,
            Some("sand") => SoilType::Sand,
            Some("peat") => SoilType::Peat,
            Some("silt") => SoilType::Silt,
            _ => SoilType::Loam,
        }
    }

    /// Сравнить модель с наблюдениями
    pub fn validate_model(&self, params: &EnvironmentParams, years: u32) -> ValidationResult {
        let config = SimulationConfig {
            years,
            time_step: 1,
            save_intermediate: false,
            save_interval: 1,
        };

        let engine = SimulationEngine::new(params.clone(), config);
        let sim_result = match engine.run() {
            Ok(r) => r,
            Err(_) => return ValidationResult::default(),
        };

        let final_lens = match sim_result.lenses.last() {
            Some(l) => l,
            None => return ValidationResult::default(),
        };

        // Сравнение с наблюдениями
        let stats = self.dataset.statistics();

        let mut errors = Vec::new();

        // Ошибка по глубине
        if let Some(alt_stats) = &stats.active_layer_stats {
            let depth_error = (final_lens.depth - alt_stats.mean).abs() / alt_stats.mean;
            errors.push(depth_error);
        }

        // Ошибка по просадке
        if let Some(sub_stats) = &stats.subsidence_stats {
            let subsidence_error = (final_lens.depth - sub_stats.mean).abs() / sub_stats.mean;
            errors.push(subsidence_error);
        }

        let mean_error = if !errors.is_empty() {
            errors.iter().sum::<f64>() / errors.len() as f64
        } else {
            0.0
        };

        ValidationResult {
            mean_relative_error: mean_error,
            observations_count: self.dataset.observations.len(),
            model_depth: final_lens.depth,
            model_diameter: final_lens.diameter,
            observed_depth_mean: stats.active_layer_stats.as_ref().map(|s| s.mean),
            observed_subsidence_mean: stats.subsidence_stats.as_ref().map(|s| s.mean),
        }
    }
}

/// Результат валидации модели
#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    pub mean_relative_error: f64,
    pub observations_count: usize,
    pub model_depth: f64,
    pub model_diameter: f64,
    pub observed_depth_mean: Option<f64>,
    pub observed_subsidence_mean: Option<f64>,
}

impl ValidationResult {
    pub fn is_good_fit(&self) -> bool {
        self.mean_relative_error < 0.3 // Ошибка < 30%
    }
}

/// Оценка температуры воздуха по температуре грунта
fn estimate_air_temp_from_ground(ground_temp: f64, active_layer: f64) -> f64 {
    // Эмпирическая связь для Якутии
    // T_air ≈ T_ground + 5-7°C (зависит от глубины активного слоя)
    let offset = 5.0 + 2.0 * (active_layer / 2.0).min(1.0);
    ground_temp + offset
}

/// Оценка льдистости по просадке
fn estimate_ice_content(subsidence: f64, active_layer: f64) -> f64 {
    if active_layer <= 0.0 {
        return 0.5;
    }

    // subsidence ≈ active_layer * ice_content * 0.9
    let estimated = subsidence / (active_layer * 0.9);
    estimated.max(0.1).min(0.95)
}

#[cfg(test)]
mod tests {
    use super::*;
    use thermokarst_core::dataset::create_example_yakutia_dataset;

    #[test]
    fn test_estimate_params() {
        let dataset = create_example_yakutia_dataset();
        let calibrator = ModelCalibrator::new(dataset);

        let params = calibrator.estimate_environment_params().unwrap();

        assert!(params.air_temp > -10.0 && params.air_temp < 20.0);
        assert!(params.ice_content > 0.0 && params.ice_content < 1.0);
    }

    #[test]
    fn test_validate_model() {
        let dataset = create_example_yakutia_dataset();
        let calibrator = ModelCalibrator::new(dataset);

        let params = calibrator.estimate_environment_params().unwrap();
        let result = calibrator.validate_model(&params, 10);

        assert!(result.observations_count > 0);
        assert!(result.model_depth > 0.0);
    }
}
