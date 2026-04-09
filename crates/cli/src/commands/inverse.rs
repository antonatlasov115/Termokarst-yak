//! Команда обратного моделирования - определение времени образования термокарста

use anyhow::Result;
use clap::Args;
use thermokarst_core::SoilType;
use thermokarst_simulation::{InverseModelingCalculator, ObservedThermokarst, SatelliteDataClient, print_data_access_guide};

#[derive(Args, Debug)]
pub struct InverseArgs {
    /// Глубина термокарста (м)
    #[arg(short, long, required_unless_present = "satellite_guide")]
    depth: Option<f64>,

    /// Диаметр термокарста (м)
    #[arg(short = 'D', long, required_unless_present = "satellite_guide")]
    diameter: Option<f64>,

    /// Широта
    #[arg(long, required_unless_present = "satellite_guide")]
    latitude: Option<f64>,

    /// Долгота
    #[arg(long, required_unless_present = "satellite_guide")]
    longitude: Option<f64>,

    /// Год наблюдения
    #[arg(short, long, default_value = "2026")]
    year: i32,

    /// NDVI окружающей территории (0-1)
    #[arg(long, default_value = "0.4")]
    ndvi: f64,

    /// Тип грунта
    #[arg(short, long, default_value = "silt")]
    soil: String,

    /// Показать руководство по получению спутниковых данных
    #[arg(long)]
    satellite_guide: bool,

    /// Поиск спутниковых снимков для области
    #[arg(long)]
    search_satellite: bool,
}

pub fn run(args: InverseArgs) -> Result<()> {
    if args.satellite_guide {
        print_data_access_guide();
        return Ok(());
    }

    // Проверяем что все обязательные параметры указаны
    let depth = args.depth.ok_or_else(|| anyhow::anyhow!("--depth обязателен"))?;
    let diameter = args.diameter.ok_or_else(|| anyhow::anyhow!("--diameter обязателен"))?;
    let latitude = args.latitude.ok_or_else(|| anyhow::anyhow!("--latitude обязателен"))?;
    let longitude = args.longitude.ok_or_else(|| anyhow::anyhow!("--longitude обязателен"))?;

    println!("🔄 Обратное моделирование термокарста\n");

    // Парсинг типа грунта
    let soil_type = match args.soil.to_lowercase().as_str() {
        "peat" | "торф" => SoilType::Peat,
        "sand" | "песок" => SoilType::Sand,
        "clay" | "глина" => SoilType::Clay,
        "silt" | "суглинок" => SoilType::Silt,
        _ => {
            eprintln!("⚠️  Неизвестный тип грунта, используется суглинок");
            SoilType::Silt
        }
    };

    // Создаем наблюдаемый объект
    let observed = ObservedThermokarst {
        depth_m: depth,
        diameter_m: diameter,
        surface_area_m2: std::f64::consts::PI * (diameter / 2.0).powi(2),
        surrounding_ndvi: args.ndvi,
        observation_year: args.year,
        coordinates: (latitude, longitude),
    };

    println!("📊 Наблюдаемые параметры:");
    println!("   Координаты: {:.4}°N, {:.4}°E", latitude, longitude);
    println!("   Глубина: {:.2} м", depth);
    println!("   Диаметр: {:.2} м", diameter);
    println!("   Площадь: {:.1} м²", observed.surface_area_m2);
    println!("   NDVI окружения: {:.2}", args.ndvi);
    println!("   Год наблюдения: {}", args.year);
    println!("   Тип грунта: {:?}\n", soil_type);

    // Выполняем обратное моделирование
    let calculator = InverseModelingCalculator::new(args.year);
    let result = calculator.estimate_formation_time(&observed, soil_type)?;

    // Валидация
    if !calculator.validate_result(&result) {
        eprintln!("⚠️  Предупреждение: результаты могут быть неточными");
    }

    // Вывод результатов
    println!("✅ Результаты обратного моделирования:\n");
    println!("🕐 Оценка возраста:");
    println!("   Возраст термокарста: {:.1} лет", result.estimated_age_years);
    println!("   Доверительный интервал: {:.1} - {:.1} лет",
             result.confidence_interval.0, result.confidence_interval.1);
    println!("   Вероятная дата начала: ~{} год", result.estimated_start_year);
    println!("   Качество оценки: {:.1}%\n", result.estimation_quality * 100.0);

    println!("🌍 Восстановленные параметры среды:");
    println!("   Летняя температура воздуха: {:.1}°C", result.inferred_params.air_temp);
    println!("   Температура мерзлоты: {:.1}°C", result.inferred_params.permafrost_temp);
    println!("   Льдистость грунта: {:.1}%", result.inferred_params.ice_content * 100.0);
    println!("   Растительный покров: {:.1}%", result.inferred_params.vegetation_cover * 100.0);
    println!("   Доступность воды: {:.1}%\n", result.inferred_params.water_availability * 100.0);

    // Интерпретация
    println!("💡 Интерпретация:");
    if result.estimated_age_years < 10.0 {
        println!("   Молодой термокарст, активная фаза развития");
    } else if result.estimated_age_years < 50.0 {
        println!("   Зрелый термокарст, стадия активного роста");
    } else if result.estimated_age_years < 100.0 {
        println!("   Старый термокарст, возможна стабилизация");
    } else {
        println!("   Очень старый термокарст, вероятно стабилизирован");
    }

    if result.estimation_quality > 0.8 {
        println!("   Высокая уверенность в оценке ✓");
    } else if result.estimation_quality > 0.6 {
        println!("   Средняя уверенность в оценке");
    } else {
        println!("   Низкая уверенность - требуется больше данных ⚠️");
    }

    // Рекомендации
    println!("\n📌 Рекомендации:");
    println!("   • Используйте спутниковые снимки для уточнения диаметра");
    println!("   • Батиметрия поможет точнее определить глубину");
    println!("   • Временной ряд снимков покажет динамику роста");
    println!("   • Полевые измерения повысят точность оценки\n");

    // Поиск спутниковых данных
    if args.search_satellite {
        println!("🛰️  Поиск доступных спутниковых снимков...\n");
        println!("💡 Для получения реальных спутниковых данных используйте:");
        println!("   cargo run --release -- inverse --satellite-guide\n");

        println!("📌 Рекомендуемые источники для координат {:.4}°N, {:.4}°E:", latitude, longitude);
        println!("   • Sentinel-2: https://scihub.copernicus.eu/");
        println!("   • Landsat: https://earthexplorer.usgs.gov/");
        println!("   • Google Earth Engine: https://earthengine.google.com/\n");
    }

    Ok(())
}
