# Примеры использования

## Быстрый старт

### 1. Простая симуляция

```bash
# Симуляция для центральной Якутии на 50 лет
cargo run --release -- simulate -r central -y 50

# С сохранением результатов
cargo run --release -- simulate -r north -y 100 -o north_100y.json -v
```

### 2. Батч-симуляция всех регионов

```bash
# Параллельная обработка
cargo run --release -- batch -y 50 -o results/ -p

# Последовательная обработка
cargo run --release -- batch -y 30 -o results/
```

### 3. Анализ результатов

```bash
cargo run --release -- analyze -i results/Северная_Якутия.json
```

## Сценарии использования

### Сценарий 1: Сравнение регионов

```bash
# Запустить симуляцию для всех регионов
./target/release/thermokarst batch -y 50 -o comparison/ -p

# Проанализировать каждый регион
for file in comparison/*.json; do
    echo "=== Анализ: $file ==="
    ./target/release/thermokarst analyze -i "$file"
    echo ""
done
```

### Сценарий 2: Долгосрочный прогноз

```bash
# Симуляция на 100 лет для северной Якутии
./target/release/thermokarst simulate -r north -y 100 -o north_longterm.json -v

# Анализ стабильности
./target/release/thermokarst analyze -i north_longterm.json
```

### Сценарий 3: Создание пользовательской конфигурации

```bash
# Создать шаблон конфигурации
./target/release/thermokarst config -o my_config.json

# Отредактировать параметры в my_config.json
# Затем использовать в симуляции (требует доработки CLI)
```

## Интерпретация результатов

### Стадии развития термокарста

- **Initiation** - Начальная стадия, активное протаивание
- **ActiveDevelopment** - Активное развитие, быстрый рост
- **Stabilization** - Стабилизация, замедление роста
- **Degradation** - Деградация образования

### Оценка стабильности

- **0.8-1.0** - Высокая стабильность
- **0.5-0.8** - Умеренная стабильность
- **0.0-0.5** - Низкая стабильность, активное развитие

### Риск обрушения

- **< 30%** - Низкий риск
- **30-60%** - Умеренный риск
- **> 60%** - Высокий риск

## Программное использование

### Пример: Использование библиотеки в своем проекте

```toml
[dependencies]
thermokarst-core = { path = "path/to/thermokarst-yakutia/crates/core" }
thermokarst-simulation = { path = "path/to/thermokarst-yakutia/crates/simulation" }
```

```rust
use thermokarst_core::EnvironmentParams;
use thermokarst_simulation::{SimulationConfig, SimulationEngine};

fn main() {
    // Настройка параметров
    let params = EnvironmentParams::northern_yakutia();
    
    let config = SimulationConfig {
        years: 50,
        time_step: 1,
        save_intermediate: true,
        save_interval: 5,
    };
    
    // Запуск симуляции
    let engine = SimulationEngine::new(params, config);
    let result = engine.run().unwrap();
    
    // Обработка результатов
    if let Some(final_lens) = result.lenses.last() {
        println!("Финальная глубина: {:.2} м", final_lens.depth);
        println!("Финальный диаметр: {:.2} м", final_lens.diameter);
        println!("Финальный объем: {:.1} м³", final_lens.volume);
    }
}
```

### Пример: Параллельная обработка

```rust
use thermokarst_simulation::{BatchSimulator, SimulationConfig};

fn main() {
    let config = SimulationConfig {
        years: 50,
        ..Default::default()
    };
    
    let mut batch = BatchSimulator::new(config);
    batch.add_yakutia_scenarios();
    
    // Параллельное выполнение
    let results = batch.run_parallel();
    
    for result in results {
        println!("Сценарий: {}", result.scenario_name);
        if let Ok(sim_result) = result.result {
            println!("  Стадия: {:?}", sim_result.stage);
        }
    }
}
```

## Расширение функциональности

### Добавление нового региона

```rust
// В вашем коде
use thermokarst_core::{EnvironmentParams, SoilType};

let custom_params = EnvironmentParams {
    air_temp: 4.5,
    permafrost_temp: -3.0,
    ice_content: 0.65,
    soil_type: SoilType::Loam,
    vegetation_cover: 0.5,
    water_availability: 0.7,
    permafrost_depth: 1.8,
    warm_season_days: 110,
};
```

### Создание собственного анализатора

```rust
use thermokarst_core::ThermokarstLens;
use thermokarst_geology::StabilityAnalyzer;

fn custom_analysis(lens: &ThermokarstLens) {
    let stability = StabilityAnalyzer::long_term_stability_score(lens);
    let risk = StabilityAnalyzer::collapse_risk(lens);
    
    // Ваша логика анализа
    if stability < 0.5 && risk > 0.6 {
        println!("Критическое состояние!");
    }
}
```
