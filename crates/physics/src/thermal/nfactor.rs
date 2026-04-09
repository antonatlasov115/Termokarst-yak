//! N-факторы для преобразования температуры воздуха в температуру поверхности грунта
//!
//! N-факторы учитывают влияние растительности, снежного покрова и других
//! поверхностных условий на передачу тепла между воздухом и грунтом.

/// N-факторы для сезонов оттаивания и промерзания
#[derive(Debug, Clone, Copy)]
pub struct NFactor {
    /// N-фактор для сезона оттаивания (0-1.5)
    pub thawing: f64,
    /// N-фактор для сезона промерзания (0-1.5)
    pub freezing: f64,
}

impl NFactor {
    /// Создать N-факторы с заданными значениями
    pub fn new(thawing: f64, freezing: f64) -> Self {
        Self { thawing, freezing }
    }

    /// Рассчитать N-факторы на основе растительности и снега
    ///
    /// # Аргументы
    /// * `vegetation_cover` - Покрытие растительностью (0-1)
    /// * `vegetation_height` - Высота растительности (м)
    /// * `snow_depth` - Глубина снега (м)
    /// * `snow_density` - Плотность снега (кг/м³)
    pub fn from_surface_conditions(
        vegetation_cover: f64,
        vegetation_height: f64,
        snow_depth: f64,
        snow_density: f64,
    ) -> Self {
        // N-фактор оттаивания: растительность снижает передачу тепла
        let n_thaw = Self::calculate_thawing_nfactor(vegetation_cover, vegetation_height);

        // N-фактор промерзания: снег изолирует грунт
        let n_freeze = Self::calculate_freezing_nfactor(snow_depth, snow_density, vegetation_cover);

        Self::new(n_thaw, n_freeze)
    }

    /// Рассчитать N-фактор оттаивания
    fn calculate_thawing_nfactor(vegetation_cover: f64, vegetation_height: f64) -> f64 {
        // Базовое значение для голой поверхности
        let base = 1.0;

        // Растительность снижает N-фактор (меньше тепла достигает грунта)
        // Эмпирическая формула на основе литературы
        let veg_effect = if vegetation_cover > 0.0 {
            let height_factor = (vegetation_height / 0.5).min(1.0); // Нормализация к 0.5м
            0.3 * vegetation_cover * (1.0 + height_factor)
        } else {
            0.0
        };

        (base - veg_effect).max(0.5).min(1.2)
    }

    /// Рассчитать N-фактор промерзания
    fn calculate_freezing_nfactor(
        snow_depth: f64,
        snow_density: f64,
        vegetation_cover: f64,
    ) -> f64 {
        // Базовое значение для голой поверхности без снега
        let base = 1.0;

        // Снег изолирует грунт, снижая N-фактор
        let snow_effect = if snow_depth > 0.0 {
            // Эффективная изоляция зависит от глубины и плотности
            let snow_insulation = snow_depth * (400.0 - snow_density) / 400.0;
            let insulation_factor = (snow_insulation / 0.5).min(1.0); // Нормализация к 0.5м
            0.5 * insulation_factor
        } else {
            0.0
        };

        // Растительность также немного изолирует зимой
        let veg_effect = 0.1 * vegetation_cover;

        (base - snow_effect - veg_effect).max(0.3).min(1.0)
    }

    /// Типичные N-факторы для различных типов поверхности в Якутии
    pub fn yakutia_typical(surface_type: SurfaceType) -> Self {
        match surface_type {
            SurfaceType::BareSoil => Self::new(1.0, 1.0),
            SurfaceType::Tundra => Self::new(0.8, 0.6),
            SurfaceType::LightForest => Self::new(0.7, 0.5),
            SurfaceType::DenseForest => Self::new(0.5, 0.4),
            SurfaceType::Peatland => Self::new(0.6, 0.5),
            SurfaceType::WaterBody => Self::new(1.2, 0.8),
        }
    }

    /// Применить N-фактор к температуре воздуха для получения температуры поверхности
    pub fn apply_to_air_temp(&self, air_temp: f64) -> f64 {
        if air_temp > 0.0 {
            air_temp * self.thawing
        } else {
            air_temp * self.freezing
        }
    }

    /// Применить N-факторы к индексам оттаивания/промерзания
    pub fn apply_to_indices(&self, ddt_air: f64, ddf_air: f64) -> (f64, f64) {
        let ddt_surface = ddt_air * self.thawing;
        let ddf_surface = ddf_air * self.freezing;
        (ddt_surface, ddf_surface)
    }
}

/// Типы поверхности для Якутии
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SurfaceType {
    /// Голая почва
    BareSoil,
    /// Тундра
    Tundra,
    /// Редкий лес
    LightForest,
    /// Густой лес
    DenseForest,
    /// Торфяник
    Peatland,
    /// Водоем
    WaterBody,
}

/// Калькулятор температуры поверхности грунта
pub struct SurfaceTemperatureCalculator {
    n_factor: NFactor,
}

