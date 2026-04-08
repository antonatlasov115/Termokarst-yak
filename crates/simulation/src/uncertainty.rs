//! Модуль оценки неопределенности прогнозов
//!
//! Предоставляет инструменты для количественной оценки неопределенности
//! в прогнозах развития термокарста с использованием методов Монте-Карло
//! и анализа чувствительности.

use rayon::prelude::*;
use thermokarst_core::{EnvironmentParams, Result, ThermokarstError};

/// Параметры неопределенности для входных данных
#[derive(Debug, Clone)]
pub struct UncertaintyParams {
    /// Относительная неопределенность температуры воздуха (±%)
    pub air_temp_uncertainty: f64,
    /// Относительная неопределенность льдистости (±%)
    pub ice_content_uncertainty: f64,
    /// Относительная неопределенность растительного покрова (±%)
    pub vegetation_uncertainty: f64,
    /// Относительная неопределенность теплопроводности (±%)
    pub thermal_conductivity_uncertainty: f64,
    /// Количество симуляций Монте-Карло
    pub n_simulations: usize,
}

impl Default for UncertaintyParams {
    fn default() -> Self {
        Self {
            air_temp_uncertainty: 10.0,             // ±10%
            ice_content_uncertainty: 15.0,          // ±15%
            vegetation_uncertainty: 20.0,           // ±20%
            thermal_conductivity_uncertainty: 10.0, // ±10%
            n_simulations: 1000,
        }
    }
}

/// Результат анализа неопределенности
#[derive(Debug, Clone)]
pub struct UncertaintyResult {
    /// Среднее значение
    pub mean: f64,
    /// Медиана
    pub median: f64,
    /// Стандартное отклонение
    pub std_dev: f64,
    /// 5-й процентиль (нижняя граница 90% доверительного интервала)
    pub percentile_5: f64,
    /// 95-й процентиль (верхняя граница 90% доверительного интервала)
    pub percentile_95: f64,
    /// Коэффициент вариации (CV = std_dev / mean)
    pub coefficient_of_variation: f64,
    /// Все результаты симуляций
    pub samples: Vec<f64>,
}

impl UncertaintyResult {
    /// Создать результат из набора симуляций
    pub fn from_samples(mut samples: Vec<f64>) -> Self {
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let n = samples.len() as f64;
        let mean = samples.iter().sum::<f64>() / n;
        let median = samples[samples.len() / 2];

        let variance = samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
        let std_dev = variance.sqrt();

        let idx_5 = (n * 0.05) as usize;
        let idx_95 = (n * 0.95) as usize;
        let percentile_5 = samples[idx_5];
        let percentile_95 = samples[idx_95];

        let coefficient_of_variation = if mean.abs() > 1e-10 {
            std_dev / mean.abs()
        } else {
            0.0
        };

        Self {
            mean,
            median,
            std_dev,
            percentile_5,
            percentile_95,
            coefficient_of_variation,
            samples,
        }
    }

    /// Ширина доверительного интервала (90%)
    pub fn confidence_interval_width(&self) -> f64 {
        self.percentile_95 - self.percentile_5
    }

    /// Относительная ширина доверительного интервала (%)
    pub fn relative_uncertainty(&self) -> f64 {
        if self.mean.abs() < 1e-10 {
            return 0.0;
        }
        (self.confidence_interval_width() / self.mean.abs()) * 100.0
    }

    /// Вывести отчет
    pub fn report(&self) -> String {
        format!(
            "Uncertainty Analysis:\n\
             Mean: {:.3}\n\
             Median: {:.3}\n\
             Std Dev: {:.3}\n\
             90% CI: [{:.3}, {:.3}]\n\
             CV: {:.2}%\n\
             Relative Uncertainty: {:.1}%",
            self.mean,
            self.median,
            self.std_dev,
            self.percentile_5,
            self.percentile_95,
            self.coefficient_of_variation * 100.0,
            self.relative_uncertainty()
        )
    }
}

