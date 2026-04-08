//! Расчет глубины протаивания

use thermokarst_core::{EnvironmentParams, Result, ThermokarstError};

/// Калькулятор глубины протаивания
pub struct ThawDepthCalculator {
    params: EnvironmentParams,
}

impl ThawDepthCalculator {
    pub fn new(params: EnvironmentParams) -> Self {
        Self { params }
    }

    /// Расчет глубины протаивания по модифицированной формуле Стефана
    ///
    /// Учитывает:
    /// - Теплопроводность грунта
    /// - Температурный режим
    /// - Растительный покров
    /// - Продолжительность теплого сезона
    pub fn calculate(&self, year: u32) -> Result<f64> {
        if year == 0 {
            return Err(ThermokarstError::InvalidParameters(
                "Год должен быть больше 0".to_string(),
            ));
        }

        // Коэффициент теплопроводности
        let k = self.params.soil_type.thermal_conductivity();

        // Эффективная температура с учетом продолжительности сезона
        let effective_temp = self.params.air_temp
            * (self.params.warm_season_days as f64 / 365.0);

        // Фактор растительного покрова (замедляет протаивание)
        let vegetation_factor = 1.0 - 0.35 * self.params.vegetation_cover;

        // Фактор льдистости (больше льда - больше энергии на таяние)
        let ice_factor = 1.0 - 0.2 * self.params.ice_content;

        // Базовая формула Стефана с модификациями
        let base_depth = (k * effective_temp * year as f64).sqrt();

        let depth = base_depth * vegetation_factor * ice_factor;

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
    pub fn active_layer_depth(&self) -> Result<f64> {
        let k = self.params.soil_type.thermal_conductivity();
        let temp_amplitude = self.params.air_temp.abs();

        // Упрощенная формула для активного слоя
        let depth = 0.5 * (k * temp_amplitude).sqrt();

        Ok(depth.max(0.3).min(3.0)) // Типично 0.3-3.0 м
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use thermokarst_core::SoilType;

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
        params1.vegetation_cover = 0.0;

        let mut params2 = EnvironmentParams::default();
        params2.vegetation_cover = 0.8;

        let calc1 = ThawDepthCalculator::new(params1);
        let calc2 = ThawDepthCalculator::new(params2);

        let depth1 = calc1.calculate(5).unwrap();
        let depth2 = calc2.calculate(5).unwrap();

        assert!(depth1 > depth2);
    }
}
