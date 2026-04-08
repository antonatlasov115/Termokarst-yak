//! Модель теплопроводности Johansen
//!
//! Реализует модель Johansen (1975) для расчета теплопроводности грунта
//! в зависимости от пористости, насыщенности, состава и размера зерен.

use thermokarst_core::EnvironmentParams;

/// Размер зерен грунта
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GrainSize {
    /// Мелкозернистый (глина, ил)
    Fine,
    /// Крупнозернистый (песок, гравий)
    Coarse,
}

/// Калькулятор теплопроводности по модели Johansen
pub struct JohansenConductivity {
    /// Пористость (0-1)
    porosity: f64,
    /// Степень насыщения (0-1)
    saturation: f64,
    /// Содержание кварца (0-1)
    quartz_content: f64,
    /// Размер зерен
    grain_size: GrainSize,
    /// Плотность сухого грунта (кг/м³)
    dry_density: f64,
}

impl JohansenConductivity {
    /// Создать новый калькулятор
    pub fn new(
        porosity: f64,
        saturation: f64,
        quartz_content: f64,
        grain_size: GrainSize,
        dry_density: f64,
    ) -> Self {
        Self {
            porosity,
            saturation,
            quartz_content,
            grain_size,
            dry_density,
        }
    }

    /// Создать из параметров окружения
    pub fn from_params(params: &EnvironmentParams) -> Self {
        let porosity = params.soil_type.porosity();
        let saturation = params.ice_content; // Используем льдистость как насыщенность
        let quartz_content = match params.soil_type {
            thermokarst_core::SoilType::Sand => 0.9,
            thermokarst_core::SoilType::Silt => 0.4,
            thermokarst_core::SoilType::Clay => 0.2,
            thermokarst_core::SoilType::Peat => 0.1,
            thermokarst_core::SoilType::Loam => 0.5,
        };
        let grain_size = match params.soil_type {
            thermokarst_core::SoilType::Sand => GrainSize::Coarse,
            thermokarst_core::SoilType::Silt => GrainSize::Fine,
            thermokarst_core::SoilType::Clay => GrainSize::Fine,
            thermokarst_core::SoilType::Peat => GrainSize::Fine,
            thermokarst_core::SoilType::Loam => GrainSize::Fine,
        };
        // Типичная плотность сухого грунта (кг/м³)
        let dry_density = match params.soil_type {
            thermokarst_core::SoilType::Sand => 1600.0,
            thermokarst_core::SoilType::Silt => 1400.0,
            thermokarst_core::SoilType::Clay => 1300.0,
            thermokarst_core::SoilType::Peat => 800.0,
            thermokarst_core::SoilType::Loam => 1500.0,
        };

        Self::new(
            porosity,
            saturation,
            quartz_content,
            grain_size,
            dry_density,
        )
    }

    /// Рассчитать число Керстена (Kersten number)
    fn kersten_number(&self) -> f64 {
        let ke = match self.grain_size {
            GrainSize::Fine => {
                // Для мелкозернистых: Ke = log10(S) + 1
                self.saturation.log10() + 1.0
            }
            GrainSize::Coarse => {
                // Для крупнозернистых: Ke = 0.7 * log10(S) + 1
                0.7 * self.saturation.log10() + 1.0
            }
        };
        ke.max(0.0).min(1.0)
    }

    /// Теплопроводность твердой фазы (Вт/(м·К))
    fn solid_conductivity(&self) -> f64 {
        const K_QUARTZ: f64 = 7.7; // Теплопроводность кварца

        let k_other: f64 = if self.quartz_content < 0.2 && self.grain_size == GrainSize::Coarse {
            3.0 // Другие минералы в крупнозернистых
        } else {
            2.0 // Другие минералы в мелкозернистых
        };

        // Среднее геометрическое
        K_QUARTZ.powf(self.quartz_content) * k_other.powf(1.0 - self.quartz_content)
    }

    /// Теплопроводность сухого грунта (Вт/(м·К))
    fn dry_conductivity(&self) -> f64 {
        // Формула Johansen для сухого грунта
        (0.135 * self.dry_density + 64.7) / (2700.0 - 0.947 * self.dry_density)
    }

    /// Теплопроводность насыщенного грунта (Вт/(м·К))
    fn saturated_conductivity(&self, frozen: bool) -> f64 {
        const K_WATER: f64 = 0.57; // Теплопроводность воды
        const K_ICE: f64 = 2.2; // Теплопроводность льда

        let k_solids = self.solid_conductivity();
        let k_fluid = if frozen { K_ICE } else { K_WATER };

        // Среднее геометрическое
        k_solids.powf(1.0 - self.porosity) * k_fluid.powf(self.porosity)
    }

    /// Рассчитать эффективную теплопроводность (Вт/(м·К))
    pub fn calculate(&self, frozen: bool) -> f64 {
        let k_dry = self.dry_conductivity();
        let k_sat = self.saturated_conductivity(frozen);
        let ke = self.kersten_number();

        // Модель Johansen
        k_dry + (k_sat - k_dry) * ke
    }

    /// Обновить степень насыщения
    pub fn set_saturation(&mut self, saturation: f64) {
        self.saturation = saturation.max(0.0).min(1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use thermokarst_core::SoilType;

    #[test]
    fn test_johansen_sand() {
        let calc = JohansenConductivity::new(
            0.4, // porosity
            0.8, // saturation
            0.9, // quartz content
            GrainSize::Coarse,
            1600.0, // dry density
        );

        let k_unfrozen = calc.calculate(false);
        let k_frozen = calc.calculate(true);

        // Мерзлый грунт должен иметь большую теплопроводность
        assert!(k_frozen > k_unfrozen);

        // Разумные значения для песка
        assert!(k_unfrozen > 1.0 && k_unfrozen < 4.0);
        assert!(k_frozen > 1.5 && k_frozen < 5.0);
    }

    #[test]
    fn test_johansen_clay() {
        let calc = JohansenConductivity::new(
            0.5, // porosity
            0.9, // saturation
            0.2, // quartz content
            GrainSize::Fine,
            1350.0, // dry density
        );

        let k = calc.calculate(false);

        // Глина имеет меньшую теплопроводность
        assert!(k > 0.5 && k < 2.0);
    }

    #[test]
    fn test_saturation_effect() {
        let mut calc = JohansenConductivity::new(0.4, 0.2, 0.9, GrainSize::Coarse, 1600.0);

        let k_low = calc.calculate(false);

        calc.set_saturation(0.9);
        let k_high = calc.calculate(false);

        // Большая насыщенность -> большая теплопроводность
        assert!(k_high > k_low);
    }

    #[test]
    fn test_from_params() {
        let params = EnvironmentParams {
            air_temp: -5.0,
            permafrost_temp: -3.0,
            ice_content: 0.7,
            soil_type: SoilType::Sand,
            vegetation_cover: 0.3,
            permafrost_depth: 100.0,
            warm_season_days: 120,
            water_availability: 0.5,
        };

        let calc = JohansenConductivity::from_params(&params);
        let k = calc.calculate(true);

        assert!(k > 0.0);
    }
}
