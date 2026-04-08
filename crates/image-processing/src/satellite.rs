//! Анализ спутниковых данных для детекции термокарста

use anyhow::{Result, Context};
use ndarray::Array2;
use std::path::Path;

/// Анализатор спутниковых снимков
pub struct SatelliteAnalyzer {
    /// Размеры изображения (width, height)
    dimensions: (usize, usize),
    /// Разрешение в метрах на пиксель
    resolution_m: f64,
}

impl SatelliteAnalyzer {
    /// Создать анализатор с заданными параметрами
    pub fn new(width: usize, height: usize, resolution_m: f64) -> Self {
        Self {
            dimensions: (width, height),
            resolution_m,
        }
    }

    /// Рассчитать NDVI (Normalized Difference Vegetation Index)
    /// NDVI = (NIR - Red) / (NIR + Red)
    pub fn calculate_ndvi(
        &self,
        nir_band: &Array2<f32>,
        red_band: &Array2<f32>,
    ) -> Result<Array2<f32>> {
        if nir_band.shape() != red_band.shape() {
            anyhow::bail!("NIR и Red каналы должны иметь одинаковый размер");
        }

        let ndvi = (nir_band - red_band) / (nir_band + red_band);
        Ok(ndvi)
    }

    /// Рассчитать NDWI (Normalized Difference Water Index)
    /// NDWI = (Green - NIR) / (Green + NIR)
    /// Используется для детекции водных поверхностей
    pub fn calculate_ndwi(
        &self,
        green_band: &Array2<f32>,
        nir_band: &Array2<f32>,
    ) -> Result<Array2<f32>> {
        if green_band.shape() != nir_band.shape() {
            anyhow::bail!("Green и NIR каналы должны иметь одинаковый размер");
        }

        let ndwi = (green_band - nir_band) / (green_band + nir_band);
        Ok(ndwi)
    }

    /// Детектировать водные поверхности (термокарстовые озера)
    pub fn detect_water_bodies(
        &self,
        ndwi: &Array2<f32>,
        threshold: f32,
    ) -> Result<Vec<WaterBody>> {
        let mut water_bodies = Vec::new();
        let (height, width) = ndwi.dim();
        let mut visited = Array2::from_elem((height, width), false);

        for y in 0..height {
            for x in 0..width {
                if !visited[[y, x]] && ndwi[[y, x]] > threshold {
                    let pixels = self.flood_fill(ndwi, &mut visited, x, y, threshold);

                    if pixels.len() > 10 {
                        // Минимум 10 пикселей для водного объекта
                        let water_body = self.analyze_water_body(&pixels);
                        water_bodies.push(water_body);
                    }
                }
            }
        }

        Ok(water_bodies)
    }

    /// Flood fill для поиска связных водных областей
    fn flood_fill(
        &self,
        ndwi: &Array2<f32>,
        visited: &mut Array2<bool>,
        start_x: usize,
        start_y: usize,
        threshold: f32,
    ) -> Vec<(usize, usize)> {
        let mut pixels = Vec::new();
        let mut stack = vec![(start_x, start_y)];
        let (height, width) = ndwi.dim();

        while let Some((x, y)) = stack.pop() {
            if x >= width || y >= height || visited[[y, x]] {
                continue;
            }

            if ndwi[[y, x]] <= threshold {
                continue;
            }

            visited[[y, x]] = true;
            pixels.push((x, y));

            // Добавить соседей
            if x > 0 {
                stack.push((x - 1, y));
            }
            if x < width - 1 {
                stack.push((x + 1, y));
            }
            if y > 0 {
                stack.push((x, y - 1));
            }
            if y < height - 1 {
                stack.push((x, y + 1));
            }
        }

        pixels
    }

    /// Анализировать водный объект
    fn analyze_water_body(&self, pixels: &[(usize, usize)]) -> WaterBody {
        // Найти центр масс
        let sum_x: usize = pixels.iter().map(|(x, _)| x).sum();
        let sum_y: usize = pixels.iter().map(|(_, y)| y).sum();
        let count = pixels.len();

        let center_x = sum_x / count;
        let center_y = sum_y / count;

        // Найти границы
        let min_x = pixels.iter().map(|(x, _)| x).min().unwrap();
        let max_x = pixels.iter().map(|(x, _)| x).max().unwrap();
        let min_y = pixels.iter().map(|(_, y)| y).min().unwrap();
        let max_y = pixels.iter().map(|(_, y)| y).max().unwrap();

        // Рассчитать площадь в м²
        let area_m2 = count as f64 * self.resolution_m * self.resolution_m;

        // Оценить диаметр (предполагая круглую форму)
        let diameter_m = 2.0 * (area_m2 / std::f64::consts::PI).sqrt();

        WaterBody {
            center: (center_x, center_y),
            pixel_count: count,
            area_m2,
            diameter_m,
            bbox: BoundingBox {
                min_x: *min_x,
                max_x: *max_x,
                min_y: *min_y,
                max_y: *max_y,
            },
        }
    }

