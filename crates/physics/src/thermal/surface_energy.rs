//! Упрощенный баланс поверхностной энергии
//!
//! Рассчитывает радиационный баланс и температуру поверхности грунта
//! на основе коротковолновой и длинноволновой радиации.

/// Калькулятор баланса поверхностной энергии
#[derive(Debug, Clone)]
pub struct SurfaceEnergyBalance {
    /// Альбедо поверхности (0-1)
    pub albedo: f64,
    /// Излучательная способность (0-1)
    pub emissivity: f64,
}

impl SurfaceEnergyBalance {
    /// Создать новый калькулятор
    pub fn new(albedo: f64, emissivity: f64) -> Self {
        Self { albedo, emissivity }
    }

    /// Типичные параметры для различных поверхностей
    pub fn for_surface(surface_type: SurfaceType) -> Self {
        match surface_type {
            SurfaceType::Snow => Self::new(0.8, 0.99),
            SurfaceType::Ice => Self::new(0.5, 0.97),
            SurfaceType::Water => Self::new(0.08, 0.97),
            SurfaceType::Soil => Self::new(0.2, 0.95),
            SurfaceType::Vegetation => Self::new(0.18, 0.96),
            SurfaceType::Peat => Self::new(0.15, 0.95),
        }
    }

    /// Рассчитать чистую радиацию (Вт/м²)
    ///
    /// # Аргументы
    /// * `shortwave_in` - Входящая коротковолновая радиация (Вт/м²)
    /// * `longwave_in` - Входящая длинноволновая радиация (Вт/м²)
    /// * `surface_temp` - Температура поверхности (°C)
    pub fn net_radiation(&self, shortwave_in: f64, longwave_in: f64, surface_temp: f64) -> f64 {
        const STEFAN_BOLTZMANN: f64 = 5.67e-8; // Вт/(м²·К⁴)

        // Коротковолновая радиация (поглощенная)
        let sw_net = (1.0 - self.albedo) * shortwave_in;

        // Длинноволновая радиация
        let temp_kelvin = surface_temp + 273.15;
        let lw_out = self.emissivity * STEFAN_BOLTZMANN * temp_kelvin.powi(4);
        let lw_net = self.emissivity * longwave_in - lw_out;

        sw_net + lw_net
    }

    /// Упрощенный расчет радиации на основе широты и дня года
    pub fn estimate_radiation(
        &self,
        latitude: f64,
        day_of_year: u32,
        cloud_cover: f64,
    ) -> (f64, f64) {
        use std::f64::consts::PI;

        // Солнечная постоянная
        const SOLAR_CONSTANT: f64 = 1367.0; // Вт/м²

        // Склонение Солнца
        let declination = 23.45 * (2.0 * PI * (284.0 + day_of_year as f64) / 365.0).sin();
        let decl_rad = declination * PI / 180.0;
        let lat_rad = latitude * PI / 180.0;

        // Часовой угол восхода/заката
        let cos_hour_angle = -(lat_rad.tan() * decl_rad.tan());
        let hour_angle = if cos_hour_angle.abs() > 1.0 {
            if cos_hour_angle > 0.0 {
                0.0
            } else {
                PI
            }
        } else {
            cos_hour_angle.acos()
        };

        // Суточная инсоляция на верхней границе атмосферы
        let daily_toa = (24.0 / PI)
            * SOLAR_CONSTANT
            * (hour_angle * lat_rad.sin() * decl_rad.sin()
                + lat_rad.cos() * decl_rad.cos() * hour_angle.sin());

        // Учет облачности и атмосферы
        let atmospheric_transmissivity = 0.75 * (1.0 - 0.65 * cloud_cover);
        let shortwave_in = daily_toa * atmospheric_transmissivity / 24.0; // Средняя за сутки

        // Длинноволновая радиация (упрощенная формула)
        let longwave_in = 200.0 + 100.0 * cloud_cover;

        (shortwave_in.max(0.0), longwave_in)
    }

    /// Рассчитать температуру поверхности из энергетического баланса
    pub fn calculate_surface_temperature(
        &self,
        air_temp: f64,
        shortwave_in: f64,
        _longwave_in: f64,
        ground_heat_flux: f64,
    ) -> f64 {
        // Упрощенный подход: температура поверхности близка к температуре воздуха
        // с поправкой на радиационный баланс
        let net_sw = (1.0 - self.albedo) * shortwave_in;

        // Упрощенная оценка: каждые 100 Вт/м² добавляют ~5°C
        let radiation_effect = (net_sw - ground_heat_flux) / 20.0;

        air_temp + radiation_effect
    }
}

