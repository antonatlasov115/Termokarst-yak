//! Модуль Richards equation для моделирования потоков воды в ненасыщенной зоне
//!
//! Портировано из PFLOTRAN (richards.F90, richards_common.F90)
//! Реализует уравнение Ричардса для однофазного потока воды

use nalgebra as na;

/// Параметры материала для уравнения Ричардса
#[derive(Debug, Clone, Copy)]
pub struct MaterialProperties {
    /// Проницаемость (m²)
    pub permeability: f64,
    /// Пористость (безразмерная, 0-1)
    pub porosity: f64,
    /// Объем ячейки (m³)
    pub volume: f64,
}

/// Вспомогательные переменные для Richards
#[derive(Debug, Clone, Copy)]
pub struct RichardsAuxVar {
    /// Давление жидкости (Pa)
    pub pressure: f64,
    /// Насыщенность (0-1)
    pub saturation: f64,
    /// Плотность жидкости (kg/m³)
    pub density: f64,
    /// Вязкость жидкости (Pa·s)
    pub viscosity: f64,
    /// Относительная проницаемость (0-1)
    pub relative_permeability: f64,
    /// k/μ * kr (проницаемость/вязкость * относительная проницаемость)
    pub kvr: f64,
    /// Производная плотности по давлению
    pub dden_dp: f64,
    /// Производная насыщенности по давлению
    pub dsat_dp: f64,
    /// Производная относительной проницаемости по давлению
    pub dkvr_dp: f64,
    /// Эффективная пористость
    pub effective_porosity: f64,
    /// Производная пористости по давлению
    pub dpor_dp: f64,
}

impl RichardsAuxVar {
    /// Создать новые вспомогательные переменные
    pub fn new() -> Self {
        Self {
            pressure: 1.0e5,
            saturation: 1.0,
            density: 1000.0,
            viscosity: 1.0e-3,
            relative_permeability: 1.0,
            kvr: 0.0,
            dden_dp: 0.0,
            dsat_dp: 0.0,
            dkvr_dp: 0.0,
            effective_porosity: 0.3,
            dpor_dp: 0.0,
        }
    }
}

/// Параметры для расчета Richards
#[derive(Debug, Clone, Copy)]
pub struct RichardsParameters {
    /// Референсное давление (Pa)
    pub reference_pressure: f64,
    /// Ускорение свободного падения (m/s²)
    pub gravity: f64,
    /// Временной шаг (s)
    pub dt: f64,
    /// Коэффициент перевода плотности kmol/m³ -> kg/m³
    pub density_kmol_to_kg: f64,
}

impl Default for RichardsParameters {
    fn default() -> Self {
        Self {
            reference_pressure: 1.0e5,
            gravity: 9.81,
            dt: 1.0,
            density_kmol_to_kg: 18.015, // Молекулярная масса воды
        }
    }
}

/// Калькулятор уравнения Ричардса
pub struct RichardsCalculator {
    pub params: RichardsParameters,
}

impl RichardsCalculator {
    /// Создать новый калькулятор
    pub fn new(params: RichardsParameters) -> Self {
        Self { params }
    }

    /// Рассчитать член аккумуляции (накопления)
    ///
    /// Accumulation = φ * S * ρ * V / Δt
    ///
    /// где:
    /// - φ - пористость
    /// - S - насыщенность
    /// - ρ - плотность
    /// - V - объем
    /// - Δt - временной шаг
    pub fn accumulation(&self, auxvar: &RichardsAuxVar, material: &MaterialProperties) -> f64 {
        let vol_over_dt = material.volume / self.params.dt;
        auxvar.saturation * auxvar.density * auxvar.effective_porosity * vol_over_dt
    }

    /// Рассчитать производную члена аккумуляции по давлению
    ///
    /// dAccum/dP = (dφ/dP * S * ρ + φ * (S * dρ/dP + dS/dP * ρ)) * V / Δt
    pub fn accumulation_derivative(
        &self,
        auxvar: &RichardsAuxVar,
        material: &MaterialProperties,
    ) -> f64 {
        let vol_over_dt = material.volume / self.params.dt;

        (auxvar.dpor_dp * auxvar.saturation * auxvar.density
            + (auxvar.saturation * auxvar.dden_dp + auxvar.dsat_dp * auxvar.density)
                * auxvar.effective_porosity)
            * vol_over_dt
    }

