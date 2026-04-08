# Работа с изображениями и спутниковыми данными

## Обзор

Для извлечения данных о термокарстовых образованиях из фотографий и спутниковых снимков можно использовать несколько подходов.

## Подходы к обработке изображений

### 1. Анализ фотографий с дронов/наземных съемок

**Что можно извлечь:**
- Диаметр термокарстового образования (по масштабу)
- Глубина (если есть тени или стереопара)
- Форма и геометрия
- Наличие воды
- Растительность по краям

**Технологии:**
- OpenCV - обработка изображений
- Photogrammetry - построение 3D моделей
- Image segmentation - выделение объектов

### 2. Спутниковые данные

**Источники:**
- Sentinel-2 (10-60м разрешение, бесплатно)
- Landsat 8/9 (15-30м разрешение, бесплатно)
- Planet Labs (3-5м разрешение, платно)
- Maxar/WorldView (0.3-0.5м разрешение, платно)

**Что можно извлечь:**
- Площадь термокарстовых озер
- Изменения во времени (временные ряды)
- NDVI (растительность)
- Температура поверхности
- Влажность почвы

### 3. InSAR (Interferometric SAR)

**Источники:**
- Sentinel-1 (бесплатно)
- ALOS PALSAR
- TerraSAR-X

**Что можно извлечь:**
- Просадка поверхности (мм точность)
- Деформации грунта
- Скорость изменений

## Реализация в проекте

### Архитектура модуля

```rust
crates/
├── image-processing/     # Новый крейт
│   ├── src/
│   │   ├── lib.rs
│   │   ├── photo.rs      # Обработка фотографий
│   │   ├── satellite.rs  # Спутниковые данные
│   │   ├── insar.rs      # InSAR данные
│   │   └── extraction.rs # Извлечение параметров
```

### Зависимости

```toml
[dependencies]
image = "0.24"           # Обработка изображений
imageproc = "0.23"       # Алгоритмы обработки
opencv = "0.88"          # OpenCV биндинги (опционально)
ndarray = "0.15"         # Массивы для растров
geo = "0.27"             # Геопространственные типы
gdal = "0.16"            # Чтение GeoTIFF
exif = "0.5"             # Метаданные фото
```

## Пример 1: Извлечение диаметра из фотографии

```rust
use image::{DynamicImage, GenericImageView};
use imageproc::edges::canny;
use imageproc::hough::detect_circles;

pub struct PhotoAnalyzer {
    image: DynamicImage,
    scale_meters_per_pixel: f64,
}

impl PhotoAnalyzer {
    pub fn new(image_path: &str, scale: f64) -> Result<Self> {
        let image = image::open(image_path)?;
        Ok(Self {
            image,
            scale_meters_per_pixel: scale,
        })
    }

    /// Определить диаметр термокарстового образования
    pub fn detect_thermokarst_diameter(&self) -> Result<f64> {
        // 1. Конвертировать в grayscale
        let gray = self.image.to_luma8();

        // 2. Детекция краев (Canny)
        let edges = canny(&gray, 50.0, 100.0);

        // 3. Поиск окружностей (Hough transform)
        let circles = detect_circles(&edges, 20, 200);

        // 4. Выбрать наибольшую окружность
        let largest_circle = circles.iter()
            .max_by_key(|c| c.radius)
            .ok_or("No circles detected")?;

        // 5. Конвертировать в метры
        let diameter_pixels = largest_circle.radius * 2.0;
        let diameter_meters = diameter_pixels * self.scale_meters_per_pixel;

        Ok(diameter_meters)
    }

    /// Оценить глубину по теням (если есть солнечный свет)
    pub fn estimate_depth_from_shadows(&self, sun_angle: f64) -> Result<f64> {
        // Анализ теней для оценки глубины
        // depth = shadow_length * tan(sun_angle)
        todo!("Implement shadow analysis")
    }

    /// Определить наличие воды
    pub fn detect_water(&self) -> bool {
        // Анализ цвета и текстуры
        // Вода обычно темнее и более однородная
        todo!("Implement water detection")
    }
}
```

## Пример 2: Анализ спутниковых данных

