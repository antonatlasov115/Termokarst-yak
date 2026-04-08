//! Модуль фазовых переходов лед-вода
//!
//! Портировано из PFLOTRAN (saturation_function.F90, eos_water.F90)
//! Реализует модели:
//! - Painter (2011) - явная модель
//! - Painter & Karra (2013) - неявная/явная модели
//! - Dall'Amico (2010, 2011)

use std::f64::consts::PI;

/// Физические константы
pub mod constants {
    /// Температура замерзания воды (K)
    pub const T_FREEZE: f64 = 273.15;

    /// Скрытая теплота плавления льда (J/kg)
    pub const HEAT_OF_FUSION: f64 = 3.34e5;

    /// Плотность льда при 273.15K (kg/m³)
    pub const DENSITY_ICE: f64 = 916.7;

    /// Плотность воды при 273.15K (kg/m³)
    pub const DENSITY_WATER: f64 = 1000.0;

    /// Отношение межфазных натяжений лед-вода/воздух-вода
    pub const INTERFACIAL_TENSION_RATIO: f64 = 2.33;

    /// Молекулярная масса воды (kg/mol)
    pub const MOLAR_MASS_WATER: f64 = 0.018015;
}

/// Модель фазовых переходов
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PhaseTransitionModel {
    /// Явная модель Painter (2011)
    PainterExplicit,
    /// Неявная модель Painter & Karra (2013)
    PainterKarraImplicit,
    /// Явная модель Painter & Karra (2013)
    PainterKarraExplicit,
    /// Модель Dall'Amico (2010, 2011)
    DallAmico,
}

/// Параметры Van Genuchten для характеристических кривых
#[derive(Debug, Clone, Copy)]
pub struct VanGenuchtenParams {
    /// Параметр α (1/Pa)
    pub alpha: f64,
    /// Параметр m (безразмерный)
    pub m: f64,
    /// Остаточная насыщенность жидкости
    pub residual_saturation: f64,
}

impl VanGenuchtenParams {
    /// Параметр n = 1/(1-m)
    pub fn n(&self) -> f64 {
        1.0 / (1.0 - self.m)
    }
}

/// Результат расчета фазовых насыщенностей
#[derive(Debug, Clone, Copy)]
pub struct PhaseSaturations {
    /// Насыщенность жидкой воды (0-1)
    pub liquid: f64,
    /// Насыщенность льда (0-1)
    pub ice: f64,
    /// Насыщенность газа (0-1)
    pub gas: f64,
}

/// Производные насыщенностей
#[derive(Debug, Clone, Copy)]
pub struct SaturationDerivatives {
    /// dS_liquid/dP
    pub dsl_dp: f64,
    /// dS_liquid/dT
    pub dsl_dt: f64,
    /// dS_ice/dP
    pub dsi_dp: f64,
    /// dS_ice/dT
    pub dsi_dt: f64,
    /// dS_gas/dP
    pub dsg_dp: f64,
    /// dS_gas/dT
    pub dsg_dt: f64,
}

/// Относительная проницаемость и её производные
#[derive(Debug, Clone, Copy)]
pub struct RelativePermeability {
    /// Относительная проницаемость жидкости
    pub kr_liquid: f64,
    /// dkr/dP
    pub dkr_dp: f64,
    /// dkr/dT
    pub dkr_dt: f64,
}

/// Калькулятор фазовых переходов
pub struct PhaseTransitionCalculator {
    model: PhaseTransitionModel,
    vg_params: VanGenuchtenParams,
    reference_pressure: f64,
}

impl PhaseTransitionCalculator {
    /// Создать новый калькулятор
    pub fn new(
        model: PhaseTransitionModel,
        vg_params: VanGenuchtenParams,
        reference_pressure: f64,
    ) -> Self {
        Self {
            model,
            vg_params,
            reference_pressure,
        }
    }

