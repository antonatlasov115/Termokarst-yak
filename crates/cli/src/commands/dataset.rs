//! Команда для работы с датасетами

use anyhow::{Context, Result};
use std::path::PathBuf;
use thermokarst_core::dataset::{create_example_yakutia_dataset, ObservationDataset};
use thermokarst_core::iryp::IrypParser;
use thermokarst_simulation::calibration::ModelCalibrator;
use thermokarst_simulation::iryp_visualization;

pub fn create(output: PathBuf) -> Result<()> {
    println!("📊 Создание примера датасета для Якутии");

    let dataset = create_example_yakutia_dataset();

    println!("\n✅ Датасет создан:");
    println!("  Название: {}", dataset.name);
    println!("  Наблюдений: {}", dataset.observations.len());

    // Статистика
    let stats = dataset.statistics();
    println!("\n📈 Статистика:");
    println!("  Всего наблюдений: {}", stats.total_observations);

    if let Some(alt_stats) = &stats.active_layer_stats {
        println!("\n  Глубина активного слоя:");
        println!("    Среднее: {:.2} м", alt_stats.mean);
        println!(
            "    Мин-Макс: {:.2} - {:.2} м",
            alt_stats.min, alt_stats.max
        );
    }

    if let Some(temp_stats) = &stats.temperature_stats {
        println!("\n  Температура грунта:");
        println!("    Среднее: {:.1}°C", temp_stats.mean);
        println!(
            "    Мин-Макс: {:.1} - {:.1}°C",
            temp_stats.min, temp_stats.max
        );
    }

    // Сохранение
    println!("\n💾 Сохранение в {:?}...", output);
    dataset.to_json_file(&output).context("Ошибка сохранения")?;

    println!("✅ Датасет сохранен");

    Ok(())
}

pub fn info(input: PathBuf) -> Result<()> {
    println!("📊 Информация о датасете");
    println!("📁 Файл: {:?}\n", input);

    let dataset = ObservationDataset::from_json_file(&input).context("Ошибка загрузки")?;

    println!("✅ Датасет загружен\n");
    println!("📋 Информация:");
    println!("  Название: {}", dataset.name);
    println!("  Описание: {}", dataset.description);
    println!("  Источник: {}", dataset.source);
    println!("  Наблюдений: {}", dataset.observations.len());

    // Статистика
    let stats = dataset.statistics();
    println!("\n📈 Статистика:");

    if let Some(alt_stats) = &stats.active_layer_stats {
        println!(
            "\n  Глубина активного слоя ({} измерений):",
            alt_stats.count
        );
        println!("    Среднее: {:.2} м", alt_stats.mean);
        println!("    Мин: {:.2} м", alt_stats.min);
        println!("    Макс: {:.2} м", alt_stats.max);
        println!("    Ст. откл: {:.2} м", alt_stats.std_dev);
    }

    if let Some(temp_stats) = &stats.temperature_stats {
        println!("\n  Температура грунта ({} измерений):", temp_stats.count);
        println!("    Среднее: {:.1}°C", temp_stats.mean);
        println!("    Мин: {:.1}°C", temp_stats.min);
        println!("    Макс: {:.1}°C", temp_stats.max);
        println!("    Ст. откл: {:.1}°C", temp_stats.std_dev);
    }

    if let Some(sub_stats) = &stats.subsidence_stats {
        println!("\n  Просадка ({} измерений):", sub_stats.count);
        println!("    Среднее: {:.2} м", sub_stats.mean);
        println!("    Мин: {:.2} м", sub_stats.min);
        println!("    Макс: {:.2} м", sub_stats.max);
        println!("    Ст. откл: {:.2} м", sub_stats.std_dev);
    }

    // Географическое распределение
    println!("\n🗺️  Географическое распределение:");
    let mut min_lat: f64 = 90.0;
    let mut max_lat: f64 = -90.0;
    let mut min_lon: f64 = 180.0;
    let mut max_lon: f64 = -180.0;

    for obs in &dataset.observations {
        let (lat, lon) = obs.coordinates;
        min_lat = min_lat.min(lat);
        max_lat = max_lat.max(lat);
        min_lon = min_lon.min(lon);
        max_lon = max_lon.max(lon);
    }

    println!("  Широта: {:.2}° - {:.2}°", min_lat, max_lat);
    println!("  Долгота: {:.2}° - {:.2}°", min_lon, max_lon);

    Ok(())
}

