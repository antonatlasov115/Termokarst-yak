//! Команды для работы с изображениями

use anyhow::{Context, Result};
use std::path::PathBuf;
use thermokarst_core::dataset::ObservationDataset;
use thermokarst_image_processing::{ImageDownloader, PhotoAnalyzer};

#[cfg(feature = "download")]
pub fn download(output_dir: PathBuf) -> Result<()> {
    println!("📥 Загрузка примеров изображений термокарста");
    println!("📁 Директория: {:?}\n", output_dir);

    let downloaded =
        ImageDownloader::download_examples(&output_dir).context("Ошибка загрузки изображений")?;

    println!("\n✅ Загружено {} изображений:", downloaded.len());
    for filename in &downloaded {
        println!("  • {}", filename);
    }

    Ok(())
}

#[cfg(not(feature = "download"))]
pub fn download(_output_dir: PathBuf) -> Result<()> {
    anyhow::bail!("Функция загрузки недоступна. Пересоберите с флагом --features download")
}

pub fn analyze(
    input: PathBuf,
    scale: f64,
    output: Option<PathBuf>,
    site_id: String,
    coordinates: Option<String>,
) -> Result<()> {
    println!("📸 Анализ изображения термокарста");
    println!("📁 Файл: {:?}", input);
    println!("📏 Масштаб: {} м/пиксель\n", scale);

    // Загрузить и проанализировать
    let analyzer = PhotoAnalyzer::from_file(input.to_str().unwrap(), scale)
        .context("Ошибка загрузки изображения")?;

    println!("🔄 Выполнение анализа...");
    let result = analyzer.analyze().context("Ошибка анализа")?;

    println!("\n✅ Анализ завершен!\n");
    println!("📊 Результаты:");
    println!("  Найдено образований: {}", result.total_count);

    for (i, feature) in result.features.iter().enumerate() {
        println!("\n  Образование #{}:", i + 1);
        println!(
            "    Центр: ({}, {}) пикселей",
            feature.center_x, feature.center_y
        );
        println!("    Радиус: {} пикселей", feature.radius_pixels);
        println!("    Диаметр: {:.2} м", feature.diameter_meters);
        println!("    Площадь: {:.1} м²", feature.area_m2);
    }

    // Создать датасет, если указаны координаты
    if let Some(coords_str) = coordinates {
        let coords: Vec<f64> = coords_str
            .split(',')
            .map(|s| s.trim().parse())
            .collect::<Result<Vec<_>, _>>()
            .context("Неверный формат координат (ожидается: lat,lon)")?;

        if coords.len() != 2 {
            anyhow::bail!("Координаты должны быть в формате: lat,lon");
        }

        let observations =
            analyzer.to_observation_dataset(&site_id, (coords[0], coords[1]), "2025-08-15")?;

        let dataset = ObservationDataset {
            name: format!("Photo Analysis: {}", site_id),
            description: format!("Extracted from image: {:?}", input),
            source: "Photo analysis".to_string(),
            observations,
        };

        if let Some(output_path) = output {
            println!("\n💾 Сохранение датасета в {:?}...", output_path);
            dataset
                .to_json_file(&output_path)
                .context("Ошибка сохранения")?;
            println!("✅ Датасет сохранен");
        }
    }

    Ok(())
}

pub fn create_synthetic(output: PathBuf, diameter: u32) -> Result<()> {
    println!("🎨 Создание синтетического изображения термокарста");
    println!("📁 Файл: {:?}", output);
    println!("📏 Диаметр: {} пикселей\n", diameter);

    ImageDownloader::create_synthetic_thermokarst(&output, diameter)
        .context("Ошибка создания изображения")?;

    println!("✅ Изображение создано");

    Ok(())
}