    /// Рассчитать поток Дарси между двумя ячейками
    ///
    /// q = -K * kr * (∇P - ρg∇z)
    ///
    /// где:
    /// - K - проницаемость
    /// - kr - относительная проницаемость
    /// - P - давление
    /// - ρ - плотность
    /// - g - гравитация
    /// - z - высота
    pub fn flux(
        &self,
        auxvar_up: &RichardsAuxVar,
        material_up: &MaterialProperties,
        auxvar_dn: &RichardsAuxVar,
        material_dn: &MaterialProperties,
        area: f64,
        distance: &[f64; 3], // [dist_up, dist_dn, dist_gravity]
    ) -> f64 {
        const EPS: f64 = 1e-8;
        const FLOW_EPS: f64 = 1e-24;

        let dd_up = distance[0];
        let dd_dn = distance[1];
        let dist_gravity = distance[2];

        // Гармоническое среднее проницаемости
        let perm_up = material_up.permeability;
        let perm_dn = material_dn.permeability;
        let dq = (perm_up * perm_dn) / (dd_up * perm_dn + dd_dn * perm_up);

        // Проверка наличия потока
        if auxvar_up.kvr <= EPS && auxvar_dn.kvr <= EPS {
            return 0.0;
        }

        // Upwind схема для насыщенности
        let upweight = if auxvar_up.saturation < EPS {
            0.0
        } else if auxvar_dn.saturation < EPS {
            1.0
        } else {
            // Стандартное upwind взвешивание
            0.5
        };

        // Средняя плотность
        let density_ave = upweight * auxvar_up.density + (1.0 - upweight) * auxvar_dn.density;

        // Гравитационный член
        let gravity = (upweight * auxvar_up.density + (1.0 - upweight) * auxvar_dn.density)
            * self.params.density_kmol_to_kg
            * dist_gravity;

        // Разность потенциалов
        let dphi = auxvar_up.pressure - auxvar_dn.pressure + gravity;

        // Upwind для относительной проницаемости
        let ukvr = if dphi >= 0.0 {
            auxvar_up.kvr
        } else {
            auxvar_dn.kvr
        };

        if ukvr <= FLOW_EPS {
            return 0.0;
        }

        // Скорость Дарси
        let v_darcy = dq * ukvr * dphi;

        // Массовый поток
        v_darcy * area * density_ave
    }

    /// Рассчитать производные потока по давлению
    ///
    /// Возвращает (dq/dP_up, dq/dP_dn)
    pub fn flux_derivatives(
        &self,
        auxvar_up: &RichardsAuxVar,
        material_up: &MaterialProperties,
        auxvar_dn: &RichardsAuxVar,
        material_dn: &MaterialProperties,
        area: f64,
        distance: &[f64; 3],
    ) -> (f64, f64) {
        const EPS: f64 = 1e-8;
        const FLOW_EPS: f64 = 1e-24;

        let dd_up = distance[0];
        let dd_dn = distance[1];
        let dist_gravity = distance[2];

        let perm_up = material_up.permeability;
        let perm_dn = material_dn.permeability;
        let dq = (perm_up * perm_dn) / (dd_up * perm_dn + dd_dn * perm_up);

        if auxvar_up.kvr <= EPS && auxvar_dn.kvr <= EPS {
            return (0.0, 0.0);
        }

        let upweight = if auxvar_up.saturation < EPS {
            0.0
        } else if auxvar_dn.saturation < EPS {
            1.0
        } else {
            0.5
        };

        let density_ave = upweight * auxvar_up.density + (1.0 - upweight) * auxvar_dn.density;
        let dden_ave_dp_up = upweight * auxvar_up.dden_dp;
        let dden_ave_dp_dn = (1.0 - upweight) * auxvar_dn.dden_dp;

        let dgravity_dden_up = upweight * self.params.density_kmol_to_kg * dist_gravity;
        let dgravity_dden_dn = (1.0 - upweight) * self.params.density_kmol_to_kg * dist_gravity;

        let gravity = (upweight * auxvar_up.density + (1.0 - upweight) * auxvar_dn.density)
            * self.params.density_kmol_to_kg
            * dist_gravity;

        let dphi = auxvar_up.pressure - auxvar_dn.pressure + gravity;
        let dphi_dp_up = 1.0 + dgravity_dden_up * auxvar_up.dden_dp;
        let dphi_dp_dn = -1.0 + dgravity_dden_dn * auxvar_dn.dden_dp;

        let (ukvr, dukvr_dp_up, dukvr_dp_dn) = if dphi >= 0.0 {
            (auxvar_up.kvr, auxvar_up.dkvr_dp, 0.0)
        } else {
            (auxvar_dn.kvr, 0.0, auxvar_dn.dkvr_dp)
        };

        if ukvr <= FLOW_EPS {
            return (0.0, 0.0);
        }

        let v_darcy = dq * ukvr * dphi;
        let q = v_darcy * area;

        let dq_dp_up = dq * (dukvr_dp_up * dphi + ukvr * dphi_dp_up) * area;
        let dq_dp_dn = dq * (dukvr_dp_dn * dphi + ukvr * dphi_dp_dn) * area;

        let jup = dq_dp_up * density_ave + q * dden_ave_dp_up;
        let jdn = dq_dp_dn * density_ave + q * dden_ave_dp_dn;

        (jup, jdn)
    }

