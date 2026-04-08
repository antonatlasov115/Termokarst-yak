//! Комплексный тест всех улучшений модели Termokarst-yak
//!
//! Демонстрирует:
//! 1. Улучшенную формулу Атласова с новыми параметрами
//! 2. Полный энергетический баланс поверхности
//! 3. Анализ неопределенности прогнозов
//! 4. 2D теплоперенос
//! 5. Влияние влажности грунта
//! 6. Новые стадии развития термокарста

use thermokarst_core::{EnvironmentParams, SoilType};
use thermokarst_geology::StabilityAnalyzer;
use thermokarst_physics::{
    FullEnergyBalance, Grid2D, HeatTransfer2D, SurfaceEnergyBalance,
    SurfaceType as SurfaceEnergyType, ThawDepthCalculator,
};
use thermokarst_simulation::{SimulationEngine, UncertaintyAnalyzer, UncertaintyParams};

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  КОМПЛЕКСНЫЙ ТЕСТ УЛУЧШЕНИЙ МОДЕЛИ TERMOKARST-YAK          ║");
    println!("║  Дата: 2026-04-08                                           ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Тест 1: Улучшенная формула Атласова
    test_improved_atlasov_formula();

    // Тест 2: Полный энергетический баланс
    test_full_energy_balance();

    // Тест 3: Анализ неопределенности
    test_uncertainty_analysis();

    // Тест 4: 2D теплоперенос
    test_2d_heat_transfer();

    // Тест 5: Влияние влажности
    test_moisture_effect();

    // Тест 6: Новые стадии термокарста
    test_thermokarst_stages();

    // Тест 7: Интегрированная симуляция
    test_integrated_simulation();

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  ✅ ВСЕ ТЕСТЫ УСПЕШНО ЗАВЕРШЕНЫ                             ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}

fn test_improved_atlasov_formula() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("ТЕСТ 1: Улучшенная формула Атласова");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Центральная Якутия
    let mut params = EnvironmentParams::default();
    params.air_temp = 5.0;
    params.ice_content = 0.70;
    params.vegetation_cover = 0.8; // Лес
    params.water_availability = 0.6;
    params.temperature_amplitude = 88.0;
    params.warm_season_days = 120;

    let calc = ThawDepthCalculator::new(params.clone());

    println!("📍 Параметры: Центральная Якутия");
    println!("   Температура: {:.1}°C", params.air_temp);
    println!("   Льдистость: {:.0}%", params.ice_content * 100.0);
    println!("   Растительность: {:.0}%", params.vegetation_cover * 100.0);
    println!("   Влажность: {:.0}%", params.water_availability * 100.0);
    println!("   Амплитуда ΔT: {:.0}°C", params.temperature_amplitude);

    // Расчет глубины протаивания
    let depth_year1 = calc.calculate(1).unwrap();
    let depth_year10 = calc.calculate(10).unwrap();
    let depth_year50 = calc.calculate(50).unwrap();

    println!("\n📊 Результаты (новая формула):");
    println!("   Год 1:  {:.2} м (активный слой)", depth_year1);
    println!("   Год 10: {:.2} м", depth_year10);
    println!("   Год 50: {:.2} м", depth_year50);

    // Коэффициенты
    println!("\n🔬 Коэффициенты формулы:");
    println!("   K_fire (β=0.30): {:.3}", calc.k_fire());
    println!("   f_continental:   {:.3}", calc.f_continental());

    // Сравнение: лес vs гарь
    params.vegetation_cover = 0.0; // Гарь
    let calc_burned = ThawDepthCalculator::new(params);
    let depth_burned = calc_burned.calculate(10).unwrap();
    let increase = (depth_burned / depth_year10 - 1.0) * 100.0;

    println!("\n🔥 Эффект пожара (V: 0.8 → 0.0):");
    println!("   Лес:  {:.2} м", depth_year10);
    println!("   Гарь: {:.2} м", depth_burned);
    println!("   Увеличение: +{:.1}% (ожидается 30-50%)", increase);

    assert!(
        increase > 20.0 && increase < 60.0,
        "Эффект пожара в допустимых пределах"
    );
    println!("   ✅ Эффект пожара соответствует наблюдениям\n");
}

