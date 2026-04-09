# 🌐 Веб-интерфейс Термокарст Якутии

Современный веб-интерфейс для визуализации симуляций термокарстовых образований.

## 🚀 Быстрый старт

```bash
cd web-ui
npm install
npm run dev
```

Откройте http://localhost:5173 в браузере.

## 📦 Технологии

- **React 18** + **TypeScript** - современный UI фреймворк
- **Vite** - быстрая сборка и hot reload
- **Recharts** - интерактивные графики
- **Lucide React** - красивые иконки

## ✨ Возможности

### 1. Интерактивные параметры
- 🌍 Выбор региона (Северная/Центральная/Южная Якутия)
- 🌡️ Настройка температуры (0-5°C)
- 💧 Льдистость грунта (0-100%)
- 🌲 Растительный покров (0-100%)
- ⏱️ Период симуляции (10-100 лет)

### 2. Визуализация результатов
- 📊 **Глубина протаивания** - динамика по годам
- 📈 **Латеральное расширение** - рост диаметра
- 📉 **Объем термокарста** - накопление объема
- ⚠️ **Стабильность** - оценка рисков

### 3. Экспорт данных
- 💾 Сохранение результатов в JSON
- 📋 Готовые данные для анализа

## 🎨 Дизайн

- Современный градиентный интерфейс
- Адаптивная верстка (desktop/tablet/mobile)
- Плавные анимации и переходы
- Интуитивные элементы управления

## 🔗 Интеграция с Rust CLI

В будущих версиях веб-интерфейс будет вызывать Rust CLI через:

### Вариант 1: WebAssembly (WASM)
```bash
# Компиляция в WASM
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/thermokarst.wasm --out-dir web-ui/src/wasm
```

### Вариант 2: Backend API
```bash
# Запуск REST API сервера
cargo run --bin api-server
```

### Вариант 3: CLI через Node.js
```typescript
import { exec } from 'child_process';

const runSimulation = (params) => {
  return new Promise((resolve, reject) => {
    exec(
      `cargo run --release -- simulate -r ${params.region} -y ${params.years}`,
      (error, stdout) => {
        if (error) reject(error);
        else resolve(JSON.parse(stdout));
      }
    );
  });
};
```

## 📱 Скриншоты

### Главный экран
- Панель параметров слева
- 4 интерактивных графика справа
- Сводка результатов

### Адаптивность
- Desktop: 2-колоночная сетка
- Tablet/Mobile: 1 колонка, параметры внизу

## 🛠️ Разработка

### Структура проекта
```
web-ui/
├── src/
│   ├── App.tsx          # Главный компонент
│   ├── App.css          # Стили
│   ├── main.tsx         # Точка входа
│   └── index.css        # Глобальные стили
├── public/              # Статические файлы
├── package.json         # Зависимости
└── vite.config.ts       # Конфигурация Vite
```

### Команды

```bash
# Разработка
npm run dev

# Сборка для продакшена
npm run build

# Предпросмотр продакшен-сборки
npm run preview

# Линтинг
npm run lint
```

## 🚀 Деплой

### Vercel
```bash
npm run build
vercel --prod
```

### Netlify
```bash
npm run build
netlify deploy --prod --dir=dist
```

### GitHub Pages
```bash
npm run build
# Загрузите содержимое dist/ в gh-pages ветку
```

## 🔮 Планы развития

- [ ] Интеграция с реальным Rust CLI
- [ ] 3D визуализация термокарста
- [ ] Сравнение нескольких сценариев
- [ ] Карта Якутии с точками наблюдений
- [ ] Экспорт в PDF/PNG
- [ ] Мультиязычность (RU/EN)
- [ ] Темная/светлая тема
- [ ] История симуляций

## 📄 Лицензия

MIT

## 📞 Контакты

- Email: antonatlasov115@gmail.com
- Телефон: 89963178892