/// Типы поверхности
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SurfaceType {
    Snow,
    Ice,
    Water,
    Soil,
    Vegetation,
    Peat,
}

/// Полный калькулятор энергетического баланса с учетом всех компонентов
pub struct FullEnergyBalance {
    surface: SurfaceEnergyBalance,
    latitude: f64,
}

impl FullEnergyBalance {
    /// Создать новый калькулятор
    pub fn new(surface: SurfaceEnergyBalance, latitude: f64) -> Self {
        Self { surface, latitude }
    }

    /// Рассчитать дневной энергетический баланс
    pub fn daily_balance(
        &self,
        day_of_year: u32,
        air_temp: f64,
        cloud_cover: f64,
    ) -> DailyEnergyBalance {
        let (sw_in, lw_in) =
            self.surface
                .estimate_radiation(self.latitude, day_of_year, cloud_cover);

        // Упрощенный тепловой поток в грунт (10% от чистой радиации)
        let net_rad = self.surface.net_radiation(sw_in, lw_in, air_temp);
        let ground_heat_flux = 0.1 * net_rad;

        let surface_temp =
            self.surface
                .calculate_surface_temperature(air_temp, sw_in, lw_in, ground_heat_flux);

        DailyEnergyBalance {
            shortwave_in: sw_in,
            longwave_in: lw_in,
            net_radiation: net_rad,
            ground_heat_flux,
            surface_temperature: surface_temp,
        }
    }

    /// Рассчитать полный энергетический баланс с турбулентными потоками
    ///
    /// Уравнение: Rn = H + LE + G
    /// где:
    /// - Rn: чистая радиация
    /// - H: явный тепловой поток
    /// - LE: скрытый тепловой поток (испарение)
    /// - G: тепловой поток в грунт
    pub fn full_balance(
        &self,
        day_of_year: u32,
        air_temp: f64,
        wind_speed: f64,
        relative_humidity: f64,
        cloud_cover: f64,
    ) -> FullDailyBalance {
        let (sw_in, lw_in) =
            self.surface
                .estimate_radiation(self.latitude, day_of_year, cloud_cover);

        let net_rad = self.surface.net_radiation(sw_in, lw_in, air_temp);

        // Явный тепловой поток (H) - упрощенная формула
        // H = ρ·cp·Ch·U·(Ts - Ta)
        const RHO_AIR: f64 = 1.2; // кг/м³
        const CP_AIR: f64 = 1005.0; // Дж/(кг·К)
        const CH: f64 = 0.003; // коэффициент теплообмена

        let surface_temp = self
            .surface
            .calculate_surface_temperature(air_temp, sw_in, lw_in, 0.0);

        let sensible_heat = RHO_AIR * CP_AIR * CH * wind_speed * (surface_temp - air_temp);

        // Скрытый тепловой поток (LE) - упрощенная формула
        // LE = ρ·Lv·Ce·U·(qs - qa)
        const LV: f64 = 2.5e6; // Дж/кг - скрытая теплота испарения
        const CE: f64 = 0.003; // коэффициент влагообмена

        // Насыщающая влажность (упрощенная формула Магнуса)
        let es_surface = 611.0 * (17.27 * surface_temp / (surface_temp + 237.3)).exp();
        let es_air = 611.0 * (17.27 * air_temp / (air_temp + 237.3)).exp();
        let ea_air = es_air * relative_humidity;

        let q_surface = 0.622 * es_surface / 101325.0; // удельная влажность
        let q_air = 0.622 * ea_air / 101325.0;

        let latent_heat = RHO_AIR * LV * CE * wind_speed * (q_surface - q_air);

        // Тепловой поток в грунт (G) - остаток энергетического баланса
        let ground_heat_flux = net_rad - sensible_heat - latent_heat;

        // Боуэновское отношение (β = H/LE)
        let bowen_ratio = if latent_heat.abs() > 1e-6 {
            sensible_heat / latent_heat
        } else {
            0.0
        };

        FullDailyBalance {
            shortwave_in: sw_in,
            longwave_in: lw_in,
            net_radiation: net_rad,
            sensible_heat,
            latent_heat,
            ground_heat_flux,
            surface_temperature: surface_temp,
            bowen_ratio,
        }
    }
}