/// Анализатор неопределенности
pub struct UncertaintyAnalyzer {
    uncertainty_params: UncertaintyParams,
}

impl UncertaintyAnalyzer {
    /// Создать новый анализатор
    pub fn new(uncertainty_params: UncertaintyParams) -> Self {
        Self { uncertainty_params }
    }

    /// Выполнить анализ неопределенности методом Монте-Карло
    ///
    /// # Аргументы
    /// * `base_params` - Базовые параметры окружающей среды
    /// * `simulation_fn` - Функция симуляции, принимающая параметры и возвращающая результат
    pub fn monte_carlo_analysis<F>(
        &self,
        base_params: &EnvironmentParams,
        simulation_fn: F,
    ) -> Result<UncertaintyResult>
    where
        F: Fn(&EnvironmentParams) -> Result<f64> + Sync,
    {
        use rand::Rng;
        use rand::SeedableRng;

        // Параллельное выполнение симуляций
        let results: Vec<f64> = (0..self.uncertainty_params.n_simulations)
            .into_par_iter()
            .filter_map(|i| {
                let mut rng = rand::rngs::StdRng::seed_from_u64(i as u64);

                // Создать возмущенные параметры
                let mut perturbed = base_params.clone();

                // Возмущение температуры
                let temp_factor = 1.0
                    + rng.gen_range(
                        -self.uncertainty_params.air_temp_uncertainty / 100.0
                            ..self.uncertainty_params.air_temp_uncertainty / 100.0,
                    );
                perturbed.air_temp *= temp_factor;

                // Возмущение льдистости
                let ice_factor = 1.0
                    + rng.gen_range(
                        -self.uncertainty_params.ice_content_uncertainty / 100.0
                            ..self.uncertainty_params.ice_content_uncertainty / 100.0,
                    );
                perturbed.ice_content = (perturbed.ice_content * ice_factor).clamp(0.1, 0.95);

                // Возмущение растительности
                let veg_factor = 1.0
                    + rng.gen_range(
                        -self.uncertainty_params.vegetation_uncertainty / 100.0
                            ..self.uncertainty_params.vegetation_uncertainty / 100.0,
                    );
                perturbed.vegetation_cover =
                    (perturbed.vegetation_cover * veg_factor).clamp(0.0, 1.0);

                // Выполнить симуляцию
                simulation_fn(&perturbed).ok()
            })
            .collect();

        if results.is_empty() {
            return Err(ThermokarstError::SimulationFailed(
                "Все симуляции Монте-Карло завершились с ошибкой".to_string(),
            ));
        }

        Ok(UncertaintyResult::from_samples(results))
    }

    /// Анализ чувствительности: изменение одного параметра
    pub fn sensitivity_analysis<F>(
        &self,
        base_params: &EnvironmentParams,
        parameter_name: &str,
        variation_range: (f64, f64),
        n_steps: usize,
        simulation_fn: F,
    ) -> Result<SensitivityResult>
    where
        F: Fn(&EnvironmentParams) -> Result<f64>,
    {
        let mut parameter_values = Vec::new();
        let mut output_values = Vec::new();

        let step = (variation_range.1 - variation_range.0) / (n_steps - 1) as f64;

        for i in 0..n_steps {
            let value = variation_range.0 + i as f64 * step;
            let mut params = base_params.clone();

            // Изменить соответствующий параметр
            match parameter_name {
                "air_temp" => params.air_temp = value,
                "ice_content" => params.ice_content = value.clamp(0.1, 0.95),
                "vegetation_cover" => params.vegetation_cover = value.clamp(0.0, 1.0),
                "water_availability" => params.water_availability = value.clamp(0.0, 1.0),
                _ => {
                    return Err(ThermokarstError::InvalidParameters(format!(
                        "Неизвестный параметр: {}",
                        parameter_name
                    )))
                }
            }

            if let Ok(result) = simulation_fn(&params) {
                parameter_values.push(value);
                output_values.push(result);
            }
        }

        Ok(SensitivityResult {
            parameter_name: parameter_name.to_string(),
            parameter_values,
            output_values,
        })
    }
}