fn test_full_energy_balance() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("ТЕСТ 2: Полный энергетический баланс поверхности (SEB)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let seb = SurfaceEnergyBalance::for_surface(SurfaceEnergyType::Vegetation);
    let calc = FullEnergyBalance::new(seb, 62.0); // Центральная Якутия

    // Летний день
    println!("☀️  Летний день (июль):");
    let summer = calc.full_balance(
        180,  // день года
        20.0, // температура воздуха
        3.0,  // скорость ветра
        0.6,  // относительная влажность
        0.3,  // облачность
    );

    println!(
        "   Rn (чистая радиация):    {:>7.1} Вт/м²",
        summer.net_radiation
    );
    println!(
        "   H  (явный поток):        {:>7.1} Вт/м²",
        summer.sensible_heat
    );
    println!(
        "   LE (скрытый поток):      {:>7.1} Вт/м²",
        summer.latent_heat
    );
    println!(
        "   G  (поток в грунт):      {:>7.1} Вт/м²",
        summer.ground_heat_flux
    );
    println!("   β  (Боуэновское отн.):   {:>7.2}", summer.bowen_ratio);
    println!(
        "   Ошибка замыкания:        {:>7.1}%",
        summer.closure_error_percent()
    );

    assert!(
        summer.closure_error_percent() < 10.0,
        "Ошибка замыкания энергобаланса < 10%"
    );
    println!("   ✅ Энергетический баланс замыкается");

    // Зимний день
    println!("\n❄️  Зимний день (январь):");
    let winter = calc.full_balance(1, -30.0, 2.0, 0.3, 0.5);

    println!(
        "   Rn (чистая радиация):    {:>7.1} Вт/м²",
        winter.net_radiation
    );
    println!(
        "   H  (явный поток):        {:>7.1} Вт/м²",
        winter.sensible_heat
    );
    println!(
        "   LE (скрытый поток):      {:>7.1} Вт/м²",
        winter.latent_heat
    );
    println!(
        "   G  (поток в грунт):      {:>7.1} Вт/м²",
        winter.ground_heat_flux
    );

    assert!(
        winter.latent_heat.abs() < 50.0,
        "Зимой испарение минимально"
    );
    println!("   ✅ Зимой испарение минимально\n");
}

fn test_uncertainty_analysis() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("ТЕСТ 3: Анализ неопределенности прогнозов (Монте-Карло)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let params = EnvironmentParams::default();

    let uncertainty_params = UncertaintyParams {
        air_temp_uncertainty: 10.0,
        ice_content_uncertainty: 15.0,
        vegetation_uncertainty: 20.0,
        thermal_conductivity_uncertainty: 10.0,
        n_simulations: 500, // Уменьшено для скорости
    };

    let analyzer = UncertaintyAnalyzer::new(uncertainty_params);

    println!("🎲 Параметры Монте-Карло:");
    println!("   Количество симуляций: 500");
    println!("   Неопределенность температуры: ±10%");
    println!("   Неопределенность льдистости: ±15%");
    println!("   Неопределенность растительности: ±20%");

    // Симуляция глубины протаивания на 10 лет
    println!("\n⏳ Выполнение 500 симуляций...");
    let result = analyzer
        .monte_carlo_analysis(&params, |p| {
            let calc = ThawDepthCalculator::new(p.clone());
            calc.calculate(10)
        })
        .unwrap();

    println!("\n📊 Результаты анализа неопределенности:");
    println!("   Среднее:              {:.2} м", result.mean);
    println!("   Медиана:              {:.2} м", result.median);
    println!("   Стандартное откл.:    {:.2} м", result.std_dev);
    println!(
        "   90% ДИ:               [{:.2}, {:.2}] м",
        result.percentile_5, result.percentile_95
    );
    println!(
        "   Коэфф. вариации:      {:.1}%",
        result.coefficient_of_variation * 100.0
    );
    println!(
        "   Относит. неопред.:    {:.1}%",
        result.relative_uncertainty()
    );

    assert!(
        result.relative_uncertainty() < 50.0,
        "Неопределенность < 50%"
    );
    println!("   ✅ Неопределенность в допустимых пределах\n");
}

