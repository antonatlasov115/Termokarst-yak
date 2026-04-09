//! Модуль визуализации результатов симуляции
//!
//! Создает графики развития термокарста, профили глубины,
//! диаграммы стадий и другие визуализации.

#[cfg(feature = "visualization")]
use plotters::prelude::*;

use std::path::Path;
use thermokarst_core::{SimulationResult, ThermokarstStage};

#[cfg(feature = "visualization")]
/// Визуализатор результатов симуляции
pub struct SimulationVisualizer;

#[cfg(feature = "visualization")]
impl SimulationVisualizer {
    /// Создать график развития термокарста во времени
    ///
    /// # Аргументы
    /// * `results` - Результаты симуляции по годам
    /// * `output_path` - Путь для сохранения графика
    pub fn plot_development(
        results: &[SimulationResult],
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(output_path, (1200, 800)).into_drawing_area();
        root.fill(&WHITE)?;

        // Берем последнюю линзу из каждого результата
        let max_year = results.len() as f64;
        let max_depth = results
            .iter()
            .filter_map(|r| r.lenses.last())
            .map(|lens| lens.depth)
            .fold(0.0, f64::max);
        let max_diameter = results
            .iter()
            .filter_map(|r| r.lenses.last())
            .map(|lens| lens.diameter)
            .fold(0.0, f64::max);

        let mut chart = ChartBuilder::on(&root)
            .caption("Развитие термокарста во времени", ("sans-serif", 40))
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0.0..max_year, 0.0..max_depth.max(max_diameter))?;

        chart
            .configure_mesh()
            .x_desc("Годы")
            .y_desc("Размер (м)")
            .draw()?;

        // График глубины
        chart
            .draw_series(LineSeries::new(
                results
                    .iter()
                    .enumerate()
                    .filter_map(|(i, r)| r.lenses.last().map(|lens| (i as f64, lens.depth))),
                &BLUE,
            ))?
            .label("Глубина (м)")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

        // График диаметра
        chart
            .draw_series(LineSeries::new(
                results
                    .iter()
                    .enumerate()
                    .filter_map(|(i, r)| r.lenses.last().map(|lens| (i as f64, lens.diameter))),
                &RED,
            ))?
            .label("Диаметр (м)")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;

