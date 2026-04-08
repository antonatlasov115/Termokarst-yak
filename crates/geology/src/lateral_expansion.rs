//! Расчет бокового расширения термокарстовых образований

use thermokarst_core::EnvironmentParams;

/// Калькулятор бокового расширения
pub struct LateralExpansionCalculator {
    params: EnvironmentParams,
}

impl LateralExpansionCalculator {
    pub fn new(params: EnvironmentParams) -> Self {
        Self { params }
    }

    /// Расчет диаметра термокарстовой линзы
    ///
    /// Учитывает:
    /// - Глубину просадки
    /// - Возраст образования
    /// - Доступность воды
    /// - Тип грунта
    pub fn calculate_diameter(&self, depth: f64, year: u32) -> f64 {
        if depth <= 0.0 || year == 0 {
            return 0.0;
        }

        // Базовый коэффициент расширения (боковое расширение медленнее вертикального)
        let base_expansion_rate = 0.4;

        // Фактор доступности воды (ускоряет боковое расширение)
        let water_factor = 1.0 + 0.6 * self.params.water_availability;

        // Фактор типа грунта (песок расширяется быстрее, глина медленнее)
        let soil_factor = match self.params.soil_type {
            thermokarst_core::SoilType::Sand => 1.3,
            thermokarst_core::SoilType::Clay => 0.7,
            thermokarst_core::SoilType::Peat => 1.1,
            thermokarst_core::SoilType::Loam => 1.0,
            thermokarst_core::SoilType::Silt => 1.2,
        };

        // Логарифмический рост со временем
        let time_factor = (year as f64 + 1.0).ln();

        // Итоговый диаметр
        let diameter = depth * base_expansion_rate * water_factor * soil_factor * time_factor;

        diameter.max(0.0)
    }

    /// Расчет скорости бокового расширения (м/год)
    pub fn expansion_rate(&self, depth: f64, year: u32) -> f64 {
        if year == 0 {
            return 0.0;
        }

        let current = self.calculate_diameter(depth, year);
        let previous = if year > 1 {
            self.calculate_diameter(depth, year - 1)
        } else {
            0.0
        };

        current - previous
    }

    /// Расчет площади термокарстового образования (м²)
    pub fn calculate_area(&self, depth: f64, year: u32) -> f64 {
        let diameter = self.calculate_diameter(depth, year);
        let radius = diameter / 2.0;

        std::f64::consts::PI * radius * radius
    }

    /// Расчет периметра (м)
    pub fn calculate_perimeter(&self, depth: f64, year: u32) -> f64 {
        let diameter = self.calculate_diameter(depth, year);

        std::f64::consts::PI * diameter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use thermokarst_core::SoilType;

    #[test]
    fn test_diameter_increases_with_time() {
        let params = EnvironmentParams::default();
        let calc = LateralExpansionCalculator::new(params);

        let d1 = calc.calculate_diameter(2.0, 1);
        let d10 = calc.calculate_diameter(2.0, 10);

        assert!(d10 > d1);
    }

    #[test]
    fn test_water_increases_expansion() {
        let mut params1 = EnvironmentParams::default();
        params1.water_availability = 0.2;

        let mut params2 = EnvironmentParams::default();
        params2.water_availability = 0.9;

        let calc1 = LateralExpansionCalculator::new(params1);
        let calc2 = LateralExpansionCalculator::new(params2);

        let d1 = calc1.calculate_diameter(2.0, 5);
        let d2 = calc2.calculate_diameter(2.0, 5);

        assert!(d2 > d1);
    }
}
