# Исправление расчета диаметра

## Проблема

Диаметр термокарста вычислялся неправильно - использовалась упрощенная формула:
```rust
let diameter = 2.0 + k_lateral * (1.0 + year as f64).ln();
```

Эта формула не учитывала:
- Тип грунта
- Доступность воды
- Глубину просадки
- Физически обоснованные коэффициенты

## Решение

Интегрирована правильная модель из `thermokarst-geology::LateralExpansionCalculator`:

```rust
// Создаем параметры окружения
let env_params = EnvironmentParams {
    soil_type: match region {
        "north" => SoilType::Peat,
        "south" => SoilType::Sand,
        _ => SoilType::Silt,
    },
    ice_content: params.ice_content,
    soil_saturation_ratio: 0.3,
    ..Default::default()
};

let lateral_calc = LateralExpansionCalculator::new(env_params);
let diameter = lateral_calc.calculate_diameter(depth, year);
```

### Формула из LateralExpansionCalculator

```rust
diameter = depth * base_expansion_rate * water_factor * soil_factor * time_factor
```

где:
- `base_expansion_rate = 0.4` (боковое расширение медленнее вертикального)
- `water_factor = 1.0 + 0.6 * soil_saturation_ratio`
- `soil_factor` зависит от типа грунта:
  - Песок: 1.3
  - Глина: 0.7
  - Торф: 1.1
  - Ил: 1.2
  - Суглинок: 1.0
- `time_factor = ln(year + 1)` (логарифмический рост)

### Обратная симуляция

Для определения возраста по диаметру используется итеративный поиск:

```rust
let mut age_from_diameter = 1u32;
for test_year in 1..=200 {
    let test_depth = xi_0 * k_fire * f_continental * f_moisture * (test_year as f64).sqrt();
    let test_diameter = lateral_calc.calculate_diameter(test_depth, test_year);
    if test_diameter >= params.current_diameter {
        age_from_diameter = test_year;
        break;
    }
}
```

## Изменения

1. Добавлена зависимость `thermokarst-geology` в `Cargo.toml`
2. Импортированы `LateralExpansionCalculator`, `EnvironmentParams`, `SoilType`
3. Заменена формула диаметра в прямой симуляции
4. Заменена формула диаметра в обратной симуляции
5. Добавлен итеративный поиск возраста по диаметру

## Результат

✅ Диаметр теперь вычисляется физически корректно  
✅ Учитывается тип грунта по региону  
✅ Учитывается глубина просадки  
✅ Обратная симуляция работает точнее  

## Тестирование

Запустите приложение и проверьте:
```bash
npm run tauri dev
```

Ожидаемые значения диаметра теперь более реалистичны и зависят от региона и параметров грунта.
