//! Симуляция термокарста для реальных точек наблюдений IRYP

use std::fs;
use thermokarst_core::iryp::IrypSite;
use thermokarst_core::EnvironmentParams;
use thermokarst_simulation::{SimulationConfig, SimulationEngine};

fn main() -> anyhow::Result<()> {
    println!("🔬 Симуляция для точек наблюдений IRYP\n");

    // Загружаем IRYP данные
    let json = fs::read_to_string("iryp_yakutia.json")?;
    let sites: Vec<IrypSite> = serde_json::from_str(&json)?;

    println!("✅ Загружено {} точек наблюдений\n", sites.len());

    // Берем несколько интересных точек
    let interesting_sites = vec![
        ("Syrdakh", 62.557200, 130.893400),
        ("Tabaga", 61.664200, 130.943400),
        ("Churapcha", 61.966389, 132.612778),
    ];

    for (name, lat, lon) in interesting_sites {
        println!("📍 Симуляция для: {} ({:.4}°N, {:.4}°E)", name, lat, lon);

        // Определяем параметры на основе региона
        let mut params = if lat > 68.0 {
            EnvironmentParams::northern_yakutia()
        } else if lat >= 60.0 {
            EnvironmentParams::central_yakutia()
        } else {
            EnvironmentParams::southern_yakutia()
        };

        // Настраиваем параметры для едомы
        params.ice_content = 0.65; // Типичная едома

        let config = SimulationConfig {
            years: 50,
            time_step: 1,
            save_intermediate: true,
            save_interval: 5,
        };

        let engine = SimulationEngine::new(params, config);
        let result = engine.run()?;

        let final_lens = result.lenses.last().unwrap();
        println!("  Результат через 50 лет:");
        println!("    Глубина: {:.2} м", final_lens.depth);
        println!("    Диаметр: {:.2} м", final_lens.diameter);
        println!("    Объем: {:.2} м³", final_lens.volume);
        println!("    Стадия: {:?}\n", result.stage);
    }

    Ok(())
}
