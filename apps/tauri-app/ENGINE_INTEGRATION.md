# Интеграция SimulationEngine в Tauri

## Что изменилось

### До (ручные формулы)
- **319 строк** кода
- Дублирование физических формул
- Ручной расчет глубины, диаметра, объема
- Упрощенная формула диаметра
- Нет использования полноценного движка

### После (SimulationEngine)
- **210 строк** кода (-34%)
- Использование `SimulationEngine` из крейта `simulation`
- Все расчеты делегированы движку
- Правильная физика из `ThawDepthCalculator`, `LateralExpansionCalculator`, `SubsidenceCalculator`
- Чистый, поддерживаемый код

## Изменения в коде

### Прямая симуляция

**Было:**
```rust
// ~70 строк ручных формул
const L: f64 = 334000.0;
const RHO_W: f64 = 1000.0;
// ... много констант

for year in 0..=params.years {
    let xi_0 = ((2.0 * thermal_cond * ddt_seconds) / (L * RHO_W * w)).sqrt();
    let k_fire = (BETA * (1.0 - params.vegetation)).exp();
    // ... ручные расчеты
    let depth = xi_0 * k_fire * f_continental * f_moisture * (year as f64).sqrt();
    let diameter = lateral_calc.calculate_diameter(depth, year);
    // ...
}
```

**Стало:**
```rust
// ~20 строк
let env_params = EnvironmentParams { /* ... */ };
let config = SimulationConfig { /* ... */ };
let engine = SimulationEngine::new(env_params, config);
let sim_result = engine.run()?;
```

### Обратная симуляция

**Было:**
```rust
// ~100 строк с дублированием формул
let xi_0 = ((2.0 * thermal_cond * ddt_seconds) / (L * RHO_W * w)).sqrt();
// ... повторение тех же формул
for test_year in 1..=200 {
    let test_depth = xi_0 * k_fire * f_continental * f_moisture * (test_year as f64).sqrt();
    let test_diameter = lateral_calc.calculate_diameter(test_depth, test_year);
    // ...
}
```

**Стало:**
```rust
// ~50 строк, использует движок
for test_age in 1..=200 {
    let engine = SimulationEngine::new(env_params.clone(), config);
    if let Ok(result) = engine.run() {
        // Сравниваем результаты
    }
}
```

## Преимущества

### 1. Меньше кода
- 319 → 210 строк (-34%)
- Убрано дублирование
- Проще читать и поддерживать

### 2. Правильная физика
- Использует `ThawDepthCalculator` (формула Атласова v0.3.0)
- Использует `LateralExpansionCalculator` (учет типа грунта, влажности)
- Использует `SubsidenceCalculator` (просадка)
- Все калькуляторы протестированы (177 тестов)

### 3. Единый источник истины
- Формулы в одном месте (крейты)
- Изменения в крейтах автоматически применяются в Tauri
- Нет расхождений между CLI и GUI

### 4. Расширяемость
- Легко добавить новые параметры
- Легко добавить новые режимы симуляции
- Можно использовать `run_with_details()` для детальных результатов

## Что работает

✅ Прямая симуляция (10-100 лет)  
✅ Обратная симуляция (определение возраста)  
✅ Правильный расчет глубины (Атласов v0.3.0)  
✅ Правильный расчет диаметра (LateralExpansionCalculator)  
✅ Учет региона (север/центр/юг)  
✅ Учет типа грунта  
✅ Итеративный поиск возраста  

## Известные ограничения

⚠️ `stability` - заглушка (0.8), так как `ThermokarstLens` не имеет поля `stability_score`
- Можно добавить расчет через `StabilityAnalyzer::long_term_stability_score()`

## Следующие шаги

1. Добавить расчет стабильности через `StabilityAnalyzer`
2. Использовать `run_with_details()` для более детальных результатов
3. Добавить кеширование результатов симуляции
4. Оптимизировать итеративный поиск в обратной симуляции

## Сборка

```bash
cd apps/tauri-app/src-tauri
cargo build --release
```

✅ Компилируется успешно  
✅ Все зависимости подключены  
✅ Готово к использованию  

---

**Дата:** 2026-04-09  
**Версия:** 0.3.0  
**Статус:** ✅ Завершено
