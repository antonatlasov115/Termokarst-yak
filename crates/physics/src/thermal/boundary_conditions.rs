//! Модуль граничных условий для потоков воды и тепла
//!
//! Портировано из PFLOTRAN (condition.F90)
//! Реализует различные типы граничных условий:
//! - Dirichlet (заданное значение)
//! - Neumann (заданный поток)
//! - Source/Sink (источники/стоки)
//! - Временные зависимости

/// Типы граничных условий
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryConditionType {
    /// Dirichlet BC - заданное давление или температура
    Dirichlet,
    /// Neumann BC - заданный поток
    Neumann,
    /// Источник/сток массы (kg/s)
    MassRate,
    /// Источник/сток энергии (W)
    EnergyRate,
    /// Объемный расход (m³/s)
    VolumetricRate,
    /// Гидростатическое условие
    Hydrostatic,
    /// Нулевой градиент
    ZeroGradient,
}

/// Временная зависимость граничного условия
#[derive(Debug, Clone)]
pub enum TimeVariation {
    /// Постоянное значение
    Constant(f64),
    /// Линейная интерполяция между точками (время, значение)
    Linear(Vec<(f64, f64)>),
    /// Периодическое (период, амплитуда, фаза, среднее)
    Periodic {
        period: f64,
        amplitude: f64,
        phase: f64,
        mean: f64,
    },
}

impl TimeVariation {
    /// Получить значение в заданный момент времени
    pub fn value_at(&self, time: f64) -> f64 {
        match self {
            TimeVariation::Constant(val) => *val,
            TimeVariation::Linear(points) => {
                if points.is_empty() {
                    return 0.0;
                }
                if points.len() == 1 {
                    return points[0].1;
                }

                // Линейная интерполяция
                if time <= points[0].0 {
                    return points[0].1;
                }
                if time >= points[points.len() - 1].0 {
                    return points[points.len() - 1].1;
                }

                for i in 0..points.len() - 1 {
                    if time >= points[i].0 && time <= points[i + 1].0 {
                        let t0 = points[i].0;
                        let t1 = points[i + 1].0;
                        let v0 = points[i].1;
                        let v1 = points[i + 1].1;
                        let alpha = (time - t0) / (t1 - t0);
                        return v0 + alpha * (v1 - v0);
                    }
                }

                points[points.len() - 1].1
            }
            TimeVariation::Periodic {
                period,
                amplitude,
                phase,
                mean,
            } => {
                use std::f64::consts::PI;
                mean + amplitude * (2.0 * PI * time / period + phase).sin()
            }
        }
    }
}

/// Граничное условие для потока
#[derive(Debug, Clone)]
pub struct FlowBoundaryCondition {
    /// Тип граничного условия
    pub bc_type: BoundaryConditionType,
    /// Временная зависимость
    pub time_variation: TimeVariation,
    /// Единицы измерения
    pub units: String,
    /// Имя условия
    pub name: String,
}

impl FlowBoundaryCondition {
    /// Создать Dirichlet BC (заданное давление)
    pub fn dirichlet(pressure: f64, name: &str) -> Self {
        Self {
            bc_type: BoundaryConditionType::Dirichlet,
            time_variation: TimeVariation::Constant(pressure),
            units: "Pa".to_string(),
            name: name.to_string(),
        }
    }

    /// Создать Neumann BC (заданный поток)
    pub fn neumann(flux: f64, name: &str) -> Self {
        Self {
            bc_type: BoundaryConditionType::Neumann,
            time_variation: TimeVariation::Constant(flux),
            units: "kg/m²/s".to_string(),
            name: name.to_string(),
        }
    }

    /// Создать источник массы
    pub fn mass_rate(rate: f64, name: &str) -> Self {
        Self {
            bc_type: BoundaryConditionType::MassRate,
            time_variation: TimeVariation::Constant(rate),
            units: "kg/s".to_string(),
            name: name.to_string(),
        }
    }

    /// Создать источник энергии
    pub fn energy_rate(rate: f64, name: &str) -> Self {
        Self {
            bc_type: BoundaryConditionType::EnergyRate,
            time_variation: TimeVariation::Constant(rate),
            units: "W".to_string(),
            name: name.to_string(),
        }
    }

    /// Создать объемный расход
    pub fn volumetric_rate(rate: f64, name: &str) -> Self {
        Self {
            bc_type: BoundaryConditionType::VolumetricRate,
            time_variation: TimeVariation::Constant(rate),
            units: "m³/s".to_string(),
            name: name.to_string(),
        }
    }

    /// Создать условие с временной зависимостью
    pub fn with_time_variation(mut self, variation: TimeVariation) -> Self {
        self.time_variation = variation;
        self
    }

