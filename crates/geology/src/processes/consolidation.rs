//! Модуль консолидации грунта и улучшенной просадки
//!
//! Рассчитывает механические свойства грунта, эффективное напряжение,
//! коэффициент пористости и осадку при оттаивании мерзлоты.

use thermokarst_core::EnvironmentParams;

/// Калькулятор консолидации грунта
pub struct ConsolidationCalculator {
    params: EnvironmentParams,
}

impl ConsolidationCalculator {
    /// Создать новый калькулятор
    pub fn new(params: EnvironmentParams) -> Self {
        Self { params }
    }

    /// Рассчитать коэффициент пористости
    /// e = n / (1 - n), где n - пористость
    pub fn void_ratio(&self, depth: f64) -> f64 {
        let porosity = self.params.soil_type.porosity();

        // Пористость уменьшается с глубиной из-за уплотнения
        let depth_factor = (-depth / 50.0).exp(); // Уменьшение на 50м
        let effective_porosity = porosity * (0.7 + 0.3 * depth_factor);

        effective_porosity / (1.0 - effective_porosity)
    }

    /// Рассчитать эффективное напряжение (Па)
    ///
    /// # Аргументы
    /// * `depth` - Глубина (м)
    /// * `water_table` - Уровень грунтовых вод (м)
    pub fn effective_stress(&self, depth: f64, water_table: f64) -> f64 {
        const UNIT_WEIGHT_SOIL: f64 = 18000.0; // Н/м³ (удельный вес грунта)
        const UNIT_WEIGHT_WATER: f64 = 9810.0; // Н/м³ (удельный вес воды)

        if depth <= 0.0 {
            return 0.0;
        }

        if depth < water_table {
            // Выше уровня грунтовых вод - полное напряжение
            UNIT_WEIGHT_SOIL * depth
        } else {
            // Ниже уровня грунтовых вод - эффективное напряжение
            let total_stress = UNIT_WEIGHT_SOIL * depth;
            let pore_pressure = UNIT_WEIGHT_WATER * (depth - water_table);
            total_stress - pore_pressure
        }
    }

    /// Индекс компрессии (безразмерный)
    /// Зависит от типа грунта и начального коэффициента пористости
    pub fn compression_index(&self, void_ratio: f64) -> f64 {
        let _porosity = self.params.soil_type.porosity();

        // Эмпирическая формула: Cc = 0.009 * (LL - 10)
        // где LL - предел текучести в %
        // Упрощенная версия на основе пористости
        let cc = match self.params.soil_type {
            thermokarst_core::SoilType::Clay => 0.3 + 0.5 * void_ratio,
            thermokarst_core::SoilType::Silt => 0.2 + 0.3 * void_ratio,
            thermokarst_core::SoilType::Sand => 0.05 + 0.1 * void_ratio,
            thermokarst_core::SoilType::Peat => 0.8 + 1.0 * void_ratio,
            thermokarst_core::SoilType::Loam => 0.15 + 0.2 * void_ratio,
        };

        cc.max(0.01)
    }

    /// Индекс набухания (безразмерный)
    /// Обычно Cs = 0.1 * Cc до 0.2 * Cc
    pub fn swelling_index(&self, void_ratio: f64) -> f64 {
        let cc = self.compression_index(void_ratio);
        0.15 * cc
    }

    /// Рассчитать осадку при консолидации (м)
    ///
    /// # Аргументы
    /// * `initial_depth` - Начальная глубина слоя (м)
    /// * `initial_stress` - Начальное эффективное напряжение (Па)
    /// * `final_stress` - Конечное эффективное напряжение (Па)
    pub fn settlement(&self, initial_depth: f64, initial_stress: f64, final_stress: f64) -> f64 {
        if initial_depth <= 0.0 || final_stress <= initial_stress {
            return 0.0;
        }

        let e0 = self.void_ratio(initial_depth);
        let cc = self.compression_index(e0);

        // Формула осадки: ΔH = H * Cc / (1 + e0) * log10(σ'f / σ'0)
        let settlement = initial_depth * cc / (1.0 + e0) * (final_stress / initial_stress).log10();

        settlement.max(0.0)
    }

    /// Рассчитать осадку при оттаивании мерзлоты (улучшенная модель)
    ///
    /// # Аргументы
    /// * `thaw_depth` - Глубина оттаивания (м)
    /// * `water_table` - Уровень грунтовых вод (м)
    pub fn thaw_settlement(&self, thaw_depth: f64, water_table: f64) -> f64 {
        if thaw_depth <= 0.0 {
            return 0.0;
        }

        // Начальное напряжение (мерзлый грунт)
        let initial_stress = self.effective_stress(thaw_depth / 2.0, water_table);

        // Конечное напряжение (оттаявший грунт + вес воды от растаявшего льда)
        let ice_content = self.params.ice_content;
        let additional_water_weight = ice_content * 9810.0 * thaw_depth;
        let final_stress = initial_stress + additional_water_weight;

        // Осадка от консолидации
        let consolidation_settlement = self.settlement(thaw_depth, initial_stress, final_stress);

        // Дополнительная осадка от таяния льда
        // При таянии лед уменьшается в объеме на ~9%
        let ice_melt_settlement = thaw_depth * ice_content * 0.09;

        consolidation_settlement + ice_melt_settlement
    }

