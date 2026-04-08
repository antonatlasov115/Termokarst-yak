//! Пример использования Newton solver для решения Richards equation
//!
//! Демонстрирует решение 1D задачи инфильтрации воды в ненасыщенную зону

use nalgebra as na;
use thermokarst_physics::{
    MaterialProperties, NewtonSolver, NewtonSolverParams, RichardsAuxVar, RichardsCalculator,
    RichardsParameters,
};

/// Простая 1D сетка для Richards equation
struct Grid1D {
    /// Число ячеек
    n_cells: usize,
    /// Размер ячейки (m)
    cell_size: f64,
    /// Площадь поперечного сечения (m²)
    area: f64,
}

impl Grid1D {
    fn new(n_cells: usize, length: f64, area: f64) -> Self {
        Self {
            n_cells,
            cell_size: length / n_cells as f64,
            area,
        }
    }
}

/// Решатель Richards equation с использованием Newton solver
struct RichardsNewtonSolver {
    grid: Grid1D,
    richards_calc: RichardsCalculator,
    newton_solver: NewtonSolver,
    materials: Vec<MaterialProperties>,
    auxvars: Vec<RichardsAuxVar>,
}

impl RichardsNewtonSolver {
    fn new(
        grid: Grid1D,
        richards_params: RichardsParameters,
        newton_params: NewtonSolverParams,
        materials: Vec<MaterialProperties>,
        initial_auxvars: Vec<RichardsAuxVar>,
    ) -> Self {
        assert_eq!(materials.len(), grid.n_cells);
        assert_eq!(initial_auxvars.len(), grid.n_cells);

        Self {
            grid,
            richards_calc: RichardsCalculator::new(richards_params),
            newton_solver: NewtonSolver::new(newton_params),
            materials,
            auxvars: initial_auxvars,
        }
    }

    /// Обновить вспомогательные переменные на основе давления
    fn update_auxvars(&mut self, pressures: &na::DVector<f64>) {
        for i in 0..self.grid.n_cells {
            self.auxvars[i].pressure = pressures[i];
            // В упрощенной версии используем линейные зависимости
            self.update_auxvar_properties(i);
        }
    }

    /// Обновить свойства для одной ячейки (упрощенная модель)
    fn update_auxvar_properties(&mut self, i: usize) {
        let auxvar = &mut self.auxvars[i];

        // Упрощенная модель: насыщенность зависит от давления
        // S = 1 для P > 0, S = exp(P/P0) для P < 0
        const P0: f64 = 1e4; // Характерное давление

        if auxvar.pressure >= 0.0 {
            auxvar.saturation = 1.0;
            auxvar.dsat_dp = 0.0;
        } else {
            auxvar.saturation = (auxvar.pressure / P0).exp();
            auxvar.dsat_dp = auxvar.saturation / P0;
        }

        // Относительная проницаемость (модель Corey)
        let s_eff = auxvar.saturation;
        auxvar.relative_permeability = s_eff.powi(3);
        auxvar.dkvr_dp = if auxvar.pressure >= 0.0 {
            0.0
        } else {
            3.0 * s_eff.powi(2) * auxvar.dsat_dp
        };

        // kvr = k/μ * kr
        auxvar.kvr = auxvar.relative_permeability / auxvar.viscosity;
        auxvar.dkvr_dp = auxvar.dkvr_dp / auxvar.viscosity;

        // Плотность (слабо сжимаемая жидкость)
        const COMPRESSIBILITY: f64 = 4.5e-10; // 1/Pa
        auxvar.density = 1000.0 * (1.0 + COMPRESSIBILITY * auxvar.pressure);
        auxvar.dden_dp = 1000.0 * COMPRESSIBILITY;
    }

    /// Вычислить невязку F(P) = 0
    fn compute_residual(&mut self, pressures: &na::DVector<f64>) -> na::DVector<f64> {
        self.update_auxvars(pressures);
        self.compute_residual_from_auxvars(&self.auxvars.clone())
    }

    /// Вычислить невязку из готовых auxvars
    fn compute_residual_from_auxvars(&self, auxvars: &[RichardsAuxVar]) -> na::DVector<f64> {
        let mut residual = na::DVector::zeros(self.grid.n_cells);

        for i in 0..self.grid.n_cells {
            // Член аккумуляции
            let accum = self
                .richards_calc
                .accumulation(&auxvars[i], &self.materials[i]);
            residual[i] = accum;

            // Потоки между ячейками
            if i > 0 {
                // Поток сверху
                let distance = [
                    self.grid.cell_size / 2.0,
                    self.grid.cell_size / 2.0,
                    self.grid.cell_size,
                ];
                let flux = self.richards_calc.flux(
                    &auxvars[i - 1],
                    &self.materials[i - 1],
                    &auxvars[i],
                    &self.materials[i],
                    self.grid.area,
                    &distance,
                );
                residual[i] -= flux;
            } else {
                // Верхняя граница: инфильтрация
                let infiltration_rate = 0.001; // kg/m²/s
                let boundary_flux = self
                    .richards_calc
                    .boundary_flux_neumann(infiltration_rate, self.grid.area);
                residual[i] -= boundary_flux;
            }

            if i < self.grid.n_cells - 1 {
                // Поток снизу
                let distance = [
                    self.grid.cell_size / 2.0,
                    self.grid.cell_size / 2.0,
                    self.grid.cell_size,
                ];
                let flux = self.richards_calc.flux(
                    &auxvars[i],
                    &self.materials[i],
                    &auxvars[i + 1],
                    &self.materials[i + 1],
                    self.grid.area,
                    &distance,
                );
                residual[i] += flux;
            } else {
                // Нижняя граница: свободный дренаж
                // Поток = -K * kr * ρ * g
                let gravity_flux = -self.materials[i].permeability
                    * auxvars[i].relative_permeability
                    / auxvars[i].viscosity
                    * auxvars[i].density
                    * self.richards_calc.params.gravity
                    * self.grid.area;
                residual[i] += gravity_flux;
            }
        }

        residual
    }

