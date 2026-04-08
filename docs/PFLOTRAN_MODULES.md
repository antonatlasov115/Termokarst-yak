# 🎯 Конкретные модули PFLOTRAN для Termokarst-yak

**Дата:** 2026-04-08 16:15 UTC  
**Статус:** Готов к изучению

---

## 📦 НАЙДЕННЫЕ КЛЮЧЕВЫЕ МОДУЛИ

### 1. ТЕПЛОВЫЕ ПРОЦЕССЫ

#### `characteristic_curves_thermal.F90`
**Что там:**
- Тепловые характеристические кривые
- Зависимость свойств от температуры
- Теплопроводность vs температура

**Применение в Termokarst-yak:**
```rust
// crates/physics/src/thermal_properties.rs
pub struct ThermalProperties {
    // Из PFLOTRAN
    pub fn thermal_conductivity(&self, temperature: f64) -> f64;
    pub fn heat_capacity(&self, temperature: f64) -> f64;
}
```

**Приоритет:** ⭐⭐⭐ (P0)

---

#### `th.F90` + `th_aux.F90`
**Что там:**
- TH = Thermal-Hydrological coupling
- Связанные тепло-гидрологические процессы
- Фазовые переходы в потоке

**Применение:**
```rust
// crates/physics/src/thermal_hydraulic.rs
pub struct ThermalHydraulicCoupling {
    // Из PFLOTRAN TH mode
}
```

**Приоритет:** ⭐⭐⭐ (P0 + P1)

---

### 2. ГИДРОЛОГИЯ

#### `richards.F90` + `richards_aux.F90` + `richards_common.F90`
**Что там:**
- Уравнение Ричардса (ненасыщенный поток)
- Капиллярное давление
- Влагоперенос в мерзлых грунтах

**Применение:**
```rust
// crates/physics/src/richards_equation.rs
pub struct RichardsEquation {
    // Из PFLOTRAN Richards mode
    pub fn calculate_flux(&self) -> f64;
    pub fn capillary_pressure(&self, saturation: f64) -> f64;
}
```

**Приоритет:** ⭐⭐⭐ (P1)

---

#### `hydrate.F90` + `hydrate_aux.F90` + `hydrate_common.F90`
**Что там:**
- Гидраты (лед + газ)
- Фазовые переходы в присутствии газа
- Может быть полезно для метана в термокарсте

**Применение:**
```rust
// Для будущего (v0.4.0+)
// Выбросы метана из термокарста
pub struct HydrateModel {
    // Из PFLOTRAN hydrate mode
}
```

**Приоритет:** ⭐ (будущее)

---

### 3. ФАЗОВЫЕ ПЕРЕХОДЫ

#### `eos.F90` + `eos_water.F90`
**Что там:**
- EOS = Equation of State (уравнение состояния)
- Свойства воды при разных температурах
- Плотность, вязкость, теплоемкость

**Применение:**
```rust
// crates/physics/src/water_properties.rs
pub struct WaterEOS {
    // Из PFLOTRAN EOS
    pub fn density(&self, temperature: f64, pressure: f64) -> f64;
    pub fn viscosity(&self, temperature: f64) -> f64;
}
```

**Приоритет:** ⭐⭐ (P0)

---

### 4. ХАРАКТЕРИСТИЧЕСКИЕ КРИВЫЕ

#### `characteristic_curves_*.F90` (несколько файлов)
**Что там:**
- Кривые водоудержания (water retention curves)
- Относительная проницаемость
- Капиллярное давление vs насыщенность

**Применение:**
```rust
// crates/physics/src/retention_curves.rs
pub struct RetentionCurve {
    // Из PFLOTRAN characteristic curves
    pub fn saturation(&self, capillary_pressure: f64) -> f64;
    pub fn relative_permeability(&self, saturation: f64) -> f64;
}
```

**Приоритет:** ⭐⭐ (P1)

---

