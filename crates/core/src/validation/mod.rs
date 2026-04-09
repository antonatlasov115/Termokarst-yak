//! Фреймворк валидации модели
//!
//! Предоставляет метрики для оценки точности модели:
//! - RMSE (Root Mean Square Error)
//! - MAE (Mean Absolute Error)
//! - R² (Coefficient of Determination)
//! - Bias (Систематическая ошибка)

/// Метрики валидации модели
#[derive(Debug, Clone)]
pub struct ValidationMetrics {
    /// Среднеквадратичная ошибка (RMSE)
    pub rmse: f64,
    /// Средняя абсолютная ошибка (MAE)
    pub mae: f64,
    /// Коэффициент детерминации (R²)
    pub r_squared: f64,
    /// Систематическая ошибка (Bias)
    pub bias: f64,
    /// Количество наблюдений
    pub n: usize,
}

impl ValidationMetrics {
    /// Рассчитать метрики валидации
    ///
    /// # Аргументы
    /// * `observed` - Наблюдаемые значения
    /// * `simulated` - Смоделированные значения
    pub fn calculate(observed: &[f64], simulated: &[f64]) -> Self {
        assert_eq!(
            observed.len(),
            simulated.len(),
            "Arrays must have same length"
        );

        let n = observed.len();
        if n == 0 {
            return Self::empty();
        }

        let n_f64 = n as f64;

        // Среднее наблюдаемых значений
        let obs_mean = observed.iter().sum::<f64>() / n_f64;

        // Остатки (residuals)
        let residuals: Vec<f64> = observed
            .iter()
            .zip(simulated.iter())
            .map(|(o, s)| o - s)
            .collect();

        // RMSE = sqrt(mean(residuals²))
        let rmse = (residuals.iter().map(|r| r.powi(2)).sum::<f64>() / n_f64).sqrt();

        // MAE = mean(|residuals|)
        let mae = residuals.iter().map(|r| r.abs()).sum::<f64>() / n_f64;

        // Bias = mean(residuals)
        let bias = residuals.iter().sum::<f64>() / n_f64;

        // R² = 1 - SS_res / SS_tot
        let ss_tot = observed.iter().map(|o| (o - obs_mean).powi(2)).sum::<f64>();

        let ss_res = residuals.iter().map(|r| r.powi(2)).sum::<f64>();

        let r_squared = if ss_tot > 0.0 {
            1.0 - ss_res / ss_tot
        } else {
            0.0
        };

        Self {
            rmse,
            mae,
            r_squared,
            bias,
            n,
        }
    }

    /// Создать пустые метрики
    fn empty() -> Self {
        Self {
            rmse: 0.0,
            mae: 0.0,
            r_squared: 0.0,
            bias: 0.0,
            n: 0,
        }
    }

    /// Проверить, является ли модель хорошей
    /// Критерии: R² > 0.7, RMSE < 20% от среднего
    pub fn is_good_fit(&self, mean_observed: f64) -> bool {
        self.r_squared > 0.7 && self.rmse < 0.2 * mean_observed.abs()
    }

    /// Относительная RMSE (%)
    pub fn relative_rmse(&self, mean_observed: f64) -> f64 {
        if mean_observed.abs() < 1e-10 {
            return 0.0;
        }
        (self.rmse / mean_observed.abs()) * 100.0
    }

    /// Относительная MAE (%)
    pub fn relative_mae(&self, mean_observed: f64) -> f64 {
        if mean_observed.abs() < 1e-10 {
            return 0.0;
        }
        (self.mae / mean_observed.abs()) * 100.0
    }

