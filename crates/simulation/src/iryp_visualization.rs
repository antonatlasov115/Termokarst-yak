//! Визуализация точек наблюдений IRYP на карте

use plotters::prelude::*;
use std::path::Path;
use thermokarst_core::iryp::IrypSite;

/// Создать карту с точками наблюдений
pub fn create_map<P: AsRef<Path>>(
    sites: &[IrypSite],
    output_path: P,
    title: &str,
) -> anyhow::Result<()> {
    let root = BitMapBackend::new(output_path.as_ref(), (1200, 800)).into_drawing_area();
    root.fill(&WHITE)?;

    // Определяем границы карты (Якутия: 56-74°N, 105-162°E)
    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 40).into_font())
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(105.0..162.0, 56.0..74.0)?;

    chart
        .configure_mesh()
        .x_desc("Долгота (°E)")
        .y_desc("Широта (°N)")
        .draw()?;

    // Группируем точки по регионам
    let mut northern = Vec::new();
    let mut central = Vec::new();
    let mut southern = Vec::new();

    for site in sites {
        match site.yakutia_region() {
            Some("Northern Yakutia") => northern.push(site),
            Some("Central Yakutia") => central.push(site),
            Some("Southern Yakutia") => southern.push(site),
            _ => {}
        }
    }

    // Рисуем точки по регионам
    // Северная Якутия - синий
    chart
        .draw_series(
            northern
                .iter()
                .map(|site| Circle::new((site.longitude, site.latitude), 4, BLUE.filled())),
        )?
        .label("Северная Якутия")
        .legend(|(x, y)| Circle::new((x + 10, y), 4, BLUE.filled()));

    // Центральная Якутия - красный
    chart
        .draw_series(
            central
                .iter()
                .map(|site| Circle::new((site.longitude, site.latitude), 4, RED.filled())),
        )?
        .label("Центральная Якутия")
        .legend(|(x, y)| Circle::new((x + 10, y), 4, RED.filled()));

    // Южная Якутия - зеленый
    chart
        .draw_series(
            southern
                .iter()
                .map(|site| Circle::new((site.longitude, site.latitude), 4, GREEN.filled())),
        )?
        .label("Южная Якутия")
        .legend(|(x, y)| Circle::new((x + 10, y), 4, GREEN.filled()));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;
    Ok(())
}

/// Создать гистограмму распределения по широте
pub fn create_latitude_histogram<P: AsRef<Path>>(
    sites: &[IrypSite],
    output_path: P,
) -> anyhow::Result<()> {
    let root = BitMapBackend::new(output_path.as_ref(), (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    // Собираем широты
    let latitudes: Vec<f64> = sites.iter().map(|s| s.latitude).collect();

    // Создаем гистограмму (бины по 2 градуса)
    let min_lat = 56.0;
    let max_lat = 74.0;
    let bin_size = 2.0;
    let num_bins = ((max_lat - min_lat) / bin_size) as usize;

    let mut bins = vec![0; num_bins];
    for lat in &latitudes {
        let bin_idx = ((*lat - min_lat) / bin_size) as usize;
        if bin_idx < num_bins {
            bins[bin_idx] += 1;
        }
    }

    let max_count = *bins.iter().max().unwrap_or(&1);

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Распределение точек наблюдений по широте",
            ("sans-serif", 30).into_font(),
        )
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(min_lat..max_lat, 0..max_count + 5)?;

    chart
        .configure_mesh()
        .x_desc("Широта (°N)")
        .y_desc("Количество точек")
        .draw()?;

    chart.draw_series(bins.iter().enumerate().map(|(i, &count)| {
        let lat = min_lat + i as f64 * bin_size;
        let x0 = lat;
        let x1 = lat + bin_size;
        Rectangle::new([(x0, 0), (x1, count)], BLUE.mix(0.6).filled())
    }))?;

    root.present()?;
    Ok(())
}
