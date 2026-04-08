//! Двумерный теплоперенос в мерзлых грунтах
//!
//! Базовая реализация 2D модели теплопереноса методом конечных разностей
//! для моделирования латеральных термических процессов в термокарсте.

use thermokarst_core::{Result, ThermokarstError};

/// Двумерная сетка для теплопереноса
#[derive(Debug, Clone)]
pub struct Grid2D {
    /// Количество узлов по X (горизонталь)
    pub nx: usize,
    /// Количество узлов по Z (глубина)
    pub nz: usize,
    /// Шаг сетки по X (м)
    pub dx: f64,
    /// Шаг сетки по Z (м)
    pub dz: f64,
    /// Температура в узлах (°C)
    pub temperature: Vec<Vec<f64>>,
    /// Теплопроводность в узлах (Вт/(м·К))
    pub thermal_conductivity: Vec<Vec<f64>>,
    /// Объемная теплоемкость (Дж/(м³·К))
    pub heat_capacity: Vec<Vec<f64>>,
}

impl Grid2D {
    /// Создать новую сетку
    pub fn new(nx: usize, nz: usize, dx: f64, dz: f64) -> Self {
        let temperature = vec![vec![0.0; nx]; nz];
        let thermal_conductivity = vec![vec![1.5; nx]; nz];
        let heat_capacity = vec![vec![2.0e6; nx]; nz];

        Self {
            nx,
            nz,
            dx,
            dz,
            temperature,
            thermal_conductivity,
            heat_capacity,
        }
    }

    /// Установить начальные условия
    pub fn set_initial_temperature<F>(&mut self, temp_fn: F)
    where
        F: Fn(f64, f64) -> f64,
    {
        for iz in 0..self.nz {
            for ix in 0..self.nx {
                let x = ix as f64 * self.dx;
                let z = iz as f64 * self.dz;
                self.temperature[iz][ix] = temp_fn(x, z);
            }
        }
    }

    /// Установить теплопроводность
    pub fn set_thermal_conductivity<F>(&mut self, k_fn: F)
    where
        F: Fn(f64, f64) -> f64,
    {
        for iz in 0..self.nz {
            for ix in 0..self.nx {
                let x = ix as f64 * self.dx;
                let z = iz as f64 * self.dz;
                self.thermal_conductivity[iz][ix] = k_fn(x, z);
            }
        }
    }
}

/// Решатель 2D уравнения теплопроводности
pub struct HeatTransfer2D {
    grid: Grid2D,
    /// Шаг по времени (с)
    dt: f64,
}

impl HeatTransfer2D {
    /// Создать новый решатель
    pub fn new(grid: Grid2D, dt: f64) -> Result<Self> {
        // Проверка устойчивости (критерий Куранта)
        let max_k = grid
            .thermal_conductivity
            .iter()
            .flat_map(|row| row.iter())
            .cloned()
            .fold(0.0, f64::max);

        let min_c = grid
            .heat_capacity
            .iter()
            .flat_map(|row| row.iter())
            .cloned()
            .fold(f64::INFINITY, f64::min);

        let alpha = max_k / min_c; // температуропроводность
        let dx2 = grid.dx.powi(2);
        let dz2 = grid.dz.powi(2);

        let dt_max = 0.25 * dx2.min(dz2) / alpha;

        if dt > dt_max {
            return Err(ThermokarstError::InvalidParameters(format!(
                "Шаг по времени {} с слишком велик. Максимум: {} с",
                dt, dt_max
            )));
        }

        Ok(Self { grid, dt })
    }

    /// Выполнить один шаг по времени (явная схема)
    pub fn step(&mut self) -> Result<()> {
        let mut new_temp = self.grid.temperature.clone();

        for iz in 1..self.grid.nz - 1 {
            for ix in 1..self.grid.nx - 1 {
                let t = self.grid.temperature[iz][ix];
                let c = self.grid.heat_capacity[iz][ix];

                // Теплопроводность на гранях (гармоническое среднее)
                let k_xp = 2.0
                    * self.grid.thermal_conductivity[iz][ix]
                    * self.grid.thermal_conductivity[iz][ix + 1]
                    / (self.grid.thermal_conductivity[iz][ix]
                        + self.grid.thermal_conductivity[iz][ix + 1]);

                let k_xm = 2.0
                    * self.grid.thermal_conductivity[iz][ix]
                    * self.grid.thermal_conductivity[iz][ix - 1]
                    / (self.grid.thermal_conductivity[iz][ix]
                        + self.grid.thermal_conductivity[iz][ix - 1]);

                let k_zp = 2.0
                    * self.grid.thermal_conductivity[iz][ix]
                    * self.grid.thermal_conductivity[iz + 1][ix]
                    / (self.grid.thermal_conductivity[iz][ix]
                        + self.grid.thermal_conductivity[iz + 1][ix]);

                let k_zm = 2.0
                    * self.grid.thermal_conductivity[iz][ix]
                    * self.grid.thermal_conductivity[iz - 1][ix]
                    / (self.grid.thermal_conductivity[iz][ix]
                        + self.grid.thermal_conductivity[iz - 1][ix]);

                // Потоки тепла
                let qx_plus = k_xp * (self.grid.temperature[iz][ix + 1] - t) / self.grid.dx;
                let qx_minus = k_xm * (t - self.grid.temperature[iz][ix - 1]) / self.grid.dx;

                let qz_plus = k_zp * (self.grid.temperature[iz + 1][ix] - t) / self.grid.dz;
                let qz_minus = k_zm * (t - self.grid.temperature[iz - 1][ix]) / self.grid.dz;

                // Дивергенция потока
                let div_q =
                    (qx_plus - qx_minus) / self.grid.dx + (qz_plus - qz_minus) / self.grid.dz;

                // Обновление температуры
                new_temp[iz][ix] = t + self.dt * div_q / c;
            }
        }

        self.grid.temperature = new_temp;
        Ok(())
    }

