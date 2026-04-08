# Руководство по работе с данными

## Обзор

Проект теперь поддерживает работу с реальными данными наблюдений для калибровки и валидации модели.

## Новые возможности

### 1. Работа с датасетами

```bash
# Создать пример датасета
./target/release/thermokarst dataset create -o my_data.json

# Показать информацию о датасете
./target/release/thermokarst dataset info -i my_data.json

# Калибровать модель по данным
./target/release/thermokarst dataset calibrate -i my_data.json -o params.json
```

### 2. Формат данных

Датасет хранится в JSON формате:

```json
{
  "name": "Название датасета",
  "description": "Описание",
  "source": "Источник данных",
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

### 3. Поля данных

**Обязательные:**
- `site_id` - идентификатор точки
- `coordinates` - [широта, долгота]
- `date` - дата наблюдения (ISO 8601)

**Опциональные:**
- `active_layer_thickness` - глубина активного слоя (м)
- `ground_temperature` - температура грунта (°C)
- `temperature_depth` - глубина измерения температуры (м)
- `subsidence` - просадка поверхности (м)
- `diameter` - диаметр термокарстового образования (м)
- `metadata` - дополнительные данные

## Калибровка модели

### Процесс калибровки

1. **Загрузка данных** - чтение датасета из JSON
2. **Статистический анализ** - расчет средних значений
3. **Оценка параметров** - определение параметров среды
4. **Валидация** - сравнение модели с наблюдениями

### Оцениваемые параметры

Из данных наблюдений автоматически оцениваются:

- **Температура воздуха** - из температуры грунта и глубины активного слоя
- **Льдистость** - из просадки и глубины протаивания
- **Тип грунта** - из метаданных наблюдений
- **Глубина мерзлоты** - из средней глубины активного слоя

### Метрики валидации

- **Средняя относительная ошибка** - отклонение модели от наблюдений
- **Критерий качества** - ошибка < 30% считается хорошей

## Примеры использования

### Пример 1: Создание датасета из полевых данных

```bash
# 1. Создать шаблон
./target/release/thermokarst dataset create -o template.json

# 2. Отредактировать template.json, добавить свои данные

# 3. Проверить датасет
./target/release/thermokarst dataset info -i template.json

# 4. Калибровать модель
./target/release/thermokarst dataset calibrate -i template.json -o my_params.json
```

### Пример 2: Региональная калибровка

```bash
# Создать датасеты для разных регионов
./target/release/thermokarst dataset create -o north_data.json
./target/release/thermokarst dataset create -o central_data.json
./target/release/thermokarst dataset create -o south_data.json

# Калибровать для каждого региона
./target/release/thermokarst dataset calibrate -i north_data.json -o north_params.json
./target/release/thermokarst dataset calibrate -i central_data.json -o central_params.json
./target/release/thermokarst dataset calibrate -i south_data.json -o south_params.json
```

### Пример 3: Программное использование

```rust
use thermokarst_core::dataset::ObservationDataset;
use thermokarst_simulation::calibration::ModelCalibrator;

// Загрузить данные
let dataset = ObservationDataset::from_json_file("observations.json")?;

// Статистика
let stats = dataset.statistics();
println!("Наблюдений: {}", stats.total_observations);

// Фильтрация по региону
let filtered = dataset.filter_by_bbox(60.0, 65.0, 125.0, 135.0);

// Калибровка
let calibrator = ModelCalibrator::new(dataset);
let params = calibrator.estimate_environment_params()?;

// Валидация
let validation = calibrator.validate_model(&params, 10);
if validation.is_good_fit() {
    println!("Модель хорошо соответствует данным");
}
```

## Источники данных

См. файл `DATA_SOURCES.md` для списка открытых баз данных:

- GTN-P (Global Terrestrial Network for Permafrost)
- CALM (Circumpolar Active Layer Monitoring)
- ESA CCI Permafrost
- PANGAEA
- Arctic Data Center

## Рекомендации

### Минимальный набор данных

Для надежной калибровки рекомендуется:
- Минимум 10 точек наблюдений
- Данные за несколько лет
- Разнообразие типов грунтов
- Географическое распределение

### Качество данных

- Проверяйте координаты (должны быть в Якутии)
- Убедитесь в корректности единиц измерения
- Удаляйте выбросы и аномальные значения
- Документируйте источник данных

### Интерпретация результатов

- Ошибка < 20% - отличное соответствие
- Ошибка 20-30% - хорошее соответствие
- Ошибка 30-50% - удовлетворительное
- Ошибка > 50% - требуется доработка

## Ограничения

Текущая версия калибровки:
- Использует простые статистические методы
- Не учитывает временную динамику
- Предполагает однородность региона
- Требует ручной настройки для сложных случаев

## Будущие улучшения

Планируется добавить:
- Импорт CSV и других форматов
- Байесовская калибровка
- Учет неопределенности
- Пространственная интерполяция
- Временные ряды
- Автоматическая оптимизация параметров

## Тестовые данные

Проект включает пример датасета с типичными значениями для Якутии:

```bash
./target/release/thermokarst dataset create -o example.json
./target/release/thermokarst dataset info -i example.json
```

Этот датасет содержит 3 точки наблюдений:
- Центральная Якутия (Якутск)
- Северная Якутия
- Южная Якутия

## Поддержка

Для вопросов и предложений:
- Создайте issue в репозитории
- Обратитесь к документации `DATA_SOURCES.md`
- Изучите примеры в `EXAMPLES.md`
