# Источники данных для калибровки модели

## Открытые базы данных по мерзлоте

### 1. GTN-P (Global Terrestrial Network for Permafrost)
**URL:** https://gtnp.arcticportal.org/

**Описание:** Глобальная сеть мониторинга многолетней мерзлоты

**Данные:**
- Температура грунта на различных глубинах
- Глубина активного слоя
- Координаты станций мониторинга
- Временные ряды с 1990-х годов

**Регионы:** Включая Россию, Сибирь, Якутию

### 2. CALM (Circumpolar Active Layer Monitoring)
**URL:** https://www2.gwu.edu/~calm/

**Описание:** Сеть мониторинга активного слоя в циркумполярных регионах

**Данные:**
- Глубина сезонного протаивания
- Ежегодные измерения с 1990-х
- Сетки измерений 100×100 м
- Данные по России и Якутии

### 3. ESA CCI Permafrost
**URL:** https://climate.esa.int/en/projects/permafrost/

**Описание:** Спутниковые данные по температуре поверхности мерзлоты

**Данные:**
- Температура поверхности грунта (GST)
- Глобальное покрытие
- Разрешение 1 км
- Период: 1997-2020+

### 4. PANGAEA
**URL:** https://www.pangaea.de/

**Описание:** Репозиторий научных данных по наукам о Земле

**Поиск:** "permafrost Yakutia", "thermokarst Siberia"

**Данные:**
- Полевые измерения
- Керны грунта
- Льдистость
- Геохимические данные

### 5. Arctic Data Center
**URL:** https://arcticdata.io/

**Описание:** Репозиторий данных по Арктике

**Данные:**
- Исследовательские датасеты
- Полевые наблюдения
- Метаданные экспедиций

## Научные публикации с данными

### Ключевые исследования по Якутии:

1. **Fedorov et al. (2014)** - "Permafrost-Landscape Map of the Republic of Sakha (Yakutia)"
   - Карты распространения мерзлоты
   - Типы ландшафтов
   - Льдистость по регионам

2. **Ulrich et al. (2017)** - "Thermokarst in Siberian ice-rich permafrost"
   - Скорости развития термокарста
   - Измерения просадки
   - Полевые данные

3. **Nitze et al. (2020)** - "Remote sensing of permafrost thaw"
   - Спутниковые данные InSAR
   - Скорости просадки
   - Временные ряды

4. **Grosse et al. (2016)** - "Thermokarst lakes, drainage, and drained basins"
   - Размеры термокарстовых озер
   - Скорости расширения
   - Статистика по регионам

## Формат данных для импорта

### JSON формат (используется в проекте)

```json
{
  "name": "Yakutia Observations 2025",
  "description": "Field measurements from Central Yakutia",
  "source": "Research expedition",
  "observations": [
    {
      "site_id": "YAK-001",
      "coordinates": [62.0, 129.7],
      "date": "2025-08-15",
      "active_layer_thickness": 1.2,
      "ground_temperature": -1.5,
      "temperature_depth": 10.0,
      "subsidence": 0.15,
      "diameter": 5.2,
      "metadata": {
        "soil_type": "loam",
        "vegetation": "sparse"
      }
    }
  ]
}
```

### CSV формат

```csv
site_id,lat,lon,date,active_layer_m,ground_temp_c,subsidence_m,diameter_m,soil_type
YAK-001,62.0,129.7,2025-08-15,1.2,-1.5,0.15,5.2,loam
YAK-002,70.5,135.2,2025-07-20,0.8,-4.2,0.08,3.1,peat
```

## Типичные значения для Якутии

### Северная Якутия (70-73°N)
- Температура мерзлоты: -5 до -10°C
- Глубина активного слоя: 0.4-1.0 м
- Льдистость: 70-90%
- Тип грунта: торф, ил
- Скорость просадки: 1-5 см/год

### Центральная Якутия (60-65°N)
- Температура мерзлоты: -2 до -5°C
- Глубина активного слоя: 1.0-2.0 м
- Льдистость: 50-70%
- Тип грунта: суглинок, супесь
- Скорость просадки: 5-15 см/год

### Южная Якутия (55-60°N)
- Температура мерзлоты: -1 до -3°C
- Глубина активного слоя: 1.5-2.5 м
- Льдистость: 30-50%
- Тип грунта: суглинок, песок
- Скорость просадки: 10-25 см/год

## Использование данных в проекте

### 1. Создание датасета

```bash
# Создать пример датасета
cargo run --release -- dataset create -o yakutia_data.json

# Импортировать CSV
cargo run --release -- dataset import -i field_data.csv -o dataset.json
```

### 2. Калибровка модели

```bash
# Оценить параметры из данных
cargo run --release -- calibrate -i yakutia_data.json -o calibrated_params.json

# Валидация модели
cargo run --release -- validate -i yakutia_data.json -p calibrated_params.json
```

### 3. Программное использование

```rust
use thermokarst_core::dataset::ObservationDataset;
use thermokarst_simulation::calibration::ModelCalibrator;

// Загрузить данные
let dataset = ObservationDataset::from_json_file("data.json")?;

// Калибровка
let calibrator = ModelCalibrator::new(dataset);
let params = calibrator.estimate_environment_params()?;

// Валидация
let validation = calibrator.validate_model(&params, 10);
println!("Ошибка: {:.1}%", validation.mean_relative_error * 100.0);
```

## Рекомендации по сбору данных

### Минимальный набор для калибровки:
1. Координаты точки наблюдения
2. Глубина активного слоя (м)
3. Температура грунта на глубине 10 м (°C)
4. Тип грунта
5. Дата наблюдения

### Дополнительные параметры:
- Просадка поверхности (для термокарста)
- Диаметр образования
- Растительный покров
- Влажность грунта
- Снежный покров

## Ссылки на литературу

1. Romanovsky et al. (2010) - "Thermal state of permafrost in Russia"
2. Shur & Jorgenson (2007) - "Patterns of permafrost formation"
3. Jorgenson et al. (2006) - "Abrupt increase in permafrost degradation"
4. Kokelj & Jorgenson (2013) - "Advances in thermokarst research"

## Контакты организаций

- **Институт мерзлотоведения СО РАН** (Якутск)
- **Arctic and Antarctic Research Institute** (Санкт-Петербург)
- **Permafrost Laboratory, University of Alaska Fairbanks**
- **Alfred Wegener Institute** (Германия)
