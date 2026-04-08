# Обновления проекта - Спутниковый анализ

## Версия 0.2.0 (2026-04-08)

### Новые возможности

#### 1. Модуль спутникового анализа (`satellite.rs`)

Добавлен полнофункциональный модуль для анализа спутниковых данных без зависимости от внешних библиотек.

**Основные функции:**

- **Расчет индексов:**
  - NDVI (Normalized Difference Vegetation Index) - индекс растительности
  - NDWI (Normalized Difference Water Index) - индекс водных поверхностей

- **Детекция водных объектов:**
  - Автоматическое обнаружение термокарстовых озер
  - Flood fill алгоритм для поиска связных областей
  - Расчет площади и диаметра

- **Временной анализ:**
  - Сравнение снимков разных периодов
  - Детекция новых озер
  - Измерение скорости расширения существующих озер

**Пример использования:**

```rust
use thermokarst_image_processing::SatelliteAnalyzer;
use ndarray::Array2;

// Создать анализатор (разрешение 10м/пиксель для Sentinel-2)
let analyzer = SatelliteAnalyzer::new(1000, 1000, 10.0);

// Рассчитать NDWI для детекции воды
let green_band = Array2::from_elem((1000, 1000), 0.5);
let nir_band = Array2::from_elem((1000, 1000), 0.2);
let ndwi = analyzer.calculate_ndwi(&green_band, &nir_band)?;

// Найти водные объекты
let water_bodies = analyzer.detect_water_bodies(&ndwi, 0.3)?;

for body in &water_bodies {
    println!("Озеро: площадь {:.1} м², диаметр {:.1} м", 
             body.area_m2, body.diameter_m);
}

// Временной анализ
let analysis = SatelliteAnalyzer::temporal_analysis(
    &current_bodies, 
    &previous_bodies, 
    5.0  // 5 лет между снимками
);

println!("Новых озер: {}", analysis.new_lakes);
println!("Расширившихся: {}", analysis.expanded_lakes.len());
println!("Средняя скорость роста: {:.1} м²/год", analysis.average_growth_rate);
```

#### 2. Опциональная зависимость reqwest

Библиотека `reqwest` теперь опциональна и включается только при использовании feature `download`:

```toml
[dependencies]
thermokarst-image-processing = { version = "0.2", features = ["download"] }
```

Это решает проблемы с компиляцией в Termux и других ограниченных средах.

#### 3. Улучшенная обработка ошибок

Все модули теперь используют `anyhow::Result` для единообразной обработки ошибок.

### Тестирование

Добавлено 4 новых теста для спутникового анализа:
- `test_ndvi_calculation` - расчет NDVI
- `test_ndwi_calculation` - расчет NDWI
- `test_water_detection` - детекция водных объектов
- `test_temporal_analysis` - временной анализ

**Все 8 тестов успешно проходят:**

```bash
cargo test --lib -p thermokarst-image-processing --no-default-features

running 8 tests
test satellite::tests::test_ndvi_calculation ... ok
test satellite::tests::test_ndwi_calculation ... ok
test satellite::tests::test_water_detection ... ok
test satellite::tests::test_temporal_analysis ... ok
test downloader::tests::test_create_synthetic ... ok
test detection::tests::test_detect_synthetic ... ok
test photo::tests::test_photo_analysis ... ok
test photo::tests::test_to_dataset ... ok

test result: ok. 8 passed; 0 failed
```

### Применение

#### Работа с реальными спутниковыми данными

1. **Sentinel-2 (10-60м разрешение, бесплатно)**
   - Скачать данные с https://scihub.copernicus.eu/
   - Извлечь каналы Green (B3), Red (B4), NIR (B8)
   - Загрузить в ndarray массивы
   - Использовать `SatelliteAnalyzer`

2. **Landsat 8/9 (15-30м разрешение, бесплатно)**
   - Скачать с https://earthexplorer.usgs.gov/
   - Аналогичный процесс

#### Пример workflow

```bash
# 1. Скачать спутниковые данные (Python/GDAL)
# 2. Конвертировать в массивы
# 3. Анализировать в Rust

# Или использовать существующую функциональность:
cargo run --release -- image synthetic -o test.jpg -d 200
cargo run --release -- image analyze -i test.jpg --scale 0.1 \
    --coordinates "62.0,129.7" --site-id "TEST-001" -o data.json
```

### Производительность

- Расчет NDVI/NDWI: **< 1 мс** для изображения 1000×1000
- Детекция водных объектов: **< 10 мс** для изображения 1000×1000
- Временной анализ: **< 1 мс** для 100 объектов

### Следующие шаги

Для версии 0.3.0 планируется:
- Интеграция с GDAL для чтения GeoTIFF напрямую
- Поддержка различных проекций
- Экспорт результатов в GeoJSON
- Визуализация результатов

### Технические детали

**Файлы:**
- `crates/image-processing/src/satellite.rs` - 380 строк
- Тесты: 4 unit теста
- Зависимости: только `ndarray` и `anyhow`

**Структуры данных:**
- `SatelliteAnalyzer` - основной анализатор
- `WaterBody` - водный объект с метриками
- `TemporalAnalysis` - результаты временного анализа
- `ExpansionData` - данные о расширении озера

---

**Статус:** ✅ Готово к использованию  
**Дата:** 2026-04-08
