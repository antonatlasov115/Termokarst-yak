//! Расчет просадки грунта при оттаивании

use thermokarst_core::EnvironmentParams;

/// Калькулятор просадки грунта
pub struct SubsidenceCalculator {
    params: EnvironmentParams,
}

impl SubsidenceCalculator {
    pub fn new(params: EnvironmentParams) -> Self {
        Self { params }
    }

    /// Расчет просадки при оттаивании
    ///
    /// Учитывает:
    /// - Объем растаявшего льда
    /// - Тип грунта и его сжимаемость
    /// - Льдистость
    pub fn calculate_subsidence(&self, thaw_depth: f64) -> f64 {
        // Объем льда, который растаял
        let ice_volume_fraction = self.params.ice_content;

        // Коэффициент сжимаемости грунта
        let compression = self.params.soil_type.compression_coefficient();

        // Базовая просадка от таяния льда
        let ice_subsidence = thaw_depth * ice_volume_fraction * 0.9; // 90% объема льда

        // Дополнительная просадка от сжатия грунта
        let compression_subsidence = thaw_depth * compression;

        ice_subsidence + compression_subsidence
    }

    /// Расчет коэффициента просадочности
    pub fn subsidence_coefficient(&self) -> f64 {
        let ice_factor = self.params.ice_content * 0.9;
        let compression_factor = self.params.soil_type.compression_coefficient();

        ice_factor + compression_factor
    }

    /// Расчет объема просадки (м³) для заданной площади
    pub fn subsidence_volume(&self, thaw_depth: f64, area: f64) -> f64 {
        let subsidence = self.calculate_subsidence(thaw_depth);
        subsidence * area
    }

    /// Расчет деформации грунта (относительная)
    pub fn strain(&self, thaw_depth: f64) -> f64 {
        if thaw_depth <= 0.0 {
            return 0.0;
        }

        let subsidence = self.calculate_subsidence(thaw_depth);
        subsidence / thaw_depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use thermokarst_core::SoilType;

    #[test]
    fn test_subsidence_increases_with_ice_content() {
        let mut params1 = EnvironmentParams::default();
        params1.ice_content = 0.3;

        let mut params2 = EnvironmentParams::default();
        params2.ice_content = 0.8;

        let calc1 = SubsidenceCalculator::new(params1);
        let calc2 = SubsidenceCalculator::new(params2);

        let subsidence1 = calc1.calculate_subsidence(2.0);
        let subsidence2 = calc2.calculate_subsidence(2.0);

        assert!(subsidence2 > subsidence1);
    }

    #[test]
    fn test_peat_has_high_subsidence() {
        let mut params = EnvironmentParams::default();
        params.soil_type = SoilType::Peat;
        params.ice_content = 0.7;

        let calc = SubsidenceCalculator::new(params);
        let coef = calc.subsidence_coefficient();

        assert!(coef > 0.6); // Торф имеет высокую просадочность
    }
}
