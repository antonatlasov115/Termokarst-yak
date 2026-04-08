use serde_json;
use std::fs;

fn main() {
    let json = fs::read_to_string("iryp_yakutia.json").unwrap();
    let sites: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
    
    println!("Проверка параметров для первых 5 точек Центральной Якутии:\n");
    
    for site in sites.iter().filter(|s| {
        s["latitude"].as_f64().unwrap() >= 60.0 && 
        s["latitude"].as_f64().unwrap() <= 68.0
    }).take(5) {
        let lat = site["latitude"].as_f64().unwrap();
        let area = site["area"].as_str().unwrap();
        
        println!("📍 {} ({:.2}°N)", site["event"].as_str().unwrap(), lat);
        println!("   Область: {}", area);
        
        // Симулируем логику estimate_params_from_site
        let air_temp = if lat > 70.0 { 2.0 } else if lat > 65.0 { 4.0 } else if lat > 60.0 { 6.0 } else { 8.0 };
        let ice_content = if lat > 68.0 { 0.75 } else { 0.65 };
        let veg_cover = if lat > 70.0 { 0.3 } else if lat > 65.0 { 0.5 } else { 0.7 };
        let temp_amp = if lat > 70.0 { 98.0 } else if lat > 65.0 { 92.0 } else if lat > 60.0 { 88.0 } else { 75.0 };
        let warm_days = if lat > 70.0 { 80 } else if lat > 65.0 { 100 } else if lat > 60.0 { 120 } else { 140 };
        
        println!("   air_temp: {}", air_temp);
        println!("   ice_content: {}", ice_content);
        println!("   vegetation_cover: {}", veg_cover);
        println!("   temperature_amplitude: {}", temp_amp);
        println!("   warm_season_days: {}", warm_days);
        println!();
    }
}
