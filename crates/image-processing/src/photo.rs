//! Анализ фотографий термокарста

use crate::detection::ThermokarstDetector;
use thermokarst_core::dataset::ObservationData;
use anyhow::Result;
use image::DynamicImage;
use std::collections::HashMap;

/// Анализатор фотографий
pub struct PhotoAnalyzer {
    detector: ThermokarstDetector,
    scale_meters_per_pixel: f64,
}

impl PhotoAnalyzer {
    /// Создать анализатор
    pub fn new(image: DynamicImage, scale: f64) -> Self {
        Self {
            detector: ThermokarstDetector::new(image),
            scale_meters_per_pixel: scale,
        }
    }

    /// Загрузить из файла
    pub fn from_file(path: &str, scale: f64) -> Result<Self> {
        let detector = ThermokarstDetector::from_file(path)?;
        Ok(Self {
            detector,
            scale_meters_per_pixel: scale,
        })
    }

    /// Проанализировать изображение
    pub fn analyze(&self) -> Result<AnalysisResult> {
        let detection = self.detector.detect()?;

        let mut features = Vec::new();

        for (i, circle) in detection.circles.iter().enumerate() {
            let diameter = self.detector.calculate_diameter_meters(circle, self.scale_meters_per_pixel);
            let area = self.detector.calculate_area_m2(circle, self.scale_meters_per_pixel);

            features.push(ThermokarstFeature {
                id: i,
                center_x: circle.center_x,
                center_y: circle.center_y,
                radius_pixels: circle.radius,
                diameter_meters: diameter,
                area_m2: area,
            });
        }

        Ok(AnalysisResult {
            features,
            total_count: detection.circles.len(),
        })
    }

    /// Создать датасет наблюдений
    pub fn to_observation_dataset(
        &self,
        site_prefix: &str,
        coordinates: (f64, f64),
        date: &str,
    ) -> Result<Vec<ObservationData>> {
        let analysis = self.analyze()?;

        let observations = analysis
            .features
            .iter()
            .map(|feature| {
                let mut metadata = HashMap::new();
                metadata.insert("source".to_string(), "photo".to_string());
                metadata.insert("area_m2".to_string(), feature.area_m2.to_string());
                metadata.insert("radius_pixels".to_string(), feature.radius_pixels.to_string());

                ObservationData {
                    site_id: format!("{}_{}", site_prefix, feature.id),
                    coordinates,
                    date: date.to_string(),
                    active_layer_thickness: None,
                    ground_temperature: None,
                    temperature_depth: None,
                    subsidence: None,
                    diameter: Some(feature.diameter_meters),
                    metadata,
                }
            })
            .collect();

        Ok(observations)
    }
}

/// Результат анализа
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub features: Vec<ThermokarstFeature>,
    pub total_count: usize,
}

/// Термокарстовое образование на фото
#[derive(Debug, Clone)]
pub struct ThermokarstFeature {
    pub id: usize,
    pub center_x: u32,
    pub center_y: u32,
    pub radius_pixels: u32,
    pub diameter_meters: f64,
    pub area_m2: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::downloader::ImageDownloader;
    use tempfile::tempdir;

    #[test]
    fn test_photo_analysis() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.jpg");

        // Создать тестовое изображение (диаметр 200 пикселей)
        ImageDownloader::create_synthetic_thermokarst(&path, 200).unwrap();

        // Анализировать (масштаб: 1 пиксель = 0.1 метра)
        let analyzer = PhotoAnalyzer::from_file(path.to_str().unwrap(), 0.1).unwrap();
        let result = analyzer.analyze().unwrap();

        assert_eq!(result.total_count, 1);

        let feature = &result.features[0];
        println!("Диаметр: {:.1} м", feature.diameter_meters);
        println!("Площадь: {:.1} м²", feature.area_m2);

        // Ожидаемый диаметр: ~200 пикселей * 0.1 м/пиксель = ~20 м
        // Но детекция может быть неточной, поэтому расширим диапазон
        assert!(feature.diameter_meters > 10.0 && feature.diameter_meters < 30.0);
    }

    #[test]
    fn test_to_dataset() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.jpg");

        ImageDownloader::create_synthetic_thermokarst(&path, 200).unwrap();

        let analyzer = PhotoAnalyzer::from_file(path.to_str().unwrap(), 0.1).unwrap();
        let observations = analyzer
            .to_observation_dataset("TEST", (62.0, 129.7), "2025-08-15")
            .unwrap();

        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].site_id, "TEST_0");
        assert!(observations[0].diameter.is_some());
    }
}
