//! Команда создания конфигурации

use anyhow::{Context, Result};
use std::path::PathBuf;
use thermokarst_core::EnvironmentParams;

pub fn run(output: PathBuf) -> Result<()> {
    println!("⚙️  Создание примера конфигурации");
    println!("📁 Файл: {:?}\n", output);

    let params = EnvironmentParams::default();

    let json = serde_json::to_string_pretty(&params).context("Ошибка сериализации")?;

    std::fs::write(&output, json).context("Ошибка записи файла")?;

    println!("✅ Конфигурация создана");
    println!("\nПример содержимого:");
    println!("  Температура воздуха: {:.1}°C", params.air_temp);
    println!("  Температура мерзлоты: {:.1}°C", params.permafrost_temp);
    println!("  Льдистость: {:.1}%", params.ice_content * 100.0);
    println!("  Тип грунта: {:?}", params.soil_type);
    println!("\nОтредактируйте файл для настройки параметров симуляции");

    Ok(())
}
