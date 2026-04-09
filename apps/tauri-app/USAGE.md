# Запуск Tauri приложения

## Быстрый старт

### Режим разработки

```bash
cd apps/tauri-app
npm run tauri dev
```

Приложение откроется в отдельном окне с горячей перезагрузкой.

### Production сборка

```bash
cd apps/tauri-app
npm run tauri build
```

Результат будет в `src-tauri/target/release/bundle/`:
- Windows: `.msi` и `.exe` установщики
- macOS: `.dmg` и `.app`
- Linux: `.deb`, `.AppImage`

## Команды npm

```bash
# Разработка frontend
npm run dev

# Сборка frontend
npm run build

# Preview production build
npm run preview

# Запуск Tauri dev
npm run tauri dev

# Сборка Tauri app
npm run tauri build
```

## Системные требования

- Node.js 18+
- Rust 1.70+
- Windows: WebView2 (обычно уже установлен)
- macOS: macOS 10.15+
- Linux: webkit2gtk, libayatana-appindicator

## Первый запуск

1. Установите зависимости:
```bash
npm install
```

2. Запустите в dev режиме:
```bash
npm run tauri dev
```

3. Выберите режим симуляции (прямая/обратная)

4. Кликните на карту для выбора местоположения

5. Настройте параметры и запустите симуляцию

## Структура результатов

### Прямая симуляция
```json
{
  "mode": "forward",
  "coordinates": { "lat": 62.5, "lon": 129.3 },
  "params": {
    "region": "central",
    "years": 50,
    "temperature": 2.5,
    "ice_content": 0.4,
    "vegetation": 0.6
  },
  "results": [
    {
      "year": 0,
      "depth": 0.0,
      "diameter": 2.0,
      "volume": 0.0,
      "stability": 1.0
    },
    ...
  ]
}
```

### Обратная симуляция
```json
{
  "mode": "inverse",
  "coordinates": { "lat": 62.5, "lon": 129.3 },
  "measuredDiameter": 15.0,
  "inverseResult": {
    "estimated_age": 44,
    "start_year": 1981,
    "confidence": 0.85,
    "results": [...]
  }
}
```

## Troubleshooting

### Ошибка "WebView2 not found" (Windows)
Установите WebView2 Runtime: https://developer.microsoft.com/microsoft-edge/webview2/

### Ошибка компиляции Rust
```bash
cd src-tauri
cargo clean
cargo build
```

### Карта не загружается
Проверьте интернет соединение - тайлы загружаются с серверов OpenStreetMap и ArcGIS.

### Большой размер приложения
Это нормально для Tauri приложений (~10-20 MB). Включает Rust runtime и WebView.

## Разработка

### Добавление новых Tauri команд

1. Добавьте функцию в `src-tauri/src/main.rs`:
```rust
#[tauri::command]
fn my_command(param: String) -> Result<String, String> {
    Ok(format!("Hello {}", param))
}
```

2. Зарегистрируйте в `invoke_handler`:
```rust
.invoke_handler(tauri::generate_handler![
    run_forward_simulation,
    run_inverse_simulation,
    get_system_info,
    my_command  // добавьте здесь
])
```

3. Вызовите из React:
```typescript
import { invoke } from '@tauri-apps/api/core';

const result = await invoke<string>('my_command', { param: 'World' });
```

### Отладка

**Frontend:**
- Откройте DevTools: F12 или Ctrl+Shift+I
- Console, Network, Elements работают как в браузере

**Backend:**
- Логи выводятся в терминал где запущен `tauri dev`
- Используйте `println!()` или `dbg!()` для отладки

## Производительность

- Первый запуск: ~2-3 секунды
- Симуляция 50 лет: ~100-200ms
- Обратная симуляция: ~150-300ms
- Рендеринг карты: зависит от количества точек

## Безопасность

- CSP отключен для разработки (включите в production)
- Asset protocol включен для загрузки локальных файлов
- Все Tauri команды валидируются на стороне Rust

## Обновления

Для обновления зависимостей:

```bash
# Frontend
npm update

# Backend
cd src-tauri
cargo update
```
