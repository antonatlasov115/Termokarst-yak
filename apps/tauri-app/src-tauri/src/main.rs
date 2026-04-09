// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use thermokarst_core::{EnvironmentParams, SoilType};
use thermokarst_simulation::{SimulationConfig, SimulationEngine};

#[derive(Debug, Serialize, Deserialize)]
struct SimulationParams {
    region: String,
    years: u32,
    temperature: f64,
    ice_content: f64,
    vegetation: f64,
    latitude: f64,
    longitude: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct SimulationResult {
    year: u32,
    depth: f64,
    diameter: f64,
    volume: f64,
    stability: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct InverseParams {
    current_depth: f64,
    current_diameter: f64,
    observation_year: u32,
    latitude: f64,
    longitude: f64,
    ice_content: f64,
    vegetation: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct InverseResult {
    estimated_age: u32,
    start_year: u32,
    confidence: f64,
    results: Vec<SimulationResult>,
}

// Tauri команда для прямой симуляции
#[tauri::command]
fn run_forward_simulation(params: SimulationParams) -> Result<Vec<SimulationResult>, String> {
    // Создаем параметры окружения
    let env_params = EnvironmentParams {
        soil_type: match params.region.as_str() {
            "north" => SoilType::Peat,
            "south" => SoilType::Sand,
            _ => SoilType::Silt,
        },
        ice_content: params.ice_content,
        vegetation_cover: params.vegetation,
        soil_saturation_ratio: 0.3,
        air_temp: params.temperature,
        permafrost_temp: -2.0,
        permafrost_depth: 1.5,
        warm_season_days: match params.region.as_str() {
            "north" => 90,
            "south" => 140,
            _ => 120,
        },
        temperature_amplitude: match params.region.as_str() {
            "north" => 95.0,
            "south" => 75.0,
            _ => 88.0,
        },
    };

    // Конфигурация симуляции
    let config = SimulationConfig {
        years: params.years,
        time_step: 1,
        save_intermediate: true,
        save_interval: 1,
    };

    // Запускаем движок симуляции
    let engine = SimulationEngine::new(env_params, config);
    let sim_result = engine.run().map_err(|e| e.to_string())?;

    // Конвертируем результаты в формат для frontend
    let mut results = vec![SimulationResult {
        year: 0,
        depth: 0.0,
        diameter: 2.0,
        volume: 0.0,
        stability: 1.0,
    }];

    for lens in sim_result.lenses {
        results.push(SimulationResult {
            year: lens.age,
            depth: (lens.depth * 100.0).round() / 100.0,
            diameter: (lens.diameter * 100.0).round() / 100.0,
            volume: (lens.volume * 100.0).round() / 100.0,
            stability: 0.8, // Заглушка, так как stability_score нет в ThermokarstLens
        });
    }

    Ok(results)
}

// Tauri команда для обратной симуляции
#[tauri::command]
fn run_inverse_simulation(params: InverseParams) -> Result<InverseResult, String> {
    // Создаем параметры окружения
    let env_params = EnvironmentParams {
        soil_type: if params.latitude > 68.0 {
            SoilType::Peat
        } else if params.latitude > 64.0 {
            SoilType::Silt
        } else {
            SoilType::Sand
        },
        ice_content: params.ice_content,
        vegetation_cover: params.vegetation,
        soil_saturation_ratio: 0.3,
        air_temp: 2.5,
        permafrost_temp: -2.0,
        permafrost_depth: 1.5,
        warm_season_days: if params.latitude > 68.0 {
            90
        } else if params.latitude > 64.0 {
            120
        } else {
            140
        },
        temperature_amplitude: if params.latitude > 68.0 {
            95.0
        } else if params.latitude > 64.0 {
            88.0
        } else {
            75.0
        },
    };

    // Итеративный поиск возраста
    let mut best_age = 1u32;
    let mut min_error = f64::MAX;

    for test_age in 1..=200 {
        let config = SimulationConfig {
            years: test_age,
            time_step: 1,
            save_intermediate: false,
            save_interval: test_age,
        };

        let engine = SimulationEngine::new(env_params.clone(), config);
        if let Ok(result) = engine.run() {
            if let Some(final_lens) = result.lenses.last() {
                let depth_error = (final_lens.depth - params.current_depth).abs();
                let diameter_error = (final_lens.diameter - params.current_diameter).abs();
                let total_error = depth_error + diameter_error;

                if total_error < min_error {
                    min_error = total_error;
                    best_age = test_age;
                }

                if total_error < 0.5 {
                    break;
                }
            }
        }
    }

    let final_age = best_age.min(params.observation_year);
    let start_year = params.observation_year.saturating_sub(final_age);

    // Генерируем полную историю
    let config = SimulationConfig {
        years: final_age,
        time_step: 1,
        save_intermediate: true,
        save_interval: 1,
    };

    let engine = SimulationEngine::new(env_params, config);
    let sim_result = engine.run().map_err(|e| e.to_string())?;

    let mut results = vec![SimulationResult {
        year: start_year,
        depth: 0.0,
        diameter: 2.0,
        volume: 0.0,
        stability: 1.0,
    }];

    for lens in sim_result.lenses {
        results.push(SimulationResult {
            year: start_year + lens.age,
            depth: (lens.depth * 100.0).round() / 100.0,
            diameter: (lens.diameter * 100.0).round() / 100.0,
            volume: (lens.volume * 100.0).round() / 100.0,
            stability: 0.8, // Заглушка
        });
    }

    let confidence = (1.0 - (min_error / 10.0)).max(0.5).min(0.95);

    Ok(InverseResult {
        estimated_age: final_age,
        start_year,
        confidence: (confidence * 100.0).round() / 100.0,
        results,
    })
}

#[tauri::command]
fn get_system_info() -> Result<String, String> {
    Ok(format!(
        "Thermokarst Yakutia v0.3.0\nRust Backend with SimulationEngine\nBuild: {}",
        env!("CARGO_PKG_VERSION")
    ))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            run_forward_simulation,
            run_inverse_simulation,
            get_system_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
