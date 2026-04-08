//! Расчет параметров термокарста по датасету

/// Константы из статьи
const LAMBDA_T: f64 = 1.5; // Теплопроводность (Вт/(м·К))
const L: f64 = 334000.0; // Теплота плавления (Дж/кг)
const RHO_W: f64 = 1000.0; // Плотность воды (кг/м³)
const I_0: f64 = 0.30; // Пороговая льдистость
const ALPHA: f64 = 0.60; // Коэффициент альфа
const BETA: f64 = 0.45; // Коэффициент бета
const GAMMA: f64 = 0.12; // Коэффициент гамма
const K_COMP: f64 = 1.0; // Коэффициент компрессии
const SECONDS_PER_DAY: f64 = 86400.0;

struct InputData {
    id: u32,
    description: &'static str,
    ddt: f64, // Градусо-дни оттаивания
    w: f64,   // Льдистость (0-1)
    v: f64,   // Растительный покров (0-1)
    dt: f64,  // Амплитуда температуры
}

fn main() {
    let inputs = vec![
        InputData {
            id: 1,
            description: "Обычный грунт, целая тайга",
            ddt: 900.0,
            w: 0.20,
            v: 0.90,
            dt: 60.0,
        },
        InputData {
            id: 2,
            description: "Обычный грунт, сильная гарь",
            ddt: 900.0,
            w: 0.20,
            v: 0.00,
            dt: 60.0,
        },
        InputData {
            id: 3,
            description: "Типичная едома, целая тайга",
            ddt: 1100.0,
            w: 0.60,
            v: 0.85,
            dt: 85.0,
        },
        InputData {
            id: 4,
            description: "Классическая едома, гарь",
            ddt: 1100.0,
            w: 0.60,
            v: 0.10,
            dt: 85.0,
        },
        InputData {
            id: 5,
            description: "Экстремальная едома",
            ddt: 1200.0,
            w: 0.75,
            v: 0.80,
            dt: 95.0,
        },
        InputData {
            id: 6,
            description: "Экстремальная едома после пожара",
            ddt: 1200.0,
            w: 0.75,
            v: 0.05,
            dt: 95.0,
        },
        InputData {
            id: 7,
            description: "Пороговая льдистость, частичная гарь",
            ddt: 1000.0,
            w: 0.30,
            v: 0.50,
            dt: 75.0,
        },
        InputData {
            id: 8,
            description: "Максимальная континентальность",
            ddt: 1400.0,
            w: 0.50,
            v: 1.00,
            dt: 100.0,
        },
        InputData {
            id: 9,
            description: "Минимальная льдистость, холодно",
            ddt: 800.0,
            w: 0.15,
            v: 0.20,
            dt: 50.0,
        },
        InputData {
            id: 10,
            description: "Максимальная льдистость",
            ddt: 1300.0,
            w: 0.80,
            v: 0.95,
            dt: 90.0,
        },
    ];

    println!("[");
    for (i, input) in inputs.iter().enumerate() {
        let xi_a = calculate_thaw_depth(input.ddt, input.w, input.v);
        let s_a = calculate_subsidence(xi_a, input.w, input.v);

        print!(
            "  {{ \"id\": {}, \"xi_A\": {:.2}, \"s_A\": {:.2} }}",
            input.id, xi_a, s_a
        );

        if i < inputs.len() - 1 {
            println!(",");
        } else {
            println!();
        }
    }
    println!("]");
}

/// Расчет глубины оттаивания по формуле Stefan
fn calculate_thaw_depth(ddt: f64, w: f64, v: f64) -> f64 {
    // Конвертация DDT из градусо-дней в градусо-секунды
    let ddt_seconds = ddt * SECONDS_PER_DAY;

    // N-фактор для растительности (уменьшает теплопередачу)
    let n_factor = 1.0 - ALPHA * v;

    // Эффективный DDT с учетом растительности
    let ddt_eff = ddt_seconds * n_factor;

    // Объемная теплота плавления
    let volumetric_latent_heat = L * RHO_W * w;

    // Формула Stefan: xi = sqrt(2 * lambda * DDT / L_vol)
    let xi_squared = (2.0 * LAMBDA_T * ddt_eff) / volumetric_latent_heat;

    xi_squared.sqrt()
}

/// Расчет просадки грунта
fn calculate_subsidence(xi_a: f64, w: f64, v: f64) -> f64 {
    // Базовая просадка от таяния льда (лед уменьшается на ~9%)
    let ice_melt_subsidence = xi_a * w * 0.09;

    // Дополнительная просадка от компрессии
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