### 5. ЧИСЛЕННЫЕ МЕТОДЫ

#### `timestepper_*.F90` (несколько файлов)
**Что там:**
- Адаптивный шаг по времени
- Контроль ошибок
- Стратегии шага

**Применение:**
```rust
// crates/simulation/src/adaptive_timestepper.rs
pub struct AdaptiveTimestepper {
    // Из PFLOTRAN timestepper
    pub fn calculate_next_dt(&self, error: f64) -> f64;
}
```

**Приоритет:** ⭐⭐ (улучшение)

---

#### `solver.F90`
**Что там:**
- Решатели линейных систем
- Предобуславливатели
- Итерационные методы

**Применение:**
```rust
// Улучшение HeatTransfer2D
pub struct LinearSolver {
    // Концепции из PFLOTRAN
}
```

**Приоритет:** ⭐ (оптимизация)

---

### 6. МАТЕРИАЛЫ И СВОЙСТВА

#### `material.F90` + `material_aux.F90`
**Что там:**
- Свойства материалов (грунтов)
- Зависимость от температуры и насыщенности
- Гетерогенность

**Применение:**
```rust
// Расширение существующего SoilType
pub struct MaterialProperties {
    // Из PFLOTRAN material
    pub fn update_from_state(&mut self, T: f64, saturation: f64);
}
```

**Приоритет:** ⭐⭐ (P1)

---

### 7. ГЕОМЕХАНИКА

#### `geomechanics_*.F90` (много файлов)
**Что там:**
- Механические деформации
- Напряжения и деформации
- Связка с гидрологией (poroelasticity)

**Применение:**
```rust
// Улучшение существующего Subsidence
pub struct Geomechanics {
    // Из PFLOTRAN geomechanics
    pub fn calculate_stress(&self) -> f64;
    pub fn calculate_strain(&self) -> f64;
}
```

**Приоритет:** ⭐ (будущее, v1.0+)

---

## 🎯 ПРИОРИТЕТНЫЙ СПИСОК ДЛЯ ИЗУЧЕНИЯ

### Для P0 (Фазовые переходы) - МАЙ 2026

1. **`th.F90`** - основной модуль TH
2. **`characteristic_curves_thermal.F90`** - тепловые свойства
3. **`eos_water.F90`** - свойства воды
4. **`saturation_function.F90`** - функции насыщенности

**Действия:**
```bash
# Изучить эти файлы
cd .ideo/pflotran/src/pflotran
less th.F90
less characteristic_curves_thermal.F90
less eos_water.F90
less saturation_function.F90
```

---

### Для P1 (Гидрология) - ИЮНЬ 2026

1. **`richards.F90`** - уравнение Ричардса
2. **`richards_common.F90`** - общие функции
3. **`material.F90`** - свойства материалов
4. **`characteristic_curves_base.F90`** - базовые кривые

**Действия:**
```bash
cd .ideo/pflotran/src/pflotran
less richards.F90
less richards_common.F90
less material.F90
```

---

## 📋 ПЛАН ДЕЙСТВИЙ

### Неделя 1 (8-14 апреля 2026)

**Задача:** Изучение модулей фазовых переходов

- [ ] День 1-2: Читать `th.F90` и понять структуру
- [ ] День 3-4: Изучить `characteristic_curves_thermal.F90`
- [ ] День 5: Выписать ключевые формулы
- [ ] День 6-7: Набросать Rust прототип

**Результат:** Понимание алгоритмов PFLOTRAN

---

### Неделя 2-3 (15-28 апреля 2026)

**Задача:** Портирование на Rust

- [ ] Создать `crates/physics/src/phase_change.rs`
- [ ] Портировать алгоритмы из PFLOTRAN
- [ ] Написать unit тесты
- [ ] Валидация на Stefan problem

**Результат:** Рабочий модуль фазовых переходов

---

### Неделя 4 (29 апреля - 5 мая 2026)