    /// Временной анализ - сравнить два снимка
    pub fn temporal_analysis(
        current_bodies: &[WaterBody],
        previous_bodies: &[WaterBody],
        years_between: f64,
    ) -> TemporalAnalysis {
        let mut new_count = 0;
        let mut expanded = Vec::new();
        let mut total_growth = 0.0;

        // Простое сопоставление по близости центров
        for current in current_bodies {
            let mut found_match = false;

            for previous in previous_bodies {
                let dx = current.center.0 as f64 - previous.center.0 as f64;
                let dy = current.center.1 as f64 - previous.center.1 as f64;
                let distance = (dx * dx + dy * dy).sqrt();

                // Если центры близко (в пределах 50 пикселей)
                if distance < 50.0 {
                    found_match = true;
                    let growth = current.area_m2 - previous.area_m2;

                    if growth > 0.0 {
                        expanded.push(ExpansionData {
                            previous_area: previous.area_m2,
                            current_area: current.area_m2,
                            growth_m2: growth,
                            growth_rate: growth / years_between,
                        });
                        total_growth += growth;
                    }
                    break;
                }
            }

            if !found_match {
                new_count += 1;
            }
        }

        TemporalAnalysis {
            new_lakes: new_count,
            expanded_lakes: expanded,
            average_growth_rate: if years_between > 0.0 {
                total_growth / years_between
            } else {
                0.0
            },
        }
    }
}

/// Водный объект (термокарстовое озеро)
#[derive(Debug, Clone)]
pub struct WaterBody {
    /// Центр (x, y) в пикселях
    pub center: (usize, usize),
    /// Количество пикселей
    pub pixel_count: usize,
    /// Площадь в м²
    pub area_m2: f64,
    /// Диаметр в метрах
    pub diameter_m: f64,
    /// Ограничивающий прямоугольник
    pub bbox: BoundingBox,
}

/// Ограничивающий прямоугольник
#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub min_x: usize,
    pub max_x: usize,
    pub min_y: usize,
    pub max_y: usize,
}

/// Данные о расширении озера
#[derive(Debug, Clone)]
pub struct ExpansionData {
    pub previous_area: f64,
    pub current_area: f64,
    pub growth_m2: f64,
    pub growth_rate: f64, // м²/год
}

/// Результат временного анализа
#[derive(Debug, Clone)]
pub struct TemporalAnalysis {
    pub new_lakes: usize,
    pub expanded_lakes: Vec<ExpansionData>,
    pub average_growth_rate: f64, // м²/год
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::Array2;

    #[test]
    fn test_ndvi_calculation() {
        let analyzer = SatelliteAnalyzer::new(10, 10, 10.0);

        let nir = Array2::from_elem((10, 10), 0.8);
        let red = Array2::from_elem((10, 10), 0.2);

        let ndvi = analyzer.calculate_ndvi(&nir, &red).unwrap();

        // NDVI = (0.8 - 0.2) / (0.8 + 0.2) = 0.6
        assert!((ndvi[[0, 0]] - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_ndwi_calculation() {
        let analyzer = SatelliteAnalyzer::new(10, 10, 10.0);

        let green = Array2::from_elem((10, 10), 0.7);
        let nir = Array2::from_elem((10, 10), 0.3);

        let ndwi = analyzer.calculate_ndwi(&green, &nir).unwrap();

        // NDWI = (0.7 - 0.3) / (0.7 + 0.3) = 0.4
        assert!((ndwi[[0, 0]] - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_water_detection() {
        let analyzer = SatelliteAnalyzer::new(20, 20, 10.0);

        // Создать NDWI с водным объектом в центре
        let mut ndwi = Array2::from_elem((20, 20), 0.0);
        for y in 8..12 {
            for x in 8..12 {
                ndwi[[y, x]] = 0.5; // Высокий NDWI = вода
            }
        }

        let water_bodies = analyzer.detect_water_bodies(&ndwi, 0.3).unwrap();

        assert_eq!(water_bodies.len(), 1);
        assert_eq!(water_bodies[0].pixel_count, 16); // 4x4 пикселей

        println!("Площадь: {:.1} м²", water_bodies[0].area_m2);
        println!("Диаметр: {:.1} м", water_bodies[0].diameter_m);
    }

    #[test]
    fn test_temporal_analysis() {
        let previous = vec![
            WaterBody {
                center: (10, 10),
                pixel_count: 100,
                area_m2: 10000.0,
                diameter_m: 100.0,
                bbox: BoundingBox { min_x: 5, max_x: 15, min_y: 5, max_y: 15 },
            },
        ];

        let current = vec![
            WaterBody {
                center: (10, 10),
                pixel_count: 150,
                area_m2: 15000.0,
                diameter_m: 120.0,
                bbox: BoundingBox { min_x: 5, max_x: 15, min_y: 5, max_y: 15 },
            },
        ];

        let analysis = SatelliteAnalyzer::temporal_analysis(&current, &previous, 5.0);

        assert_eq!(analysis.new_lakes, 0);
        assert_eq!(analysis.expanded_lakes.len(), 1);
        assert_eq!(analysis.expanded_lakes[0].growth_m2, 5000.0);
        assert_eq!(analysis.expanded_lakes[0].growth_rate, 1000.0); // 5000 / 5 лет
    }
}