pub fn calibrate(input: PathBuf, output: Option<PathBuf>) -> Result<()> {
    println!("🔧 Калибровка модели по данным наблюдений");
    println!("📁 Датасет: {:?}\n", input);

    let dataset = ObservationDataset::from_json_file(&input).context("Ошибка загрузки")?;

    println!(
        "✅ Датасет загружен ({} наблюдений)",
        dataset.observations.len()
    );

    let calibrator = ModelCalibrator::new(dataset);

    println!("\n🔄 Оценка параметров...");
    let params = calibrator
        .estimate_environment_params()
        .context("Ошибка калибровки")?;

    println!("\n✅ Параметры оценены:\n");
    println!("  Температура воздуха: {:.1}°C", params.air_temp);
    println!("  Температура мерзлоты: {:.1}°C", params.permafrost_temp);
    println!("  Льдистость: {:.1}%", params.ice_content * 100.0);
    println!("  Тип грунта: {:?}", params.soil_type);
    println!("  Растительность: {:.1}%", params.vegetation_cover * 100.0);
    println!("  Глубина мерзлоты: {:.2} м", params.permafrost_depth);

    // Валидация
    println!("\n🔍 Валидация модели (10 лет)...");
    let validation = calibrator.validate_model(&params, 10);

    println!("\n📊 Результаты валидации:");
    println!(
        "  Средняя относительная ошибка: {:.1}%",
        validation.mean_relative_error * 100.0
    );
    println!("  Модель - глубина: {:.2} м", validation.model_depth);
    println!("  Модель - диаметр: {:.2} м", validation.model_diameter);

    if let Some(obs_depth) = validation.observed_depth_mean {
        println!("  Наблюдения - глубина: {:.2} м", obs_depth);
    }

    if validation.is_good_fit() {
        println!("\n✅ Модель хорошо соответствует данным");
    } else {
        println!("\n⚠️  Модель требует дополнительной настройки");
    }

    // Сохранение параметров
    if let Some(output_path) = output {
        println!("\n💾 Сохранение параметров в {:?}...", output_path);
        let json = serde_json::to_string_pretty(&params).context("Ошибка сериализации")?;
        std::fs::write(&output_path, json).context("Ошибка записи")?;
        println!("✅ Параметры сохранены");
    }

    Ok(())
}

pub fn import_iryp(input: PathBuf, output: PathBuf) -> Result<()> {
    println!("📥 Импорт данных IRYP (Ice-Rich Yedoma Permafrost)");
    println!("📁 Источник: {:?}", input);
    println!("📁 Назначение: {:?}\n", output);

    // Парсинг IRYP файла
    println!("🔄 Парсинг IRYP датасета...");
    let sites = IrypParser::parse_file(&input).context("Ошибка парсинга IRYP файла")?;

    println!("✅ Загружено {} точек наблюдений", sites.len());

    // Фильтрация только Якутии
    println!("\n🔍 Фильтрация точек из Якутии...");
    let yakutia_sites = IrypParser::filter_yakutia(sites);

    println!("✅ Найдено {} точек в Якутии", yakutia_sites.len());

    // Группировка по регионам
    let groups = IrypParser::group_by_region(&yakutia_sites);

    println!("\n📊 Распределение по регионам:");
    for (region, sites) in &groups {
        println!("  {}: {} точек", region, sites.len());
    }

    // Вывод примеров
    println!("\n📍 Примеры точек наблюдений:");
    for site in yakutia_sites.iter().take(5) {
        println!("\n  🔹 {}", site.event);
        println!(
            "     Координаты: {:.4}°N, {:.4}°E",
            site.latitude, site.longitude
        );
        println!("     Область: {}", site.area);
        if let Some(date) = &site.date {
            println!("     Дата: {}", date);
        }
        if let Some(investigator) = &site.investigator {
            println!("     Исследователь: {}", investigator);
        }
        println!("     Описание: {}", site.comment);
    }

    // Сохранение в JSON
    println!("\n💾 Сохранение данных...");
    let json = serde_json::to_string_pretty(&yakutia_sites).context("Ошибка сериализации")?;
    std::fs::write(&output, json).context("Ошибка записи файла")?;

    println!("✅ Данные сохранены в {:?}", output);
    println!("\n📈 Итого:");
    println!("  Всего точек IRYP: {}", yakutia_sites.len());
    println!("  Регионов: {}", groups.len());

    Ok(())
}