    /// Время консолидации (годы)
    ///
    /// # Аргументы
    /// * `layer_thickness` - Толщина слоя (м)
    /// * `drainage_path` - Путь дренажа (м) - обычно H/2 для двустороннего дренажа
    pub fn consolidation_time(&self, _layer_thickness: f64, drainage_path: f64) -> f64 {
        // Коэффициент консолидации (м²/год)
        let cv = match self.params.soil_type {
            thermokarst_core::SoilType::Clay => 0.5,
            thermokarst_core::SoilType::Silt => 2.0,
            thermokarst_core::SoilType::Sand => 10.0,
            thermokarst_core::SoilType::Peat => 0.2,
            thermokarst_core::SoilType::Loam => 1.5,
        };

        // Фактор времени для 90% консолидации
        const T90: f64 = 0.848;

        // Время: t = T * H² / cv
        T90 * drainage_path.powi(2) / cv
    }

    /// Степень консолидации в момент времени t (0-1)
    pub fn degree_of_consolidation(&self, time_years: f64, layer_thickness: f64) -> f64 {
        let drainage_path = layer_thickness / 2.0; // Двусторонний дренаж
        let t_total = self.consolidation_time(layer_thickness, drainage_path);

        if t_total <= 0.0 {
            return 1.0;
        }

        let time_factor = time_years / t_total;

        // Упрощенная формула степени консолидации
        if time_factor < 0.6 {
            (4.0 * time_factor / std::f64::consts::PI).sqrt()
        } else {
            1.0 - 10.0_f64.powf(-time_factor)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use thermokarst_core::SoilType;

    #[test]
    fn test_void_ratio() {
        let params = EnvironmentParams {
            air_temp: -5.0,
            permafrost_temp: -3.0,
            ice_content: 0.5,
            soil_type: SoilType::Silt,
            vegetation_cover: 0.3,
            permafrost_depth: 100.0,
            warm_season_days: 120,
            soil_saturation_ratio: 0.5,
            temperature_amplitude: 88.0,
        };

        let calc = ConsolidationCalculator::new(params);

        let e_surface = calc.void_ratio(0.0);
        let e_deep = calc.void_ratio(50.0);

        // Пористость уменьшается с глубиной
        assert!(e_deep < e_surface);

        println!("Void ratio at surface: {:.3}", e_surface);
        println!("Void ratio at 50m: {:.3}", e_deep);
    }

    #[test]
    fn test_effective_stress() {
        let params = EnvironmentParams::default();
        let calc = ConsolidationCalculator::new(params);

        let water_table = 2.0;

        let stress_above = calc.effective_stress(1.0, water_table);
        let stress_below = calc.effective_stress(5.0, water_table);

        // Напряжение увеличивается с глубиной
        assert!(stress_below > stress_above);

        println!("Stress at 1m: {:.0} Pa", stress_above);
        println!("Stress at 5m: {:.0} Pa", stress_below);
    }

    #[test]
    fn test_compression_index() {
        let params = EnvironmentParams {
            soil_type: SoilType::Clay,
            ..Default::default()
        };

        let calc = ConsolidationCalculator::new(params);
        let e = calc.void_ratio(5.0);
        let cc = calc.compression_index(e);

        // Глина имеет высокий индекс компрессии
        assert!(cc > 0.2);

        println!("Compression index (clay): {:.3}", cc);
    }

    #[test]
    fn test_settlement() {
        let params = EnvironmentParams::default();
        let calc = ConsolidationCalculator::new(params);

        let settlement = calc.settlement(
            5.0,      // 5м слой
            50000.0,  // Начальное напряжение
            100000.0, // Конечное напряжение (удвоение)
        );

        assert!(settlement > 0.0);
        assert!(settlement < 5.0); // Осадка меньше толщины слоя

        println!("Settlement: {:.3} m", settlement);
    }

    #[test]
    fn test_thaw_settlement() {
        let params = EnvironmentParams {
            ice_content: 0.6,
            soil_type: SoilType::Silt,
            ..Default::default()
        };

        let calc = ConsolidationCalculator::new(params);

        let settlement = calc.thaw_settlement(3.0, 5.0);

        assert!(settlement > 0.0);

        println!("Thaw settlement for 3m depth: {:.3} m", settlement);
    }

    #[test]
    fn test_consolidation_time() {
        let params = EnvironmentParams {
            soil_type: SoilType::Clay,
            ..Default::default()
        };

        let calc = ConsolidationCalculator::new(params);

        let time = calc.consolidation_time(10.0, 5.0);

        // Глина консолидируется медленно
        assert!(time > 1.0);

        println!("Consolidation time (clay, 10m): {:.1} years", time);
    }

    #[test]
    fn test_degree_of_consolidation() {
        let params = EnvironmentParams {
            soil_type: SoilType::Sand,
            ..Default::default()
        };

        let calc = ConsolidationCalculator::new(params);

        let u_1year = calc.degree_of_consolidation(1.0, 5.0);
        let u_5year = calc.degree_of_consolidation(5.0, 5.0);

        // Степень консолидации увеличивается со временем
        assert!(u_5year > u_1year);
        assert!(u_5year <= 1.0);

        println!("Consolidation after 1 year: {:.1}%", u_1year * 100.0);
        println!("Consolidation after 5 years: {:.1}%", u_5year * 100.0);
    }

    #[test]
    fn test_ice_content_effect() {
        let params_low_ice = EnvironmentParams {
            ice_content: 0.3,
            ..Default::default()
        };

        let params_high_ice = EnvironmentParams {
            ice_content: 0.8,
            ..Default::default()
        };

        let calc_low = ConsolidationCalculator::new(params_low_ice);
        let calc_high = ConsolidationCalculator::new(params_high_ice);

        let settlement_low = calc_low.thaw_settlement(3.0, 5.0);
        let settlement_high = calc_high.thaw_settlement(3.0, 5.0);

        // Больше льда -> больше осадка
        assert!(settlement_high > settlement_low);

        println!("Settlement (30% ice): {:.3} m", settlement_low);
        println!("Settlement (80% ice): {:.3} m", settlement_high);
    }
}
