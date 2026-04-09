//! Парсер для IRYP (Ice-Rich Yedoma Permafrost) датасета
//! Источник: PANGAEA, https://doi.org/10.1594/PANGAEA.940078

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Точка наблюдения едомы из IRYP датасета
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrypSite {
    /// Название события/локации
    pub event: String,

    /// Тип файла/данных
    pub file_type: String,

    /// Область/регион
    pub area: String,

    /// Широта (градусы)
    pub latitude: f64,

    /// Долгота (градусы)
    pub longitude: f64,

    /// Комментарий/описание
    pub comment: String,

    /// Дата наблюдения
    pub date: Option<String>,

    /// ID записи
    pub id: Option<u32>,

    /// Исследователь
    pub investigator: Option<String>,
}

impl IrypSite {
    /// Проверка, находится ли точка в Якутии
    pub fn is_in_yakutia(&self) -> bool {
        let area_lower = self.area.to_lowercase();
        let comment_lower = self.comment.to_lowercase();

        area_lower.contains("yakutia")
            || area_lower.contains("sakha")
            || area_lower.contains("central yakutia")
            || comment_lower.contains("yakutia")
            || comment_lower.contains("sakha")
            || self.is_yakutia_by_coords()
    }

    /// Проверка по координатам (Якутия: 56-74°N, 105-162°E)
    fn is_yakutia_by_coords(&self) -> bool {
        self.latitude >= 56.0
            && self.latitude <= 74.0
            && self.longitude >= 105.0
            && self.longitude <= 162.0
    }

    /// Определение региона Якутии
    pub fn yakutia_region(&self) -> Option<&'static str> {
        if !self.is_in_yakutia() {
            return None;
        }

        // Северная Якутия: > 68°N
        if self.latitude > 68.0 {
            Some("Northern Yakutia")
        }
        // Центральная Якутия: 60-68°N
        else if self.latitude >= 60.0 && self.latitude <= 68.0 {
            Some("Central Yakutia")
        }
        // Южная Якутия: < 60°N
        else {
            Some("Southern Yakutia")
        }
    }
}

/// Парсер IRYP датасета
pub struct IrypParser;

impl IrypParser {
    /// Парсинг IRYP .tab файла
    pub fn parse_file<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<IrypSite>> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self::parse_content(&content))
    }

    /// Парсинг содержимого IRYP файла
    pub fn parse_content(content: &str) -> Vec<IrypSite> {
        let mut sites = Vec::new();
        let mut in_data_section = false;

        for line in content.lines() {
            // Пропускаем пустые строки
            if line.trim().is_empty() {
                continue;
            }

            // Ищем начало табличных данных
            if line.starts_with("Event\tFile type\tArea") {
                in_data_section = true;
                continue;
            }

            // Парсим только табличные данные
            if in_data_section {
                if let Some(site) = Self::parse_line(line) {
                    sites.push(site);
                }
            }
        }

        sites
    }

    /// Парсинг одной строки данных
    fn parse_line(line: &str) -> Option<IrypSite> {
        let parts: Vec<&str> = line.split('\t').collect();

        // Минимум нужно: Event, File type, Area, Latitude, Longitude, Comment
        if parts.len() < 6 {
            return None;
        }

        let event = parts[0].to_string();
        let file_type = parts[1].to_string();
        let area = parts[2].to_string();

        // Парсим координаты
        let latitude = parts[3].parse::<f64>().ok()?;
        let longitude = parts[4].parse::<f64>().ok()?;

        let comment = parts[5].to_string();

        // Опциональные поля
        let date = if parts.len() > 6 && !parts[6].is_empty() {
            Some(parts[6].to_string())
        } else {
            None
        };

        let id = if parts.len() > 7 {
            parts[7].parse::<u32>().ok()
        } else {
            None
        };

        let investigator = if parts.len() > 8 && !parts[8].is_empty() {
            Some(parts[8].to_string())
        } else {
            None
        };

        Some(IrypSite {
            event,
            file_type,
            area,
            latitude,
            longitude,
            comment,
            date,
            id,
            investigator,
        })
    }

    /// Фильтрация только точек из Якутии
    pub fn filter_yakutia(sites: Vec<IrypSite>) -> Vec<IrypSite> {
        sites
            .into_iter()
            .filter(|site| site.is_in_yakutia())
            .collect()
    }

    /// Группировка по регионам Якутии
    pub fn group_by_region(
        sites: &[IrypSite],
    ) -> std::collections::HashMap<String, Vec<&IrypSite>> {
        let mut groups: std::collections::HashMap<String, Vec<&IrypSite>> =
            std::collections::HashMap::new();

        for site in sites {
            if let Some(region) = site.yakutia_region() {
                groups
                    .entry(region.to_string())
                    .or_insert_with(Vec::new)
                    .push(site);
            }
        }

        groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yakutia_detection() {
        let site = IrypSite {
            event: "Syrdakh_1976".to_string(),
            file_type: "IRYP_v1 Photographs".to_string(),
            area: "Syrdakh, Central Yakutia".to_string(),
            latitude: 62.557200,
            longitude: 130.893400,
            comment: "Yedoma exposed at a thermokarst lake".to_string(),
            date: Some("1976-06-30".to_string()),
            id: Some(35),
            investigator: Some("Christine Siegert".to_string()),
        };

        assert!(site.is_in_yakutia());
        assert_eq!(site.yakutia_region(), Some("Central Yakutia"));
    }

    #[test]
    fn test_region_classification() {
        // Северная Якутия
        let north = IrypSite {
            event: "Test".to_string(),
            file_type: "Test".to_string(),
            area: "Test".to_string(),
            latitude: 70.0,
            longitude: 130.0,
            comment: "Test".to_string(),
            date: None,
            id: None,
            investigator: None,
        };
        assert_eq!(north.yakutia_region(), Some("Northern Yakutia"));

        // Центральная Якутия
        let central = IrypSite {
            event: "Test".to_string(),
            file_type: "Test".to_string(),
            area: "Test".to_string(),
            latitude: 63.0,
            longitude: 130.0,
            comment: "Test".to_string(),
            date: None,
            id: None,
            investigator: None,
        };
        assert_eq!(central.yakutia_region(), Some("Central Yakutia"));

        // Южная Якутия
        let south = IrypSite {
            event: "Test".to_string(),
            file_type: "Test".to_string(),
            area: "Test".to_string(),
            latitude: 58.0,
            longitude: 130.0,
            comment: "Test".to_string(),
            date: None,
            id: None,
            investigator: None,
        };
        assert_eq!(south.yakutia_region(), Some("Southern Yakutia"));
    }
}