fn test_2d_heat_transfer() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("ТЕСТ 4: 2D теплоперенос (метод конечных разностей)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Создаем 2D сетку 21x21 узлов, 10м x 10м
    let mut grid = Grid2D::new(21, 21, 0.5, 0.5);

    println!("🔲 Параметры сетки:");
    println!("   Размер: 21×21 узлов");
    println!("   Область: 10×10 м");
    println!("   Шаг: 0.5 м");

    // Начальное условие: горячая точка в центре (термокарст)
    grid.set_initial_temperature(|x, z| {
        let cx = 5.0; // центр по X
        let cz = 5.0; // центр по Z
        let r2 = (x - cx).powi(2) + (z - cz).powi(2);
        if r2 < 1.0 {
            10.0 // Теплая зона (протаявший грунт)
        } else {
            -5.0 // Мерзлый грунт
        }
    });

    println!("\n🌡️  Начальные условия:");
    println!("   Центр (термокарст): +10°C");
    println!("   Окружение (мерзлота): -5°C");

    let dt = 100.0; // 100 секунд
    let mut solver = HeatTransfer2D::new(grid, dt).unwrap();

    let t_initial_center = solver.temperature_at(10, 10).unwrap();
    let t_initial_edge = solver.temperature_at(0, 10).unwrap();

    // Симуляция 1 год (365 дней)
    let simulation_time = 365.0 * 24.0 * 3600.0; // секунды
    println!("\n⏳ Симуляция: 1 год...");
    solver.simulate(simulation_time).unwrap();

    let t_final_center = solver.temperature_at(10, 10).unwrap();
    let t_final_edge = solver.temperature_at(0, 10).unwrap();

    println!("\n📊 Результаты 2D теплопереноса:");
    println!("   Центр:");
    println!("     Начало: {:>6.1}°C", t_initial_center);
    println!("     Конец:  {:>6.1}°C", t_final_center);
    println!("     Δ:      {:>6.1}°C", t_final_center - t_initial_center);
    println!("   Край:");
    println!("     Начало: {:>6.1}°C", t_initial_edge);
    println!("     Конец:  {:>6.1}°C", t_final_edge);
    println!("     Δ:      {:>6.1}°C", t_final_edge - t_initial_edge);

    // Средняя температура в области
    let avg_temp = solver.average_temperature((0, 21), (0, 21));
    println!("   Средняя температура: {:.1}°C", avg_temp);

    assert!(
        t_final_center < t_initial_center,
        "Центр охлаждается (диффузия)"
    );
    assert!(t_final_edge > t_initial_edge, "Край нагревается (диффузия)");
    println!("   ✅ Латеральная диффузия тепла работает корректно\n");
}

fn test_moisture_effect() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("ТЕСТ 5: Влияние влажности грунта на протаивание");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut params = EnvironmentParams::default();
    params.air_temp = 5.0;
    params.ice_content = 0.7;
    params.vegetation_cover = 0.5;

    println!("💧 Сравнение: сухой vs влажный грунт");

    // Сухой грунт
    params.water_availability = 0.2;
    let calc_dry = ThawDepthCalculator::new(params.clone());
    let depth_dry = calc_dry.calculate(10).unwrap();

    // Влажный грунт
    params.water_availability = 0.9;
    let calc_wet = ThawDepthCalculator::new(params);
    let depth_wet = calc_wet.calculate(10).unwrap();

    let increase = (depth_wet / depth_dry - 1.0) * 100.0;

    println!("   Сухой грунт (20% влажность):  {:.2} м", depth_dry);
    println!("   Влажный грунт (90% влажность): {:.2} м", depth_wet);
    println!("   Увеличение глубины:            +{:.1}%", increase);
    println!("   Фактор влажности (сухой):      {:.2}", 1.0 + 0.3 * 0.2);
    println!("   Фактор влажности (влажный):    {:.2}", 1.0 + 0.3 * 0.9);

    assert!(depth_wet > depth_dry, "Влажный грунт протаивает глубже");
    assert!(increase > 15.0 && increase < 35.0, "Увеличение 15-35%");
    println!("   ✅ Влияние влажности реалистично\n");
}

