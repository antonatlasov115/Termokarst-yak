# 💡 Что можно взять из PFLOTRAN для Termokarst-yak

**Дата:** 2026-04-08  
**Источник:** `.ideo/pflotran/`  
**Статус:** Анализ возможностей интеграции

---

## 🔍 ЧТО ТАКОЕ PFLOTRAN

**PFLOTRAN** - это open-source симулятор для моделирования:
- Подземных потоков (groundwater flow)
- Реактивного транспорта (reactive transport)
- Тепломассопереноса
- Многофазных потоков

**Технологии:**
- Язык: Fortran 2003 (объектно-ориентированный)
- Параллелизм: MPI (массивно-параллельный)
- Лицензия: GNU LGPL (можно интегрировать)

---

## 🎯 ЧТО МОЖНО ВЗЯТЬ ДЛЯ TERMOKARST-YAK

### 1. 🌊 ГИДРОЛОГИЯ (P1 - Высокий приоритет)

**Из PFLOTRAN:**
- Закон Дарси для потоков в мерзлых грунтах
- Уравнение Ричардса (ненасыщенный поток)
- Многофазные потоки (вода + лед)
- Капиллярное давление

**Применение в Termokarst-yak:**
```rust
// Адаптация из PFLOTRAN
pub struct DarcyFlow {
    hydraulic_conductivity: f64,
    pressure_gradient: Vec3,
}

impl DarcyFlow {
    // q = -K * ∇h (из PFLOTRAN)
    pub fn calculate_flux(&self) -> Vec3 {
        -self.hydraulic_conductivity * self.pressure_gradient
    }
}
```

**Приоритет:** ⭐⭐⭐ (P1 в ROADMAP)

---

### 2. ❄️ ФАЗОВЫЕ ПЕРЕХОДЫ (P0 - Критично)

**Из PFLOTRAN:**
- Модель замерзания/таяния
- Скрытая теплота фазовых переходов
- Температурная зависимость теплоемкости
- Кривые замерзания (freezing curves)

**Применение в Termokarst-yak:**
```rust
// Концепция из PFLOTRAN
pub struct PhaseChange {
    latent_heat: f64,
    melting_point: f64,
}

impl PhaseChange {
    // Доля жидкой воды при температуре T
    pub fn liquid_fraction(&self, temperature: f64) -> f64 {
        // Из PFLOTRAN: сигмоидальная функция
        0.5 * (1.0 + tanh((temperature - self.melting_point) / 0.5))
    }
}
```

**Приоритет:** ⭐⭐⭐ (P0 в ROADMAP)

---

### 3. 🔥 ТЕПЛОПЕРЕНОС С ФАЗОВЫМИ ПЕРЕХОДАМИ

**Из PFLOTRAN:**
- Уравнение теплопроводности с источниками
- Эффективная теплоемкость (с учетом фазовых переходов)
- Конвективный теплоперенос (с потоком воды)

**Применение:**
```rust
// Расширение существующего HeatTransfer2D
pub struct HeatTransferWithPhaseChange {
    grid: Grid2D,
    phase_change: PhaseChangeModel,
}

impl HeatTransferWithPhaseChange {
    // C_eff = C + L * dθ/dT (из PFLOTRAN)
    pub fn effective_heat_capacity(&self, T: f64) -> f64 {
        let base_capacity = self.heat_capacity;
        let phase_term = self.phase_change.latent_heat 
            * self.phase_change.liquid_fraction_derivative(T);
        base_capacity + phase_term
    }
}
```

**Приоритет:** ⭐⭐⭐ (P0)

---

### 4. 🧮 ЧИСЛЕННЫЕ МЕТОДЫ

**Из PFLOTRAN:**
- Неявные схемы (implicit schemes) для устойчивости
- Решатели линейных систем (PETSc)
- Адаптивный шаг по времени
- Ньютоновские итерации для нелинейных задач

**Применение:**
```rust
// Улучшение существующего HeatTransfer2D
pub struct ImplicitSolver {
    tolerance: f64,
    max_iterations: usize,
}

impl ImplicitSolver {
    // Неявная схема (более устойчивая)
    pub fn solve_implicit(&mut self, dt: f64) -> Result<()> {
        // Из PFLOTRAN: Newton-Raphson
        for iter in 0..self.max_iterations {
            let residual = self.calculate_residual();
            if residual < self.tolerance {
                break;
            }
            self.update_solution();
        }
        Ok(())
    }
}
```

**Приоритет:** ⭐⭐ (улучшение производительности)

---

### 5. 🌡️ СВЯЗАННЫЕ ПРОЦЕССЫ (THM - Thermo-Hydro-Mechanical)

**Из PFLOTRAN:**
- Связка тепло ↔ гидрология
- Зависимость свойств от температуры
- Обратные связи

**Применение:**
```rust
// P3 в ROADMAP: Связка SEB ↔ грунт
pub struct CoupledTHM {
    thermal: HeatTransfer2D,
    hydraulic: DarcyFlow,
    mechanical: Subsidence,
}

impl CoupledTHM {
    // Итерационное решение связанной системы (из PFLOTRAN)
    pub fn solve_coupled(&mut self, dt: f64) -> Result<()> {
        loop {
            self.thermal.step(dt)?;
            self.hydraulic.update_from_thermal(&self.thermal);
            self.mechanical.update_from_hydraulic(&self.hydraulic);
            
            if self.converged() {
                break;
            }
        }
        Ok(())
    }
}
```

**Приоритет:** ⭐ (P3 в ROADMAP)

