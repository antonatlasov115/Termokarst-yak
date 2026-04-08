//! Базовый Newton-Raphson решатель для нелинейных систем
//!
//! Упрощенная версия, портированная из PFLOTRAN (solver.F90)
//! Использует nalgebra для линейной алгебры

use nalgebra as na;

/// Параметры Newton solver
#[derive(Debug, Clone, Copy)]
pub struct NewtonSolverParams {
    /// Абсолютная толерантность (absolute tolerance)
    pub atol: f64,
    /// Относительная толерантность (relative tolerance)
    pub rtol: f64,
    /// Толерантность на изменение (step tolerance)
    pub stol: f64,
    /// Максимальное число итераций
    pub max_iterations: usize,
    /// Минимальное число итераций
    pub min_iterations: usize,
    /// Использовать line search
    pub use_line_search: bool,
}

impl Default for NewtonSolverParams {
    fn default() -> Self {
        Self {
            atol: 1e-8,
            rtol: 1e-8,
            stol: 1e-8,
            max_iterations: 50,
            min_iterations: 1,
            use_line_search: true,
        }
    }
}

/// Результат Newton solver
#[derive(Debug, Clone)]
pub struct NewtonSolverResult {
    /// Решение
    pub solution: na::DVector<f64>,
    /// Число итераций
    pub iterations: usize,
    /// Финальная невязка
    pub residual_norm: f64,
    /// Сошелся ли метод
    pub converged: bool,
    /// Причина остановки
    pub stop_reason: StopReason,
}

/// Причина остановки итераций
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    /// Достигнута абсолютная толерантность
    AbsoluteTolerance,
    /// Достигнута относительная толерантность
    RelativeTolerance,
    /// Достигнута толерантность на изменение
    StepTolerance,
    /// Достигнуто максимальное число итераций
    MaxIterations,
    /// Расходимость
    Divergence,
}

/// Базовый Newton-Raphson решатель
pub struct NewtonSolver {
    params: NewtonSolverParams,
}

impl NewtonSolver {
    /// Создать новый решатель
    pub fn new(params: NewtonSolverParams) -> Self {
        Self { params }
    }

    /// Решить нелинейную систему F(x) = 0
    ///
    /// # Аргументы
    /// * `x0` - начальное приближение
    /// * `residual_fn` - функция вычисления невязки F(x)
    /// * `jacobian_fn` - функция вычисления якобиана J(x) = dF/dx
    ///
    /// # Возвращает
    /// Результат решения
    pub fn solve<F, J>(
        &self,
        x0: na::DVector<f64>,
        mut residual_fn: F,
        mut jacobian_fn: J,
    ) -> NewtonSolverResult
    where
        F: FnMut(&na::DVector<f64>) -> na::DVector<f64>,
        J: FnMut(&na::DVector<f64>) -> na::DMatrix<f64>,
    {
        let mut x = x0.clone();
        let mut residual = residual_fn(&x);
        let mut residual_norm = residual.norm();
        let initial_residual_norm = residual_norm;

        for iter in 0..self.params.max_iterations {
            // Проверка сходимости
            if iter >= self.params.min_iterations {
                // Абсолютная толерантность
                if residual_norm < self.params.atol {
                    return NewtonSolverResult {
                        solution: x,
                        iterations: iter,
                        residual_norm,
                        converged: true,
                        stop_reason: StopReason::AbsoluteTolerance,
                    };
                }

                // Относительная толерантность
                if residual_norm < self.params.rtol * initial_residual_norm {
                    return NewtonSolverResult {
                        solution: x,
                        iterations: iter,
                        residual_norm,
                        converged: true,
                        stop_reason: StopReason::RelativeTolerance,
                    };
                }
            }

            // Вычислить якобиан
            let jacobian = jacobian_fn(&x);

            // Решить линейную систему J * dx = -F
            let dx = match self.solve_linear_system(&jacobian, &(-&residual)) {
                Some(dx) => dx,
                None => {
                    // Не удалось решить линейную систему
                    return NewtonSolverResult {
                        solution: x,
                        iterations: iter,
                        residual_norm,
                        converged: false,
                        stop_reason: StopReason::Divergence,
                    };
                }
            };

            // Line search (опционально)
            let alpha = if self.params.use_line_search {
                self.line_search(&x, &dx, &mut residual_fn, residual_norm)
            } else {
                1.0
            };

            // Обновить решение
            let x_new = &x + alpha * &dx;

            // Проверка толерантности на изменение
            let dx_norm = dx.norm();
            let x_norm = x.norm();
            if dx_norm < self.params.stol * (1.0 + x_norm) && iter >= self.params.min_iterations {
                return NewtonSolverResult {
                    solution: x_new,
                    iterations: iter + 1,
                    residual_norm,
                    converged: true,
                    stop_reason: StopReason::StepTolerance,
                };
            }

            // Обновить для следующей итерации
            x = x_new;
            residual = residual_fn(&x);
            let new_residual_norm = residual.norm();

            // Проверка расходимости
            if new_residual_norm > 1e10 * initial_residual_norm {
                return NewtonSolverResult {
                    solution: x,
                    iterations: iter + 1,
                    residual_norm: new_residual_norm,
                    converged: false,
                    stop_reason: StopReason::Divergence,
                };
            }

            residual_norm = new_residual_norm;
        }

        // Достигнуто максимальное число итераций
        NewtonSolverResult {
            solution: x,
            iterations: self.params.max_iterations,
            residual_norm,
            converged: false,
            stop_reason: StopReason::MaxIterations,
        }
    }

