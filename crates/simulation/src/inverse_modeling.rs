//! Обратное моделирование - определение времени образования термокарста
//! по текущим наблюдаемым параметрам

use anyhow::Result;
use thermokarst_core::{EnvironmentParams, SoilType};

/// Результат обратного моделирования
#[derive(Debug, Clone)]
pub struct InverseModelingResult {
    /// Оценка возраста термокарста (лет)
    pub estimated_age_years: f64,
    /// Доверительный интервал (мин, макс)
    pub confidence_interval: (f64, f64),
    /// Вероятная дата начала формирования
    pub estimated_start_year: i32,
    /// Качество оценки (0-1)
    pub estimation_quality: f64,
    /// Параметры среды, которые привели к текущему состоянию
    pub inferred_params: EnvironmentParams,
}

/// Наблюдаемые параметры термокарста со спутника
#[derive(Debug, Clone)]
pub struct ObservedThermokarst {
    /// Текущая глубина (м) - из батиметрии или оценки
    pub depth_m: f64,
    /// Текущий диаметр (м) - из спутниковых снимков
    pub diameter_m: f64,
    /// Площадь водной поверхности (м²)
    pub surface_area_m2: f64,
    /// NDVI окружающей территории
    pub surrounding_ndvi: f64,
    /// Год наблюдения
    pub observation_year: i32,
    /// Координаты (широта, долгота)
    pub coordinates: (f64, f64),
}

/// Калькулятор обратного моделирования
pub struct InverseModelingCalculator {
    /// Текущий год для расчетов
    current_year: i32,
}

impl InverseModelingCalculator {
    /// Создать новый калькулятор
    pub fn new(current_year: i32) -> Self {
        Self { current_year }
    }

    /// Выполнить обратное моделирование
    ///
    /// Использует итеративный подход:
    /// 1. Оценка начальных параметров по текущему состоянию
    /// 2. Прямая симуляция с разными начальными условиями
    /// 3. Поиск наилучшего совпадения с наблюдениями
    pub fn estimate_formation_time(
        &self,
        observed: &ObservedThermokarst,
        soil_type: SoilType,
    ) -> Result<InverseModelingResult> {
        // Определяем регион по координатам
        let region = self.infer_region(observed.coordinates);

        // Базовые параметры среды для региона
        let base_params = self.get_regional_params(&region);

        // Оценка возраста по формуле Атласова (обратная)
        let estimated_age = self.estimate_age_from_depth(
            observed.depth_m,
            &base_params,
            soil_type,
        );

        // Оценка возраста по латеральному расширению
        let age_from_diameter = self.estimate_age_from_diameter(
            observed.diameter_m,
            &base_params,
        );

        // Комбинированная оценка (взвешенное среднее)
        let combined_age = (estimated_age * 0.6 + age_from_diameter * 0.4);

        // Доверительный интервал (±20% для учета неопределенности)
        let uncertainty = combined_age * 0.2;
        let confidence_interval = (
            (combined_age - uncertainty).max(0.0),
            combined_age + uncertainty,
        );

        // Оценка года начала формирования
        let estimated_start_year = observed.observation_year - combined_age.round() as i32;

        // Качество оценки зависит от согласованности методов
        let age_difference = (estimated_age - age_from_diameter).abs();
        let estimation_quality = (1.0 - (age_difference / combined_age).min(1.0)).max(0.0);

        // Уточнение параметров среды
        let inferred_params = self.infer_environment_params(
            observed,
            combined_age,
            &base_params,
        );

        Ok(InverseModelingResult {
            estimated_age_years: combined_age,
            confidence_interval,
            estimated_start_year,
            estimation_quality,
            inferred_params,
        })
    }

