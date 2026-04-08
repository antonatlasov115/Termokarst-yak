//! Тестовая программа для расчета параметров термокарста по датасету

use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Входные данные
#[derive(Debug, Deserialize)]
struct InputData {
    id: u32,
    description: String,
    #[serde(rename = "DDT")]
    ddt: f64, // Градусо-дни оттаивания
    w: f64, // Льдистость (0-1)
    #[serde(rename = "V")]
    v: f64, // Растительный покров (0-1)
    #[serde(rename = "dT")]
    dt: f64, // Амплитуда температуры
}

/// Выходные данные
#[derive(Debug, Serialize)]
struct OutputData {
    id: u32,
    xi_A: f64, // Глубина оттаивания (м)
    s_A: f64,  // Просадка (м)
}

/// Константы из статьи
const LAMBDA_T: f64 = 1.5; // Теплопроводность (Вт/(м·К))
const L: f64 = 334000.0; // Теплота плавления (Дж/кг)
const RHO_W: f64 = 1000.0; // Плотность воды (кг/м³)
const I_0: f64 = 0.30; // Пороговая льдистость
const DT_0: f64 = 40.0; // Базовая амплитуда (К)
const ALPHA: f64 = 0.60; // Коэффициент альфа
const BETA: f64 = 0.45; // Коэффициент бета
const GAMMA: f64 = 0.12; // Коэффициент гамма
const K_COMP: f64 = 1.0; // Коэффициент компрессии
const SECONDS_PER_DAY: f64 = 86400.0;

fn main() {
    let input_json = r#"[
  { "id": 1, "description": "Обычный грунт, целая тайга", "DDT": 900, "w": 0.20, "V": 0.90, "dT": 60 },
  { "id": 2, "description": "Обычный грунт, сильная гарь", "DDT": 900, "w": 0.20, "V": 0.00, "dT": 60 },
  { "id": 3, "description": "Типичная едома, целая тайга", "DDT": 1100, "w": 0.60, "V": 0.85, "dT": 85 },
  { "id": 4, "description": "Классическая едома, гарь", "DDT": 1100, "w": 0.60, "V": 0.10, "dT": 85 },
  { "id": 5, "description": "Экстремальная едома", "DDT": 1200, "w": 0.75, "V": 0.80, "dT": 95 },
  { "id": 6, "description": "Экстремальная едома после пожара", "DDT": 1200, "w": 0.75, "V": 0.05, "dT": 95 },
  { "id": 7, "description": "Пороговая льдистость, частичная гарь", "DDT": 1000, "w": 0.30, "V": 0.50, "dT": 75 },
  { "id": 8, "description": "Максимальная континентальность", "DDT": 1400, "w": 0.50, "V": 1.00, "dT": 100 },
  { "id": 9, "description": "Минимальная льдистость, холодно", "DDT": 800, "w": 0.15, "V": 0.20, "dT": 50 },
  { "id": 10, "description": "Максимальная льдистость", "DDT": 1300, "w": 0.80, "V": 0.95, "dT": 90 }
]"#;

    let inputs: Vec<InputData> = serde_json::from_str(input_json).expect("Ошибка парсинга JSON");

    let mut outputs = Vec::new();

    for input in inputs {
        // Расчет глубины оттаивания (xi_A) по формуле Stefan
        let xi_a = calculate_thaw_depth(input.ddt, input.w, input.v, input.dt);

        // Расчет просадки (s_A)
        let s_a = calculate_subsidence(xi_a, input.w, input.v);

        outputs.push(OutputData {
            id: input.id,
            xi_A: (xi_a * 100.0).round() / 100.0, // Округление до 2 знаков
            s_A: (s_a * 100.0).round() / 100.0,
        });
    }

    // Вывод результатов
    let output_json = serde_json::to_string_pretty(&outputs).expect("Ошибка сериализации");

    println!("{}", output_json);
}

/// Расчет глубины оттаивания по формуле Stefan
fn calculate_thaw_depth(ddt: f64, w: f64, v: f64, dt: f64) -> f64 {
    // Конвертация DDT из градусо-дней в градусо-секунды
    let ddt_seconds = ddt * SECONDS_PER_DAY;

    // N-фактор для растительности (уменьшает теплопередачу)
    let n_factor = 1.0 - ALPHA * v;

    // Эффективный DDT с учетом растительности
    let ddt_eff = ddt_seconds * n_factor;

    // Эффективная льдистость (объемная доля льда)
    let ice_volume_fraction = w;

    // Объемная теплота плавления
    let volumetric_latent_heat = L * RHO_W * ice_volume_fraction;

    // Формула Stefan: xi = sqrt(2 * lambda * DDT / L_vol)
    let xi_squared = (2.0 * LAMBDA_T * ddt_eff) / volumetric_latent_heat;

    xi_squared.sqrt()
}

/// Расчет просадки грунта
fn calculate_subsidence(xi_a: f64, w: f64, v: f64) -> f64 {
    // Базовая просадка от таяния льда
    // При таянии лед уменьшается в объеме на ~9%
    let ice_melt_subsidence = xi_a * w * 0.09;

    // Дополнительная просадка от компрессии
    // Зависит от льдистости (больше льда = больше компрессия)
    let compression_factor = if w > I_0 {
        K_COMP * (w - I_0) * BETA
    } else {
        0.0
    };

    let compression_subsidence = xi_a * compression_factor;

    // Влияние растительности (корни удерживают грунт)
    let vegetation_factor = 1.0 - GAMMA * v;

    // Общая просадка
    (ice_melt_subsidence + compression_subsidence) * vegetation_factor
}