    /// Вывести отчет о метриках
    pub fn report(&self, mean_observed: f64) -> String {
        format!(
            "Validation Metrics (n={}):\n\
             RMSE: {:.4} ({:.1}%)\n\
             MAE: {:.4} ({:.1}%)\n\
             R²: {:.4}\n\
             Bias: {:.4}\n\
             Fit quality: {}",
            self.n,
            self.rmse,
            self.relative_rmse(mean_observed),
            self.mae,
            self.relative_mae(mean_observed),
            self.r_squared,
            self.bias,
            if self.is_good_fit(mean_observed) {
                "Good"
            } else {
                "Poor"
            }
        )
    }
}

/// Кросс-валидация модели
pub struct CrossValidation {
    /// Количество фолдов
    k_folds: usize,
}

impl CrossValidation {
    /// Создать новый объект кросс-валидации
    pub fn new(k_folds: usize) -> Self {
        assert!(k_folds > 1, "k_folds must be > 1");
        Self { k_folds }
    }

    /// Разделить данные на фолды
    pub fn split_folds<T: Clone>(&self, data: &[T]) -> Vec<(Vec<T>, Vec<T>)> {
        let n = data.len();
        let fold_size = n / self.k_folds;
        let mut folds = Vec::new();

        for i in 0..self.k_folds {
            let test_start = i * fold_size;
            let test_end = if i == self.k_folds - 1 {
                n
            } else {
                (i + 1) * fold_size
            };

            let mut train = Vec::new();
            let mut test = Vec::new();

            for (j, item) in data.iter().enumerate() {
                if j >= test_start && j < test_end {
                    test.push(item.clone());
                } else {
                    train.push(item.clone());
                }
            }

            folds.push((train, test));
        }

        folds
    }

    /// Выполнить кросс-валидацию
    pub fn validate<F>(&self, observed: &[f64], predict_fn: F) -> Vec<ValidationMetrics>
    where
        F: Fn(&[f64]) -> Vec<f64>,
    {
        let folds = self.split_folds(observed);
        let mut metrics = Vec::new();

        for (train, test) in folds {
            let predictions = predict_fn(&train);
            let test_predictions: Vec<f64> = predictions.iter().take(test.len()).copied().collect();

            let fold_metrics = ValidationMetrics::calculate(&test, &test_predictions);
            metrics.push(fold_metrics);
        }

        metrics
    }
}

/// Статистика временных рядов
pub struct TimeSeriesMetrics {
    /// Коэффициент Нэша-Сатклиффа (Nash-Sutcliffe Efficiency)
    pub nse: f64,
    /// Индекс согласия (Index of Agreement)
    pub d_index: f64,
    /// Коэффициент корреляции Пирсона
    pub pearson_r: f64,
}

impl TimeSeriesMetrics {
    /// Рассчитать метрики временных рядов
    pub fn calculate(observed: &[f64], simulated: &[f64]) -> Self {
        assert_eq!(observed.len(), simulated.len());

        let n = observed.len() as f64;
        let obs_mean = observed.iter().sum::<f64>() / n;

        // Nash-Sutcliffe Efficiency
        let numerator: f64 = observed
            .iter()
            .zip(simulated.iter())
            .map(|(o, s)| (o - s).powi(2))
            .sum();

        let denominator: f64 = observed.iter().map(|o| (o - obs_mean).powi(2)).sum();

        let nse = if denominator > 0.0 {
            1.0 - numerator / denominator
        } else {
            0.0
        };

        // Index of Agreement
        let d_numerator: f64 = observed
            .iter()
            .zip(simulated.iter())
            .map(|(o, s)| (o - s).powi(2))
            .sum();

        let d_denominator: f64 = observed
            .iter()
            .zip(simulated.iter())
            .map(|(o, s)| ((s - obs_mean).abs() + (o - obs_mean).abs()).powi(2))
            .sum();

        let d_index = if d_denominator > 0.0 {
            1.0 - d_numerator / d_denominator
        } else {
            0.0
        };

        // Pearson correlation
        let sim_mean = simulated.iter().sum::<f64>() / n;

        let cov: f64 = observed
            .iter()
            .zip(simulated.iter())
            .map(|(o, s)| (o - obs_mean) * (s - sim_mean))
            .sum::<f64>()
            / n;

        let obs_std = (observed.iter().map(|o| (o - obs_mean).powi(2)).sum::<f64>() / n).sqrt();

        let sim_std = (simulated
            .iter()
            .map(|s| (s - sim_mean).powi(2))
            .sum::<f64>()
            / n)
            .sqrt();

        let pearson_r = if obs_std > 0.0 && sim_std > 0.0 {
            cov / (obs_std * sim_std)
        } else {
            0.0
        };

        Self {
            nse,
            d_index,
            pearson_r,
        }
    }