```rust
use gdal::Dataset;
use ndarray::Array2;

pub struct SatelliteAnalyzer {
    dataset: Dataset,
}

impl SatelliteAnalyzer {
    /// Загрузить спутниковый снимок (GeoTIFF)
    pub fn from_geotiff(path: &str) -> Result<Self> {
        let dataset = Dataset::open(path)?;
        Ok(Self { dataset })
    }

    /// Рассчитать NDVI (Normalized Difference Vegetation Index)
    pub fn calculate_ndvi(&self) -> Result<Array2<f32>> {
        // NDVI = (NIR - Red) / (NIR + Red)
        let nir = self.read_band(4)?;  // Near-infrared
        let red = self.read_band(3)?;  // Red

        let ndvi = (&nir - &red) / (&nir + &red);
        Ok(ndvi)
    }

    /// Детекция термокарстовых озер
    pub fn detect_thermokarst_lakes(&self) -> Result<Vec<ThermokarstFeature>> {
        // 1. Рассчитать водный индекс (NDWI)
        let green = self.read_band(2)?;
        let nir = self.read_band(4)?;
        let ndwi = (&green - &nir) / (&green + &nir);

        // 2. Пороговая сегментация (вода > 0.3)
        let water_mask = ndwi.mapv(|v| v > 0.3);

        // 3. Найти связные компоненты
        let features = self.extract_features(&water_mask)?;

        Ok(features)
    }

    /// Измерить площадь термокарстового образования
    pub fn measure_area(&self, feature: &ThermokarstFeature) -> f64 {
        let pixel_size = self.get_pixel_size(); // метры
        feature.pixel_count as f64 * pixel_size * pixel_size
    }

    /// Временной анализ (сравнение снимков)
    pub fn temporal_analysis(
        &self,
        previous: &SatelliteAnalyzer,
        years_between: f64,
    ) -> Result<ChangeAnalysis> {
        let current_lakes = self.detect_thermokarst_lakes()?;
        let previous_lakes = previous.detect_thermokarst_lakes()?;

        // Сопоставить озера и рассчитать изменения
        let changes = self.match_and_compare(&current_lakes, &previous_lakes)?;

        Ok(ChangeAnalysis {
            new_lakes: changes.new_count,
            expanded_lakes: changes.expanded,
            average_growth_rate: changes.total_growth / years_between,
        })
    }
}

#[derive(Debug)]
pub struct ThermokarstFeature {
    pub id: usize,
    pub centroid: (f64, f64),  // lat, lon
    pub pixel_count: usize,
    pub perimeter: f64,
    pub bbox: BoundingBox,
}

#[derive(Debug)]
pub struct ChangeAnalysis {
    pub new_lakes: usize,
    pub expanded_lakes: Vec<ExpansionData>,
    pub average_growth_rate: f64,  // м²/год
}
```

## Пример 3: InSAR данные (просадка)

```rust
pub struct InSARAnalyzer {
    displacement_map: Array2<f32>,  // мм
    coherence_map: Array2<f32>,
}

impl InSARAnalyzer {
    /// Загрузить карту смещений
    pub fn from_displacement_map(path: &str) -> Result<Self> {
        // Загрузить GeoTIFF с данными InSAR
        todo!()
    }

    /// Извлечь скорость просадки в точке
    pub fn get_subsidence_rate(&self, lat: f64, lon: f64) -> Result<f64> {
        let (x, y) = self.latlon_to_pixel(lat, lon)?;
        let displacement_mm = self.displacement_map[[y, x]];
        let coherence = self.coherence_map[[y, x]];

        // Проверить качество данных
        if coherence < 0.3 {
            return Err("Low coherence - unreliable data".into());
        }

        // Конвертировать в м/год
        let subsidence_m_per_year = displacement_mm / 1000.0;
        Ok(subsidence_m_per_year)
    }

    /// Создать датасет наблюдений из InSAR
    pub fn to_observation_dataset(&self) -> ObservationDataset {
        let mut observations = Vec::new();

        // Сэмплировать точки с высокой когерентностью
        for (y, x) in self.high_coherence_points() {
            let (lat, lon) = self.pixel_to_latlon(x, y);
            let subsidence = self.displacement_map[[y, x]] / 1000.0;

            observations.push(ObservationData {
                site_id: format!("INSAR_{x}_{y}"),
                coordinates: (lat, lon),
                date: "2025-08-01".to_string(),
                subsidence: Some(subsidence),
                ..Default::default()
            });
        }

        ObservationDataset {
            name: "InSAR Subsidence Data".to_string(),
            source: "Sentinel-1 InSAR".to_string(),
            observations,
            ..Default::default()
        }
    }
}
```

## Пример 4: Автоматическое извлечение параметров