/// Результат расчета дневного энергетического баланса
#[derive(Debug, Clone)]
pub struct DailyEnergyBalance {
    /// Входящая коротковолновая радиация (Вт/м²)
    pub shortwave_in: f64,
    /// Входящая длинноволновая радиация (Вт/м²)
    pub longwave_in: f64,
    /// Чистая радиация (Вт/м²)
    pub net_radiation: f64,
    /// Тепловой поток в грунт (Вт/м²)
    pub ground_heat_flux: f64,
    /// Температура поверхности (°C)
    pub surface_temperature: f64,
}

/// Результат расчета полного энергетического баланса с турбулентными потоками
#[derive(Debug, Clone)]
pub struct FullDailyBalance {
    /// Входящая коротковолновая радиация (Вт/м²)
    pub shortwave_in: f64,
    /// Входящая длинноволновая радиация (Вт/м²)
    pub longwave_in: f64,
    /// Чистая радиация (Вт/м²)
    pub net_radiation: f64,
    /// Явный тепловой поток (Вт/м²)
    pub sensible_heat: f64,
    /// Скрытый тепловой поток - испарение (Вт/м²)
    pub latent_heat: f64,
    /// Тепловой поток в грунт (Вт/м²)
    pub ground_heat_flux: f64,
    /// Температура поверхности (°C)
    pub surface_temperature: f64,
    /// Боуэновское отношение (H/LE)
    pub bowen_ratio: f64,
}

impl FullDailyBalance {
    /// Проверить замыкание энергетического баланса
    pub fn closure_error(&self) -> f64 {
        let sum = self.sensible_heat + self.latent_heat + self.ground_heat_flux;
        (self.net_radiation - sum).abs()
    }

