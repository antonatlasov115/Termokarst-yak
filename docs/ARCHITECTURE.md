# Архитектура проекта

## Структура крейтов

### `thermokarst-core`
Базовые типы и модели данных.

```
core/
├── types/           # Базовые типы
│   ├── mod.rs      # SoilType, EnvironmentParams, ThermokarstLens
│   └── error.rs    # Типы ошибок
├── models/          # Модели данных
│   ├── dataset.rs  # Работа с датасетами
│   ├── iryp.rs     # IRYP модели
│   └── iryp_params.rs  # Параметры IRYP
└── validation/      # Валидация данных
    └── mod.rs
```

### `thermokarst-physics`
Физические процессы в мерзлых грунтах.

```
physics/
├── thermal/         # Тепловые процессы
│   ├── heat_transfer.rs
│   ├── heat_transfer_2d.rs
│   ├── frozen_ground.rs
│   ├── phase_transition.rs
│   ├── snow.rs
│   ├── surface_energy.rs
│   ├── thawing_index.rs
│   ├── nfactor.rs
│   ├── boundary_conditions.rs
│   ├── thaw.rs
│   └── thermal_conductivity.rs
├── hydraulic/       # Гидравлические процессы
│   └── richards.rs  # Уравнение Ричардса
├── mechanical/      # Механические процессы
│   └── subsidence.rs
└── solvers/         # Численные решатели
    └── newton_solver.rs
```

### `thermokarst-geology`
Геологические процессы термокарста.

```
geology/
├── processes/       # Геологические процессы
│   ├── consolidation.rs      # Консолидация грунта
│   └── lateral_expansion.rs  # Латеральное расширение
└── analysis/        # Анализ
    └── stability.rs  # Анализ стабильности
```

### `thermokarst-simulation`
Движок симуляции.

```
simulation/
├── modeling/        # Моделирование
│   ├── engine.rs              # Основной движок
│   ├── inverse_modeling.rs    # Обратное моделирование
│   └── satellite_integration.rs
├── analysis/        # Анализ и калибровка
│   ├── batch.rs      # Пакетная обработка
│   ├── calibration.rs
│   └── uncertainty.rs
└── visualization/   # Визуализация
    ├── visualization.rs
    └── iryp_visualization.rs
```

## Ключевые изменения v0.3.0

### 1. Модель Йоханзена для теплопроводности
```rust
// Старый подход (захардкоженные значения)
let k = match soil_type {
    SoilType::Peat => 0.5,
    SoilType::Sand => 2.0,
    // ...
};

// Новый подход (динамическая теплопроводность)
let k = soil_type.thermal_conductivity(saturation_ratio);
```

### 2. Переименование параметров
- `water_availability` → `soil_saturation_ratio` (более точное название)
- Добавлен `temperature_amplitude` для формулы Атласова

### 3. Исправление размерностей в inverse_modeling
- DDT теперь в секундах (не в днях)
- Используется `temperature_amplitude` вместо `air_temp` в логарифме
- Прямой расчет через ALT: `t = (depth / xi_alt)²`

## Импорты

Все публичные API доступны через корневые модули:

```rust
use thermokarst_core::{EnvironmentParams, SoilType};
use thermokarst_physics::{HeatTransferCalculator, NewtonSolver};
use thermokarst_geology::{ConsolidationModel, StabilityAnalyzer};
use thermokarst_simulation::{SimulationEngine, InverseModelingCalculator};
```