```rust
pub struct AutoExtractor;

impl AutoExtractor {
    /// Извлечь все параметры из фотографии
    pub fn extract_from_photo(
        photo_path: &str,
        metadata: PhotoMetadata,
    ) -> Result<ObservationData> {
        let analyzer = PhotoAnalyzer::new(photo_path, metadata.scale)?;

        let diameter = analyzer.detect_thermokarst_diameter()?;
        let has_water = analyzer.detect_water();
        let depth = if metadata.sun_angle.is_some() {
            analyzer.estimate_depth_from_shadows(metadata.sun_angle.unwrap())?
        } else {
            0.0
        };

        Ok(ObservationData {
            site_id: metadata.site_id,
            coordinates: metadata.coordinates,
            date: metadata.date,
            diameter: Some(diameter),
            subsidence: if depth > 0.0 { Some(depth) } else { None },
            metadata: [
                ("source".to_string(), "photo".to_string()),
                ("has_water".to_string(), has_water.to_string()),
            ].iter().cloned().collect(),
        })
    }

    /// Извлечь параметры из спутникового снимка
    pub fn extract_from_satellite(
        geotiff_path: &str,
        region: BoundingBox,
    ) -> Result<Vec<ObservationData>> {
        let analyzer = SatelliteAnalyzer::from_geotiff(geotiff_path)?;
        let lakes = analyzer.detect_thermokarst_lakes()?;

        let observations = lakes.iter()
            .filter(|lake| region.contains(lake.centroid))
            .map(|lake| {
                let area = analyzer.measure_area(lake);
                let diameter = (area / std::f64::consts::PI).sqrt() * 2.0;

                ObservationData {
                    site_id: format!("SAT_{}", lake.id),
                    coordinates: lake.centroid,
                    date: "2025-08-01".to_string(),
                    diameter: Some(diameter),
                    metadata: [
                        ("source".to_string(), "satellite".to_string()),
                        ("area_m2".to_string(), area.to_string()),
                    ].iter().cloned().collect(),
                }
            })
            .collect();

        Ok(observations)
    }
}
```

## Практическое использование

### CLI команды (расширение)

```bash
# Извлечь данные из фотографии
thermokarst image analyze -i photo.jpg --scale 0.05 -o observation.json

# Обработать спутниковый снимок
thermokarst satellite analyze -i sentinel2.tif --region yakutia.geojson

# InSAR анализ
thermokarst insar extract -i displacement.tif -o subsidence_data.json

# Временной анализ
thermokarst satellite compare -i1 2020.tif -i2 2025.tif --years 5
```

### Workflow с изображениями

```bash
# 1. Обработать фотографии с дрона
thermokarst image batch -i photos/*.jpg --scale 0.05 -o photo_data.json

# 2. Обработать спутниковые снимки
thermokarst satellite analyze -i sentinel2.tif -o satellite_data.json

# 3. Объединить данные
thermokarst dataset merge -i photo_data.json satellite_data.json -o combined.json

# 4. Калибровать модель
thermokarst dataset calibrate -i combined.json -o params.json

# 5. Запустить симуляцию
thermokarst simulate --params params.json -y 50 -o results.json
```

## Источники спутниковых данных

### Бесплатные

1. **Copernicus Open Access Hub** (Sentinel-1, Sentinel-2)
   - URL: https://scihub.copernicus.eu/
   - Разрешение: 10-60м
   - Обновление: каждые 5 дней

2. **USGS EarthExplorer** (Landsat)
   - URL: https://earthexplorer.usgs.gov/
   - Разрешение: 15-30м
   - Архив с 1972 года

3. **NASA Earthdata**
   - URL: https://earthdata.nasa.gov/
   - Различные продукты (MODIS, ASTER, etc.)

### Коммерческие

1. **Planet Labs** - ежедневные снимки 3-5м
2. **Maxar** - 0.3-0.5м разрешение
3. **Airbus** - Pleiades 0.5м

## Инструменты обработки

### Python (для предобработки)

```python
# Скачать Sentinel-2 данные
from sentinelsat import SentinelAPI

api = SentinelAPI('user', 'password')
products = api.query(
    area='POLYGON((125 60, 135 60, 135 70, 125 70, 125 60))',
    date=('20250101', '20250801'),
    platformname='Sentinel-2'
)

# Обработать в GeoTIFF
import rasterio
# ... обработка и экспорт
```

### QGIS

- Визуализация
- Ручная оцифровка
- Экспорт в GeoJSON/GeoTIFF

## Точность методов

| Метод | Параметр | Точность |
|-------|----------|----------|
| Фото с дрона | Диаметр | ±0.1-0.5м |
| Фото с дрона | Глубина | ±0.5-2м |
| Sentinel-2 | Площадь | ±100-500м² |
| Landsat | Площадь | ±500-2000м² |
| InSAR | Просадка | ±1-5мм/год |
| Planet Labs | Площадь | ±10-50м² |

## Рекомендации

1. **Для точных измерений** - использовать дроны или высокоразрешающие спутники
2. **Для мониторинга больших территорий** - Sentinel-2/Landsat
3. **Для измерения просадки** - InSAR (Sentinel-1)
4. **Для временного анализа** - комбинировать разные источники

## Следующие шаги

1. Создать крейт `image-processing`
2. Реализовать базовые алгоритмы детекции
3. Добавить CLI команды для работы с изображениями
4. Интегрировать с существующей системой калибровки
5. Добавить примеры и документацию
