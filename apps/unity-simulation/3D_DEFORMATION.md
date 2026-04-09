# 3D Thermokarst Deformation System

Полная реализация трехмерной деформации террейна с клеточным автоматом и физикой.

## 🎯 Архитектура

### Клеточный автомат на деформируемой сетке

Каждая ячейка террейна (`TerrainCell`) хранит:

**Геометрия:**
- `currentHeight` - текущая высота
- `initialHeight` - высота до таяния
- `waterDepth` - глубина воды

**Физика:**
- `iceContent` - льдистость (0-1)
- `vegetationCover` - растительный покров (0-1)
- `saturationRatio` - влажность для Йоханзена (0-1)
- `soilType` - тип грунта

**Состояние:**
- `cumulativeThawDepth` - накопленная глубина протаивания ξ_thermo
- `timeActiveYears` - время активного таяния
- `isActive` - активна ли ячейка

## 🧊 Генерация ледяных жил (Voronoi)

### Полигональная структура мерзлоты

```csharp
// Генерация карты льдистости с ледяными жилами
float[,] iceMap = VoronoiNoise.GenerateRealisticIceMap(
    width, height, 
    cellSize: 15f,  // Размер полигонов
    seed
);
```

**Результат:**
- Границы полигонов: `iceContent = 0.8-0.95` (ледяные жилы)
- Центры полигонов: `iceContent = 0.3-0.6` (меньше льда)

**Эффект:** Котлованы формируются вдоль ледяных жил, создавая характерные для Якутии угловатые, соединенные озера.

## 🔄 Цикл симуляции

### Шаг 1: Вертикальное протаивание

```csharp
// Формула Атласова v0.3.0
ξ_ALT = √(2λₜ·DDT / (L·ρw·w^0.7)) · exp(0.30·(1-V)) · (1 + 0.12·ln(ΔT/40))

// Кумулятивное таяние
ξ_thermo = ξ_ALT * √t

// Просадка грунта
subsidence = Δξ * iceContent
```

### Шаг 2: Горизонтальная эрозия

**Тепловой бонус от воды:**
```csharp
if (cell.IsUnderwater(waterLevel))
{
    // Ячейка под водой нагревает соседей
    foreach (neighbor in GetNeighbors(cell))
    {
        if (!neighbor.IsUnderwater())
        {
            // Ускоряем таяние берега
            neighbor.timeActiveYears += thermalBonus;
        }
    }
}
```

**Эрозия крутых склонов:**
```csharp
float heightDiff = neighbor.height - cell.height;
if (heightDiff > 2f)
{
    // Гравитационное обрушение
    float slopeErosion = heightDiff * 0.05f * dtYears;
    neighbor.height -= slopeErosion;
}
```

### Шаг 3: Обновление воды

```csharp
if (cell.currentHeight < waterLevel)
{
    cell.waterDepth = waterLevel - cell.currentHeight;
}
```

## 🎨 Визуализация

### Цветовая кодировка

- **Синий** - вода (`waterDepth > 0`)
- **Желтый→Красный** - активное таяние (по глубине)
- **Зеленый** - нормальный грунт

### Обновление меша

Каждый кадр (или реже) обновляется Vertex Buffer:

```csharp
vertices[i] = new Vector3(x, cell.currentHeight, y);
colors[i] = GetCellColor(cell);
```

## 🌊 Продвинутая эрозия

### Термическая абразия

```csharp
// Подмыв берега теплой водой
float tempDiff = waterTemp - permafrostTemp;
float erosion = tempDiff * iceContent * 0.05f;
```

### Волновое воздействие

```csharp
// Зависит от размера озера
float fetch = √(lakeArea);
float waveHeight = 0.5 * √fetch * windSpeed / 10;
```

### Сезонная вариация

```csharp
// Максимум летом, минимум зимой
float seasonalMultiplier = GetSeasonalMultiplier(dayOfYear);
erosion *= seasonalMultiplier;
```

## 🎮 Использование

### Базовая настройка

```csharp
GameObject terrain = new GameObject("Terrain");
var grid = terrain.AddComponent<DeformableTerrainGrid>();

grid.gridWidth = 128;
grid.gridHeight = 128;
grid.voronoiCellSize = 15f; // Размер полигонов
grid.autoSimulate = true;
grid.simulationSpeed = 1f; // 1 год в секунду
```

### Управление симуляцией

```csharp
// Симулировать 10 лет
grid.SimulateYears(10f);

// Получить ячейку
TerrainCell cell = grid.GetCell(x, y);
Debug.Log($"Ice: {cell.iceContent}, Depth: {cell.cumulativeThawDepth}");
```

## 🔬 Параметры для реализма

### Условия формирования термокарста

1. **Нарушение растительности:** `vegetationCover < 0.3`
2. **Наличие воды:** `waterDepth > 0`
3. **Высокая льдистость:** `iceContent > 0.6` (ледяные жилы)

### Факторы разнообразия форм

1. **Шум Вороного** - полигональные ледяные жилы
2. **Шум Перлина** - вариация растительности
3. **Рельеф** - сток воды и эрозия склонов
4. **Тепловой бонус** - боковое расширение от воды

## 📊 Оптимизация

### Для больших карт (256x256+)

```csharp
// Обновлять меш реже
if (frameCount % 10 == 0)
    UpdateMesh();

// Симулировать блоками
SimulateRegion(x0, y0, x1, y1);
```

### Для мобильных

```csharp
grid.gridWidth = 64;
grid.gridHeight = 64;
grid.enableLateralErosion = false; // Отключить эрозию
```

## 🎯 Результат

**Получаем:**
- ✅ Реалистичные формы аласов (не круглые!)
- ✅ Полигональная структура (ледяные жилы)
- ✅ Динамическое расширение берегов
- ✅ Учет рельефа и стока воды
- ✅ Физически достоверное время формирования

**Формы озер:**
- Угловатые (вдоль ледяных жил)
- Соединенные (слияние котлованов)
- Неровные берега (эрозия)
- Пологие склоны (обрушение)

---

**Версия:** 0.3.0  
**Дата:** 2026-04-09
