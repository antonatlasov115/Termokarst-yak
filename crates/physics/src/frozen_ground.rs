//! Интеграционный модуль для связки фазовых переходов и потоков воды
//!
//! Объединяет Richards equation с моделью фазовых переходов лед-вода
//! для моделирования потоков в мерзлых грунтах

use crate::phase_transition::{
    PhaseTransitionCalculator, PhaseTransitionModel, VanGenuchtenParams,
};
use crate::richards::{MaterialProperties, RichardsAuxVar, RichardsCalculator, RichardsParameters};

/// Состояние ячейки с учетом фазовых переходов
#[derive(Debug, Clone, Copy)]
pub struct FrozenGroundState {
    /// Давление жидкости (Pa)
    pub pressure: f64,
    /// Температура (°C)
    pub temperature: f64,
    /// Насыщенность жидкой воды (0-1)
    pub liquid_saturation: f64,
    /// Насыщенность льда (0-1)
    pub ice_saturation: f64,
    /// Насыщенность газа (0-1)
    pub gas_saturation: f64,
    /// Плотность жидкости (kg/m³)
    pub liquid_density: f64,
    /// Относительная проницаемость жидкости (0-1)
    pub relative_permeability: f64,
}

/// Калькулятор для мерзлых грунтов
pub struct FrozenGroundCalculator {
    phase_calc: PhaseTransitionCalculator,
    richards_calc: RichardsCalculator,
}

impl FrozenGroundCalculator {
    /// Создать новый калькулятор
    pub fn new(
        phase_model: PhaseTransitionModel,
        vg_params: VanGenuchtenParams,
        richards_params: RichardsParameters,
    ) -> Self {
        let phase_calc = PhaseTransitionCalculator::new(
            phase_model,
            vg_params,
            richards_params.reference_pressure,
        );
        let richards_calc = RichardsCalculator::new(richards_params);

        Self {
            phase_calc,
            richards_calc,
        }
    }

    /// Обновить состояние ячейки на основе давления и температуры
    pub fn update_state(&self, pressure: f64, temperature: f64) -> FrozenGroundState {
        // Рассчитать фазовые насыщенности
        let (sats, _derivs, rel_perm) = self.phase_calc.compute(pressure, temperature);

        // Плотность воды (упрощенная модель)
        let liquid_density = 1000.0 * (1.0 - 1.9e-4 * (temperature - 4.0).powi(2));

        FrozenGroundState {
            pressure,
            temperature,
            liquid_saturation: sats.liquid,
            ice_saturation: sats.ice,
            gas_saturation: sats.gas,
            liquid_density,
            relative_permeability: rel_perm.kr_liquid,
        }
    }

    /// Преобразовать состояние в RichardsAuxVar для расчета потоков
    pub fn state_to_richards_auxvar(&self, state: &FrozenGroundState) -> RichardsAuxVar {
        // Вязкость воды (упрощенная модель)
        let viscosity = 1.79e-3 * (-1.704e-2 * state.temperature).exp();

        // k/μ * kr
        let kvr = state.relative_permeability / viscosity;

        RichardsAuxVar {
            pressure: state.pressure,
            saturation: state.liquid_saturation,
            density: state.liquid_density,
            viscosity,
            relative_permeability: state.relative_permeability,
            kvr,
            dden_dp: 4.5e-7,         // Упрощенная производная
            dsat_dp: 0.0,            // Будет рассчитано из фазовых переходов
            dkvr_dp: 0.0,            // Будет рассчитано из фазовых переходов
            effective_porosity: 0.3, // Должно передаваться из материала
            dpor_dp: 0.0,
        }
    }

    /// Рассчитать поток между двумя ячейками с учетом фазовых переходов
    pub fn compute_flux(
        &self,
        state_up: &FrozenGroundState,
        material_up: &MaterialProperties,
        state_dn: &FrozenGroundState,
        material_dn: &MaterialProperties,
        area: f64,
        distance: &[f64; 3],
    ) -> f64 {
        let auxvar_up = self.state_to_richards_auxvar(state_up);
        let auxvar_dn = self.state_to_richards_auxvar(state_dn);

        self.richards_calc.flux(
            &auxvar_up,
            material_up,
            &auxvar_dn,
            material_dn,
            area,
            distance,
        )
    }

