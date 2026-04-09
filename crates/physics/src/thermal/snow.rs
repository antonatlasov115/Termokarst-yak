//! Модуль снежного покрова
//!
//! Моделирует накопление, уплотнение и таяние снега,
//! а также его влияние на теплоизоляцию грунта.

/// Модуль снежного покрова
#[derive(Debug, Clone)]
pub struct SnowModule {
    /// Глубина снега (м)
    pub depth: f64,
    /// Плотность снега (кг/м³)
    pub density: f64,
    /// Водный эквивалент (м)
    pub water_equivalent: f64,
    /// Дни с момента последнего снегопада
    days_since_snowfall: u32,
}

impl SnowModule {
    /// Создать новый модуль снега
    pub fn new(depth: f64, density: f64) -> Self {
        let water_equivalent = depth * density / 1000.0; // кг/м³ -> т/м³
        Self {
            depth,
            density,
            water_equivalent,
            days_since_snowfall: 0,
        }
    }

    /// Создать пустой (без снега)
    pub fn empty() -> Self {
        Self::new(0.0, 0.0)
    }

    /// Типичные условия для Якутии
    pub fn yakutia_winter() -> Self {
        // Типичная глубина снега в Центральной Якутии: 30-40 см
        // Плотность: 200-300 кг/м³
        Self::new(0.35, 250.0)
    }

    /// Теплопроводность снега по модели Sturm et al. (1997)
    /// k = 0.138 - 1.01 * ρ + 3.233 * ρ²
    /// где ρ - плотность в г/см³
    pub fn thermal_conductivity(&self) -> f64 {
        if self.depth <= 0.0 {
            return 0.0;
        }

        let rho = self.density / 1000.0; // кг/м³ -> г/см³
        let k = 0.138 - 1.01 * rho + 3.233 * rho.powi(2);
        k.max(0.05) // Минимальное значение для свежего снега
    }

    /// Фактор теплоизоляции снега
    /// Показывает, во сколько раз снег снижает теплопередачу
    pub fn insulation_factor(&self) -> f64 {
        if self.depth <= 0.0 {
            return 1.0;
        }

        let k_snow = self.thermal_conductivity();
        let k_air = 0.025; // Теплопроводность воздуха

        // Эффективное термическое сопротивление
        let r_snow = self.depth / k_snow;
        let r_ref = 0.1 / k_air; // Референсное сопротивление

        1.0 / (1.0 + r_snow / r_ref)
    }

    /// Обновить плотность снега (уплотнение со временем)
    pub fn update_density(&mut self, days_elapsed: u32) {
        self.days_since_snowfall += days_elapsed;

        // Эмпирическая модель уплотнения
        // Свежий снег: ~100 кг/м³
        // Старый снег: ~400 кг/м³
        const RHO_FRESH: f64 = 100.0;
        const RHO_OLD: f64 = 400.0;
        const COMPACTION_RATE: f64 = 0.01; // 1/день

        let target_density = RHO_OLD
            - (RHO_OLD - RHO_FRESH) * (-COMPACTION_RATE * self.days_since_snowfall as f64).exp();

        // Плавное изменение плотности
        self.density = self.density * 0.9 + target_density * 0.1;

        // Обновить глубину при сохранении водного эквивалента
        if self.density > 0.0 {
            self.depth = self.water_equivalent * 1000.0 / self.density;
        }
    }

    /// Добавить снегопад
    pub fn add_snowfall(&mut self, new_snow_depth: f64, new_snow_density: f64) {
        if new_snow_depth <= 0.0 {
            return;
        }

        let new_we = new_snow_depth * new_snow_density / 1000.0;
        let total_we = self.water_equivalent + new_we;

        if total_we > 0.0 {
            // Средневзвешенная плотность
            self.density =
                (self.water_equivalent * self.density + new_we * new_snow_density) / total_we;
            self.water_equivalent = total_we;
            self.depth = self.water_equivalent * 1000.0 / self.density;
        }

        self.days_since_snowfall = 0;
    }