impl SurfaceTemperatureCalculator {
    /// Создать новый калькулятор
    pub fn new(n_factor: NFactor) -> Self {
        Self { n_factor }
    }

    /// Создать для типичных условий Якутии
    pub fn for_yakutia(surface_type: SurfaceType) -> Self {
        Self::new(NFactor::yakutia_typical(surface_type))
    }

    /// Рассчитать среднегодовую температуру поверхности (MAGST)
    pub fn calculate_magst(&self, _maat: f64, ddt_air: f64, ddf_air: f64) -> f64 {
        let (ddt_surface, ddf_surface) = self.n_factor.apply_to_indices(ddt_air, ddf_air);

        // MAGST = (DDT_surface - DDF_surface) / 365
        (ddt_surface - ddf_surface) / 365.0
    }

    /// Рассчитать температуру поверхности для заданного дня
    pub fn calculate_surface_temp(&self, air_temp: f64) -> f64 {
        self.n_factor.apply_to_air_temp(air_temp)
    }

    /// Получить N-факторы
    pub fn n_factors(&self) -> &NFactor {
        &self.n_factor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nfactor_bare_soil() {
        let nf = NFactor::yakutia_typical(SurfaceType::BareSoil);

        assert_eq!(nf.thawing, 1.0);
        assert_eq!(nf.freezing, 1.0);
    }

    #[test]
    fn test_nfactor_forest() {
        let nf = NFactor::yakutia_typical(SurfaceType::DenseForest);

        // Лес снижает оба фактора
        assert!(nf.thawing < 1.0);
        assert!(nf.freezing < 1.0);

        // Зимой изоляция сильнее
        assert!(nf.freezing < nf.thawing);
    }

    #[test]
    fn test_nfactor_from_conditions() {
        let nf = NFactor::from_surface_conditions(
            0.5,   // 50% растительности
            0.3,   // 30 см высота
            0.4,   // 40 см снега
            250.0, // плотность снега
        );

        // Растительность снижает N_thaw
        assert!(nf.thawing < 1.0);

        // Снег снижает N_freeze
        assert!(nf.freezing < 1.0);

        println!("N_thaw: {:.2}, N_freeze: {:.2}", nf.thawing, nf.freezing);
    }

    #[test]
    fn test_apply_to_air_temp() {
        let nf = NFactor::new(0.8, 0.6);

        let t_surface_summer = nf.apply_to_air_temp(20.0);
        let t_surface_winter = nf.apply_to_air_temp(-30.0);

        // Летом: 20 * 0.8 = 16
        assert!((t_surface_summer - 16.0).abs() < 0.1);

        // Зимой: -30 * 0.6 = -18
        assert!((t_surface_winter + 18.0).abs() < 0.1);
    }

    #[test]
    fn test_apply_to_indices() {
        let nf = NFactor::new(0.8, 0.6);

        let (ddt_surface, ddf_surface) = nf.apply_to_indices(1000.0, 3000.0);

        assert_eq!(ddt_surface, 800.0);
        assert_eq!(ddf_surface, 1800.0);
    }

    #[test]
    fn test_surface_temp_calculator() {
        let calc = SurfaceTemperatureCalculator::for_yakutia(SurfaceType::Tundra);

        let maat = -10.0;
        let ddt_air = 800.0;
        let ddf_air = 3000.0;

        let magst = calc.calculate_magst(maat, ddt_air, ddf_air);

        // MAGST должен быть холоднее MAAT из-за N-факторов
        println!("MAAT: {:.1}°C, MAGST: {:.1}°C", maat, magst);

        // Проверка разумности - MAGST может быть теплее из-за изоляции снега зимой
        assert!(magst > -15.0 && magst < 0.0);
    }

    #[test]
    fn test_snow_insulation() {
        // Без снега
        let nf_no_snow = NFactor::from_surface_conditions(0.0, 0.0, 0.0, 0.0);

        // С глубоким снегом
        let nf_deep_snow = NFactor::from_surface_conditions(0.0, 0.0, 0.8, 200.0);

        // Снег должен сильно снизить N_freeze
        assert!(nf_deep_snow.freezing < nf_no_snow.freezing);

        println!(
            "No snow N_freeze: {:.2}, Deep snow N_freeze: {:.2}",
            nf_no_snow.freezing, nf_deep_snow.freezing
        );
    }

    #[test]
    fn test_vegetation_effect() {
        // Без растительности
        let nf_bare = NFactor::from_surface_conditions(0.0, 0.0, 0.0, 0.0);

        // С густой растительностью
        let nf_veg = NFactor::from_surface_conditions(0.8, 0.5, 0.0, 0.0);

        // Растительность должна снизить N_thaw
        assert!(nf_veg.thawing < nf_bare.thawing);

        println!(
            "Bare N_thaw: {:.2}, Vegetated N_thaw: {:.2}",
            nf_bare.thawing, nf_veg.thawing
        );
    }
}
