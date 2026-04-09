# Unity Thermokarst Simulation

Порт алгоритмов термокарста на C# для Unity с процедурной генерацией и физикой.

## Структура

```
Scripts/
├── Core/              # Базовые типы и модели
│   ├── SoilType.cs
│   ├── EnvironmentParams.cs
│   └── ThermokarstLens.cs
├── Physics/           # Физические расчеты
│   ├── ThermalConductivity.cs
│   ├── HeatTransfer.cs
│   └── PhaseTransition.cs
├── Simulation/        # Симуляция
│   ├── ThermokarstEngine.cs
│   └── InverseModeling.cs
├── Procedural/        # Процедурная генерация
│   ├── TerrainGenerator.cs
│   ├── ThermokarstSpawner.cs
│   └── NoiseGenerator.cs
└── Integration/       # Интеграция с Unity
    ├── ThermokarstBehaviour.cs
    └── PhysicsIntegration.cs
```

## Использование

1. Добавьте `ThermokarstBehaviour` на GameObject
2. Настройте параметры среды в Inspector
3. Запустите симуляцию

## Особенности

- Реалистичная физика термокарста (модель Атласова v0.3.0)
- Процедурная генерация ландшафта
- Интеграция с Unity Physics
- Визуализация в реальном времени
