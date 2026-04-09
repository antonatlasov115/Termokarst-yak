# Исправление ошибки overflow

## Проблема

При запуске приложения возникала ошибка:
```
thread 'main' panicked at src\main.rs:160:22:
attempt to subtract with overflow
```

## Причина

В функции `run_inverse_simulation` на строке 160 происходило вычитание:
```rust
let start_year = params.observation_year - final_age;
```

Если `final_age` больше чем `observation_year`, происходит переполнение при вычитании беззнаковых целых чисел (u32).

## Решение

Применены два исправления:

1. **Ограничение возраста**: добавлен `.min(params.observation_year)` чтобы возраст не превышал год наблюдения
2. **Безопасное вычитание**: использован `saturating_sub()` вместо обычного вычитания

```rust
let final_age = ((age_from_depth as f64 * w_depth + age_from_diameter as f64 * (1.0 - w_depth))
    .round() as u32)
    .max(1)
    .min(params.observation_year); // Ограничиваем возраст годом наблюдения
let start_year = params.observation_year.saturating_sub(final_age);
```

`saturating_sub()` возвращает 0 вместо паники при переполнении.

## Статус

✅ Исправлено  
✅ Приложение компилируется  
✅ Готово к запуску

Теперь можно запустить:
```bash
npm run tauri dev
```
