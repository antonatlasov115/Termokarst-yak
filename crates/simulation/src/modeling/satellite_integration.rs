//! Интеграция со спутниковыми данными
//! Поддержка Sentinel-2, Landsat-8/9, Planet Labs

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Источник спутниковых данных
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SatelliteSource {
    /// Sentinel-2 (ESA) - 10м разрешение
    Sentinel2,
    /// Landsat 8/9 (NASA/USGS) - 30м разрешение
    Landsat,
    /// Planet Labs - до 3м разрешение
    Planet,
}

/// Спутниковое изображение
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatelliteImage {
    /// Источник данных
    pub source: SatelliteSource,
    /// Дата съемки
    pub acquisition_date: String,
    /// Координаты центра (широта, долгота)
    pub center_coords: (f64, f64),
    /// Разрешение (метров на пиксель)
    pub resolution_m: f64,
    /// Облачность (0-100%)
    pub cloud_cover_percent: f64,
    /// URL для скачивания (если доступно)
    pub download_url: Option<String>,
}

/// Детектированный термокарст
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedThermokarst {
    /// Координаты центра
    pub center_coords: (f64, f64),
    /// Диаметр (м)
    pub diameter_m: f64,
    /// Площадь (м²)
    pub area_m2: f64,
    /// NDWI (индекс воды)
    pub ndwi: f64,
    /// NDVI окружения
    pub surrounding_ndvi: f64,
    /// Уверенность детекции (0-1)
    pub confidence: f64,
}

/// API клиент для спутниковых данных
pub struct SatelliteDataClient {
    /// API ключ (если требуется)
    api_key: Option<String>,
}

impl SatelliteDataClient {
    /// Создать новый клиент
    pub fn new(api_key: Option<String>) -> Self {
        Self { api_key }
    }

    /// Поиск доступных снимков для области
    ///
    /// # Аргументы
    /// * `bbox` - Bounding box (min_lat, min_lon, max_lat, max_lon)
    /// * `start_date` - Начальная дата (YYYY-MM-DD)
    /// * `end_date` - Конечная дата (YYYY-MM-DD)
    /// * `max_cloud_cover` - Максимальная облачность (0-100)
    pub async fn search_images(
        &self,
        bbox: (f64, f64, f64, f64),
        start_date: &str,
        end_date: &str,
        max_cloud_cover: f64,
    ) -> Result<Vec<SatelliteImage>> {
        // TODO: Реальная интеграция с API
        // Sentinel Hub API: https://www.sentinel-hub.com/
        // USGS Earth Explorer: https://earthexplorer.usgs.gov/
        // Planet API: https://www.planet.com/

        println!("🛰️  Поиск спутниковых снимков...");
        println!("   Область: {:?}", bbox);
        println!("   Период: {} - {}", start_date, end_date);
        println!("   Макс. облачность: {}%", max_cloud_cover);

        // Заглушка - возвращаем примеры
        Ok(vec![
            SatelliteImage {
                source: SatelliteSource::Sentinel2,
                acquisition_date: "2025-07-15".to_string(),
                center_coords: ((bbox.0 + bbox.2) / 2.0, (bbox.1 + bbox.3) / 2.0),
                resolution_m: 10.0,
                cloud_cover_percent: 5.0,
                download_url: Some("https://scihub.copernicus.eu/...".to_string()),
            },
            SatelliteImage {
                source: SatelliteSource::Landsat,
                acquisition_date: "2025-07-20".to_string(),
                center_coords: ((bbox.0 + bbox.2) / 2.0, (bbox.1 + bbox.3) / 2.0),
                resolution_m: 30.0,
                cloud_cover_percent: 12.0,
                download_url: Some("https://earthexplorer.usgs.gov/...".to_string()),
            },
        ])
    }

    /// Детекция термокарстовых озер на снимке
    pub fn detect_thermokarts(
        &self,
        image: &SatelliteImage,
        min_diameter_m: f64,
    ) -> Result<Vec<DetectedThermokarst>> {
        println!("🔍 Детекция термокарстовых озер...");
        println!("   Источник: {:?}", image.source);
        println!("   Дата: {}", image.acquisition_date);
        println!("   Мин. диаметр: {} м", min_diameter_m);

        // TODO: Реальная обработка изображений
        // 1. Загрузка снимка
        // 2. Расчет NDWI для детекции воды
        // 3. Сегментация водных объектов
        // 4. Фильтрация по размеру
        // 5. Расчет параметров

        // Заглушка - возвращаем примеры
        Ok(vec![
            DetectedThermokarst {
                center_coords: (62.5, 129.3),
                diameter_m: 15.5,
                area_m2: 188.7,
                ndwi: 0.45,
                surrounding_ndvi: 0.35,
                confidence: 0.92,
            },
            DetectedThermokarst {
                center_coords: (62.52, 129.35),
                diameter_m: 22.0,
                area_m2: 380.1,
                ndwi: 0.52,
                surrounding_ndvi: 0.28,
                confidence: 0.88,
            },
        ])
    }

