//! Расчет бокового расширения термокарстовых образований
//!
//! Включает диффузионную модель на основе CryoGrid3 и
//! адвективно-диффузионный транспорт седиментов.

use thermokarst_core::EnvironmentParams;

/// Калькулятор бокового расширения
pub struct LateralExpansionCalculator {
    params: EnvironmentParams,
}

impl LateralExpansionCalculator {
    pub fn new(params: EnvironmentParams) -> Self {
        Self { params }
    }

    /// Расчет диаметра термокарстовой линзы (улучшенная модель)
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
            thermokarst_core::SoilType::Silt => 1.2,
            thermokarst_core::SoilType::Loam => 1.0,
        };

        // Логарифмический рост со временем
        let time_factor = (year as f64 + 1.0).ln();

        // Итоговый диаметр
        let diameter = depth * base_expansion_rate * water_factor * soil_factor * time_factor;

        diameter.max(0.0)
    }

    /// Расчет диаметра с использованием диффузионной модели (CryoGrid3)
    ///
    /// # Аргументы
    /// * `depth` - Глубина просадки (м)
    /// * `year` - Возраст образования (годы)
    /// * `water_depth` - Глубина воды в термокарсте (м)
    pub fn calculate_diameter_diffusion(&self, depth: f64, year: u32, water_depth: f64) -> f64 {
        if depth <= 0.0 || year == 0 {
            return 0.0;
        }

        // Коэффициенты диффузии (м²/с)
        const K_LAND: f64 = 3e-10; // Диффузия на суше (медленная)
        const K_WATER: f64 = 3e-8; // Диффузия в воде (быстрая)

        // Эффективный коэффициент диффузии
        let k_eff = if water_depth > 0.0 { K_WATER } else { K_LAND };

        // Время в секундах
        let time_seconds = year as f64 * 365.25 * 86400.0;

        // Длина диффузии: L = sqrt(k * t)
        let diffusion_length = (k_eff * time_seconds).sqrt();

        // Диаметр с учетом глубины (более глубокие образования расширяются больше)
        let diameter = 2.0 * diffusion_length * (1.0 + depth / 5.0).sqrt();

        diameter.max(0.0)
    }

    /// Расчет скорости эрозии берега (м/год)
    ///
    /// # Аргументы
    /// * `slope_angle` - Угол наклона берега (радианы)
    /// * `water_present` - Наличие воды в термокарсте
    pub fn calculate_erosion_rate(&self, slope_angle: f64, water_present: bool) -> f64 {
        use std::f64::consts::PI;

        // Коэффициенты диффузии
        let k_eff = if water_present { 3e-8 } else { 3e-10 };

        // Критический угол (45°)
        let alpha_crit = PI / 4.0;

        if slope_angle.abs() < alpha_crit {
            // Диффузионный + адвективный транспорт
            let diffusive = k_eff * slope_angle;
            let advective = k_eff * slope_angle * slope_angle.atan().powi(2)
                / (alpha_crit.powi(2) - slope_angle.atan().powi(2));

            // Конвертация в м/год
            (diffusive + advective) * 365.25 * 86400.0
        } else {
            // Склон слишком крутой - обвал
            0.0
        }
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

    #[test]
    fn test_diffusion_model() {
        let params = EnvironmentParams::default();
        let calc = LateralExpansionCalculator::new(params);

        // Без воды (медленная диффузия)
        let d_land = calc.calculate_diameter_diffusion(2.0, 10, 0.0);

        // С водой (быстрая диффузия)
        let d_water = calc.calculate_diameter_diffusion(2.0, 10, 1.0);

        // Вода должна ускорять расширение
        assert!(d_water > d_land);

        println!(
            "Diameter (land): {:.2}m, Diameter (water): {:.2}m",
            d_land, d_water
        );
    }

    #[test]
    fn test_diffusion_time_dependency() {
        let params = EnvironmentParams::default();
        let calc = LateralExpansionCalculator::new(params);

        let d1 = calc.calculate_diameter_diffusion(2.0, 1, 1.0);
        let d10 = calc.calculate_diameter_diffusion(2.0, 10, 1.0);
        let d100 = calc.calculate_diameter_diffusion(2.0, 100, 1.0);

        // Диаметр должен расти со временем
        assert!(d10 > d1);
        assert!(d100 > d10);

        println!(
            "Year 1: {:.2}m, Year 10: {:.2}m, Year 100: {:.2}m",
            d1, d10, d100
        );
    }

    #[test]
    fn test_erosion_rate() {
        let params = EnvironmentParams::default();
        let calc = LateralExpansionCalculator::new(params);

        // Небольшой угол наклона (10°)
        let slope = 10.0_f64.to_radians();

        let rate_land = calc.calculate_erosion_rate(slope, false);
        let rate_water = calc.calculate_erosion_rate(slope, true);

        // Вода должна ускорять эрозию
        assert!(rate_water > rate_land);

        println!("Erosion rate (land): {:.6} m/year", rate_land);
        println!("Erosion rate (water): {:.6} m/year", rate_water);
    }

    #[test]
    fn test_erosion_steep_slope() {
        let params = EnvironmentParams::default();
        let calc = LateralExpansionCalculator::new(params);

        // Крутой угол (60°)
        let steep_slope = 60.0_f64.to_radians();

        let rate = calc.calculate_erosion_rate(steep_slope, true);

        // Для очень крутых склонов модель возвращает 0
        assert_eq!(rate, 0.0);
    }

    #[test]
    fn test_depth_effect_on_diameter() {
        let params = EnvironmentParams::default();
        let calc = LateralExpansionCalculator::new(params);

        let d_shallow = calc.calculate_diameter_diffusion(1.0, 10, 1.0);
        let d_deep = calc.calculate_diameter_diffusion(5.0, 10, 1.0);

        // Более глубокие образования расширяются больше
        assert!(d_deep > d_shallow);

        println!("Shallow (1m): {:.2}m, Deep (5m): {:.2}m", d_shallow, d_deep);
    }
}
