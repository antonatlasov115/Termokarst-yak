# Icons

Для генерации иконок используйте:

```bash
npm install -g @tauri-apps/cli
tauri icon path/to/icon.png
```

Или создайте иконки вручную:
- 32x32.png
- 128x128.png
- 128x128@2x.png
- icon.icns (macOS)
- icon.ico (Windows)

Временно можно закомментировать секцию `bundle.icon` в `tauri.conf.json` для разработки.