    /// Вычислить якобиан J = dF/dP (численно)
    fn compute_jacobian(&mut self, pressures: &na::DVector<f64>) -> na::DMatrix<f64> {
        let n = self.grid.n_cells;
        let mut jacobian = na::DMatrix::zeros(n, n);
        let eps = 1e-6;

        let f0 = self.compute_residual(pressures);

        for j in 0..n {
            let mut p_perturbed = pressures.clone();
            p_perturbed[j] += eps;
            let f_perturbed = self.compute_residual(&p_perturbed);

            for i in 0..n {
                jacobian[(i, j)] = (f_perturbed[i] - f0[i]) / eps;
            }
        }

        jacobian
    }

    /// Решить систему на один временной шаг
    fn solve_timestep(&mut self, initial_pressures: na::DVector<f64>) -> na::DVector<f64> {
        // Клонируем необходимые данные для замыканий
        let grid = Grid1D {
            n_cells: self.grid.n_cells,
            cell_size: self.grid.cell_size,
            area: self.grid.area,
        };
        let richards_calc = RichardsCalculator::new(self.richards_calc.params);
        let materials = self.materials.clone();
        let mut auxvars = self.auxvars.clone();

        let residual_fn = move |p: &na::DVector<f64>| {
            // Обновить auxvars
            for i in 0..grid.n_cells {
                auxvars[i].pressure = p[i];
                update_auxvar_properties(&mut auxvars[i]);
            }

            let mut residual = na::DVector::zeros(grid.n_cells);

            for i in 0..grid.n_cells {
                let accum = richards_calc.accumulation(&auxvars[i], &materials[i]);
                residual[i] = accum;

                if i > 0 {
                    let distance = [grid.cell_size / 2.0, grid.cell_size / 2.0, grid.cell_size];
                    let flux = richards_calc.flux(
                        &auxvars[i - 1],
                        &materials[i - 1],
                        &auxvars[i],
                        &materials[i],
                        grid.area,
                        &distance,
                    );
                    residual[i] -= flux;
                } else {
                    let infiltration_rate = 0.001;
                    let boundary_flux =
                        richards_calc.boundary_flux_neumann(infiltration_rate, grid.area);
                    residual[i] -= boundary_flux;
                }

                if i < grid.n_cells - 1 {
                    let distance = [grid.cell_size / 2.0, grid.cell_size / 2.0, grid.cell_size];
                    let flux = richards_calc.flux(
                        &auxvars[i],
                        &materials[i],
                        &auxvars[i + 1],
                        &materials[i + 1],
                        grid.area,
                        &distance,
                    );
                    residual[i] += flux;
                } else {
                    let gravity_flux = -materials[i].permeability
                        * auxvars[i].relative_permeability
                        / auxvars[i].viscosity
                        * auxvars[i].density
                        * richards_calc.params.gravity
                        * grid.area;
                    residual[i] += gravity_flux;
                }
            }

            residual
        };

        // Используем численное дифференцирование для якобиана
        let result =
            self.newton_solver
                .solve(initial_pressures, residual_fn, |p: &na::DVector<f64>| {
                    // Численный якобиан
                    let n = self.grid.n_cells;
                    let mut jacobian = na::DMatrix::zeros(n, n);
                    let eps = 1e-6;

                    // Вычисляем невязку в текущей точке
                    let mut auxvars_base = self.auxvars.clone();
                    for i in 0..n {
                        auxvars_base[i].pressure = p[i];
                        update_auxvar_properties(&mut auxvars_base[i]);
                    }
                    let f0 = self.compute_residual_from_auxvars(&auxvars_base);

                    // Вычисляем производные
                    for j in 0..n {
                        let mut auxvars_pert = auxvars_base.clone();
                        auxvars_pert[j].pressure += eps;
                        update_auxvar_properties(&mut auxvars_pert[j]);
                        let f_pert = self.compute_residual_from_auxvars(&auxvars_pert);

                        for i in 0..n {
                            jacobian[(i, j)] = (f_pert[i] - f0[i]) / eps;
                        }
                    }

                    jacobian
                });

        println!("Newton solver:");
        println!("  Итераций: {}", result.iterations);
        println!("  Сошелся: {}", result.converged);
        println!("  Невязка: {:.2e}", result.residual_norm);
        println!("  Причина остановки: {:?}", result.stop_reason);

        result.solution
    }
}