pub fn visualize_iryp(input: PathBuf, output_dir: PathBuf) -> Result<()> {
    println!("🗺️  Визуализация точек наблюдений IRYP");
    println!("📁 Источник: {:?}", input);
    println!("📁 Выходная директория: {:?}\n", output_dir);

    // Создаем директорию если не существует
    std::fs::create_dir_all(&output_dir)?;

    // Загружаем данные
    println!("🔄 Загрузка данных...");
    let json_content = std::fs::read_to_string(&input)?;
    let sites: Vec<thermokarst_core::iryp::IrypSite> = serde_json::from_str(&json_content)?;

    println!("✅ Загружено {} точек", sites.len());

    // Создаем карту
    println!("\n🗺️  Создание карты...");
    let map_path = output_dir.join("iryp_map.png");
    iryp_visualization::create_map(
        &sites,
        &map_path,
        "Точки наблюдений едомы в Якутии (IRYP v2)",
    )
    .context("Ошибка создания карты")?;
    println!("✅ Карта сохранена: {:?}", map_path);

    // Создаем гистограмму
    println!("\n📊 Создание гистограммы...");
    let hist_path = output_dir.join("iryp_latitude_histogram.png");
    iryp_visualization::create_latitude_histogram(&sites, &hist_path)
        .context("Ошибка создания гистограммы")?;
    println!("✅ Гистограмма сохранена: {:?}", hist_path);

    println!("\n✅ Визуализация завершена!");
    println!("📂 Файлы сохранены в: {:?}", output_dir);

    Ok(())
}

pub fn simulate_iryp_sites(input: PathBuf, years: u32) -> Result<()> {
    println!("🔬 Симуляция термокарста для точек наблюдений IRYP");
    println!("📁 Источник: {:?}", input);
    println!("⏱️  Период: {} лет\n", years);

    // Загружаем IRYP данные
    let json_content = std::fs::read_to_string(&input)?;
    let sites: Vec<thermokarst_core::iryp::IrypSite> = serde_json::from_str(&json_content)?;

    println!("✅ Загружено {} точек наблюдений\n", sites.len());

    // Берем РАЗНООБРАЗНЫЕ точки из ВСЕХ регионов Якутии
    let mut all_sites: Vec<_> = sites.iter().collect();

    // Удаляем дубликаты по координатам
    all_sites.sort_by(|a, b| {
        a.latitude
            .partial_cmp(&b.latitude)
            .unwrap()
            .then(a.longitude.partial_cmp(&b.longitude).unwrap())
    });
    all_sites.dedup_by(|a, b| {
        (a.latitude - b.latitude).abs() < 0.01 && (a.longitude - b.longitude).abs() < 0.01
    });

    // Выбираем точки с МАКСИМАЛЬНЫМ разбросом широт
    // Сортируем по широте и берем равномерно распределенные
    all_sites.sort_by(|a, b| a.latitude.partial_cmp(&b.latitude).unwrap());

    let total = all_sites.len();
    let interesting_sites: Vec<_> = if total >= 5 {
        vec![
            all_sites[0],             // Самая южная
            all_sites[total / 4],     // 25%
            all_sites[total / 2],     // Середина
            all_sites[total * 3 / 4], // 75%
            all_sites[total - 1],     // Самая северная
        ]
    } else {
        all_sites.into_iter().take(5).collect()
    };

    println!(
        "📍 Выбрано {} точек для симуляции:\n",
        interesting_sites.len()
    );

    use thermokarst_core::EnvironmentParams;
    use thermokarst_simulation::{SimulationConfig, SimulationEngine};

    for site in interesting_sites {
        println!("🔹 {}", site.event);
        println!(
            "   Координаты: {:.4}°N, {:.4}°E",
            site.latitude, site.longitude
        );
        println!("   Область: {}", site.area);

        // Определяем параметры на основе IRYP точки
        // Функция автоматически учитывает широту, тип местности и т.д.
        let params = thermokarst_core::estimate_params_from_site(site);

        // ОТЛАДКА: выводим параметры
        println!("   🔧 Параметры:");
        println!("      air_temp: {:.2}°C", params.air_temp);
        println!("      warm_season_days: {}", params.warm_season_days);
        println!("      ice_content: {:.3}", params.ice_content);
        println!("      vegetation_cover: {:.3}", params.vegetation_cover);
        println!(
            "      temperature_amplitude: {:.2}°C",
            params.temperature_amplitude
        );
        let ddt = params.air_temp * params.warm_season_days as f64;
        println!("      → DDT: {:.1} °C·дней", ddt);

        let config = SimulationConfig {
            years,
            time_step: 1,
            save_intermediate: false,
            save_interval: 1,
        };

        let engine = SimulationEngine::new(params, config);
        let result = engine.run()?;

        let final_lens = result.lenses.last().unwrap();
        println!("   📊 Прогноз через {} лет:", years);
        println!("      Глубина просадки: {:.2} м", final_lens.depth);
        println!("      Диаметр аласа: {:.2} м", final_lens.diameter);
        println!("      Объем: {:.2} м³", final_lens.volume);
        println!("      Стадия: {:?}", result.stage);
        println!();
    }

    println!("✅ Симуляция завершена!");

    Ok(())
}