    /// Получить временной ряд для отслеживания роста термокарста
    pub async fn get_time_series(
        &self,
        coords: (f64, f64),
        start_year: i32,
        end_year: i32,
    ) -> Result<Vec<(i32, DetectedThermokarst)>> {
        println!("📊 Получение временного ряда...");
        println!("   Координаты: {:?}", coords);
        println!("   Период: {} - {}", start_year, end_year);

        // TODO: Реальная загрузка исторических данных
        // Google Earth Engine API может помочь с этим

        let mut time_series = Vec::new();
        for year in start_year..=end_year {
            // Симуляция роста
            let age = (year - start_year) as f64;
            let diameter = 5.0 + age.sqrt() * 2.0;
            let area = std::f64::consts::PI * (diameter / 2.0).powi(2);

            time_series.push((
                year,
                DetectedThermokarst {
                    center_coords: coords,
                    diameter_m: diameter,
                    area_m2: area,
                    ndwi: 0.45,
                    surrounding_ndvi: 0.35,
                    confidence: 0.85,
                },
            ));
        }

        Ok(time_series)
    }
}

/// Инструкции по получению спутниковых данных
pub fn print_data_access_guide() {
    println!("\n📡 Как получить спутниковые данные:\n");

    println!("1️⃣  Sentinel-2 (ESA) - БЕСПЛАТНО");
    println!("   • Разрешение: 10м (RGB, NIR)");
    println!("   • Частота: каждые 5 дней");
    println!("   • Регистрация: https://scihub.copernicus.eu/");
    println!("   • API: Sentinel Hub (https://www.sentinel-hub.com/)");
    println!("   • Python: sentinelsat, eolearn\n");

    println!("2️⃣  Landsat 8/9 (NASA/USGS) - БЕСПЛАТНО");
    println!("   • Разрешение: 30м (мультиспектральный)");
    println!("   • Частота: каждые 16 дней");
    println!("   • Регистрация: https://earthexplorer.usgs.gov/");
    println!("   • API: USGS M2M API");
    println!("   • Python: landsatxplore\n");

    println!("3️⃣  Google Earth Engine - БЕСПЛАТНО для исследований");
    println!("   • Доступ к Sentinel, Landsat, MODIS");
    println!("   • Облачная обработка");
    println!("   • Регистрация: https://earthengine.google.com/");
    println!("   • Python: earthengine-api\n");

    println!("4️⃣  Planet Labs - ПЛАТНО (но есть Education/Research программы)");
    println!("   • Разрешение: 3-5м");
    println!("   • Ежедневная съемка");
    println!("   • https://www.planet.com/\n");

    println!("💡 Рекомендация для Якутии:");
    println!("   Используйте Sentinel-2 через Google Earth Engine");
    println!("   Это даст вам бесплатный доступ к данным с 2015 года\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_satellite_client_creation() {
        let client = SatelliteDataClient::new(None);
        assert!(client.api_key.is_none());

        let client_with_key = SatelliteDataClient::new(Some("test_key".to_string()));
        assert!(client_with_key.api_key.is_some());
    }

    #[tokio::test]
    async fn test_search_images() {
        let client = SatelliteDataClient::new(None);
        let bbox = (62.0, 129.0, 63.0, 130.0);

        let images = client
            .search_images(bbox, "2025-06-01", "2025-08-31", 20.0)
            .await
            .unwrap();

        assert!(!images.is_empty());
    }

    #[tokio::test]
    async fn test_time_series() {
        let client = SatelliteDataClient::new(None);
        let coords = (62.5, 129.5);

        let series = client.get_time_series(coords, 2015, 2025).await.unwrap();

        assert_eq!(series.len(), 11); // 2015-2025 включительно
        assert!(series[0].1.diameter_m < series[10].1.diameter_m); // Рост со временем
    }
}
