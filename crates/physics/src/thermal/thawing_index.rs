//! Расчет индексов оттаивания и промерзания
//!
//! Индексы оттаивания (DDT - Degree Days of Thawing) и промерзания (DDF - Degree Days of Freezing)
//! используются для оценки сезонного теплового воздействия на грунт.

use std::f64::consts::PI;

/// Калькулятор индексов оттаивания и промерзания
pub struct ThawingIndexCalculator {
    /// Среднегодовая температура воздуха (°C)
    maat: f64,
    /// Амплитуда годовых колебаний температуры (°C)
    amplitude: f64,
}

impl ThawingIndexCalculator {
    /// Создать новый калькулятор
    ///
    /// # Аргументы
    /// * `maat` - Среднегодовая температура воздуха (°C)
    /// * `amplitude` - Амплитуда годовых колебаний (°C)
    pub fn new(maat: f64, amplitude: f64) -> Self {
        Self { maat, amplitude }
    }

    /// Создать из дневных температур
    pub fn from_daily_temps(daily_temps: &[f64]) -> Self {
        let n = daily_temps.len() as f64;
        let maat = daily_temps.iter().sum::<f64>() / n;

        // Оценка амплитуды через стандартное отклонение
        let variance = daily_temps.iter().map(|t| (t - maat).powi(2)).sum::<f64>() / n;
        let amplitude = variance.sqrt() * 2.0; // Приблизительная амплитуда

        Self::new(maat, amplitude)
    }

    /// Рассчитать индекс оттаивания (DDT) - градусо-дни положительных температур
    pub fn calculate_ddt(&self) -> f64 {
        if self.maat >= self.amplitude / 2.0 {
            // Всегда положительная температура
            return self.maat * 365.0;
        }

        if self.maat <= -self.amplitude / 2.0 {
            // Всегда отрицательная температура
            return 0.0;
        }

        // Интегрирование синусоидальной функции температуры
        // T(t) = MAAT + A/2 * sin(2πt/365)
        // DDT = ∫[T(t) > 0] T(t) dt

        let thaw_season = self.thawing_season_length();
        let mean_thaw_temp = self.mean_thawing_temperature();

        thaw_season * mean_thaw_temp
    }

    /// Рассчитать индекс промерзания (DDF) - градусо-дни отрицательных температур
    pub fn calculate_ddf(&self) -> f64 {
        if self.maat >= self.amplitude / 2.0 {
            // Всегда положительная температура
            return 0.0;
        }

        if self.maat <= -self.amplitude / 2.0 {
            // Всегда отрицательная температура
            return self.maat.abs() * 365.0;
        }

        let freeze_season = self.freezing_season_length();
        let mean_freeze_temp = self.mean_freezing_temperature();

        freeze_season * mean_freeze_temp.abs()
    }

    /// Длительность сезона оттаивания (дни)
    pub fn thawing_season_length(&self) -> f64 {
        if self.maat >= self.amplitude / 2.0 {
            return 365.0;
        }

        if self.maat <= -self.amplitude / 2.0 {
            return 0.0;
        }

        // Решение уравнения: MAAT + A/2 * sin(2πt/365) = 0
        // sin(2πt/365) = -2*MAAT/A
        let sin_value = (-2.0 * self.maat / self.amplitude).max(-1.0).min(1.0);
        let angle = sin_value.asin();

        // Длительность теплого сезона
        (PI - 2.0 * angle) * 365.0 / (2.0 * PI)
    }

    /// Длительность сезона промерзания (дни)
    pub fn freezing_season_length(&self) -> f64 {
        365.0 - self.thawing_season_length()
    }

    /// Средняя температура в период оттаивания (°C)
    pub fn mean_thawing_temperature(&self) -> f64 {
        let thaw_days = self.thawing_season_length();
        if thaw_days <= 0.0 {
            return 0.0;
        }

        // Численное интегрирование
        let ddt = self.calculate_ddt_numerical();
        ddt / thaw_days
    }

    /// Средняя температура в период промерзания (°C)
    pub fn mean_freezing_temperature(&self) -> f64 {
        let freeze_days = self.freezing_season_length();
        if freeze_days <= 0.0 {
            return 0.0;
        }

        let ddf = self.calculate_ddf_numerical();
        -ddf / freeze_days
    }

    /// Численный расчет DDT
    fn calculate_ddt_numerical(&self) -> f64 {
        let mut sum = 0.0;
        for day in 0..365 {
            let t = self.temperature_at_day(day);
            if t > 0.0 {
                sum += t;
            }
        }
        sum
    }

    /// Численный расчет DDF
    fn calculate_ddf_numerical(&self) -> f64 {
        let mut sum = 0.0;
        for day in 0..365 {
            let t = self.temperature_at_day(day);
            if t < 0.0 {
                sum += t.abs();
            }
        }
        sum
    }