        root.present()?;
        Ok(())
    }

    /// Создать график объема термокарста
    pub fn plot_volume(
        results: &[SimulationResult],
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(output_path, (1200, 600)).into_drawing_area();
        root.fill(&WHITE)?;

        let max_year = results.len() as f64;
        let max_volume = results
            .iter()
            .filter_map(|r| r.lenses.last())
            .map(|lens| lens.volume)
            .fold(0.0, f64::max);

        let mut chart = ChartBuilder::on(&root)
            .caption("Объем термокарста во времени", ("sans-serif", 40))
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0.0..max_year, 0.0..max_volume)?;

        chart
            .configure_mesh()
            .x_desc("Годы")
            .y_desc("Объем (м³)")
            .draw()?;

        chart.draw_series(LineSeries::new(
            results
                .iter()
                .enumerate()
                .filter_map(|(i, r)| r.lenses.last().map(|lens| (i as f64, lens.volume))),
            &GREEN,
        ))?;

        root.present()?;
        Ok(())
    }

    /// Создать диаграмму стадий развития
    pub fn plot_stages(
        results: &[SimulationResult],
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(output_path, (1200, 400)).into_drawing_area();
        root.fill(&WHITE)?;

        let max_year = results.len() as f64;

        let mut chart = ChartBuilder::on(&root)
            .caption("Стадии развития термокарста", ("sans-serif", 40))
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0.0..max_year, 0.0..4.0)?;

        chart
            .configure_mesh()
            .x_desc("Годы")
            .y_desc("Стадия")
            .y_label_formatter(&|y| match *y as i32 {
                0 => "Инициация".to_string(),
                1 => "Активное развитие".to_string(),
                2 => "Стабилизация".to_string(),
                3 => "Деградация".to_string(),
                _ => "".to_string(),
            })
            .draw()?;

        // Преобразуем стадии в числа для графика
        let stage_data: Vec<(f64, f64)> = results
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let stage_num = match r.stage {
                    ThermokarstStage::Initiation => 0.0,
                    ThermokarstStage::ActiveDevelopment => 1.0,
                    ThermokarstStage::Stabilization => 2.0,
                    ThermokarstStage::Degradation => 3.0,
                };
                (i as f64, stage_num)
            })
            .collect();

        chart.draw_series(LineSeries::new(stage_data, &MAGENTA))?;

        root.present()?;
        Ok(())
    }

    /// Создать профиль глубины (поперечное сечение)
    pub fn plot_cross_section(
        result: &SimulationResult,
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
        root.fill(&WHITE)?;

        let lens = result.lenses.last().ok_or("No lens data available")?;
        let radius = lens.diameter / 2.0;
        let depth = lens.depth;

        let mut chart = ChartBuilder::on(&root)
            .caption(
                format!("Поперечное сечение (год {})", lens.age),
                ("sans-serif", 30),
            )
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(-radius..radius, -depth..0.5)?;

        chart
            .configure_mesh()
            .x_desc("Расстояние (м)")
            .y_desc("Глубина (м)")
            .draw()?;

        // Рисуем профиль термокарста (упрощенная парабола)
        let profile: Vec<(f64, f64)> = (0..100)
            .map(|i| {
                let x = -radius + (2.0 * radius * i as f64 / 99.0);
                let normalized_x = x / radius;
                let y = -depth * (1.0 - normalized_x * normalized_x);
                (x, y)
            })
            .collect();

        chart.draw_series(LineSeries::new(profile, &BLUE.mix(0.8)))?;

        // Заливка
        let fill_area: Vec<(f64, f64)> = (0..100)
            .map(|i| {
                let x = -radius + (2.0 * radius * i as f64 / 99.0);
                let normalized_x = x / radius;
                let y = -depth * (1.0 - normalized_x * normalized_x);
                (x, y)
            })
            .chain(vec![(-radius, 0.0), (radius, 0.0)])
            .collect();

        chart.draw_series(std::iter::once(Polygon::new(fill_area, &BLUE.mix(0.3))))?;

        root.present()?;
        Ok(())
    }

    /// Создать комплексный отчет с несколькими графиками
    pub fn create_report(
        results: &[SimulationResult],
        output_dir: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(output_dir)?;

        Self::plot_development(results, &output_dir.join("development.png"))?;
        Self::plot_volume(results, &output_dir.join("volume.png"))?;
        Self::plot_stages(results, &output_dir.join("stages.png"))?;

        if let Some(last_result) = results.last() {
            Self::plot_cross_section(last_result, &output_dir.join("cross_section.png"))?;
        }

        Ok(())
    }
}

#[cfg(not(feature = "visualization"))]
pub struct SimulationVisualizer;

#[cfg(not(feature = "visualization"))]
impl SimulationVisualizer {
    pub fn plot_development(
        _results: &[SimulationResult],
        _output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Err("Visualization feature not enabled. Rebuild with --features visualization".into())
    }

    pub fn plot_volume(
        _results: &[SimulationResult],
        _output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Err("Visualization feature not enabled. Rebuild with --features visualization".into())
    }

    pub fn plot_stages(
        _results: &[SimulationResult],
        _output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Err("Visualization feature not enabled. Rebuild with --features visualization".into())
    }

    pub fn plot_cross_section(
        _result: &SimulationResult,
        _output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Err("Visualization feature not enabled. Rebuild with --features visualization".into())
    }

    pub fn create_report(
        _results: &[SimulationResult],
        _output_dir: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Err("Visualization feature not enabled. Rebuild with --features visualization".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use thermokarst_core::{EnvironmentParams, ThermokarstLens};

    #[test]
    #[cfg(feature = "visualization")]
    fn test_visualization_available() {
        // Тест что визуализация доступна с feature
        let lens = ThermokarstLens::new(1.0, 5.0, 10);
        let result = SimulationResult {
            lenses: vec![lens],
            environment: EnvironmentParams::default(),
            stage: ThermokarstStage::ActiveDevelopment,
            total_years: 10,
        };

        let results = vec![result];
        let temp_dir = std::env::temp_dir();

        // Проверяем что функции не паникуют
        let _ = SimulationVisualizer::plot_development(&results, &temp_dir.join("test_dev.png"));
    }
}