---

## 📋 КОНКРЕТНЫЕ МОДУЛИ ДЛЯ ИЗУЧЕНИЯ

### В PFLOTRAN искать:

1. **`src/pflotran/thermal_*.F90`** - теплоперенос
2. **`src/pflotran/phase_*.F90`** - фазовые переходы
3. **`src/pflotran/richards_*.F90`** - уравнение Ричардса
4. **`src/pflotran/freezing_*.F90`** - замерзание
5. **`src/pflotran/subsurface_*.F90`** - подземные процессы

---

## 🔄 ПЛАН ИНТЕГРАЦИИ

### Этап 1: Изучение (1 неделя)
- [ ] Прочитать документацию PFLOTRAN
- [ ] Найти модули фазовых переходов
- [ ] Изучить численные схемы
- [ ] Понять структуру данных

### Этап 2: Адаптация (2-3 недели)
- [ ] Портировать алгоритмы на Rust
- [ ] Адаптировать под Termokarst-yak
- [ ] Написать тесты
- [ ] Валидация на benchmark задачах

### Этап 3: Интеграция (1-2 недели)
- [ ] Интегрировать в существующий код
- [ ] Обновить документацию
- [ ] Добавить примеры использования

---

## ⚠️ ВАЖНЫЕ ЗАМЕЧАНИЯ

### Лицензия
- PFLOTRAN: GNU LGPL
- Termokarst-yak: MIT
- ✅ **Совместимы!** Можно использовать идеи и алгоритмы

### Язык
- PFLOTRAN: Fortran 2003
- Termokarst-yak: Rust
- ⚠️ Нужно портировать, но концепции универсальны

### Сложность
- PFLOTRAN: очень сложный (100k+ строк)
- Termokarst-yak: средний (~6500 строк)
- 💡 Брать только нужное, упрощать

---

## 🎯 ПРИОРИТЕТНЫЕ ЗАДАЧИ ИЗ PFLOTRAN

### Для версии 0.2.0 (Q2 2026)

**P0: Фазовые переходы**
- Взять из PFLOTRAN:
  - Модель замерзания/таяния
  - Кривые замерзания
  - Эффективная теплоемкость

**Ожидаемый результат:**
```rust
// Новый модуль
// crates/physics/src/phase_change.rs (на основе PFLOTRAN)

pub struct PhaseChangeModel {
    // Из PFLOTRAN
}

impl PhaseChangeModel {
    pub fn liquid_fraction(&self, T: f64) -> f64;
    pub fn effective_heat_capacity(&self, T: f64) -> f64;
    pub fn phase_change_rate(&self, T: f64, heat_flux: f64) -> f64;
}
```

---

### Для версии 0.3.0 (Q3 2026)

**P1: Гидрология**
- Взять из PFLOTRAN:
  - Закон Дарси
  - Уравнение Ричардса
  - Многофазные потоки

**Ожидаемый результат:**
```rust
// Новый модуль
// crates/physics/src/hydrology.rs (на основе PFLOTRAN)

pub struct HydrologicalModel {
    // Из PFLOTRAN
}

impl HydrologicalModel {
    pub fn darcy_flux(&self) -> f64;
    pub fn richards_equation(&self) -> f64;
    pub fn infiltration(&self) -> f64;
}
```

---

## 📚 РЕСУРСЫ

### Документация PFLOTRAN
- Официальный сайт: https://www.pflotran.org/
- GitHub: https://github.com/pflotran/pflotran
- Документация: https://documentation.pflotran.org/

### Полезные статьи
1. Hammond et al. (2014) - "PFLOTRAN: Reactive flow & transport code"
2. Lichtner et al. (2015) - "PFLOTRAN User Manual"
3. Mills et al. (2007) - "Modeling subsurface reactive transport"

---

## 💡 АЛЬТЕРНАТИВЫ

Если PFLOTRAN слишком сложен, можно посмотреть:

1. **SUTRA** (USGS) - проще, но менее функционален
2. **TOUGH2** - для геотермальных систем
3. **HYDRUS** - для ненасыщенной зоны
4. **Научные статьи** - прямые формулы

---

## ✅ РЕКОМЕНДАЦИИ

### Что точно стоит взять:
1. ✅ Модель фазовых переходов (P0)
2. ✅ Закон Дарси для мерзлых грунтов (P1)
3. ✅ Численные схемы (неявные методы)

### Что можно отложить:
- ⏳ Реактивный транспорт (не нужен для термокарста)
- ⏳ Геохимия (пока не приоритет)
- ⏳ MPI параллелизм (есть rayon)

### Что не нужно:
- ❌ Полная структура PFLOTRAN (слишком сложно)
- ❌ PETSc зависимости (есть свои решатели)
- ❌ Fortran код напрямую (портировать на Rust)

---

## 🎯 ИТОГОВЫЙ ПЛАН

### Ближайшие действия (апрель 2026):

1. **Изучить PFLOTRAN** (1 неделя)
   - Найти модули фазовых переходов
   - Понять алгоритмы
   - Выписать ключевые формулы

2. **Портировать на Rust** (2 недели)
   - Создать `phase_change.rs`
   - Написать тесты
   - Валидация на Stefan problem

3. **Интегрировать** (1 неделя)
   - Связать с `HeatTransfer2D`
   - Обновить `ThawDepthCalculator`
   - Документация

**Цель:** Версия 0.2.0 с фазовыми переходами к концу мая 2026

---

**Документ создан:** 2026-04-08 16:15 UTC  
**Статус:** План действий готов