/// Результат анализа чувствительности
#[derive(Debug, Clone)]
pub struct SensitivityResult {
    /// Имя параметра
    pub parameter_name: String,
    /// Значения параметра
    pub parameter_values: Vec<f64>,
    /// Соответствующие выходные значения
    pub output_values: Vec<f64>,
}

impl SensitivityResult {
    /// Рассчитать индекс чувствительности (нормализованный градиент)
    pub fn sensitivity_index(&self) -> f64 {
        if self.parameter_values.len() < 2 {
            return 0.0;
        }

        let n = self.parameter_values.len();
        let param_range = self.parameter_values[n - 1] - self.parameter_values[0];
        let output_range = self.output_values[n - 1] - self.output_values[0];

        if param_range.abs() < 1e-10 {
            return 0.0;
        }

        // Нормализованный градиент
        let param_mean = self.parameter_values.iter().sum::<f64>() / n as f64;
        let output_mean = self.output_values.iter().sum::<f64>() / n as f64;

        if output_mean.abs() < 1e-10 {
            return 0.0;
        }

        (output_range / param_range) * (param_mean / output_mean)
    }

    /// Вывести отчет
    pub fn report(&self) -> String {
        format!(
            "Sensitivity Analysis for '{}':\n\
             Parameter range: [{:.3}, {:.3}]\n\
             Output range: [{:.3}, {:.3}]\n\
             Sensitivity index: {:.3}",
            self.parameter_name,
            self.parameter_values.first().unwrap_or(&0.0),
            self.parameter_values.last().unwrap_or(&0.0),
            self.output_values.first().unwrap_or(&0.0),
            self.output_values.last().unwrap_or(&0.0),
            self.sensitivity_index()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uncertainty_result_from_samples() {
        let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let result = UncertaintyResult::from_samples(samples);

        assert!((result.mean - 5.5).abs() < 0.1);
        assert!((result.median - 5.5).abs() < 0.1);
        assert!(result.std_dev > 0.0);

        println!("{}", result.report());
    }

    #[test]
    fn test_monte_carlo_simple() {
        let params = EnvironmentParams::default();
        let uncertainty = UncertaintyParams {
            n_simulations: 100,
            ..Default::default()
        };

        let analyzer = UncertaintyAnalyzer::new(uncertainty);

        // Простая функция: возвращает температуру
        let result = analyzer
            .monte_carlo_analysis(&params, |p| Ok(p.air_temp))
            .unwrap();

        println!("Monte Carlo test:");
        println!("{}", result.report());

        // Среднее должно быть близко к исходной температуре
        assert!((result.mean - params.air_temp).abs() < 1.0);

        // Должна быть некоторая неопределенность
        assert!(result.std_dev > 0.0);
    }

    #[test]
    fn test_sensitivity_analysis() {
        let params = EnvironmentParams::default();
        let uncertainty = UncertaintyParams::default();
        let analyzer = UncertaintyAnalyzer::new(uncertainty);

        // Анализ чувствительности к температуре
        let result = analyzer
            .sensitivity_analysis(
                &params,
                "air_temp",
                (0.0, 10.0),
                10,
                |p| Ok(p.air_temp * 2.0), // Линейная зависимость
            )
            .unwrap();

        println!("{}", result.report());

        assert_eq!(result.parameter_values.len(), 10);
        assert_eq!(result.output_values.len(), 10);

        // Индекс чувствительности должен быть положительным
        assert!(result.sensitivity_index() > 0.0);
    }

    #[test]
    fn test_confidence_interval() {
        let samples: Vec<f64> = (0..1000).map(|i| i as f64).collect();
        let result = UncertaintyResult::from_samples(samples);

        let ci_width = result.confidence_interval_width();

        // 90% интервал должен покрывать большую часть данных
        assert!(ci_width > 800.0);

        println!("CI width: {:.1}", ci_width);
        println!(
            "Relative uncertainty: {:.1}%",
            result.relative_uncertainty()
        );
    }
}