    /// Получить значение в заданный момент времени
    pub fn value_at(&self, time: f64) -> f64 {
        self.time_variation.value_at(time)
    }
}

/// Граничное условие для температуры
#[derive(Debug, Clone)]
pub struct ThermalBoundaryCondition {
    /// Тип граничного условия
    pub bc_type: BoundaryConditionType,
    /// Временная зависимость
    pub time_variation: TimeVariation,
    /// Единицы измерения
    pub units: String,
    /// Имя условия
    pub name: String,
}

impl ThermalBoundaryCondition {
    /// Создать Dirichlet BC (заданная температура)
    pub fn dirichlet(temperature: f64, name: &str) -> Self {
        Self {
            bc_type: BoundaryConditionType::Dirichlet,
            time_variation: TimeVariation::Constant(temperature),
            units: "°C".to_string(),
            name: name.to_string(),
        }
    }

    /// Создать Neumann BC (заданный тепловой поток)
    pub fn neumann(flux: f64, name: &str) -> Self {
        Self {
            bc_type: BoundaryConditionType::Neumann,
            time_variation: TimeVariation::Constant(flux),
            units: "W/m²".to_string(),
            name: name.to_string(),
        }
    }

    /// Создать источник энергии
    pub fn energy_rate(rate: f64, name: &str) -> Self {
        Self {
            bc_type: BoundaryConditionType::EnergyRate,
            time_variation: TimeVariation::Constant(rate),
            units: "W".to_string(),
            name: name.to_string(),
        }
    }

    /// Создать условие с временной зависимостью
    pub fn with_time_variation(mut self, variation: TimeVariation) -> Self {
        self.time_variation = variation;
        self
    }

    /// Получить значение в заданный момент времени
    pub fn value_at(&self, time: f64) -> f64 {
        self.time_variation.value_at(time)
    }
}

/// Менеджер граничных условий
#[derive(Debug, Clone)]
pub struct BoundaryConditionManager {
    /// Граничные условия для потока
    pub flow_conditions: Vec<(usize, FlowBoundaryCondition)>,
    /// Граничные условия для температуры
    pub thermal_conditions: Vec<(usize, ThermalBoundaryCondition)>,
}

impl BoundaryConditionManager {
    /// Создать новый менеджер
    pub fn new() -> Self {
        Self {
            flow_conditions: Vec::new(),
            thermal_conditions: Vec::new(),
        }
    }

    /// Добавить граничное условие для потока
    pub fn add_flow_condition(&mut self, cell_id: usize, condition: FlowBoundaryCondition) {
        self.flow_conditions.push((cell_id, condition));
    }

    /// Добавить граничное условие для температуры
    pub fn add_thermal_condition(&mut self, cell_id: usize, condition: ThermalBoundaryCondition) {
        self.thermal_conditions.push((cell_id, condition));
    }

    /// Получить граничное условие для потока в ячейке
    pub fn get_flow_condition(&self, cell_id: usize) -> Option<&FlowBoundaryCondition> {
        self.flow_conditions
            .iter()
            .find(|(id, _)| *id == cell_id)
            .map(|(_, bc)| bc)
    }

    /// Получить граничное условие для температуры в ячейке
    pub fn get_thermal_condition(&self, cell_id: usize) -> Option<&ThermalBoundaryCondition> {
        self.thermal_conditions
            .iter()
            .find(|(id, _)| *id == cell_id)
            .map(|(_, bc)| bc)
    }

    /// Применить граничные условия для потока в заданный момент времени
    pub fn apply_flow_conditions(&self, time: f64) -> Vec<(usize, BoundaryConditionType, f64)> {
        self.flow_conditions
            .iter()
            .map(|(cell_id, bc)| (*cell_id, bc.bc_type, bc.value_at(time)))
            .collect()
    }

    /// Применить граничные условия для температуры в заданный момент времени
    pub fn apply_thermal_conditions(&self, time: f64) -> Vec<(usize, BoundaryConditionType, f64)> {
        self.thermal_conditions
            .iter()
            .map(|(cell_id, bc)| (*cell_id, bc.bc_type, bc.value_at(time)))
            .collect()
    }
}