    /// Температура в заданный день года
    pub fn temperature_at_day(&self, day: u32) -> f64 {
        let t = day as f64 / 365.0;
        self.maat + (self.amplitude / 2.0) * (2.0 * PI * t).sin()
    }

    /// Рассчитать число Фроста (Frost Number)
    /// F = sqrt(DDF) / sqrt(DDT)
    /// Используется для оценки вероятности существования мерзлоты
    pub fn frost_number(&self) -> f64 {
        let ddf = self.calculate_ddf();
        let ddt = self.calculate_ddt();

        if ddt <= 0.0 {
            return f64::INFINITY;
        }

        (ddf / ddt).sqrt()
    }
}

/// Калькулятор на основе реальных дневных температур
pub struct DailyThawingIndex {
    daily_temps: Vec<f64>,
}

impl DailyThawingIndex {
    /// Создать из массива дневных температур
    pub fn new(daily_temps: Vec<f64>) -> Self {
        Self { daily_temps }
    }

    /// Рассчитать DDT
    pub fn calculate_ddt(&self) -> f64 {
        self.daily_temps.iter().filter(|&&t| t > 0.0).sum()
    }

    /// Рассчитать DDF
    pub fn calculate_ddf(&self) -> f64 {
        self.daily_temps
            .iter()
            .filter(|&&t| t < 0.0)
            .map(|t| t.abs())
            .sum()
    }

    /// Длительность сезона оттаивания
    pub fn thawing_season_length(&self) -> u32 {
        self.daily_temps.iter().filter(|&&t| t > 0.0).count() as u32
    }

    /// Длительность сезона промерзания
    pub fn freezing_season_length(&self) -> u32 {
        self.daily_temps.iter().filter(|&&t| t < 0.0).count() as u32
    }

    /// Число Фроста
    pub fn frost_number(&self) -> f64 {
        let ddf = self.calculate_ddf();
        let ddt = self.calculate_ddt();

        if ddt <= 0.0 {
            return f64::INFINITY;
        }

        (ddf / ddt).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thawing_index_yakutia() {
        // Типичные условия Центральной Якутии
        let calc = ThawingIndexCalculator::new(-10.0, 40.0);

        let ddt = calc.calculate_ddt();
        let ddf = calc.calculate_ddf();

        // DDT должен быть положительным
        assert!(ddt > 0.0);
        // DDF должен быть больше DDT (холодный климат)
        assert!(ddf > ddt);

        println!("DDT: {:.1}, DDF: {:.1}", ddt, ddf);
    }

    #[test]
    fn test_season_lengths() {
        let calc = ThawingIndexCalculator::new(-5.0, 30.0);

        let thaw_days = calc.thawing_season_length();
        let freeze_days = calc.freezing_season_length();

        // Сумма должна быть 365
        assert!((thaw_days + freeze_days - 365.0).abs() < 1.0);

        // В холодном климате сезон промерзания длиннее
        assert!(freeze_days > thaw_days);
    }

    #[test]
    fn test_frost_number() {
        let calc = ThawingIndexCalculator::new(-8.0, 35.0);
        let fn_val = calc.frost_number();

        // Для мерзлоты F > 0.5
        assert!(fn_val > 0.5);
        println!("Frost Number: {:.2}", fn_val);
    }

    #[test]
    fn test_daily_temps() {
        // Создаем синтетические дневные температуры
        let mut temps = Vec::new();
        for day in 0..365 {
            let t = -10.0 + 20.0 * (2.0 * PI * day as f64 / 365.0).sin();
            temps.push(t);
        }

        let calc = DailyThawingIndex::new(temps);
        let ddt = calc.calculate_ddt();
        let ddf = calc.calculate_ddf();

        assert!(ddt > 0.0);
        assert!(ddf > 0.0);

        println!("Daily DDT: {:.1}, DDF: {:.1}", ddt, ddf);
    }

    #[test]
    fn test_temperature_at_day() {
        let calc = ThawingIndexCalculator::new(0.0, 20.0);

        // День 0 (зима)
        let t_winter = calc.temperature_at_day(0);
        assert!(t_winter < 5.0);

        // День ~91 (весна/лето)
        let t_summer = calc.temperature_at_day(91);
        assert!(t_summer > 5.0);
    }

    #[test]
    fn test_from_daily_temps() {
        let temps: Vec<f64> = (0..365)
            .map(|day| -5.0 + 15.0 * (2.0 * PI * day as f64 / 365.0).sin())
            .collect();

        let calc = ThawingIndexCalculator::from_daily_temps(&temps);

        assert!((calc.maat + 5.0).abs() < 1.0); // MAAT должен быть около -5
        assert!(calc.amplitude > 10.0); // Амплитуда должна быть значительной
    }
}