    /// Рассчитать граничный поток (Dirichlet BC)
    ///
    /// Для граничных условий с заданным давлением
    pub fn boundary_flux_dirichlet(
        &self,
        auxvar_internal: &RichardsAuxVar,
        material_internal: &MaterialProperties,
        pressure_boundary: f64,
        area: f64,
        distance: f64,
    ) -> f64 {
        const FLOW_EPS: f64 = 1e-24;

        let perm = material_internal.permeability;
        let dq = perm / distance;

        if auxvar_internal.kvr <= FLOW_EPS {
            return 0.0;
        }

        let dphi = auxvar_internal.pressure - pressure_boundary;
        let v_darcy = dq * auxvar_internal.kvr * dphi;

        v_darcy * area * auxvar_internal.density
    }

    /// Рассчитать граничный поток (Neumann BC)
    ///
    /// Для граничных условий с заданным потоком
    ///
    /// # Аргументы
    /// * `flux` - заданный поток (kg/m²/s)
    /// * `area` - площадь границы (m²)
    ///
    /// # Возвращает
    /// Массовый поток (kg/s)
    pub fn boundary_flux_neumann(&self, flux: f64, area: f64) -> f64 {
        flux * area
    }

    /// Применить источник/сток массы
    ///
    /// # Аргументы
    /// * `rate` - скорость источника/стока (kg/s)
    ///   Положительное значение - источник (добавление воды)
    ///   Отрицательное значение - сток (удаление воды)
    ///
    /// # Возвращает
    /// Массовый поток (kg/s)
    pub fn apply_source_sink(&self, rate: f64) -> f64 {
        rate
    }

