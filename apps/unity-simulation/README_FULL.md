# Unity Thermokarst Simulation v0.3.0

Полный порт алгоритмов термокарста на C# для Unity с процедурной генерацией и физикой.

## 📦 Что включено

### Core (Базовые типы)
- **SoilType.cs** - Типы грунта с моделью Йоханзена для теплопроводности
- **EnvironmentParams.cs** - Параметры среды (температура, льдистость, растительность)
- **ThermokarstLens.cs** - Структура данных термокарстовой линзы

### Physics (Физика)
- **HeatTransfer.cs** - Расчет теплопереноса в грунте

### Simulation (Симуляция)
- **ThermokarstEngine.cs** - Движок симуляции (формула Атласова v0.3.0)
  - Вертикальное протаивание
  - Латеральное расширение
  - Обратное моделирование

### Procedural (Процедурная генерация)
- **TerrainGenerator.cs** - Генерация ландшафта Якутии
- **NoiseGenerator.cs** - Многослойный шум Перлина (FBM)
- **ThermokarstSpawner.cs** - Спавн термокарстов на ландшафте

### Integration (Интеграция с Unity)
- **ThermokarstBehaviour.cs** - MonoBehaviour компонент для GameObject
- **PhysicsIntegration.cs** - Интеграция с Unity Physics

## 🚀 Быстрый старт

### 1. Импорт
```
Assets/
└── ThermokarstSimulation/
    ├── Scripts/
    │   ├── Core/
    │   ├── Physics/
    │   ├── Simulation/
    │   ├── Procedural/
    │   └── Integration/
    └── Prefabs/
```

### 2. Создание сцены

**Шаг 1: Ландшафт**
```
GameObject > Create Empty > "Terrain"
Add Component > TerrainGenerator
```

**Шаг 2: Термокарст**
```
GameObject > Create Empty > "Thermokarst"
Add Component > ThermokarstBehaviour
Add Component > MeshFilter
Add Component > MeshRenderer
```

**Шаг 3: Запуск**
```
Play ▶️
```

## 🎮 Использование

### Через Inspector
- `Auto Simulate` - автоматическая симуляция
- `Time Scale` - скорость (1 сек = 1 год)
- `Environment Params` - параметры среды

### Через код
```csharp
using ThermokarstSimulation.Core;
using ThermokarstSimulation.Integration;

var behaviour = GetComponent<ThermokarstBehaviour>();

// Симулировать год
behaviour.SimulateYear();

// Получить состояние
ThermokarstLens lens = behaviour.GetLens();
Debug.Log($"Age: {lens.age}, Depth: {lens.depth:F2}m");
```

## 🧪 Физика

### Формула Атласова v0.3.0
```
ξ = √(2λₜ·DDT / (L·ρw·w^0.7)) · exp(0.30·(1-V)) · (1 + 0.12·ln(ΔT/40))
```

**Где:**
- `λₜ` - теплопроводность по Йоханзену
- `DDT` - градусо-секунды (не дни!)
- `L` - скрытая теплота (334 кДж/кг)
- `w` - льдистость
- `V` - растительный покров
- `ΔT` - годовая амплитуда температур

### Модель Йоханзена
```csharp
λₜ(Sr) = λdry + (λsat - λdry) · Sr^0.7
```

## 🌍 Регионы Якутии

```csharp
// Северная Якутия (холодно, много льда)
var params = EnvironmentParams.NorthernYakutia();

// Центральная Якутия (умеренно)
var params = EnvironmentParams.CentralYakutia();

// Южная Якутия (теплее, меньше льда)
var params = EnvironmentParams.SouthernYakutia();
```

## 🎨 Процедурная генерация

### Условия формирования термокарста
- ✅ Низины (height < 0.5)
- ✅ Высокая влажность (moisture > 0.6)
- ✅ Случайное распределение

### Параметры шума
- **Noise Scale** - масштаб (50 = стандарт)
- **Octaves** - детализация (4 = стандарт)
- **Persistence** - влияние октав (0.5)
- **Lacunarity** - частота (2.0)

## 📊 Примеры

### Наблюдение за развитием
```csharp
void Update()
{
    ThermokarstLens lens = thermokarst.GetLens();
    
    if (!lens.IsStable())
        Debug.LogWarning("Unstable!");
        
    float aspectRatio = lens.AspectRatio;
    Debug.Log($"Aspect: {aspectRatio:F2}");
}
```

### Обратное моделирование
```csharp
var engine = new ThermokarstEngine(params);

float observedDepth = 3.5f;
float age = engine.EstimateAgeFromDepth(observedDepth);

Debug.Log($"Estimated age: {age:F0} years");
```

## ⚡ Оптимизация

### Большие ландшафты
- Используйте LOD
- Уменьшите `maxThermokarstsCount`
- Увеличьте `timeScale`

### Мобильные устройства
- Разрешение 128x128
- Отключите `enablePhysicsDeformation`
- Меньше segments в меше

## 📋 Требования

- Unity 2021.3+
- .NET Standard 2.1

## 📄 Лицензия

MIT License

## 🔗 Ссылки

- [Rust версия](../../crates/)
- [Документация](../../docs/)
- [Примеры](../../examples/)

---

**Версия:** 0.3.0  
**Дата:** 2026-04-09  
**Автор:** Thermokarst Research Team