    /// Рассчитать аккумуляцию (накопление) с учетом льда
    ///
    /// Включает как жидкую воду, так и лёд
    pub fn compute_accumulation(
        &self,
        state: &FrozenGroundState,
        material: &MaterialProperties,
    ) -> f64 {
        let auxvar = self.state_to_richards_auxvar(state);

        // Аккумуляция жидкой воды
        let liquid_accum = self.richards_calc.accumulation(&auxvar, material);

        // Аккумуляция льда (плотность льда ≈ 917 kg/m³)
        let ice_density = 917.0;
        let ice_accum = state.ice_saturation * ice_density * material.porosity * material.volume
            / self.richards_calc.params.dt;

        liquid_accum + ice_accum
    }
}

/// Параметры для моделирования мерзлых грунтов
#[derive(Debug, Clone, Copy)]
pub struct FrozenGroundParams {
    /// Параметры Van Genuchten
    pub vg_params: VanGenuchtenParams,
    /// Параметры Richards
    pub richards_params: RichardsParameters,
    /// Модель фазовых переходов
    pub phase_model: PhaseTransitionModel,
}

impl Default for FrozenGroundParams {
    fn default() -> Self {
        Self {
            vg_params: VanGenuchtenParams {
                alpha: 1.0e-4,
                m: 0.5,
                residual_saturation: 0.1,
            },
            richards_params: RichardsParameters::default(),
            phase_model: PhaseTransitionModel::PainterExplicit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frozen_ground_state_above_freezing() {
        let params = FrozenGroundParams::default();
        let calc = FrozenGroundCalculator::new(
            params.phase_model,
            params.vg_params,
            params.richards_params,
        );

        let state = calc.update_state(1.0e5, 5.0);

        // При T > 0°C льда быть не должно
        assert!(state.ice_saturation < 1e-10);
        assert!(state.liquid_saturation > 0.0);
        assert!((state.liquid_saturation + state.gas_saturation - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_frozen_ground_state_below_freezing() {
        let params = FrozenGroundParams::default();
        let calc = FrozenGroundCalculator::new(
            params.phase_model,
            params.vg_params,
            params.richards_params,
        );

        let state = calc.update_state(1.0e5, -5.0);

        // При T < 0°C должен быть лёд
        assert!(state.ice_saturation > 0.0);
        assert!(state.liquid_saturation > 0.0);
        assert!(
            (state.liquid_saturation + state.ice_saturation + state.gas_saturation - 1.0).abs()
                < 1e-6
        );
    }

    #[test]
    fn test_flux_in_frozen_ground() {
        let params = FrozenGroundParams::default();
        let calc = FrozenGroundCalculator::new(
            params.phase_model,
            params.vg_params,
            params.richards_params,
        );

        let state_up = calc.update_state(1.1e5, -2.0);
        let state_dn = calc.update_state(1.0e5, -2.0);

        let material = MaterialProperties {
            permeability: 1e-12,
            porosity: 0.3,
            volume: 1.0,
        };

        let area = 1.0;
        let distance = [0.5, 0.5, 0.0];

        let flux = calc.compute_flux(&state_up, &material, &state_dn, &material, area, &distance);

        // Поток должен быть положительным (от высокого давления к низкому)
        // Но может быть очень малым из-за низкой проницаемости мерзлого грунта
        assert!(flux >= 0.0);
    }

    #[test]
    fn test_accumulation_with_ice() {
        let params = FrozenGroundParams::default();
        let calc = FrozenGroundCalculator::new(
            params.phase_model,
            params.vg_params,
            params.richards_params,
        );

        let state = calc.update_state(1.0e5, -5.0);
        let material = MaterialProperties {
            permeability: 1e-12,
            porosity: 0.3,
            volume: 1.0,
        };

        let accum = calc.compute_accumulation(&state, &material);

        // Аккумуляция должна включать и воду, и лёд
        assert!(accum > 0.0);
    }

    #[test]
    fn test_reduced_permeability_when_frozen() {
        let params = FrozenGroundParams::default();
        let calc = FrozenGroundCalculator::new(
            params.phase_model,
            params.vg_params,
            params.richards_params,
        );

        let state_unfrozen = calc.update_state(1.0e5, 5.0);
        let state_frozen = calc.update_state(1.0e5, -5.0);

        // Относительная проницаемость должна быть ниже в мерзлом грунте
        assert!(state_frozen.relative_permeability < state_unfrozen.relative_permeability);
    }
}
