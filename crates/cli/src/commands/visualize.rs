//! Команда визуализации результатов симуляции

use anyhow::{Context, Result};
use std::path::PathBuf;
use thermokarst_core::SimulationResult;

#[cfg(feature = "visualization")]
use thermokarst_simulation::SimulationVisualizer;

pub fn run(input: PathBuf, output_dir: PathBuf, plot_type: String) -> Result<()> {
    #[cfg(not(feature = "visualization"))]
    {
        anyhow::bail!(
            "Визуализация недоступна. Пересоберите проект с флагом:\n\
             cargo build --release --features visualization"
        );
    }

    #[cfg(feature = "visualization")]
    {
        println!("📊 Визуализация результатов симуляции");
        println!("📁 Входной файл: {:?}", input);
        println!("📁 Выходная директория: {:?}\n", output_dir);

        // Загрузить результаты
        let json_data =
            std::fs::read_to_string(&input).context("Не удалось прочитать файл с результатами")?;

        let results: Vec<SimulationResult> =
            serde_json::from_str(&json_data).context("Не удалось распарсить JSON")?;

        if results.is_empty() {
            anyhow::bail!("Файл не содержит результатов симуляции");
        }

        println!("✅ Загружено {} результатов симуляции", results.len());

        // Создать директорию для графиков
        std::fs::create_dir_all(&output_dir)
            .context("Не удалось создать директорию для графиков")?;

        match plot_type.as_str() {
            "development" => {
                println!("\n🔄 Создание графика развития...");
                let output_path = output_dir.join("development.png");
                SimulationVisualizer::plot_development(&results, &output_path)
                    .map_err(|e| anyhow::anyhow!("Ошибка создания графика развития: {}", e))?;
                println!("✅ График сохранен: {:?}", output_path);
            }

            "volume" => {
                println!("\n🔄 Создание графика объема...");
                let output_path = output_dir.join("volume.png");
                SimulationVisualizer::plot_volume(&results, &output_path)
                    .map_err(|e| anyhow::anyhow!("Ошибка создания графика объема: {}", e))?;
                println!("✅ График сохранен: {:?}", output_path);
            }

            "stages" => {
                println!("\n🔄 Создание диаграммы стадий...");
                let output_path = output_dir.join("stages.png");
                SimulationVisualizer::plot_stages(&results, &output_path)
                    .map_err(|e| anyhow::anyhow!("Ошибка создания диаграммы стадий: {}", e))?;
                println!("✅ График сохранен: {:?}", output_path);
            }

            "cross-section" => {
                println!("\n🔄 Создание поперечного сечения...");
                if let Some(last_result) = results.last() {
                    let output_path = output_dir.join("cross_section.png");
                    SimulationVisualizer::plot_cross_section(last_result, &output_path).map_err(
                        |e| anyhow::anyhow!("Ошибка создания поперечного сечения: {}", e),
                    )?;
                    println!("✅ График сохранен: {:?}", output_path);
                } else {
                    anyhow::bail!("Нет данных для создания поперечного сечения");
                }
            }

            "all" => {
                println!("\n🔄 Создание всех графиков...");
                SimulationVisualizer::create_report(&results, &output_dir)
                    .map_err(|e| anyhow::anyhow!("Ошибка создания отчета: {}", e))?;

                println!("\n✅ Все графики созданы:");
                println!("  📈 development.png - График развития");
                println!("  📊 volume.png - График объема");
                println!("  🎨 stages.png - Диаграмма стадий");
                println!("  🗺️  cross_section.png - Поперечное сечение");
            }

            _ => {
                anyhow::bail!(
                    "Неизвестный тип графика: {}. Доступные: development, volume, stages, cross-section, all",
                    plot_type
                );
            }
        }

        println!("\n🎉 Визуализация завершена!");
        Ok(())
    }
}