    /// Таяние снега
    /// Возвращает количество растаявшего снега (м водного эквивалента)
    pub fn melt(&mut self, energy_input: f64) -> f64 {
        if self.depth <= 0.0 {
            return 0.0;
        }

        // Скрытая теплота плавления льда: 334 кДж/кг
        const LATENT_HEAT: f64 = 334000.0; // Дж/кг

        // Масса снега (кг/м²)
        let snow_mass = self.water_equivalent * 1000.0;

        // Масса растаявшего снега (кг/м²)
        let melted_mass = (energy_input / LATENT_HEAT).min(snow_mass);

        // Обновить состояние
        let melted_we = melted_mass / 1000.0;
        self.water_equivalent -= melted_we;

        if self.water_equivalent <= 0.0 {
            self.depth = 0.0;
            self.density = 0.0;
            self.water_equivalent = 0.0;
        } else if self.density > 0.0 {
            self.depth = self.water_equivalent * 1000.0 / self.density;
        }

        melted_we
    }

    /// Простое таяние на основе положительных температур
    pub fn melt_degree_day(&mut self, positive_temp: f64, melt_factor: f64) -> f64 {
        if positive_temp <= 0.0 || self.depth <= 0.0 {
            return 0.0;
        }

        // Фактор таяния: обычно 3-5 мм/(день·°C)
        let melt_depth = positive_temp * melt_factor / 1000.0; // м
        let melt_we = melt_depth.min(self.water_equivalent);

        self.water_equivalent -= melt_we;

        if self.water_equivalent <= 0.0 {
            self.depth = 0.0;
            self.density = 0.0;
            self.water_equivalent = 0.0;
        } else if self.density > 0.0 {
            self.depth = self.water_equivalent * 1000.0 / self.density;
        }

        melt_we
    }

    /// Проверить, есть ли снег
    pub fn has_snow(&self) -> bool {
        self.depth > 0.01 // Минимум 1 см
    }

    /// Альбедо снега (отражательная способность)
    pub fn albedo(&self) -> f64 {
        if !self.has_snow() {
            return 0.2; // Альбедо грунта
        }

        // Свежий снег: 0.8-0.9
        // Старый снег: 0.5-0.6
        let age_factor = (-0.05 * self.days_since_snowfall as f64).exp();
        0.5 + 0.4 * age_factor
    }
}

/// Симулятор снежного покрова на сезон
pub struct SnowSeasonSimulator {
    snow: SnowModule,
    /// Средняя температура для каждого дня
    daily_temps: Vec<f64>,
    /// Осадки для каждого дня (мм)
    daily_precip: Vec<f64>,
}

impl SnowSeasonSimulator {
    /// Создать новый симулятор
    pub fn new(daily_temps: Vec<f64>, daily_precip: Vec<f64>) -> Self {
        Self {
            snow: SnowModule::empty(),
            daily_temps,
            daily_precip,
        }
    }

    /// Симулировать сезон
    pub fn simulate(&mut self) -> Vec<SnowModule> {
        let mut history = Vec::new();

        for (day, (&temp, &precip)) in self
            .daily_temps
            .iter()
            .zip(self.daily_precip.iter())
            .enumerate()
        {
            // Осадки в виде снега при T < 0°C
            if temp < 0.0 && precip > 0.0 {
                let snow_depth = precip / 1000.0; // мм -> м
                let snow_density = 100.0; // Свежий снег
                self.snow.add_snowfall(snow_depth, snow_density);
            }

            // Таяние при T > 0°C
            if temp > 0.0 {
                let melt_factor = 4.0; // мм/(день·°C)
                self.snow.melt_degree_day(temp, melt_factor);
            }

            // Уплотнение
            if day % 7 == 0 {
                // Обновлять раз в неделю
                self.snow.update_density(7);
            }

            history.push(self.snow.clone());
        }

        history
    }

    /// Получить максимальную глубину снега за сезон
    pub fn max_snow_depth(&self, history: &[SnowModule]) -> f64 {
        history.iter().map(|s| s.depth).fold(0.0, f64::max)
    }