/// Обновить свойства для одной ячейки (упрощенная модель)
fn update_auxvar_properties(auxvar: &mut RichardsAuxVar) {
    // Упрощенная модель: насыщенность зависит от давления
    // S = 1 для P > 0, S = exp(P/P0) для P < 0
    const P0: f64 = 1e4; // Характерное давление

    if auxvar.pressure >= 0.0 {
        auxvar.saturation = 1.0;
        auxvar.dsat_dp = 0.0;
    } else {
        auxvar.saturation = (auxvar.pressure / P0).exp();
        auxvar.dsat_dp = auxvar.saturation / P0;
    }

    // Относительная проницаемость (модель Corey)
    let s_eff = auxvar.saturation;
    auxvar.relative_permeability = s_eff.powi(3);
    auxvar.dkvr_dp = if auxvar.pressure >= 0.0 {
        0.0
    } else {
        3.0 * s_eff.powi(2) * auxvar.dsat_dp
    };

    // kvr = k/μ * kr
    auxvar.kvr = auxvar.relative_permeability / auxvar.viscosity;
    auxvar.dkvr_dp = auxvar.dkvr_dp / auxvar.viscosity;

    // Плотность (слабо сжимаемая жидкость)
    const COMPRESSIBILITY: f64 = 4.5e-10; // 1/Pa
    auxvar.density = 1000.0 * (1.0 + COMPRESSIBILITY * auxvar.pressure);
    auxvar.dden_dp = 1000.0 * COMPRESSIBILITY;
}

fn main() {
    println!("=== Пример: Newton solver для Richards equation ===\n");

    // Параметры сетки
    let n_cells = 10;
    let length = 1.0; // 1 метр
    let area = 1.0; // 1 м²
    let grid = Grid1D::new(n_cells, length, area);

    // Параметры Richards
    let richards_params = RichardsParameters {
        dt: 3600.0, // 1 час
        ..Default::default()
    };

    // Параметры Newton solver
    let newton_params = NewtonSolverParams {
        atol: 1e-6,
        rtol: 1e-6,
        max_iterations: 20,
        use_line_search: true,
        ..Default::default()
    };

    // Материалы (однородная почва)
    let materials: Vec<_> = (0..n_cells)
        .map(|_| MaterialProperties {
            permeability: 1e-12, // 1 Darcy ≈ 1e-12 m²
            porosity: 0.4,
            volume: grid.cell_size * area,
        })
        .collect();

    // Начальные условия (ненасыщенная зона)
    let initial_auxvars: Vec<_> = (0..n_cells)
        .map(|_| RichardsAuxVar {
            pressure: -5000.0, // Отрицательное давление (ненасыщенная зона)
            saturation: 0.5,
            density: 1000.0,
            viscosity: 1e-3,
            relative_permeability: 0.125, // S^3 = 0.5^3
            kvr: 0.125 / 1e-3,
            effective_porosity: 0.4,
            ..RichardsAuxVar::new()
        })
        .collect();

    // Начальные давления
    let initial_pressures =
        na::DVector::from_vec(initial_auxvars.iter().map(|av| av.pressure).collect());

    println!("Параметры задачи:");
    println!("  Число ячеек: {}", n_cells);
    println!("  Длина: {} м", length);
    println!("  Размер ячейки: {:.3} м", grid.cell_size);
    println!(
        "  Временной шаг: {} с ({} ч)",
        richards_params.dt,
        richards_params.dt / 3600.0
    );
    println!("  Проницаемость: {:.2e} м²", materials[0].permeability);
    println!("  Пористость: {}", materials[0].porosity);
    println!("\nНачальные условия:");
    println!("  Давление: {:.1} Па", initial_pressures[0]);
    println!("  Насыщенность: {}", initial_auxvars[0].saturation);
    println!("\nГраничные условия:");
    println!("  Верх: инфильтрация 0.001 kg/m²/s");
    println!("  Низ: свободный дренаж\n");

    // Создать решатель
    let mut solver = RichardsNewtonSolver::new(
        grid,
        richards_params,
        newton_params,
        materials,
        initial_auxvars,
    );

    // Решить один временной шаг
    println!("Решение системы...\n");
    let solution = solver.solve_timestep(initial_pressures.clone());

    println!("\nРезультаты:");
    println!("  Глубина (м)  | Давление (Па) | Изменение (Па)");
    println!("  -------------|---------------|----------------");
    for i in 0..n_cells {
        let depth = (i as f64 + 0.5) * solver.grid.cell_size;
        let p_new = solution[i];
        let p_old = initial_pressures[i];
        let dp = p_new - p_old;
        println!("  {:12.3} | {:13.1} | {:14.1}", depth, p_new, dp);
    }

    println!("\n=== Пример завершен ===");
}