**Задача:** Интеграция

- [ ] Связать с `HeatTransfer2D`
- [ ] Обновить `ThawDepthCalculator`
- [ ] Интеграционные тесты
- [ ] Документация

**Результат:** Версия 0.2.0 готова

---

## 🔧 ТЕХНИЧЕСКИЕ ДЕТАЛИ

### Как читать Fortran код

**Fortran 2003 → Rust перевод:**

```fortran
! PFLOTRAN (Fortran)
real(kind=8) :: temperature
real(kind=8) :: liquid_fraction

liquid_fraction = 0.5d0 * (1.d0 + tanh((temperature - 273.15d0) / 0.5d0))
```

↓ Портируется в ↓

```rust
// Termokarst-yak (Rust)
let temperature: f64 = ...;
let liquid_fraction: f64 = 0.5 * (1.0 + ((temperature - 273.15) / 0.5).tanh());
```

**Ключевые отличия:**
- `real(kind=8)` → `f64`
- `0.5d0` → `0.5`
- `tanh()` → `.tanh()`
- Индексация массивов: Fortran с 1, Rust с 0

---

### Структура модулей PFLOTRAN

```fortran
! Типичная структура модуля PFLOTRAN
module TH_module
  use Option_module
  use Global_module
  
  implicit none
  
  type, public :: TH_type
    ! Данные
  end type
  
contains
  
  subroutine THInit(th)
    ! Инициализация
  end subroutine
  
  subroutine THSolve(th)
    ! Решение
  end subroutine
  
end module
```

↓ Портируется в ↓

```rust
// Rust эквивалент
pub struct TH {
    // Данные
}

impl TH {
    pub fn new() -> Self {
        // Инициализация
    }
    
    pub fn solve(&mut self) -> Result<()> {
        // Решение
    }
}
```

---

## 📚 ДОПОЛНИТЕЛЬНЫЕ РЕСУРСЫ

### Документация PFLOTRAN

**Онлайн:**
- https://www.pflotran.org/
- https://documentation.pflotran.org/

**Локально:**
```bash
cd .ideo/pflotran
ls regression_tests/  # Примеры тестов
ls shortcourse/       # Обучающие материалы
```

### Полезные примеры

```bash
# Найти примеры с фазовыми переходами
cd .ideo/pflotran/regression_tests
grep -r "FREEZING" .
grep -r "PHASE_CHANGE" .
grep -r "THERMAL" .
```

---

## ✅ ЧЕКЛИСТ ГОТОВНОСТИ

### Перед началом портирования:

- [ ] Установлен Fortran компилятор (для тестов PFLOTRAN)
- [ ] Изучена документация PFLOTRAN
- [ ] Найдены нужные модули
- [ ] Выписаны ключевые формулы
- [ ] Понятна структура данных
- [ ] Есть тестовые примеры

### После портирования:

- [ ] Код компилируется
- [ ] Unit тесты проходят
- [ ] Валидация на benchmark
- [ ] Документация написана
- [ ] Примеры работают

---

## 🎉 ОЖИДАЕМЫЙ РЕЗУЛЬТАТ

### Версия 0.2.0 (конец мая 2026)

**Новые возможности:**
```rust
use thermokarst_physics::{PhaseChangeModel, ThermalHydraulic};

// Фазовые переходы (из PFLOTRAN)
let phase_change = PhaseChangeModel::new();
let liquid_fraction = phase_change.liquid_fraction(-2.0); // °C
println!("Доля жидкой воды: {:.2}", liquid_fraction);

// Эффективная теплоемкость
let c_eff = phase_change.effective_heat_capacity(-2.0);
println!("C_eff: {:.1} Дж/(кг·К)", c_eff);
```

**Научная достоверность:** 8.5/10 → 9.0/10

---

**Документ создан:** 2026-04-08 16:15 UTC  
**Следующий шаг:** Начать изучение `th.F90`
