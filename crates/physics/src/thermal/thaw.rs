//! Расчет глубины протаивания по формуле Атласова

use thermokarst_core::{EnvironmentParams, Result, ThermokarstError};

// Физические константы
const L: f64 = 334_000.0; // Дж/кг - скрытая теплота плавления льда
const RHO_W: f64 = 1000.0; // кг/м³ - плотность воды
const SECONDS_PER_DAY: f64 = 86_400.0;

// Региональные константы формулы Атласова для Якутии
const BETA: f64 = 0.30; // коэффициент покрова/пожара (ослаблен с 0.45)
const GAMMA: f64 = 0.12; // коэффициент континентальности
const DT0: f64 = 40.0; // базовая амплитуда температур, °C

/// Калькулятор глубины протаивания по формуле Атласова
pub struct ThawDepthCalculator {
    params: EnvironmentParams,
}

impl ThawDepthCalculator {
    pub fn new(params: EnvironmentParams) -> Self {
        Self { params }
    }

    /// Расчет глубины протаивания по формуле Атласова
    ///
    /// Формула: ξ_A = √(2λₜ·DDT / (L·ρw·w^0.7)) · exp(β·(1-V)) · (1 + γ·ln(ΔT/ΔT₀))
    ///
    /// где:
    /// - λₜ - теплопроводность талого грунта (зависит от влажности по Йоханзену)
    /// - DDT - градусо-дни тепла (degree-days of thawing)
    /// - L - скрытая теплота плавления льда
    /// - ρw - плотность воды
    /// - w - объемная льдистость (усиленное влияние через w^0.7)
    /// - β - коэффициент покрова (0.30 для Якутии)
    /// - V - плотность растительного покрова (0-1)
    /// - γ - коэффициент континентальности (0.12 для Якутии)
    /// - ΔT - годовая амплитуда температур
    /// - ΔT₀ - базовая амплитуда (40°C)
    pub fn calculate(&self, year: u32) -> Result<f64> {
        if year == 0 {
            return Err(ThermokarstError::InvalidParameters(
                "Год должен быть больше 0".to_string(),
            ));
        }

        // 1. Базовая формула Стефана: ξ₀ = √(2λₜ·DDT / (L·ρw·w))

        // Теплопроводность талого грунта с учетом влажности (модель Йоханзена)
        let lambda_t = self.params.soil_type.thermal_conductivity(self.params.soil_saturation_ratio);

        // DDT (degree-days of thawing) в секундах
        // DDT = средняя температура × продолжительность сезона
        let ddt_days = self.params.air_temp * self.params.warm_season_days as f64;
        let ddt_seconds = ddt_days * SECONDS_PER_DAY;

        // Объемная льдистость
        let w = self.params.ice_content;

        // Проверка на деление на ноль
        if w < 0.01 {
            return Err(ThermokarstError::InvalidParameters(
                "Льдистость слишком мала".to_string(),
            ));
        }

        // Усиленное влияние льдистости: w^0.7 вместо w
        // Это делает модель более чувствительной к изменениям льдистости
        let w_effective = w.powf(0.7);

        // Базовая глубина по Стефану
        let xi_0_squared = (2.0 * lambda_t * ddt_seconds) / (L * RHO_W * w_effective);
        let xi_0 = xi_0_squared.sqrt();

        // 2. Коэффициент покрова/пожара: K_fire = exp(β·(1-V))
        let v = self.params.vegetation_cover;
        let k_fire = (BETA * (1.0 - v)).exp();

        // 3. Функция континентальности: f(ΔT) = 1 + γ·ln(ΔT/ΔT₀)
        let delta_t = self.params.temperature_amplitude;
        let f_continental = if delta_t > 0.0 {
            1.0 + GAMMA * (delta_t / DT0).ln()
        } else {
            1.0
        };

        // 4. Итоговая формула Атласова (без f_moisture - влажность уже в λₜ)
        let xi_a = xi_0 * k_fire * f_continental;

        // Учитываем время (корень из года для многолетнего процесса)
        let depth = xi_a * (year as f64).sqrt();

        Ok(depth.max(0.0))
    }