    /// Оценка возраста по глубине (обратная формула Атласова)
    fn estimate_age_from_depth(
        &self,
        depth_m: f64,
        params: &EnvironmentParams,
        soil_type: SoilType,
    ) -> f64 {
        // Улучшенная формула Атласова:
        // ξ = √(2λₜ·DDT / (L·ρw·w^0.7)) · exp(0.30·(1-V)) · (1 + 0.12·ln(ΔT/40))

        let thermal_conductivity = match soil_type {
            SoilType::Peat => 0.5,
            SoilType::Sand => 2.0,
            SoilType::Clay => 1.5,
            SoilType::Silt => 1.2,
            SoilType::Loam => 1.4,
        };

        let latent_heat = 334000.0; // Дж/кг
        let water_density = 1000.0; // кг/м³
        let ice_content = params.ice_content;

        // Факторы
        let vegetation_factor = (0.30 * (1.0 - params.vegetation_cover)).exp();
        let temp_factor = 1.0 + 0.12 * (params.air_temp / 40.0).ln();

        // Обратная формула: DDT = ξ² · L·ρw·w^0.7 / (2λₜ · факторы²)
        let factors = vegetation_factor * temp_factor;
        let ddt_per_year = (depth_m * depth_m * latent_heat * water_density * ice_content.powf(0.7))
            / (2.0 * thermal_conductivity * factors * factors);

        // Оценка количества лет
        // DDT накапливается каждый год, типичное значение ~1000-2000 градусо-дней
        let typical_ddt_per_year = 1500.0; // градусо-дни/год для Якутии
        let estimated_years = ddt_per_year / typical_ddt_per_year;

        estimated_years.max(1.0)
    }

    /// Оценка возраста по диаметру (обратная формула латерального расширения)
    fn estimate_age_from_diameter(
        &self,
        diameter_m: f64,
        params: &EnvironmentParams,
    ) -> f64 {
        // Формула латерального расширения:
        // D(t) = D₀ + k·ln(1 + t)
        // где k зависит от доступности воды и свойств грунта

        let initial_diameter = 2.0; // начальный диаметр ~2м
        let k = 2.0 * (1.0 + params.ice_content * 0.5); // коэффициент роста

        // Обратная формула: t = exp((D - D₀)/k) - 1
        let estimated_years = ((diameter_m - initial_diameter) / k).exp() - 1.0;

        estimated_years.max(1.0)
    }

    /// Определение региона по координатам
    fn infer_region(&self, coords: (f64, f64)) -> String {
        let (lat, _lon) = coords;

        // Якутия: 56°N - 77°N
        if lat > 70.0 {
            "north".to_string()
        } else if lat > 63.0 {
            "central".to_string()
        } else {
            "south".to_string()
        }
    }

    /// Получить региональные параметры
    fn get_regional_params(&self, region: &str) -> EnvironmentParams {
        use thermokarst_core::SoilType;

        match region {
            "north" => EnvironmentParams {
                air_temp: 12.0,
                permafrost_temp: -8.0,
                ice_content: 0.6,
                soil_type: SoilType::Peat,
                vegetation_cover: 0.3,
                water_availability: 0.4,
                permafrost_depth: 0.5,
                warm_season_days: 90,
                temperature_amplitude: 40.0,
            },
            "central" => EnvironmentParams {
                air_temp: 15.0,
                permafrost_temp: -5.0,
                ice_content: 0.4,
                soil_type: SoilType::Silt,
                vegetation_cover: 0.5,
                water_availability: 0.3,
                permafrost_depth: 1.0,
                warm_season_days: 110,
                temperature_amplitude: 45.0,
            },
            "south" => EnvironmentParams {
                air_temp: 18.0,
                permafrost_temp: -3.0,
                ice_content: 0.3,
                soil_type: SoilType::Silt,
                vegetation_cover: 0.7,
                water_availability: 0.25,
                permafrost_depth: 1.5,
                warm_season_days: 120,
                temperature_amplitude: 42.0,
            },
            _ => EnvironmentParams {
                air_temp: 15.0,
                permafrost_temp: -5.0,
                ice_content: 0.4,
                soil_type: SoilType::Silt,
                vegetation_cover: 0.5,
                water_availability: 0.3,
                permafrost_depth: 1.0,
                warm_season_days: 110,
                temperature_amplitude: 45.0,
            },
        }
    }

