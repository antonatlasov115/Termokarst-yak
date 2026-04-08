//! Команда батч-симуляции

use anyhow::{Context, Result};
use std::path::PathBuf;
use thermokarst_simulation::{BatchSimulator, SimulationConfig};

pub fn run(years: u32, output_dir: Option<PathBuf>, parallel: bool) -> Result<()> {
    println!("🌍 Запуск батч-симуляции для всех регионов Якутии");
    println!("⏱️  Период: {} лет", years);
    println!("⚡ Режим: {}\n", if parallel { "параллельный" } else { "последовательный" });

    let config = SimulationConfig {
        years,
        time_step: 1,
        save_intermediate: true,
        save_interval: if years > 50 { 5 } else { 1 },
    };

    let mut batch = BatchSimulator::new(config);
    batch.add_yakutia_scenarios();

    println!("📋 Сценариев для выполнения: {}\n", batch.scenario_count());

    println!("🔄 Выполнение симуляций...");
    let results = if parallel {
        batch.run_parallel()
    } else {
        batch.run_sequential()
    };

    println!("\n✅ Все симуляции завершены!\n");

    // Вывод результатов
    for batch_result in &results {
        println!("📊 Сценарий: {}", batch_result.scenario_name);

        match &batch_result.result {
            Ok(result) => {
                if let Some(last) = result.lenses.last() {
                    println!("  Финальное состояние (год {}):", last.age);
                    println!("    Глубина: {:.2} м", last.depth);
                    println!("    Диаметр: {:.2} м", last.diameter);
                    println!("    Объем: {:.1} м³", last.volume);
                    println!("    Стабильность: {}", if last.is_stable() { "✓" } else { "✗" });
                }
                println!("  Стадия: {:?}", result.stage);
            }
            Err(e) => {
                println!("  ❌ Ошибка: {}", e);
            }
        }
        println!();
    }

    // Сохранение результатов
    if let Some(dir) = output_dir {
        println!("💾 Сохранение результатов в {:?}...", dir);
        std::fs::create_dir_all(&dir).context("Ошибка создания директории")?;

        for batch_result in results {
            if let Ok(result) = batch_result.result {
                let filename = format!("{}.json", batch_result.scenario_name.replace(' ', "_"));
                let path = dir.join(filename);

                let json = serde_json::to_string_pretty(&result)
                    .context("Ошибка сериализации")?;
                std::fs::write(&path, json).context("Ошибка записи файла")?;

                println!("  ✓ {}", path.display());
            }
        }

        println!("✅ Все результаты сохранены");
    }

    Ok(())
}
