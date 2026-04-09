# Unity Thermokarst Simulation - Setup Guide

## Быстрый старт

### 1. Импорт скриптов

Скопируйте папку `Scripts/` в `Assets/` вашего Unity проекта.

### 2. Создание базовой сцены

#### Шаг 1: Создать ландшафт
1. Создайте пустой GameObject: `GameObject > Create Empty`
2. Назовите его "Terrain"
3. Добавьте компонент `TerrainGenerator`
4. Настройте параметры:
   - Width: 256
   - Height: 256
   - Noise Scale: 50
   - Octaves: 4

#### Шаг 2: Создать префаб термокарста
1. Создайте пустой GameObject: `GameObject > Create Empty`
2. Назовите его "Thermokarst"
3. Добавьте компоненты:
   - `ThermokarstBehaviour`
   - `MeshFilter`
   - `MeshRenderer`
4. Настройте параметры среды в Inspector
5. Сохраните как префаб: перетащите в папку Prefabs

#### Шаг 3: Настроить спавнер
1. На объекте "Terrain" добавьте `ThermokarstSpawner`
2. Перетащите префаб термокарста в поле "Thermokarst Prefab"
3. Настройте количество и радиус спавна

### 3. Запуск симуляции

Нажмите Play. Термокарсты будут автоматически симулироваться в реальном времени.

## Параметры среды

### Северная Якутия
```csharp
var params = EnvironmentParams.NorthernYakutia();
// Холодно, высокая льдистость, короткий теплый сезон
```

### Центральная Якутия (по умолчанию)
```csharp
var params = EnvironmentParams.CentralYakutia();
// Умеренные условия
```

### Южная Якутия
```csharp
var params = EnvironmentParams.SouthernYakutia();
// Теплее, меньше льда, длинный теплый сезон
```

## Управление симуляцией

### Через Inspector
- `Auto Simulate` - автоматическая симуляция
- `Time Scale` - скорость времени (1 = 1 секунда = 1 год)
- `Visualize Depth` - цветовая визуализация глубины

### Через код
```csharp
var behaviour = GetComponent<ThermokarstBehaviour>();

// Симулировать один год
behaviour.SimulateYear();

// Получить состояние
ThermokarstLens lens = behaviour.GetLens();
Debug.Log($"Depth: {lens.depth}m, Diameter: {lens.diameter}m");

// Сбросить
behaviour.Reset();
```

## Интеграция с Unity Physics

Добавьте компонент `PhysicsIntegration` для:
- Деформации окружающих объектов
- Эффектов частиц (таяние, обрушение)
- Взаимодействия с Rigidbody

## Процедурная генерация

### Настройка ландшафта
- **Noise Scale** - масштаб шума (больше = плавнее)
- **Octaves** - детализация (больше = детальнее)
- **Persistence** - влияние октав (0.5 = стандарт)
- **Lacunarity** - частота октав (2.0 = стандарт)

### Условия формирования термокарста
- Низины (height < 0.5)
- Высокая влажность (moisture > 0.6)
- Случайное распределение

## Оптимизация

### Для больших ландшафтов
1. Используйте LOD для термокарстов
2. Уменьшите `maxThermokarstsCount`
3. Увеличьте `timeScale` для быстрой симуляции

### Для мобильных устройств
1. Уменьшите разрешение ландшафта (128x128)
2. Отключите `enablePhysicsDeformation`
3. Упростите меши термокарстов (меньше segments)

## Примеры использования

### Пример 1: Наблюдение за развитием
```csharp
public class ThermokarstObserver : MonoBehaviour
{
    [SerializeField] private ThermokarstBehaviour thermokarst;
    
    void Update()
    {
        ThermokarstLens lens = thermokarst.GetLens();
        
        if (!lens.IsStable())
        {
            Debug.LogWarning("Thermokarst is unstable!");
        }
    }
}
```

### Пример 2: Обратное моделирование
```csharp
var engine = new ThermokarstEngine(environmentParams);

// Оценить возраст по наблюдаемой глубине
float observedDepth = 3.5f;
float estimatedAge = engine.EstimateAgeFromDepth(observedDepth);

Debug.Log($"Estimated age: {estimatedAge} years");
```

## Требования

- Unity 2021.3 или выше
- .NET Standard 2.1

## Лицензия

MIT License - см. LICENSE файл в корне проекта