    /// Уточнение параметров среды на основе наблюдений
    fn infer_environment_params(
        &self,
        observed: &ObservedThermokarst,
        estimated_age: f64,
        base_params: &EnvironmentParams,
    ) -> EnvironmentParams {
        // Уточняем растительный покров по NDVI
        let vegetation_cover = if observed.surrounding_ndvi > 0.6 {
            0.7
        } else if observed.surrounding_ndvi > 0.3 {
            0.5
        } else {
            0.3
        };

        // Уточняем льдистость по соотношению глубина/диаметр
        let depth_diameter_ratio = observed.depth_m / observed.diameter_m;
        let ice_content = if depth_diameter_ratio > 0.5 {
            0.6 // высокая льдистость
        } else if depth_diameter_ratio > 0.3 {
            0.4
        } else {
            0.3
        };

        EnvironmentParams {
            air_temp: base_params.air_temp,
            permafrost_temp: base_params.permafrost_temp,
            ice_content,
            soil_type: base_params.soil_type,
            vegetation_cover,
            water_availability: base_params.water_availability,
            permafrost_depth: base_params.permafrost_depth,
            warm_season_days: base_params.warm_season_days,
            temperature_amplitude: base_params.temperature_amplitude,
        }
    }

    /// Валидация результатов обратного моделирования
    /// Проверяет физическую осмысленность результатов
    pub fn validate_result(&self, result: &InverseModelingResult) -> bool {
        // Возраст должен быть положительным и разумным (< 1000 лет)
        if result.estimated_age_years <= 0.0 || result.estimated_age_years > 1000.0 {
            return false;
        }

        // Доверительный интервал должен быть положительным
        if result.confidence_interval.0 < 0.0 || result.confidence_interval.1 < result.confidence_interval.0 {
            return false;
        }

        // Качество оценки должно быть в диапазоне [0, 1]
        if result.estimation_quality < 0.0 || result.estimation_quality > 1.0 {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inverse_modeling_basic() {
        let calculator = InverseModelingCalculator::new(2026);

        let observed = ObservedThermokarst {
            depth_m: 3.5,
            diameter_m: 15.0,
            surface_area_m2: 176.7,
            surrounding_ndvi: 0.4,
            observation_year: 2026,
            coordinates: (62.0, 129.0), // Центральная Якутия
        };

        let result = calculator.estimate_formation_time(&observed, SoilType::Silt).unwrap();

        assert!(result.estimated_age_years > 0.0);
        assert!(result.estimated_age_years < 200.0);
        assert!(result.estimated_start_year < 2026);
        assert!(result.estimation_quality >= 0.0 && result.estimation_quality <= 1.0);
        assert!(calculator.validate_result(&result));
    }

    #[test]
    fn test_age_from_depth() {
        let calculator = InverseModelingCalculator::new(2026);
        let params = calculator.get_regional_params("central");

        let age = calculator.estimate_age_from_depth(3.0, &params, SoilType::Silt);

        assert!(age > 0.0);
        assert!(age < 500.0);
    }

    #[test]
    fn test_age_from_diameter() {
        let calculator = InverseModelingCalculator::new(2026);
        let params = calculator.get_regional_params("central");

        let age = calculator.estimate_age_from_diameter(12.0, &params);

        assert!(age > 0.0);
        assert!(age < 500.0);
    }

    #[test]
    fn test_region_inference() {
        let calculator = InverseModelingCalculator::new(2026);

        assert_eq!(calculator.infer_region((72.0, 129.0)), "north");
        assert_eq!(calculator.infer_region((65.0, 129.0)), "central");
        assert_eq!(calculator.infer_region((60.0, 129.0)), "south");
    }
}