    /// Выполнить симуляцию на заданное время
    pub fn simulate(&mut self, total_time: f64) -> Result<()> {
        let n_steps = (total_time / self.dt).ceil() as usize;

        for _ in 0..n_steps {
            self.step()?;
        }

        Ok(())
    }

    /// Получить текущую сетку
    pub fn grid(&self) -> &Grid2D {
        &self.grid
    }

    /// Получить температуру в точке
    pub fn temperature_at(&self, ix: usize, iz: usize) -> Option<f64> {
        if ix < self.grid.nx && iz < self.grid.nz {
            Some(self.grid.temperature[iz][ix])
        } else {
            None
        }
    }

    /// Рассчитать среднюю температуру в области
    pub fn average_temperature(&self, x_range: (usize, usize), z_range: (usize, usize)) -> f64 {
        let mut sum = 0.0;
        let mut count = 0;

        for iz in z_range.0..z_range.1.min(self.grid.nz) {
            for ix in x_range.0..x_range.1.min(self.grid.nx) {
                sum += self.grid.temperature[iz][ix];
                count += 1;
            }
        }

        if count > 0 {
            sum / count as f64
        } else {
            0.0
        }
    }
}

/// Граничные условия
#[derive(Debug, Clone)]
pub enum BoundaryCondition {
    /// Фиксированная температура (°C)
    Dirichlet(f64),
    /// Фиксированный поток (Вт/м²)
    Neumann(f64),
    /// Изолированная граница (нулевой поток)
    Insulated,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_creation() {
        let grid = Grid2D::new(10, 20, 0.1, 0.1);

        assert_eq!(grid.nx, 10);
        assert_eq!(grid.nz, 20);
        assert_eq!(grid.temperature.len(), 20);
        assert_eq!(grid.temperature[0].len(), 10);
    }

    #[test]
    fn test_initial_conditions() {
        let mut grid = Grid2D::new(10, 10, 1.0, 1.0);

        // Линейный градиент по глубине
        grid.set_initial_temperature(|_x, z| 10.0 - z);

        assert!((grid.temperature[0][0] - 10.0).abs() < 1e-10);
        assert!((grid.temperature[5][0] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_heat_diffusion() {
        let mut grid = Grid2D::new(11, 11, 0.1, 0.1);

        // Начальное условие: горячая точка в центре
        grid.set_initial_temperature(|x, z| {
            let cx = 0.5;
            let cz = 0.5;
            let r2 = (x - cx).powi(2) + (z - cz).powi(2);
            if r2 < 0.01 {
                100.0
            } else {
                0.0
            }
        });

        let dt = 1.0; // 1 секунда
        let mut solver = HeatTransfer2D::new(grid, dt).unwrap();

        let t_initial = solver.temperature_at(5, 5).unwrap();

        // Симуляция 100 секунд
        solver.simulate(100.0).unwrap();

        let t_final = solver.temperature_at(5, 5).unwrap();

        // Температура в центре должна уменьшиться (диффузия)
        assert!(t_final < t_initial);

        println!("Initial center temp: {:.1}°C", t_initial);
        println!("Final center temp: {:.1}°C", t_final);
    }

    #[test]
    fn test_stability_check() {
        let grid = Grid2D::new(10, 10, 0.1, 0.1);

        // Слишком большой шаг по времени должен вызвать ошибку
        let dt_large = 10000.0;
        let result = HeatTransfer2D::new(grid, dt_large);

        // Проверяем что возвращается ошибка
        assert!(result.is_err());

        // Малый шаг должен работать
        let grid2 = Grid2D::new(10, 10, 0.1, 0.1);
        let dt_small = 0.1;
        let result2 = HeatTransfer2D::new(grid2, dt_small);
        assert!(result2.is_ok());
    }

    #[test]
    fn test_average_temperature() {
        let mut grid = Grid2D::new(10, 10, 1.0, 1.0);
        grid.set_initial_temperature(|_x, z| 10.0 - z);

        let solver = HeatTransfer2D::new(grid, 1.0).unwrap();

        // Средняя температура в верхней половине
        let avg = solver.average_temperature((0, 10), (0, 5));

        // Должна быть около 7.5°C (среднее между 10 и 5)
        assert!((avg - 7.5).abs() < 1.0);

        println!("Average temperature: {:.2}°C", avg);
    }
}
