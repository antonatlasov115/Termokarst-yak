//! Детекция термокарстовых образований на изображениях

use image::{DynamicImage, GrayImage};
use imageproc::edges::canny;
use imageproc::contrast::{threshold, ThresholdType};
use anyhow::Result;

/// Результат детекции
#[derive(Debug, Clone)]
pub struct DetectionResult {
    /// Найденные окружности
    pub circles: Vec<Circle>,

    /// Обработанное изображение (края)
    pub edges: GrayImage,

    /// Бинарная маска
    pub mask: GrayImage,
}

/// Окружность (термокарстовое образование)
#[derive(Debug, Clone)]
pub struct Circle {
    pub center_x: u32,
    pub center_y: u32,
    pub radius: u32,
}

/// Детектор термокарстовых образований
pub struct ThermokarstDetector {
    image: DynamicImage,
}

impl ThermokarstDetector {
    pub fn new(image: DynamicImage) -> Self {
        Self { image }
    }

    /// Загрузить изображение из файла
    pub fn from_file(path: &str) -> Result<Self> {
        let image = image::open(path)?;
        Ok(Self::new(image))
    }

    /// Детектировать термокарстовые образования
    pub fn detect(&self) -> Result<DetectionResult> {
        // 1. Конвертировать в grayscale
        let gray = self.image.to_luma8();

        // 2. Детекция краев (Canny)
        let edges = canny(&gray, 50.0, 100.0);

        // 3. Пороговая обработка для выделения темных областей (вода)
        let mask = threshold(&gray, 100, ThresholdType::Binary);

        // 4. Найти окружности (упрощенный алгоритм)
        let circles = self.find_circles(&mask)?;

        Ok(DetectionResult {
            circles,
            edges,
            mask,
        })
    }

    /// Упрощенный поиск окружностей
    fn find_circles(&self, mask: &GrayImage) -> Result<Vec<Circle>> {
        let mut circles = Vec::new();

        // Найти связные компоненты (темные области)
        let components = self.find_connected_components(mask);

        for component in components {
            if component.pixels.len() > 100 {
                // Достаточно большая область
                let circle = self.fit_circle(&component);
                circles.push(circle);
            }
        }

        Ok(circles)
    }

    /// Найти связные компоненты
    fn find_connected_components(&self, mask: &GrayImage) -> Vec<Component> {
        let (width, height) = mask.dimensions();
        let mut visited = vec![vec![false; width as usize]; height as usize];
        let mut components = Vec::new();

        for y in 0..height {
            for x in 0..width {
                if !visited[y as usize][x as usize] && mask.get_pixel(x, y)[0] < 128 {
                    let component = self.flood_fill(mask, &mut visited, x, y);
                    if !component.pixels.is_empty() {
                        components.push(component);
                    }
                }
            }
        }

        components
    }

    /// Flood fill для поиска компонент
    fn flood_fill(
        &self,
        mask: &GrayImage,
        visited: &mut Vec<Vec<bool>>,
        start_x: u32,
        start_y: u32,
    ) -> Component {
        let mut pixels = Vec::new();
        let mut stack = vec![(start_x, start_y)];
        let (width, height) = mask.dimensions();

        while let Some((x, y)) = stack.pop() {
            if x >= width || y >= height || visited[y as usize][x as usize] {
                continue;
            }

            if mask.get_pixel(x, y)[0] >= 128 {
                continue;
            }

            visited[y as usize][x as usize] = true;
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

        Component { pixels }
    }

    /// Аппроксимировать компоненту окружностью
    fn fit_circle(&self, component: &Component) -> Circle {
        // Найти центр масс
        let sum_x: u32 = component.pixels.iter().map(|(x, _)| x).sum();
        let sum_y: u32 = component.pixels.iter().map(|(_, y)| y).sum();
        let count = component.pixels.len() as u32;

        let center_x = sum_x / count;
        let center_y = sum_y / count;

        // Найти средний радиус
        let sum_dist: f64 = component
            .pixels
            .iter()
            .map(|(x, y)| {
                let dx = *x as f64 - center_x as f64;
                let dy = *y as f64 - center_y as f64;
                (dx * dx + dy * dy).sqrt()
            })
            .sum();

        let radius = (sum_dist / count as f64) as u32;

        Circle {
            center_x,
            center_y,
            radius,
        }
    }

    /// Рассчитать диаметр в метрах
    pub fn calculate_diameter_meters(
        &self,
        circle: &Circle,
        meters_per_pixel: f64,
    ) -> f64 {
        (circle.radius * 2) as f64 * meters_per_pixel
    }

    /// Оценить площадь в м²
    pub fn calculate_area_m2(&self, circle: &Circle, meters_per_pixel: f64) -> f64 {
        let radius_m = circle.radius as f64 * meters_per_pixel;
        std::f64::consts::PI * radius_m * radius_m
    }
}

/// Связная компонента
struct Component {
    pixels: Vec<(u32, u32)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::downloader::ImageDownloader;
    use tempfile::tempdir;

    #[test]
    fn test_detect_synthetic() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.jpg");

        // Создать синтетическое изображение
        ImageDownloader::create_synthetic_thermokarst(&path, 200).unwrap();

        // Детектировать
        let detector = ThermokarstDetector::from_file(path.to_str().unwrap()).unwrap();
        let result = detector.detect().unwrap();

        // Должна быть найдена хотя бы одна окружность
        assert!(!result.circles.is_empty());

        let circle = &result.circles[0];
        println!("Найдена окружность: центр=({}, {}), радиус={}",
                 circle.center_x, circle.center_y, circle.radius);
    }
}
