//! Модуль для работы с реальными данными наблюдений

use crate::{Result, ThermokarstError};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Формат данных наблюдений
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationData {
    /// Идентификатор точки наблюдения
    pub site_id: String,

    /// Координаты (широта, долгота)
    pub coordinates: (f64, f64),

    /// Дата наблюдения (ISO 8601)
    pub date: String,

    /// Глубина активного слоя (м)
    pub active_layer_thickness: Option<f64>,

    /// Температура грунта на глубине (°C)
    pub ground_temperature: Option<f64>,

    /// Глубина измерения температуры (м)
    pub temperature_depth: Option<f64>,

    /// Просадка поверхности (м)
    pub subsidence: Option<f64>,

    /// Диаметр термокарстового образования (м)
    pub diameter: Option<f64>,

    /// Дополнительные метаданные
    pub metadata: std::collections::HashMap<String, String>,
}

/// Набор данных наблюдений
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationDataset {
    /// Название датасета
    pub name: String,

    /// Описание
    pub description: String,

    /// Источник данных
    pub source: String,

    /// Наблюдения
    pub observations: Vec<ObservationData>,
}

impl ObservationDataset {
    /// Загрузить датасет из JSON файла
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| ThermokarstError::IoError(e))?;

        let dataset: Self =
            serde_json::from_str(&content).map_err(|e| ThermokarstError::SerializationError(e))?;

        Ok(dataset)
    }

    /// Сохранить датасет в JSON файл
    pub fn to_json_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| ThermokarstError::SerializationError(e))?;

        std::fs::write(path, json).map_err(|e| ThermokarstError::IoError(e))?;

        Ok(())
    }

    /// Фильтровать наблюдения по региону (bbox)
    pub fn filter_by_bbox(&self, min_lat: f64, max_lat: f64, min_lon: f64, max_lon: f64) -> Self {
        let filtered: Vec<ObservationData> = self
            .observations
            .iter()
            .filter(|obs| {
                let (lat, lon) = obs.coordinates;
                lat >= min_lat && lat <= max_lat && lon >= min_lon && lon <= max_lon
            })
            .cloned()
            .collect();

        Self {
            name: format!("{} (filtered)", self.name),
            description: self.description.clone(),
            source: self.source.clone(),
            observations: filtered,
        }
    }

    /// Получить статистику по датасету
    pub fn statistics(&self) -> DatasetStatistics {
        let mut stats = DatasetStatistics::default();

        stats.total_observations = self.observations.len();

        let mut alt_values = Vec::new();
        let mut temp_values = Vec::new();
        let mut subsidence_values = Vec::new();

        for obs in &self.observations {
            if let Some(alt) = obs.active_layer_thickness {
                alt_values.push(alt);
            }
            if let Some(temp) = obs.ground_temperature {
                temp_values.push(temp);
            }
            if let Some(sub) = obs.subsidence {
                subsidence_values.push(sub);
            }
        }

        stats.active_layer_stats = calculate_stats(&alt_values);
        stats.temperature_stats = calculate_stats(&temp_values);
        stats.subsidence_stats = calculate_stats(&subsidence_values);

        stats
    }
}

/// Статистика по датасету
#[derive(Debug, Clone, Default)]
pub struct DatasetStatistics {
    pub total_observations: usize,
    pub active_layer_stats: Option<ValueStatistics>,
    pub temperature_stats: Option<ValueStatistics>,
    pub subsidence_stats: Option<ValueStatistics>,
}

/// Статистика по значениям
#[derive(Debug, Clone)]
pub struct ValueStatistics {
    pub count: usize,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub std_dev: f64,
}

fn calculate_stats(values: &[f64]) -> Option<ValueStatistics> {
    if values.is_empty() {
        return None;
    }

    let count = values.len();
    let sum: f64 = values.iter().sum();
    let mean = sum / count as f64;

    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count as f64;
    let std_dev = variance.sqrt();

    Some(ValueStatistics {
        count,
        mean,
        min,
        max,
        std_dev,
    })
}

/// Создать пример датасета для Якутии
pub fn create_example_yakutia_dataset() -> ObservationDataset {
    ObservationDataset {
        name: "Yakutia Permafrost Observations Example".to_string(),
        description: "Example dataset with typical values for Yakutia region".to_string(),
        source: "Synthetic data based on literature".to_string(),
        observations: vec![
            ObservationData {
                site_id: "YAK-001".to_string(),
                coordinates: (62.0, 129.7), // Якутск
                date: "2025-08-15".to_string(),
                active_layer_thickness: Some(1.2),
                ground_temperature: Some(-1.5),
                temperature_depth: Some(10.0),
                subsidence: Some(0.15),
                diameter: Some(5.2),
                metadata: [
                    ("soil_type".to_string(), "loam".to_string()),
                    ("vegetation".to_string(), "sparse".to_string()),
                ]
                .iter()
                .cloned()
                .collect(),
            },
            ObservationData {
                site_id: "YAK-002".to_string(),
                coordinates: (70.5, 135.2), // Северная Якутия
                date: "2025-07-20".to_string(),
                active_layer_thickness: Some(0.8),
                ground_temperature: Some(-4.2),
                temperature_depth: Some(10.0),
                subsidence: Some(0.08),
                diameter: Some(3.1),
                metadata: [
                    ("soil_type".to_string(), "peat".to_string()),
                    ("vegetation".to_string(), "dense".to_string()),
                ]
                .iter()
                .cloned()
                .collect(),
            },
            ObservationData {
                site_id: "YAK-003".to_string(),
                coordinates: (58.5, 125.0), // Южная Якутия
                date: "2025-08-01".to_string(),
                active_layer_thickness: Some(1.8),
                ground_temperature: Some(-0.8),
                temperature_depth: Some(10.0),
                subsidence: Some(0.25),
                diameter: Some(8.5),
                metadata: [
                    ("soil_type".to_string(), "loam".to_string()),
                    ("vegetation".to_string(), "moderate".to_string()),
                ]
                .iter()
                .cloned()
                .collect(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_example_dataset() {
        let dataset = create_example_yakutia_dataset();
        assert_eq!(dataset.observations.len(), 3);
    }

    #[test]
    fn test_dataset_statistics() {
        let dataset = create_example_yakutia_dataset();
        let stats = dataset.statistics();

        assert_eq!(stats.total_observations, 3);
        assert!(stats.active_layer_stats.is_some());
    }

    #[test]
    fn test_filter_by_bbox() {
        let dataset = create_example_yakutia_dataset();

        // Фильтр для северной Якутии
        let filtered = dataset.filter_by_bbox(65.0, 75.0, 130.0, 140.0);

        assert_eq!(filtered.observations.len(), 1);
        assert_eq!(filtered.observations[0].site_id, "YAK-002");
    }
}
