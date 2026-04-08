//! Команда для работы с датасетами

use anyhow::{Context, Result};
use std::path::PathBuf;
use thermokarst_core::dataset::{create_example_yakutia_dataset, ObservationDataset};
use thermokarst_simulation::calibration::ModelCalibrator;

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
        println!("    Мин-Макс: {:.2} - {:.2} м", alt_stats.min, alt_stats.max);
    }

    if let Some(temp_stats) = &stats.temperature_stats {
        println!("\n  Температура грунта:");
        println!("    Среднее: {:.1}°C", temp_stats.mean);
        println!("    Мин-Макс: {:.1} - {:.1}°C", temp_stats.min, temp_stats.max);
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
        println!("\n  Глубина активного слоя ({} измерений):", alt_stats.count);
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

    println!("✅ Датасет загружен ({} наблюдений)", dataset.observations.len());

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
    println!("  Средняя относительная ошибка: {:.1}%", validation.mean_relative_error * 100.0);
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