impl Default for BoundaryConditionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dirichlet_bc() {
        let bc = FlowBoundaryCondition::dirichlet(1.0e5, "inlet");
        assert_eq!(bc.bc_type, BoundaryConditionType::Dirichlet);
        assert_eq!(bc.value_at(0.0), 1.0e5);
        assert_eq!(bc.value_at(100.0), 1.0e5);
    }

    #[test]
    fn test_neumann_bc() {
        let bc = FlowBoundaryCondition::neumann(0.001, "infiltration");
        assert_eq!(bc.bc_type, BoundaryConditionType::Neumann);
        assert_eq!(bc.value_at(0.0), 0.001);
    }

    #[test]
    fn test_mass_rate_source() {
        let bc = FlowBoundaryCondition::mass_rate(0.1, "source");
        assert_eq!(bc.bc_type, BoundaryConditionType::MassRate);
        assert_eq!(bc.value_at(0.0), 0.1);
    }

    #[test]
    fn test_time_variation_constant() {
        let var = TimeVariation::Constant(100.0);
        assert_eq!(var.value_at(0.0), 100.0);
        assert_eq!(var.value_at(50.0), 100.0);
        assert_eq!(var.value_at(100.0), 100.0);
    }

    #[test]
    fn test_time_variation_linear() {
        let points = vec![(0.0, 0.0), (10.0, 100.0), (20.0, 50.0)];
        let var = TimeVariation::Linear(points);

        assert_eq!(var.value_at(0.0), 0.0);
        assert_eq!(var.value_at(10.0), 100.0);
        assert_eq!(var.value_at(20.0), 50.0);

        // Интерполяция
        assert_eq!(var.value_at(5.0), 50.0);
        assert_eq!(var.value_at(15.0), 75.0);
    }

    #[test]
    fn test_time_variation_periodic() {
        use std::f64::consts::PI;

        let var = TimeVariation::Periodic {
            period: 365.0, // годовой цикл
            amplitude: 10.0,
            phase: 0.0,
            mean: 0.0,
        };

        assert!((var.value_at(0.0) - 0.0).abs() < 1e-10);
        assert!((var.value_at(365.0 / 4.0) - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_bc_with_time_variation() {
        let points = vec![(0.0, 1.0e5), (100.0, 1.5e5)];
        let bc = FlowBoundaryCondition::dirichlet(1.0e5, "inlet")
            .with_time_variation(TimeVariation::Linear(points));

        assert_eq!(bc.value_at(0.0), 1.0e5);
        assert_eq!(bc.value_at(100.0), 1.5e5);
        assert_eq!(bc.value_at(50.0), 1.25e5);
    }

    #[test]
    fn test_boundary_condition_manager() {
        let mut manager = BoundaryConditionManager::new();

        // Добавить условия
        manager.add_flow_condition(0, FlowBoundaryCondition::dirichlet(1.0e5, "inlet"));
        manager.add_flow_condition(10, FlowBoundaryCondition::neumann(0.001, "top"));
        manager.add_thermal_condition(0, ThermalBoundaryCondition::dirichlet(20.0, "inlet_temp"));

        // Проверить получение условий
        assert!(manager.get_flow_condition(0).is_some());
        assert!(manager.get_flow_condition(10).is_some());
        assert!(manager.get_flow_condition(5).is_none());

        // Применить условия
        let flow_bcs = manager.apply_flow_conditions(0.0);
        assert_eq!(flow_bcs.len(), 2);

        let thermal_bcs = manager.apply_thermal_conditions(0.0);
        assert_eq!(thermal_bcs.len(), 1);
    }

    #[test]
    fn test_seasonal_temperature_bc() {
        use std::f64::consts::PI;

        // Сезонная температура: -20°C зимой, +20°C летом
        let var = TimeVariation::Periodic {
            period: 365.0,
            amplitude: 20.0,
            phase: -PI / 2.0, // минимум в начале года
            mean: 0.0,
        };

        let bc = ThermalBoundaryCondition::dirichlet(0.0, "surface").with_time_variation(var);

        // Зима (день 0)
        assert!((bc.value_at(0.0) - (-20.0)).abs() < 1.0);

        // Лето (день 182.5)
        assert!((bc.value_at(182.5) - 20.0).abs() < 1.0);
    }

    #[test]
    fn test_infiltration_with_precipitation() {
        // Инфильтрация с переменными осадками
        let points = vec![
            (0.0, 0.0),    // нет осадков
            (30.0, 0.001), // дождь
            (60.0, 0.0),   // сухо
            (90.0, 0.002), // сильный дождь
            (120.0, 0.0),  // сухо
        ];

        let bc = FlowBoundaryCondition::neumann(0.0, "infiltration")
            .with_time_variation(TimeVariation::Linear(points));

        assert_eq!(bc.value_at(0.0), 0.0);
        assert_eq!(bc.value_at(30.0), 0.001);
        assert_eq!(bc.value_at(90.0), 0.002);
        assert_eq!(bc.value_at(120.0), 0.0);
    }
}