    /// Вывести отчет
    pub fn report(&self) -> String {
        format!(
            "Time Series Metrics:\n\
             Nash-Sutcliffe Efficiency: {:.4}\n\
             Index of Agreement: {:.4}\n\
             Pearson Correlation: {:.4}",
            self.nse, self.d_index, self.pearson_r
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_fit() {
        let observed = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let simulated = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        let metrics = ValidationMetrics::calculate(&observed, &simulated);

        assert_eq!(metrics.rmse, 0.0);
        assert_eq!(metrics.mae, 0.0);
        assert_eq!(metrics.bias, 0.0);
        assert_eq!(metrics.r_squared, 1.0);
    }

    #[test]
    fn test_metrics_calculation() {
        let observed = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let simulated = vec![1.1, 2.2, 2.9, 4.1, 4.8];

        let metrics = ValidationMetrics::calculate(&observed, &simulated);

        assert!(metrics.rmse > 0.0);
        assert!(metrics.mae > 0.0);
        assert!(metrics.r_squared > 0.9); // Хорошая корреляция

        println!("{}", metrics.report(3.0));
    }

    #[test]
    fn test_bias() {
        let observed = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let simulated = vec![1.5, 2.5, 3.5, 4.5, 5.5]; // Систематическое завышение на 0.5

        let metrics = ValidationMetrics::calculate(&observed, &simulated);

        assert!((metrics.bias + 0.5).abs() < 0.01); // Bias должен быть -0.5
    }

    #[test]
    fn test_cross_validation() {
        let cv = CrossValidation::new(5);
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

        let folds = cv.split_folds(&data);

        assert_eq!(folds.len(), 5);

        for (train, test) in &folds {
            assert!(train.len() + test.len() == data.len());
        }
    }

    #[test]
    fn test_time_series_metrics() {
        let observed = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let simulated = vec![1.1, 2.1, 3.1, 4.1, 5.1];

        let metrics = TimeSeriesMetrics::calculate(&observed, &simulated);

        assert!(metrics.nse > 0.9); // Хорошая эффективность
        assert!(metrics.d_index > 0.9); // Хорошее согласие
        assert!(metrics.pearson_r > 0.99); // Высокая корреляция

        println!("{}", metrics.report());
    }

    #[test]
    fn test_relative_errors() {
        let observed = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let simulated = vec![11.0, 19.0, 31.0, 39.0, 51.0];

        let metrics = ValidationMetrics::calculate(&observed, &simulated);
        let mean_obs = observed.iter().sum::<f64>() / observed.len() as f64;

        let rel_rmse = metrics.relative_rmse(mean_obs);
        let rel_mae = metrics.relative_mae(mean_obs);

        assert!(rel_rmse < 10.0); // Менее 10% ошибки
        assert!(rel_mae < 10.0);

        println!("Relative RMSE: {:.2}%", rel_rmse);
        println!("Relative MAE: {:.2}%", rel_mae);
    }

    #[test]
    fn test_is_good_fit() {
        let observed = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let simulated = vec![11.0, 19.0, 31.0, 39.0, 51.0];

        let metrics = ValidationMetrics::calculate(&observed, &simulated);
        let mean_obs = observed.iter().sum::<f64>() / observed.len() as f64;

        assert!(metrics.is_good_fit(mean_obs));
    }
}