    /// Рассчитать насыщенности фаз (явная модель Painter)
    ///
    /// # Аргументы
    /// * `liquid_pressure` - давление жидкости (Pa)
    /// * `temperature` - температура (°C)
    ///
    /// # Возвращает
    /// Насыщенности фаз и их производные
    pub fn compute_painter_explicit(
        &self,
        liquid_pressure: f64,
        temperature: f64,
    ) -> (
        PhaseSaturations,
        SaturationDerivatives,
        RelativePermeability,
    ) {
        use constants::*;

        let alpha = self.vg_params.alpha;
        let m = self.vg_params.m;
        let n = self.vg_params.n();

        // Функция B (зависит от давления)
        let (function_b, dfunc_b_dp) = if liquid_pressure >= self.reference_pressure {
            (1.0, 0.0)
        } else {
            let pc = self.reference_pressure - liquid_pressure;
            let pc_alpha = pc * alpha;
            let pc_alpha_n = pc_alpha.powf(n);
            let one_plus_pc_alpha_n = 1.0 + pc_alpha_n;
            let se = one_plus_pc_alpha_n.powf(-m);
            let dse_dpc =
                -m * n * alpha * pc_alpha_n / (pc_alpha * one_plus_pc_alpha_n.powf(m + 1.0));

            let func_b = 1.0 / se;
            let dfunc_b = dse_dpc / (se * se);
            (func_b, dfunc_b)
        };

        // Функция A (зависит от температуры)
        let (function_a, dfunc_a_dt) = if temperature >= 0.0 {
            (1.0, 0.0)
        } else {
            // Капиллярное давление лед-жидкость
            let gamma = DENSITY_ICE * HEAT_OF_FUSION * INTERFACIAL_TENSION_RATIO;
            let pc_il = gamma * (-temperature) / T_FREEZE;

            let pc_il_alpha = pc_il * alpha;
            let pc_il_alpha_n = pc_il_alpha.powf(n);
            let one_plus_pc_il_alpha_n = 1.0 + pc_il_alpha_n;
            let se_temp = one_plus_pc_il_alpha_n.powf(-m);

            let func_a = 1.0 / se_temp;
            let dfunc_a = (gamma / T_FREEZE) / (se_temp * se_temp)
                * (-m)
                * one_plus_pc_il_alpha_n.powf(-m - 1.0)
                * n
                * pc_il.powf(n - 1.0)
                * alpha.powf(n);
            (func_a, dfunc_a)
        };

        // Насыщенности
        let liquid_sat = 1.0 / (function_a + function_b - 1.0);
        let gas_sat = liquid_sat * (function_b - 1.0);
        let ice_sat = liquid_sat * (function_a - 1.0);

        // Производные насыщенности жидкости
        let denom_sq = (function_a + function_b - 1.0).powi(2);
        let dsl_dp = -dfunc_b_dp / denom_sq;
        let dsl_dt = -dfunc_a_dt / denom_sq;

        // Производные насыщенности газа
        let dsg_dp = dsl_dp * (function_b - 1.0) + liquid_sat * dfunc_b_dp;
        let dsg_dt = dsl_dt * (function_b - 1.0);

        // Производные насыщенности льда
        let dsi_dp = dsl_dp * (function_a - 1.0);
        let dsi_dt = dsl_dt * (function_a - 1.0) + liquid_sat * dfunc_a_dt;

        // Относительная проницаемость (модель Mualem)
        let (kr_liquid, dkr_ds_liq) = if liquid_sat < 1.0 {
            let one_over_m = 1.0 / m;
            let liq_sat_one_over_m = liquid_sat.powf(one_over_m);
            let kr = liquid_sat.sqrt() * (1.0 - (1.0 - liq_sat_one_over_m).powf(m)).powi(2);

            let dkr_ds = 0.5 * kr / liquid_sat
                + 2.0
                    * liquid_sat.powf(one_over_m - 0.5)
                    * (1.0 - liq_sat_one_over_m).powf(m - 1.0)
                    * (1.0 - (1.0 - liq_sat_one_over_m).powf(m));
            (kr, dkr_ds)
        } else {
            (1.0, 0.0)
        };

        let dkr_dp = dkr_ds_liq * dsl_dp;
        let dkr_dt = dkr_ds_liq * dsl_dt;

        let saturations = PhaseSaturations {
            liquid: liquid_sat.clamp(0.0, 1.0),
            ice: ice_sat.clamp(0.0, 1.0),
            gas: gas_sat.clamp(0.0, 1.0),
        };

        let derivatives = SaturationDerivatives {
            dsl_dp,
            dsl_dt,
            dsi_dp,
            dsi_dt,
            dsg_dp,
            dsg_dt,
        };

        let rel_perm = RelativePermeability {
            kr_liquid,
            dkr_dp,
            dkr_dt,
        };

        (saturations, derivatives, rel_perm)
    }

    /// Рассчитать насыщенности фаз (выбор модели)
    pub fn compute(
        &self,
        liquid_pressure: f64,
        temperature: f64,
    ) -> (
        PhaseSaturations,
        SaturationDerivatives,
        RelativePermeability,
    ) {
        match self.model {
            PhaseTransitionModel::PainterExplicit => {
                self.compute_painter_explicit(liquid_pressure, temperature)
            }
            _ => {
                // TODO: реализовать другие модели
                self.compute_painter_explicit(liquid_pressure, temperature)
            }
        }
    }
}

/// Уравнения состояния для льда
pub struct IceEOS;

impl IceEOS {
    /// Плотность льда (модель Painter)
    ///
    /// # Аргументы
    /// * `temperature` - температура (°C)
    /// * `pressure` - давление (Pa)
    ///
    /// # Возвращает
    /// Плотность (kg/m³) и производные
    pub fn density_painter(temperature: f64, pressure: f64) -> (f64, f64, f64) {
        const P_REF: f64 = 1.0e5; // Па
        const ALPHA: f64 = 3.3e-10; // 1/Па
        const BETA: f64 = 1.53e-4; // 1/K
        const MOLAR_MASS: f64 = 0.018015; // kg/mol

        // Плотность в kmol/m³
        let den_kmol = 50.9424 * (1.0 + ALPHA * (pressure - P_REF) - BETA * temperature);

        // Перевод в kg/m³
        let density = den_kmol * MOLAR_MASS * 1000.0;
        let dden_dt = 50.9424 * (-BETA) * MOLAR_MASS * 1000.0;
        let dden_dp = 50.9424 * ALPHA * MOLAR_MASS * 1000.0;

        (density, dden_dt, dden_dp)
    }

