//! Основные типы данных

use serde::{Deserialize, Serialize};

/// Тип грунта
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SoilType {
    /// Глина
    Clay,
    /// Песок
    Sand,
    /// Торф
    Peat,
    /// Суглинок
    Loam,
    /// Ил
    Silt,
}

impl SoilType {
    /// Коэффициент теплопроводности (Вт/(м·К))
    pub fn thermal_conductivity(&self) -> f64 {
        match self {
            Self::Clay => 1.2,
            Self::Sand => 2.0,
            Self::Peat => 0.5,
            Self::Loam => 1.5,
            Self::Silt => 1.8,
        }
    }

    /// Пористость грунта (0-1)
    pub fn porosity(&self) -> f64 {
        match self {
            Self::Clay => 0.45,
            Self::Sand => 0.35,
            Self::Peat => 0.80,
            Self::Loam => 0.50,
            Self::Silt => 0.55,
        }
    }

    /// Коэффициент сжимаемости при оттаивании
    pub fn compression_coefficient(&self) -> f64 {
        match self {
            Self::Clay => 0.15,
            Self::Sand => 0.05,
            Self::Peat => 0.40,
            Self::Loam => 0.20,
            Self::Silt => 0.25,
        }
    }
}

/// Параметры окружающей среды
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentParams {
    /// Средняя летняя температура воздуха (°C)
    pub air_temp: f64,

    /// Температура многолетнемерзлых пород (°C)
    pub permafrost_temp: f64,

    /// Льдистость грунта (0-1)
    pub ice_content: f64,

    /// Тип грунта
    pub soil_type: SoilType,

    /// Покрытие растительностью (0-1)
    pub vegetation_cover: f64,

    /// Доступность воды (0-1)
    pub water_availability: f64,

    /// Глубина залегания мерзлоты (м)
    pub permafrost_depth: f64,

    /// Продолжительность теплого сезона (дни)
    pub warm_season_days: u32,
}

impl Default for EnvironmentParams {
    fn default() -> Self {
        // Типичные условия для Центральной Якутии
        Self {
            air_temp: 5.0,
            permafrost_temp: -2.0,
            ice_content: 0.7,
            soil_type: SoilType::Loam,
            vegetation_cover: 0.4,
            water_availability: 0.6,
            permafrost_depth: 1.5,
            warm_season_days: 120,
        }
    }
}

impl EnvironmentParams {
    /// Создать параметры для северной Якутии
    pub fn northern_yakutia() -> Self {
        Self {
            air_temp: 3.0,
            permafrost_temp: -5.0,
            ice_content: 0.85,
            soil_type: SoilType::Peat,
            vegetation_cover: 0.6,
            water_availability: 0.8,
            permafrost_depth: 0.8,
            warm_season_days: 90,
        }
    }

    /// Создать параметры для центральной Якутии
    pub fn central_yakutia() -> Self {
        Self::default()
    }

    /// Создать параметры для южной Якутии
    pub fn southern_yakutia() -> Self {
        Self {
            air_temp: 7.0,
            permafrost_temp: -1.0,
            ice_content: 0.5,
            soil_type: SoilType::Loam,
            vegetation_cover: 0.3,
            water_availability: 0.5,
            permafrost_depth: 2.0,
            warm_season_days: 140,
        }
    }
}

/// Термокарстовая линза
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermokarstLens {
    /// Глубина просадки (м)
    pub depth: f64,

    /// Диаметр (м)
    pub diameter: f64,

    /// Объем (м³)
    pub volume: f64,

    /// Возраст (годы)
    pub age: u32,

    /// Скорость роста объема (м³/год)
    pub growth_rate: f64,

    /// Площадь поверхности (м²)
    pub surface_area: f64,
}

impl ThermokarstLens {
    /// Создать новую линзу
    pub fn new(depth: f64, diameter: f64, age: u32) -> Self {
        let volume = Self::calculate_volume(depth, diameter);
        let surface_area = Self::calculate_surface_area(diameter);

        Self {
            depth,
            diameter,
            volume,
            age,
            growth_rate: 0.0,
            surface_area,
        }
    }

    /// Рассчитать объем (приближение цилиндром)
    fn calculate_volume(depth: f64, diameter: f64) -> f64 {
        let radius = diameter / 2.0;
        std::f64::consts::PI * radius * radius * depth
    }

    /// Рассчитать площадь поверхности
    fn calculate_surface_area(diameter: f64) -> f64 {
        let radius = diameter / 2.0;
        std::f64::consts::PI * radius * radius
    }

    /// Обновить параметры линзы
    pub fn update(&mut self, new_depth: f64, new_diameter: f64) {
        let old_volume = self.volume;

        self.depth = new_depth;
        self.diameter = new_diameter;
        self.volume = Self::calculate_volume(new_depth, new_diameter);
        self.surface_area = Self::calculate_surface_area(new_diameter);
        self.growth_rate = self.volume - old_volume;
    }

    /// Соотношение глубина/диаметр
    pub fn aspect_ratio(&self) -> f64 {
        if self.diameter > 0.0 {
            self.depth / self.diameter
        } else {
            0.0
        }
    }

    /// Проверка стабильности
    pub fn is_stable(&self) -> bool {
        const MAX_DEPTH: f64 = 15.0;
        const MAX_DIAMETER: f64 = 100.0;
        const MIN_ASPECT_RATIO: f64 = 0.05;
        const MAX_ASPECT_RATIO: f64 = 0.6;

        let aspect = self.aspect_ratio();

        self.depth < MAX_DEPTH
            && self.diameter < MAX_DIAMETER
            && aspect > MIN_ASPECT_RATIO
            && aspect < MAX_ASPECT_RATIO
    }
}

/// Стадия развития термокарста
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThermokarstStage {
    /// Инициация - начальное нарушение
    Initiation,
    /// Активное развитие
    ActiveDevelopment,
    /// Стабилизация
    Stabilization,
    /// Деградация
    Degradation,
}

/// Результат симуляции
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    /// История развития линзы
    pub lenses: Vec<ThermokarstLens>,

    /// Параметры среды
    pub environment: EnvironmentParams,

    /// Стадия развития
    pub stage: ThermokarstStage,

    /// Общее время симуляции (годы)
    pub total_years: u32,
}