fn test_thermokarst_stages() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("ТЕСТ 6: Новые стадии развития термокарста");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    use thermokarst_core::ThermokarstLens;

    println!("📈 Определение стадий по глубине:");

    // Инициация
    let lens1 = ThermokarstLens::new(1.5, 4.0, 3);
    let stage1 = StabilityAnalyzer::determine_stage(&lens1);
    println!("   Глубина 1.5 м → {:?}", stage1);

    // Активное развитие
    let lens2 = ThermokarstLens::new(4.5, 12.0, 15);
    let stage2 = StabilityAnalyzer::determine_stage(&lens2);
    println!("   Глубина 4.5 м → {:?}", stage2);

    // Стабилизация
    let mut lens3 = ThermokarstLens::new(8.0, 20.0, 40);
    lens3.growth_rate = 0.3;
    let stage3 = StabilityAnalyzer::determine_stage(&lens3);
    println!("   Глубина 8.0 м (медленный рост) → {:?}", stage3);

    // Деградация
    let mut lens4 = ThermokarstLens::new(12.0, 30.0, 80);
    lens4.growth_rate = -1.0;
    let stage4 = StabilityAnalyzer::determine_stage(&lens4);
    println!("   Глубина 12.0 м (уменьшение) → {:?}", stage4);

    use thermokarst_core::ThermokarstStage;
    assert_eq!(stage1, ThermokarstStage::Initiation);
    assert_eq!(stage2, ThermokarstStage::ActiveDevelopment);
    assert_eq!(stage3, ThermokarstStage::Stabilization);
    assert_eq!(stage4, ThermokarstStage::Degradation);

    println!("   ✅ Все стадии определяются корректно\n");
}

fn test_integrated_simulation() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("ТЕСТ 7: Интегрированная симуляция (все компоненты)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut params = EnvironmentParams::default();
    params.air_temp = 5.0;
    params.ice_content = 0.70;
    params.vegetation_cover = 0.6;
    params.water_availability = 0.7;
    params.soil_type = SoilType::Loam;
    params.temperature_amplitude = 88.0;

    println!("🌍 Симуляция термокарста: Центральная Якутия, 50 лет");
    println!("   Параметры:");
    println!("     Температура: {:.1}°C", params.air_temp);
    println!("     Льдистость: {:.0}%", params.ice_content * 100.0);
    println!(
        "     Растительность: {:.0}%",
        params.vegetation_cover * 100.0
    );
    println!("     Влажность: {:.0}%", params.water_availability * 100.0);
    println!("     Грунт: {:?}", params.soil_type);

    let mut engine = SimulationEngine::new(params);

    println!("\n⏳ Выполнение симуляции...");
    let result = engine.run(50).unwrap();

    println!("\n📊 Результаты симуляции:");
    println!("   Финальное состояние (год 50):");
    println!("     Глубина:     {:.2} м", result.lens.depth);
    println!("     Диаметр:     {:.2} м", result.lens.diameter);
    println!("     Объем:       {:.1} м³", result.lens.volume);
    println!("     Стадия:      {:?}", result.stage);
    println!(
        "     Стабильность: {}",
        if result.is_stable { "✓" } else { "✗" }
    );

    // Проверка реалистичности
    assert!(
        result.lens.depth > 0.0 && result.lens.depth < 20.0,
        "Глубина реалистична"
    );
    assert!(
        result.lens.diameter > 0.0 && result.lens.diameter < 100.0,
        "Диаметр реалистичен"
    );
    assert!(result.lens.volume > 0.0, "Объем положительный");

    println!("\n   ✅ Интегрированная симуляция работает корректно");
    println!("   ✅ Все компоненты взаимодействуют правильно\n");
}