    /// Применить объемный источник/сток
    ///
    /// # Аргументы
    /// * `volumetric_rate` - объемный расход (m³/s)
    /// * `density` - плотность жидкости (kg/m³)
    ///
    /// # Возвращает
    /// Массовый поток (kg/s)
    pub fn apply_volumetric_source_sink(&self, volumetric_rate: f64, density: f64) -> f64 {
        volumetric_rate * density
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accumulation() {
        let params = RichardsParameters::default();
        let calc = RichardsCalculator::new(params);

        let auxvar = RichardsAuxVar {
            saturation: 0.8,
            density: 1000.0,
            effective_porosity: 0.3,
            ..RichardsAuxVar::new()
        };

        let material = MaterialProperties {
            permeability: 1e-12,
            porosity: 0.3,
            volume: 1.0,
        };

        let accum = calc.accumulation(&auxvar, &material);

        // Accumulation = 0.8 * 1000 * 0.3 * 1.0 / 1.0 = 240
        assert!((accum - 240.0).abs() < 1e-6);
    }

    #[test]
    fn test_flux_saturated() {
        let params = RichardsParameters::default();
        let calc = RichardsCalculator::new(params);

        let mut auxvar_up = RichardsAuxVar::new();
        auxvar_up.pressure = 1.1e5;
        auxvar_up.saturation = 1.0;
        auxvar_up.density = 1000.0;
        auxvar_up.kvr = 1e-6; // k/μ * kr (увеличено для видимого потока)

        let mut auxvar_dn = RichardsAuxVar::new();
        auxvar_dn.pressure = 1.0e5;
        auxvar_dn.saturation = 1.0;
        auxvar_dn.density = 1000.0;
        auxvar_dn.kvr = 1e-6;

        let material = MaterialProperties {
            permeability: 1e-12,
            porosity: 0.3,
            volume: 1.0,
        };

        let area = 1.0;
        let distance = [0.5, 0.5, 0.0]; // без гравитации

        let flux = calc.flux(
            &auxvar_up, &material, &auxvar_dn, &material, area, &distance,
        );

        // Поток должен быть положительным (от up к dn)
        assert!(flux > 0.0);
    }

    #[test]
    fn test_flux_with_gravity() {
        let params = RichardsParameters::default();
        let calc = RichardsCalculator::new(params);

        let mut auxvar_up = RichardsAuxVar::new();
        auxvar_up.pressure = 1.0e5;
        auxvar_up.saturation = 1.0;
        auxvar_up.density = 1000.0;
        auxvar_up.kvr = 1e-6;

        let mut auxvar_dn = RichardsAuxVar::new();
        auxvar_dn.pressure = 1.0e5;
        auxvar_dn.saturation = 1.0;
        auxvar_dn.density = 1000.0;
        auxvar_dn.kvr = 1e-6;

        let material = MaterialProperties {
            permeability: 1e-12,
            porosity: 0.3,
            volume: 1.0,
        };

        let area = 1.0;
        let distance = [0.5, 0.5, 1.0]; // гравитация вниз (положительное значение)

        let flux = calc.flux(
            &auxvar_up, &material, &auxvar_dn, &material, area, &distance,
        );

        // С гравитацией поток должен быть вниз (положительный)
        assert!(flux > 0.0);
    }

    #[test]
    fn test_flux_derivatives() {
        let params = RichardsParameters::default();
        let calc = RichardsCalculator::new(params);

        let mut auxvar_up = RichardsAuxVar::new();
        auxvar_up.pressure = 1.1e5;
        auxvar_up.saturation = 0.8;
        auxvar_up.density = 1000.0;
        auxvar_up.kvr = 1e-6;
        auxvar_up.dkvr_dp = 1e-11;
        auxvar_up.dden_dp = 4.5e-7;

        let mut auxvar_dn = RichardsAuxVar::new();
        auxvar_dn.pressure = 1.0e5;
        auxvar_dn.saturation = 0.8;
        auxvar_dn.density = 1000.0;
        auxvar_dn.kvr = 1e-6;
        auxvar_dn.dkvr_dp = 1e-11;
        auxvar_dn.dden_dp = 4.5e-7;

        let material = MaterialProperties {
            permeability: 1e-12,
            porosity: 0.3,
            volume: 1.0,
        };

        let area = 1.0;
        let distance = [0.5, 0.5, 0.0];

        let (jup, jdn) = calc.flux_derivatives(
            &auxvar_up, &material, &auxvar_dn, &material, area, &distance,
        );

        // Производные должны быть ненулевыми
        assert!(jup.abs() > 0.0);
        assert!(jdn.abs() > 0.0);
    }

    #[test]
    fn test_boundary_flux_dirichlet() {
        let params = RichardsParameters::default();
        let calc = RichardsCalculator::new(params);

        let mut auxvar = RichardsAuxVar::new();
        auxvar.pressure = 1.1e5;
        auxvar.density = 1000.0;
        auxvar.kvr = 1e-6;

        let material = MaterialProperties {
            permeability: 1e-12,
            porosity: 0.3,
            volume: 1.0,
        };

        let pressure_boundary = 1.0e5;
        let area = 1.0;
        let distance = 0.5;

        let flux =
            calc.boundary_flux_dirichlet(&auxvar, &material, pressure_boundary, area, distance);

        // Поток должен быть положительным (из области наружу)
        assert!(flux > 0.0);
    }

    #[test]
    fn test_boundary_flux_neumann() {
        let params = RichardsParameters::default();
        let calc = RichardsCalculator::new(params);

        let flux_density = 0.001; // kg/m²/s (инфильтрация)
        let area = 1.0; // m²

        let flux = calc.boundary_flux_neumann(flux_density, area);

        assert_eq!(flux, 0.001); // kg/s
    }

    #[test]
    fn test_source_sink() {
        let params = RichardsParameters::default();
        let calc = RichardsCalculator::new(params);

        // Источник (положительный)
        let source_rate = 0.1; // kg/s
        let source_flux = calc.apply_source_sink(source_rate);
        assert_eq!(source_flux, 0.1);

        // Сток (отрицательный)
        let sink_rate = -0.05; // kg/s
        let sink_flux = calc.apply_source_sink(sink_rate);
        assert_eq!(sink_flux, -0.05);
    }

    #[test]
    fn test_volumetric_source_sink() {
        let params = RichardsParameters::default();
        let calc = RichardsCalculator::new(params);

        let volumetric_rate = 0.001; // m³/s
        let density = 1000.0; // kg/m³

        let mass_flux = calc.apply_volumetric_source_sink(volumetric_rate, density);

        assert_eq!(mass_flux, 1.0); // kg/s
    }
}
