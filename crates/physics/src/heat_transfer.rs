//! Расчеты теплопереноса в грунте

use thermokarst_core::{EnvironmentParams, Result};

/// Калькулятор теплопереноса
pub struct HeatTransferCalculator {
    params: EnvironmentParams,
}

impl HeatTransferCalculator {
    pub fn new(params: EnvironmentParams) -> Self {
        Self { params }
    }

    /// Расчет теплового потока (Вт/м²)
    pub fn heat_flux(&self) -> f64 {
        let k = self.params.soil_type.thermal_conductivity();
        let temp_gradient = (self.params.air_temp - self.params.permafrost_temp)
            / self.params.permafrost_depth;

        k * temp_gradient
    }

    /// Расчет температуры на глубине (°C)
    pub fn temperature_at_depth(&self, depth: f64) -> f64 {
        if depth <= 0.0 {
            return self.params.air_temp;
        }

        if depth >= self.params.permafrost_depth {
            return self.params.permafrost_temp;
        }

        // Линейная интерполяция
        let ratio = depth / self.params.permafrost_depth;
        self.params.air_temp + ratio * (self.params.permafrost_temp - self.params.air_temp)
    }

    /// Расчет энергии, необходимой для таяния льда (МДж/м³)
    pub fn latent_heat_required(&self, volume: f64) -> f64 {
        const LATENT_HEAT_ICE: f64 = 334.0; // МДж/м³

        volume * self.params.ice_content * LATENT_HEAT_ICE
    }

    /// Эффективная теплоемкость грунта (МДж/(м³·К))
    pub fn effective_heat_capacity(&self) -> f64 {
        const HEAT_CAPACITY_SOIL: f64 = 2.0; // МДж/(м³·К)
        const HEAT_CAPACITY_ICE: f64 = 1.9;
        const HEAT_CAPACITY_WATER: f64 = 4.2;

        let porosity = self.params.soil_type.porosity();
        let soil_fraction = 1.0 - porosity;
        let ice_fraction = porosity * self.params.ice_content;
        let water_fraction = porosity * (1.0 - self.params.ice_content);

        soil_fraction * HEAT_CAPACITY_SOIL
            + ice_fraction * HEAT_CAPACITY_ICE
            + water_fraction * HEAT_CAPACITY_WATER
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_gradient() {
        let params = EnvironmentParams::default();
        let calc = HeatTransferCalculator::new(params);

        let temp_surface = calc.temperature_at_depth(0.0);
        let temp_middle = calc.temperature_at_depth(0.75);
        let temp_bottom = calc.temperature_at_depth(1.5);

        assert!(temp_surface > temp_middle);
        assert!(temp_middle > temp_bottom);
    }
}