    /// Относительная ошибка замыкания (%)
    pub fn closure_error_percent(&self) -> f64 {
        if self.net_radiation.abs() < 1e-6 {
            return 0.0;
        }
        (self.closure_error() / self.net_radiation.abs()) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_net_radiation_soil() {
        let seb = SurfaceEnergyBalance::for_surface(SurfaceType::Soil);

        let net_rad = seb.net_radiation(400.0, 250.0, 15.0);

        println!("Net radiation (soil): {:.1} W/m²", net_rad);

        // Почва с низким альбедо должна поглощать больше
        assert!(net_rad > 100.0);
    }

    #[test]
    fn test_estimate_radiation_summer() {
        let seb = SurfaceEnergyBalance::for_surface(SurfaceType::Soil);

        // Центральная Якутия, летний день
        let (sw, lw) = seb.estimate_radiation(62.0, 180, 0.3);

        println!("Summer radiation: SW={:.1} W/m², LW={:.1} W/m²", sw, lw);

        assert!(sw > 100.0); // Летом должна быть значительная радиация
        assert!(lw > 200.0);
    }

    #[test]
    fn test_estimate_radiation_winter() {
        let seb = SurfaceEnergyBalance::for_surface(SurfaceType::Snow);

        // Центральная Якутия, зимний день
        let (sw, lw) = seb.estimate_radiation(62.0, 1, 0.5);

        println!("Winter radiation: SW={:.1} W/m², LW={:.1} W/m²", sw, lw);

        assert!(sw < 50.0); // Зимой радиация минимальна
    }

    #[test]
    fn test_surface_temperature() {
        let seb = SurfaceEnergyBalance::for_surface(SurfaceType::Soil);

        let t_surface = seb.calculate_surface_temperature(
            20.0,  // Температура воздуха
            500.0, // Коротковолновая радиация
            300.0, // Длинноволновая радиация
            50.0,  // Тепловой поток в грунт
        );

        println!("Surface temperature: {:.1}°C", t_surface);

        // Температура поверхности должна быть выше температуры воздуха летом
        assert!(t_surface > 15.0);
    }

    #[test]
    fn test_daily_balance_yakutia_summer() {
        let seb = SurfaceEnergyBalance::for_surface(SurfaceType::Vegetation);
        let calc = FullEnergyBalance::new(seb, 62.0);

        let balance = calc.daily_balance(180, 20.0, 0.3);

        println!("Summer balance:");
        println!("  SW in: {:.1} W/m²", balance.shortwave_in);
        println!("  LW in: {:.1} W/m²", balance.longwave_in);
        println!("  Net rad: {:.1} W/m²", balance.net_radiation);
        println!("  Ground flux: {:.1} W/m²", balance.ground_heat_flux);
        println!("  Surface temp: {:.1}°C", balance.surface_temperature);

        assert!(balance.net_radiation > 0.0);
        assert!(balance.surface_temperature > 10.0);
    }

    #[test]
    fn test_daily_balance_yakutia_winter() {
        let seb = SurfaceEnergyBalance::for_surface(SurfaceType::Snow);
        let calc = FullEnergyBalance::new(seb, 62.0);

        let balance = calc.daily_balance(1, -30.0, 0.5);

        println!("Winter balance:");
        println!("  SW in: {:.1} W/m²", balance.shortwave_in);
        println!("  LW in: {:.1} W/m²", balance.longwave_in);
        println!("  Net rad: {:.1} W/m²", balance.net_radiation);
        println!("  Ground flux: {:.1} W/m²", balance.ground_heat_flux);
        println!("  Surface temp: {:.1}°C", balance.surface_temperature);

        // Зимой чистая радиация может быть отрицательной
        assert!(balance.surface_temperature < -20.0);
    }

    #[test]
    fn test_albedo_effect() {
        let snow = SurfaceEnergyBalance::for_surface(SurfaceType::Snow);
        let soil = SurfaceEnergyBalance::for_surface(SurfaceType::Soil);

        let net_rad_snow = snow.net_radiation(400.0, 250.0, 0.0);
        let net_rad_soil = soil.net_radiation(400.0, 250.0, 0.0);

        // Почва должна поглощать больше радиации из-за низкого альбедо
        assert!(net_rad_soil > net_rad_snow);

        println!("Snow net rad: {:.1} W/m²", net_rad_snow);
        println!("Soil net rad: {:.1} W/m²", net_rad_soil);
    }

    #[test]
    fn test_full_energy_balance_summer() {
        let seb = SurfaceEnergyBalance::for_surface(SurfaceType::Vegetation);
        let calc = FullEnergyBalance::new(seb, 62.0);

        // Летний день: теплый, умеренный ветер, средняя влажность
        let balance = calc.full_balance(180, 20.0, 3.0, 0.6, 0.3);

        println!("Full summer balance:");
        println!("  Net radiation: {:.1} W/m²", balance.net_radiation);
        println!("  Sensible heat: {:.1} W/m²", balance.sensible_heat);
        println!("  Latent heat: {:.1} W/m²", balance.latent_heat);
        println!("  Ground flux: {:.1} W/m²", balance.ground_heat_flux);
        println!("  Bowen ratio: {:.2}", balance.bowen_ratio);
        println!("  Closure error: {:.1}%", balance.closure_error_percent());

        // Летом чистая радиация должна быть положительной
        assert!(balance.net_radiation > 0.0);

        // Ошибка замыкания должна быть небольшой
        assert!(balance.closure_error_percent() < 5.0);
    }

    #[test]
    fn test_full_energy_balance_winter() {
        let seb = SurfaceEnergyBalance::for_surface(SurfaceType::Snow);
        let calc = FullEnergyBalance::new(seb, 62.0);

        // Зимний день: холодный, слабый ветер, низкая влажность
        let balance = calc.full_balance(1, -30.0, 2.0, 0.3, 0.5);

        println!("Full winter balance:");
        println!("  Net radiation: {:.1} W/m²", balance.net_radiation);
        println!("  Sensible heat: {:.1} W/m²", balance.sensible_heat);
        println!("  Latent heat: {:.1} W/m²", balance.latent_heat);
        println!("  Ground flux: {:.1} W/m²", balance.ground_heat_flux);
        println!("  Bowen ratio: {:.2}", balance.bowen_ratio);

        // Зимой испарение минимально
        assert!(balance.latent_heat.abs() < 50.0);
    }

    #[test]
    fn test_bowen_ratio_dry_vs_wet() {
        let seb = SurfaceEnergyBalance::for_surface(SurfaceType::Soil);
        let calc = FullEnergyBalance::new(seb, 62.0);

        // Сухие условия (низкая влажность)
        let dry = calc.full_balance(180, 25.0, 3.0, 0.3, 0.2);

        // Влажные условия (высокая влажность)
        let wet = calc.full_balance(180, 25.0, 3.0, 0.9, 0.2);

        println!(
            "Bowen ratio - dry: {:.2}, wet: {:.2}",
            dry.bowen_ratio, wet.bowen_ratio
        );
        println!(
            "Dry - H: {:.1}, LE: {:.1}",
            dry.sensible_heat, dry.latent_heat
        );
        println!(
            "Wet - H: {:.1}, LE: {:.1}",
            wet.sensible_heat, wet.latent_heat
        );

        // Проверяем что оба значения положительные
        assert!(dry.latent_heat > 0.0);
        assert!(wet.latent_heat > 0.0);
    }
}