    /// Решить линейную систему A * x = b
    fn solve_linear_system(
        &self,
        a: &na::DMatrix<f64>,
        b: &na::DVector<f64>,
    ) -> Option<na::DVector<f64>> {
        // Используем LU разложение
        let lu = a.clone().lu();
        lu.solve(b)
    }

    /// Line search для выбора оптимального шага
    ///
    /// Возвращает коэффициент alpha в диапазоне (0, 1]
    fn line_search<F>(
        &self,
        x: &na::DVector<f64>,
        dx: &na::DVector<f64>,
        residual_fn: &mut F,
        current_norm: f64,
    ) -> f64
    where
        F: FnMut(&na::DVector<f64>) -> na::DVector<f64>,
    {
        let mut alpha = 1.0;
        let c = 1e-4; // Параметр Armijo
        let rho = 0.5; // Коэффициент уменьшения шага

        for _ in 0..10 {
            let x_new = x + alpha * dx;
            let residual_new = residual_fn(&x_new);
            let new_norm = residual_new.norm();

            // Условие Armijo
            if new_norm <= (1.0 - c * alpha) * current_norm {
                return alpha;
            }

            alpha *= rho;

            if alpha < 1e-8 {
                return alpha;
            }
        }

        alpha
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_system() {
        // Решить x^2 - 4 = 0, x0 = 1
        // F(x) = x^2 - 4
        // J(x) = 2x

        let params = NewtonSolverParams {
            atol: 1e-10,
            rtol: 1e-10,
            stol: 1e-10,
            max_iterations: 20,
            min_iterations: 1,
            use_line_search: false,
        };

        let solver = NewtonSolver::new(params);

        let x0 = na::DVector::from_vec(vec![1.0]);

        let residual_fn = |x: &na::DVector<f64>| na::DVector::from_vec(vec![x[0] * x[0] - 4.0]);

        let jacobian_fn = |x: &na::DVector<f64>| na::DMatrix::from_vec(1, 1, vec![2.0 * x[0]]);

        let result = solver.solve(x0, residual_fn, jacobian_fn);

        assert!(result.converged);
        assert!((result.solution[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_system_2d() {
        // Решить систему:
        // x^2 + y^2 - 1 = 0
        // x - y = 0
        // Решение: x = y = 1/sqrt(2)

        let params = NewtonSolverParams::default();
        let solver = NewtonSolver::new(params);

        let x0 = na::DVector::from_vec(vec![0.5, 0.5]);

        let residual_fn = |x: &na::DVector<f64>| {
            na::DVector::from_vec(vec![x[0] * x[0] + x[1] * x[1] - 1.0, x[0] - x[1]])
        };

        let jacobian_fn = |x: &na::DVector<f64>| {
            na::DMatrix::from_vec(2, 2, vec![2.0 * x[0], x[0], 2.0 * x[1], -x[1]])
        };

        let result = solver.solve(x0, residual_fn, jacobian_fn);

        assert!(result.converged);
        let expected = 1.0 / 2.0_f64.sqrt();
        assert!((result.solution[0] - expected).abs() < 1e-6);
        assert!((result.solution[1] - expected).abs() < 1e-6);
    }

    #[test]
    fn test_with_line_search() {
        let params = NewtonSolverParams {
            use_line_search: true,
            ..Default::default()
        };

        let solver = NewtonSolver::new(params);

        let x0 = na::DVector::from_vec(vec![10.0]); // Плохое начальное приближение

        let residual_fn = |x: &na::DVector<f64>| na::DVector::from_vec(vec![x[0] * x[0] - 4.0]);

        let jacobian_fn = |x: &na::DVector<f64>| na::DMatrix::from_vec(1, 1, vec![2.0 * x[0]]);

        let result = solver.solve(x0, residual_fn, jacobian_fn);

        assert!(result.converged);
        assert!((result.solution[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_max_iterations() {
        let params = NewtonSolverParams {
            max_iterations: 2,
            ..Default::default()
        };

        let solver = NewtonSolver::new(params);

        let x0 = na::DVector::from_vec(vec![0.1]);

        let residual_fn = |x: &na::DVector<f64>| na::DVector::from_vec(vec![x[0] * x[0] - 4.0]);

        let jacobian_fn = |x: &na::DVector<f64>| na::DMatrix::from_vec(1, 1, vec![2.0 * x[0]]);

        let result = solver.solve(x0, residual_fn, jacobian_fn);

        assert!(!result.converged);
        assert_eq!(result.stop_reason, StopReason::MaxIterations);
    }
}