    /// Расчет скорости протаивания (м/год)
    pub fn thaw_rate(&self, year: u32) -> Result<f64> {
        if year == 0 {
            return Ok(0.0);
        }

        let current = self.calculate(year)?;
        let previous = if year > 1 {
            self.calculate(year - 1)?
        } else {
            0.0
        };

        Ok(current - previous)
    }

    /// Расчет глубины сезонного протаивания (активный слой)
    /// Это глубина протаивания за один сезон (year = 1)
    pub fn active_layer_depth(&self) -> Result<f64> {
        self.calculate(1)
    }

    /// Получить коэффициент K_fire для текущих параметров
    pub fn k_fire(&self) -> f64 {
        (BETA * (1.0 - self.params.vegetation_cover)).exp()
    }

    /// Получить функцию континентальности для текущих параметров
    pub fn f_continental(&self) -> f64 {
        let delta_t = self.params.temperature_amplitude;
        if delta_t > 0.0 {
            1.0 + GAMMA * (delta_t / DT0).ln()
        } else {
            1.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use thermokarst_core::SoilType;

    #[test]
    fn test_atlasov_formula_basic() {
        let mut params = EnvironmentParams::default();
        params.air_temp = 10.0;
        params.warm_season_days = 100;
        params.ice_content = 0.3;
        params.vegetation_cover = 0.8;
        params.temperature_amplitude = 88.0;

        let calc = ThawDepthCalculator::new(params);
        let depth = calc.calculate(1).unwrap();

        // Глубина должна быть положительной и разумной
        assert!(depth > 0.0);
        assert!(depth < 10.0);
    }

    #[test]
    fn test_k_fire_coefficient() {
        let mut params = EnvironmentParams::default();

        // Полный покров (V=1.0) → K_fire ≈ 1.0
        params.vegetation_cover = 1.0;
        let calc = ThawDepthCalculator::new(params.clone());
        let k_fire_full = calc.k_fire();
        assert!((k_fire_full - 1.0).abs() < 0.01);

        // Нет покрова (V=0.0) → K_fire ≈ 1.35 (с новым β=0.30)
        params.vegetation_cover = 0.0;
        let calc = ThawDepthCalculator::new(params);
        let k_fire_bare = calc.k_fire();
        assert!((k_fire_bare - 1.35).abs() < 0.05);
    }

    #[test]
    fn test_continental_function() {
        let mut params = EnvironmentParams::default();

        // Базовая амплитуда (40°C) → f = 1.0
        params.temperature_amplitude = 40.0;
        let calc = ThawDepthCalculator::new(params.clone());
        let f_base = calc.f_continental();
        assert!((f_base - 1.0).abs() < 0.001);

        // Якутская амплитуда (88°C) → f ≈ 1.095
        params.temperature_amplitude = 88.0;
        let calc = ThawDepthCalculator::new(params);
        let f_yakutia = calc.f_continental();
        assert!(f_yakutia > 1.08 && f_yakutia < 1.11);
    }

    #[test]
    fn test_thaw_depth_increases_with_time() {
        let params = EnvironmentParams::default();
        let calc = ThawDepthCalculator::new(params);

        let depth_year1 = calc.calculate(1).unwrap();
        let depth_year10 = calc.calculate(10).unwrap();

        assert!(depth_year10 > depth_year1);
    }

    #[test]
    fn test_vegetation_reduces_thaw() {
        let mut params1 = EnvironmentParams::default();
        params1.vegetation_cover = 0.0; // Гарь

        let mut params2 = EnvironmentParams::default();
        params2.vegetation_cover = 0.9; // Густой лес

        let calc1 = ThawDepthCalculator::new(params1);
        let calc2 = ThawDepthCalculator::new(params2);

        let depth1 = calc1.calculate(5).unwrap();
        let depth2 = calc2.calculate(5).unwrap();

        // Гарь протаивает глубже
        assert!(depth1 > depth2);
    }
}