    /// Получить среднюю глубину снега за зиму
    pub fn mean_winter_snow_depth(&self, history: &[SnowModule]) -> f64 {
        let winter_snow: Vec<f64> = history
            .iter()
            .filter(|s| s.has_snow())
            .map(|s| s.depth)
            .collect();

        if winter_snow.is_empty() {
            return 0.0;
        }

        winter_snow.iter().sum::<f64>() / winter_snow.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snow_thermal_conductivity() {
        let snow = SnowModule::new(0.3, 250.0);
        let k = snow.thermal_conductivity();

        // Типичные значения: 0.1-0.4 Вт/(м·К)
        assert!(k > 0.05 && k < 0.5);
        println!("Snow thermal conductivity: {:.3} W/(m·K)", k);
    }

    #[test]
    fn test_snow_insulation() {
        let snow = SnowModule::new(0.4, 250.0);
        let factor = snow.insulation_factor();

        // Снег должен снижать теплопередачу
        assert!(factor < 1.0);
        println!("Insulation factor: {:.2}", factor);
    }

    #[test]
    fn test_snow_compaction() {
        let mut snow = SnowModule::new(0.5, 100.0);
        let initial_density = snow.density;

        snow.update_density(30); // 30 дней

        // Плотность должна увеличиться
        assert!(snow.density > initial_density);
        println!("Density after 30 days: {:.1} kg/m³", snow.density);
    }

    #[test]
    fn test_snowfall() {
        let mut snow = SnowModule::empty();

        snow.add_snowfall(0.1, 100.0); // 10 см свежего снега

        assert!(snow.depth > 0.0);
        assert!(snow.water_equivalent > 0.0);
        println!(
            "After snowfall: depth={:.2}m, WE={:.3}m",
            snow.depth, snow.water_equivalent
        );
    }

    #[test]
    fn test_snow_melt() {
        let mut snow = SnowModule::new(0.3, 250.0);
        let initial_we = snow.water_equivalent;

        let melted = snow.melt_degree_day(5.0, 4.0); // 5°C, 4 мм/(день·°C)

        assert!(melted > 0.0);
        assert!(snow.water_equivalent < initial_we);
        println!("Melted: {:.4}m WE", melted);
    }

    #[test]
    fn test_yakutia_winter() {
        let snow = SnowModule::yakutia_winter();

        assert!(snow.depth > 0.2 && snow.depth < 0.5);
        assert!(snow.density > 200.0 && snow.density < 300.0);

        let k = snow.thermal_conductivity();
        println!(
            "Yakutia winter snow: depth={:.2}m, k={:.3} W/(m·K)",
            snow.depth, k
        );
    }

    #[test]
    fn test_snow_albedo() {
        let mut snow = SnowModule::new(0.3, 100.0);

        let fresh_albedo = snow.albedo();
        assert!(fresh_albedo > 0.7);

        snow.days_since_snowfall = 30;
        let old_albedo = snow.albedo();
        assert!(old_albedo < fresh_albedo);

        println!(
            "Fresh albedo: {:.2}, Old albedo: {:.2}",
            fresh_albedo, old_albedo
        );
    }

    #[test]
    fn test_season_simulation() {
        use std::f64::consts::PI;

        // Создать синтетический год
        let mut temps = Vec::new();
        let mut precip = Vec::new();

        for day in 0..365 {
            let t = -10.0 + 20.0 * (2.0 * PI * day as f64 / 365.0).sin();
            temps.push(t);

            // Осадки зимой
            let p = if t < 0.0 { 2.0 } else { 0.0 };
            precip.push(p);
        }

        let mut sim = SnowSeasonSimulator::new(temps, precip);
        let history = sim.simulate();

        let max_depth = sim.max_snow_depth(&history);
        let mean_depth = sim.mean_winter_snow_depth(&history);

        assert!(max_depth > 0.0);
        println!(
            "Max snow depth: {:.2}m, Mean winter depth: {:.2}m",
            max_depth, mean_depth
        );
    }
}
