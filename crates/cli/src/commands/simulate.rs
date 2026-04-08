//! Команда запуска одиночной симуляции

use anyhow::{Context, Result};
use std::path::PathBuf;
use thermokarst_core::EnvironmentParams;
use thermokarst_simulation::{SimulationConfig, SimulationEngine};

pub fn run(region: String, years: u32, output: Option<PathBuf>, verbose: bool) -> Result<()> {
    println!("🌍 Запуск симуляции термокарстовых образований");
    println!("📍 Регион: {}", region);
    println!("⏱️  Период: {} лет\n", years);

    // Выбор параметров по региону
    let params = match region.to_lowercase().as_str() {
        "north" | "северная" => {
            println!("Параметры: Северная Якутия");
            EnvironmentParams::northern_yakutia()
        }
        "central" | "центральная" => {
            println!("Параметры: Центральная Якутия");
            EnvironmentParams::central_yakutia()
        }
        "south" | "южная" => {
            println!("Параметры: Южная Якутия");
            EnvironmentParams::southern_yakutia()
        }
        _ => {
            println!("⚠️  Неизвестный регион, используются параметры Центральной Якутии");
            EnvironmentParams::central_yakutia()
        }
    };

    if verbose {
        println!("\nПараметры среды:");
        println!("  Температура воздуха: {:.1}°C", params.air_temp);
        println!("  Температура мерзлоты: {:.1}°C", params.permafrost_temp);
        println!("  Льдистость: {:.1}%", params.ice_content * 100.0);
        println!("  Тип грунта: {:?}", params.soil_type);
        println!("  Растительность: {:.1}%", params.vegetation_cover * 100.0);
        println!();
    }

    // Конфигурация симуляции
    let config = SimulationConfig {
        years,
        time_step: 1,
        save_intermediate: true,
        save_interval: if years > 50 { 5 } else { 1 },
    };

    // Запуск симуляции
    println!("🔄 Выполнение симуляции...");
    let engine = SimulationEngine::new(params, config);
    let result = engine.run().context("Ошибка выполнения симуляции")?;

    // Вывод результатов
    println!("\n✅ Симуляция завершена!\n");
    println!("📊 Результаты:");
    println!("  Количество точек данных: {}", result.lenses.len());

    if let Some(first) = result.lenses.first() {
        println!("\n  Начальное состояние (год {}):", first.age);
        println!("    Глубина: {:.2} м", first.depth);
        println!("    Диаметр: {:.2} м", first.diameter);
        println!("    Объем: {:.1} м³", first.volume);
    }

    if let Some(last) = result.lenses.last() {
        println!("\n  Финальное состояние (год {}):", last.age);
        println!("    Глубина: {:.2} м", last.depth);
        println!("    Диаметр: {:.2} м", last.diameter);
        println!("    Объем: {:.1} м³", last.volume);
        println!("    Площадь: {:.1} м²", last.surface_area);
        println!("    Соотношение Г/Д: {:.3}", last.aspect_ratio());
        println!(
            "    Стабильность: {}",
            if last.is_stable() { "✓" } else { "✗" }
        );
    }

    println!("\n  Стадия развития: {:?}", result.stage);

    // Сохранение результатов
    if let Some(output_path) = output {
        println!("\n💾 Сохранение результатов в {:?}...", output_path);
        let json =
            serde_json::to_string_pretty(&result).context("Ошибка сериализации результатов")?;
        std::fs::write(&output_path, json).context("Ошибка записи файла")?;
        println!("✅ Результаты сохранены");
    }

    Ok(())
}
