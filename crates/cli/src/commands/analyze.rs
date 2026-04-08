//! Команда анализа результатов

use anyhow::{Context, Result};
use std::path::PathBuf;
use thermokarst_core::SimulationResult;
use thermokarst_geology::StabilityAnalyzer;

pub fn run(input: PathBuf) -> Result<()> {
    println!("📊 Анализ результатов симуляции");
    println!("📁 Файл: {:?}\n", input);

    // Загрузка данных
    let data = std::fs::read_to_string(&input).context("Ошибка чтения файла")?;
    let result: SimulationResult = serde_json::from_str(&data).context("Ошибка парсинга JSON")?;

    println!("✅ Данные загружены\n");

    // Общая информация
    println!("📋 Общая информация:");
    println!("  Период симуляции: {} лет", result.total_years);
    println!("  Точек данных: {}", result.lenses.len());
    println!("  Стадия развития: {:?}\n", result.stage);

    // Параметры среды
    println!("🌍 Параметры среды:");
    println!(
        "  Температура воздуха: {:.1}°C",
        result.environment.air_temp
    );
    println!(
        "  Температура мерзлоты: {:.1}°C",
        result.environment.permafrost_temp
    );
    println!(
        "  Льдистость: {:.1}%",
        result.environment.ice_content * 100.0
    );
    println!("  Тип грунта: {:?}", result.environment.soil_type);
    println!(
        "  Растительность: {:.1}%",
        result.environment.vegetation_cover * 100.0
    );
    println!();

    // Анализ развития
    if let (Some(first), Some(last)) = (result.lenses.first(), result.lenses.last()) {
        println!("📈 Динамика развития:");
        println!("\n  Начало (год {}):", first.age);
        println!("    Глубина: {:.2} м", first.depth);
        println!("    Диаметр: {:.2} м", first.diameter);
        println!("    Объем: {:.1} м³", first.volume);

        println!("\n  Конец (год {}):", last.age);
        println!("    Глубина: {:.2} м", last.depth);
        println!("    Диаметр: {:.2} м", last.diameter);
        println!("    Объем: {:.1} м³", last.volume);
        println!("    Площадь: {:.1} м²", last.surface_area);

        println!("\n  Изменения:");
        println!("    Δ Глубина: +{:.2} м", last.depth - first.depth);
        println!("    Δ Диаметр: +{:.2} м", last.diameter - first.diameter);
        println!("    Δ Объем: +{:.1} м³", last.volume - first.volume);

        // Анализ стабильности
        println!("\n🔍 Анализ стабильности:");
        let stability_score = StabilityAnalyzer::long_term_stability_score(last);
        let collapse_risk = StabilityAnalyzer::collapse_risk(last);
        let is_stable = StabilityAnalyzer::is_geometrically_stable(last);

        println!("  Оценка стабильности: {:.2}/1.00", stability_score);
        println!("  Риск обрушения: {:.1}%", collapse_risk * 100.0);
        println!(
            "  Геометрически стабильна: {}",
            if is_stable { "Да" } else { "Нет" }
        );
        println!("  Соотношение Г/Д: {:.3}", last.aspect_ratio());

        if let Some(years) = StabilityAnalyzer::time_to_stabilization(last) {
            if years == 0 {
                println!("  Статус: Стабилизирована");
            } else {
                println!("  Прогноз стабилизации: через {} лет", years);
            }
        } else {
            println!("  Статус: Продолжает активное развитие");
        }

        // Рекомендации
        println!("\n💡 Рекомендации:");
        if collapse_risk > 0.7 {
            println!("  ⚠️  ВЫСОКИЙ риск обрушения берегов");
            println!("  → Требуется мониторинг и возможные укрепительные работы");
        } else if collapse_risk > 0.4 {
            println!("  ⚠️  Умеренный риск обрушения");
            println!("  → Рекомендуется периодический мониторинг");
        } else {
            println!("  ✓ Низкий риск обрушения");
        }

        if stability_score < 0.5 {
            println!("  ⚠️  Низкая общая стабильность");
            println!("  → Образование находится в активной фазе развития");
        } else if stability_score > 0.8 {
            println!("  ✓ Высокая стабильность");
            println!("  → Образование близко к равновесному состоянию");
        }
    }

    Ok(())
}