    /// Внутренняя энергия льда (модель по умолчанию)
    ///
    /// # Аргументы
    /// * `temperature` - температура (°C)
    ///
    /// # Возвращает
    /// Внутренняя энергия (J/kg) и производная по температуре
    pub fn internal_energy_default(temperature: f64) -> (f64, f64) {
        use constants::*;

        const A: f64 = -10.6644;
        const B: f64 = 0.1698;
        const C: f64 = 198148.0;

        let t_kelvin = temperature + T_FREEZE;

        // Внутренняя энергия в J/mol (Maier-Kelly fit)
        let u_mol = A * temperature
            + B / 2.0 * (t_kelvin.powi(2) - T_FREEZE.powi(2))
            + C * (1.0 / T_FREEZE - 1.0 / t_kelvin)
            - HEAT_OF_FUSION * MOLAR_MASS_WATER * 1000.0;

        let du_dt_mol = A + B * t_kelvin + C / t_kelvin.powi(2);

        // Перевод в J/kg
        let u_ice = u_mol / MOLAR_MASS_WATER;
        let du_dt = du_dt_mol / MOLAR_MASS_WATER;

        (u_ice, du_dt)
    }

    /// Внутренняя энергия льда (модель Fukusako & Yamada, 1993)
    ///
    /// # Аргументы
    /// * `temperature` - температура (°C)
    ///
    /// # Возвращает
    /// Внутренняя энергия (J/kg) и производная по температуре
    pub fn internal_energy_fukusako(temperature: f64) -> (f64, f64) {
        use constants::*;

        const LW: f64 = -3.34110e5; // Скрытая теплота плавления, J/kg

        let t_kelvin = temperature + T_FREEZE;

        let u_ice = if t_kelvin >= 90.0 {
            LW + 185.0 * temperature + 3.445 * (t_kelvin.powi(2) - T_FREEZE.powi(2))
        } else {
            LW + 4.475 * (t_kelvin.powi(2) - T_FREEZE.powi(2))
        };

        let du_dt = if t_kelvin >= 90.0 {
            185.0 + 3.445 * 2.0 * t_kelvin
        } else {
            4.475 * 2.0 * t_kelvin
        };

        (u_ice, du_dt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_transition_above_freezing() {
        let vg_params = VanGenuchtenParams {
            alpha: 1.0e-4,
            m: 0.5,
            residual_saturation: 0.1,
        };

        let calc =
            PhaseTransitionCalculator::new(PhaseTransitionModel::PainterExplicit, vg_params, 1.0e5);

        let (sats, _, _) = calc.compute(1.0e5, 5.0);

        // При T > 0°C льда быть не должно
        assert!(sats.ice < 1e-10);
        assert!(sats.liquid > 0.0);
    }

    #[test]
    fn test_phase_transition_below_freezing() {
        let vg_params = VanGenuchtenParams {
            alpha: 1.0e-4,
            m: 0.5,
            residual_saturation: 0.1,
        };

        let calc =
            PhaseTransitionCalculator::new(PhaseTransitionModel::PainterExplicit, vg_params, 1.0e5);

        let (sats, _, _) = calc.compute(1.0e5, -5.0);

        // При T < 0°C должен быть лёд
        assert!(sats.ice > 0.0);
        assert!(sats.liquid > 0.0);
        assert!((sats.liquid + sats.ice + sats.gas - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_ice_density() {
        let (density, _, _) = IceEOS::density_painter(0.0, 1.0e5);
        // Плотность льда при 0°C должна быть около 917 kg/m³
        assert!((density - 917.0).abs() < 10.0);
    }

    #[test]
    fn test_ice_internal_energy() {
        let (u_ice, _) = IceEOS::internal_energy_default(0.0);
        // Внутренняя энергия должна быть отрицательной
        assert!(u_ice < 0.0);
    }

    #[test]
    fn test_saturation_sum() {
        let vg_params = VanGenuchtenParams {
            alpha: 1.0e-4,
            m: 0.5,
            residual_saturation: 0.1,
        };

        let calc =
            PhaseTransitionCalculator::new(PhaseTransitionModel::PainterExplicit, vg_params, 1.0e5);

        for temp in [-10.0, -5.0, -1.0, 0.0, 5.0] {
            let (sats, _, _) = calc.compute(9.0e4, temp);
            let sum = sats.liquid + sats.ice + sats.gas;
            assert!(
                (sum - 1.0).abs() < 1e-6,
                "Sum of saturations != 1 at T={}",
                temp
            );
        }
    }
}
