//! Отладка параметров для IRYP точек

use thermokarst_core::{IrypSite, estimate_params_from_site};
use std::fs;

fn main() -> anyhow::Result<()> {
    let json = fs::read_to_string("iryp_yakutia.json")?;
    let sites: Vec<IrypSite> = serde_json::from_str(&json)?;
    
    println!("🔍 АНАЛИЗ ПАРАМЕТРОВ ДЛЯ ЦЕНТРАЛЬНОЙ ЯКУТИИ:\n");
    
    let mut central: Vec<_> = sites.iter()
        .filter(|s| s.latitude >= 60.0 && s.latitude <= 68.0)
        .collect();
    
    central.sort_by(|a, b| a.latitude.partial_cmp(&b.latitude).unwrap());
    central.dedup_by(|a, b| {
        (a.latitude - b.latitude).abs() < 0.01 && 
        (a.longitude - b.longitude).abs() < 0.01
    });
    
    println!("Всего уникальных точек: {}\n", central.len());
    
    for site in central.iter().take(10) {
        let params = estimate_params_from_site(site);
        
        println!("📍 {} ({:.2}°N, {:.2}°E)", site.event, site.latitude, site.longitude);
        println!("   air_temp: {:.2}°C", params.air_temp);
        println!("   warm_season_days: {}", params.warm_season_days);
        println!("   ice_content: {:.3}", params.ice_content);
        println!("   vegetation_cover: {:.3}", params.vegetation_cover);
        println!("   temperature_amplitude: {:.2}°C", params.temperature_amplitude);
        
        // Вычисляем DDT
        let ddt = params.air_temp * params.warm_season_days as f64;
        println!("   → DDT: {:.1} °C·дней", ddt);
        
        // Вычисляем коэффициенты
        let k_fire = (0.45 * (1.0 - params.vegetation_cover)).exp();
        let f_cont = 1.0 + 0.12 * (params.temperature_amplitude / 40.0).ln();
        println!("   → K_fire: {:.4}", k_fire);
        println!("   → f_continental: {:.4}", f_cont);
        println!();
    }
    
    Ok(())
}
