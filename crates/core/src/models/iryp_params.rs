//! Преобразование IRYP данных в параметры симуляции

use crate::iryp::IrypSite;
use crate::types::{EnvironmentParams, SoilType};

/// Оценить параметры окружающей среды на основе IRYP точки
///
/// Использует НЕПРЕРЫВНЫЕ функции широты/долготы для точной оценки
pub fn estimate_params_from_site(site: &IrypSite) -> EnvironmentParams {
    let mut params = EnvironmentParams::default();

    let lat = site.latitude;
    let lon = site.longitude;

    // 1. Температура воздуха - ЛИНЕЙНАЯ функция широты
    // От +8°C на юге (56°N) до +2°C на севере (74°N)
    params.air_temp = 8.0 - (lat - 56.0) * 0.33;

    // 2. Температура мерзлоты - зависит от широты
    params.permafrost_temp = -1.0 - (lat - 56.0) * 0.22;

    // 3. Льдистость - НЕПРЕРЫВНАЯ функция широты
    // Едома: от 0.60 на юге до 0.80 на севере
    params.ice_content = 0.60 + (lat - 56.0) * 0.011;
    params.ice_content = params.ice_content.clamp(0.50, 0.85);

    // 4. Растительный покров - зависит от широты и континентальности
    // От 0.8 (густая тайга на юге) до 0.2 (тундра на севере)
    let base_veg = 0.85 - (lat - 56.0) * 0.036;
    // Корректировка на континентальность (восток суше)
    let continentality_factor = (lon - 105.0) / 57.0; // 0..1 для 105-162°E
    params.vegetation_cover = base_veg * (1.0 - continentality_factor * 0.15);
    params.vegetation_cover = params.vegetation_cover.clamp(0.2, 0.9);

    // 5. Тип грунта - зависит от широты и местности
    params.soil_type = if lat > 70.0 {
        SoilType::Silt // Северная едома
    } else if lat > 65.0 {
        SoilType::Peat // Лесотундра
    } else {
        SoilType::Loam // Центральная едома
    };

    // 6. Глубина мерзлоты - НЕПРЕРЫВНАЯ функция
    // От 2.5 м на юге до 0.5 м на севере
    params.permafrost_depth = 2.8 - (lat - 56.0) * 0.128;
    params.permafrost_depth = params.permafrost_depth.clamp(0.5, 3.0);

    // 7. Продолжительность теплого сезона - НЕПРЕРЫВНАЯ
    // От 150 дней на юге до 70 дней на севере
    let warm_days_f64 = 155.0 - (lat - 56.0) * 4.7;
    params.warm_season_days = warm_days_f64.round() as u32;
    params.warm_season_days = params.warm_season_days.clamp(70, 150);

    // 8. Годовая амплитуда температур - НЕПРЕРЫВНАЯ функция широты и долготы
    // Зависит от континентальности: чем восточнее и севернее, тем больше
    let lat_factor = 70.0 + (lat - 56.0) * 1.5; // 70..97°C от широты
    let lon_factor = (lon - 105.0) / 57.0; // 0..1 от долготы
    params.temperature_amplitude = lat_factor + lon_factor * 20.0; // +20°C от континентальности
    params.temperature_amplitude = params.temperature_amplitude.clamp(65.0, 115.0); // Расширенный диапазон

    // 9. Доступность воды - зависит от типа местности
    let area_lower = site.area.to_lowercase();
    params.soil_saturation_ratio = if area_lower.contains("delta") {
        0.9 // Дельты - очень высокая
    } else if area_lower.contains("river") {
        0.8 // Реки - высокая
    } else if area_lower.contains("alas") {
        0.85 // Аласы - высокая (термокарстовые озера)
    } else if area_lower.contains("lake") {
        0.8 // Озера
    } else {
        // Базовая доступность зависит от широты (север влажнее)
        0.45 + (lat - 56.0) * 0.019
    };
    params.soil_saturation_ratio = params.soil_saturation_ratio.clamp(0.3, 0.95);

    params
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_continuous_parameters() {
        // Три точки с разной широтой должны иметь РАЗНЫЕ параметры
        let site1 = IrypSite {
            event: "South".to_string(),
            file_type: "Test".to_string(),
            area: "Test".to_string(),
            latitude: 60.0,
            longitude: 130.0,
            comment: "Test".to_string(),
            date: None,
            id: None,
            investigator: None,
        };

        let site2 = IrypSite {
            event: "Central".to_string(),
            file_type: "Test".to_string(),
            area: "Test".to_string(),
            latitude: 65.0,
            longitude: 130.0,
            comment: "Test".to_string(),
            date: None,
            id: None,
            investigator: None,
        };

        let site3 = IrypSite {
            event: "North".to_string(),
            file_type: "Test".to_string(),
            area: "Test".to_string(),
            latitude: 72.0,
            longitude: 130.0,
            comment: "Test".to_string(),
            date: None,
            id: None,
            investigator: None,
        };

        let params1 = estimate_params_from_site(&site1);
        let params2 = estimate_params_from_site(&site2);
        let params3 = estimate_params_from_site(&site3);

        // Температура должна УМЕНЬШАТЬСЯ с широтой
        assert!(params1.air_temp > params2.air_temp);
        assert!(params2.air_temp > params3.air_temp);

        // Льдистость должна УВЕЛИЧИВАТЬСЯ с широтой
        assert!(params1.ice_content < params2.ice_content);
        assert!(params2.ice_content < params3.ice_content);

        // Все параметры должны быть РАЗНЫМИ
        assert_ne!(params1.air_temp, params2.air_temp);
        assert_ne!(params2.air_temp, params3.air_temp);
    }

    #[test]
    fn test_water_availability_by_area() {
        let mut site = IrypSite {
            event: "Test".to_string(),
            file_type: "Test".to_string(),
            area: "Lena Delta".to_string(),
            latitude: 72.0,
            longitude: 126.0,
            comment: "Test".to_string(),
            date: None,
            id: None,
            investigator: None,
        };

        let params_delta = estimate_params_from_site(&site);
        assert!(params_delta.soil_saturation_ratio > 0.85);

        site.area = "Alas".to_string();
        let params_alas = estimate_params_from_site(&site);
        assert!(params_alas.soil_saturation_ratio > 0.8);

        site.area = "Upland".to_string();
        let params_upland = estimate_params_from_site(&site);
        assert!(params_upland.soil_saturation_ratio < params_delta.soil_saturation_ratio);
    }
}
