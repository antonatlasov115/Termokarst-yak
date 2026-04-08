//! Анализ стабильности термокарстовых образований

use thermokarst_core::{ThermokarstLens, ThermokarstStage};

/// Анализатор стабильности
pub struct StabilityAnalyzer;

impl StabilityAnalyzer {
    /// Определение стадии развития термокарста
    pub fn determine_stage(lens: &ThermokarstLens) -> ThermokarstStage {
        let age = lens.age;

        // Инициация: молодое и пока еще маленькое образование
        if age < 5 && lens.volume < 10.0 {
            ThermokarstStage::Initiation
        }
        // Активное развитие: линза уверенно растет в объеме
        else if lens.growth_rate > 1.0 {
            ThermokarstStage::ActiveDevelopment
        }
        // Стабилизация: рост почти остановился
        else if lens.growth_rate >= -0.5 && lens.growth_rate <= 1.0 {
            ThermokarstStage::Stabilization
        }
        // Деградация: линза заплывает/уменьшается в объеме
        else {
            ThermokarstStage::Degradation
        }
    }

    /// Оценка риска обрушения берегов
    pub fn collapse_risk(lens: &ThermokarstLens) -> f64 {
        let aspect = lens.aspect_ratio();

        // Высокий риск при большом соотношении глубина/диаметр
        if aspect > 0.5 {
            0.9
        } else if aspect > 0.4 {
            0.6
        } else if aspect > 0.3 {
            0.3
        } else {
            0.1
        }
    }

    /// Проверка геометрической стабильности
    pub fn is_geometrically_stable(lens: &ThermokarstLens) -> bool {
        const MAX_DEPTH: f64 = 15.0;
        const MAX_DIAMETER: f64 = 100.0;
        const MIN_ASPECT: f64 = 0.05;
        const MAX_ASPECT: f64 = 0.6;

        let aspect = lens.aspect_ratio();

        lens.depth < MAX_DEPTH
            && lens.diameter < MAX_DIAMETER
            && aspect > MIN_ASPECT
            && aspect < MAX_ASPECT
    }

    /// Оценка долгосрочной устойчивости (0-1)
    pub fn long_term_stability_score(lens: &ThermokarstLens) -> f64 {
        let mut score: f64 = 1.0;

        // Штраф за большую глубину
        if lens.depth > 10.0 {
            score -= 0.3;
        } else if lens.depth > 7.0 {
            score -= 0.15;
        }

        // Штраф за большой диаметр
        if lens.diameter > 80.0 {
            score -= 0.2;
        } else if lens.diameter > 50.0 {
            score -= 0.1;
        }

        // Штраф за неблагоприятное соотношение
        let aspect = lens.aspect_ratio();
        if aspect > 0.5 || aspect < 0.1 {
            score -= 0.3;
        }

        // Штраф за высокую скорость роста
        if lens.growth_rate > 5.0 {
            score -= 0.2;
        }

        score.max(0.0).min(1.0)
    }

    /// Прогноз времени до стабилизации (годы)
    pub fn time_to_stabilization(lens: &ThermokarstLens) -> Option<u32> {
        if lens.growth_rate < 0.5 {
            return Some(0); // Уже стабилизировалась
        }

        // Простая экстраполяция
        let years = (lens.growth_rate / 0.1).ceil() as u32;

        if years > 100 {
            None // Слишком долго или не стабилизируется
        } else {
            Some(years)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_young_lens_is_initiating() {
        let lens = ThermokarstLens::new(0.5, 2.0, 2);
        let stage = StabilityAnalyzer::determine_stage(&lens);

        assert_eq!(stage, ThermokarstStage::Initiation);
    }

    #[test]
    fn test_old_large_fast_growing_lens_is_active() {
        // Столетнее озеро с большим объемом и высокой скоростью роста
        let mut lens = ThermokarstLens::new(5.0, 15.0, 100);
        lens.volume = 700.0;
        lens.growth_rate = 12.0; // Высокая скорость роста

        let stage = StabilityAnalyzer::determine_stage(&lens);

        // Должно быть ActiveDevelopment, а не Initiation!
        assert_eq!(stage, ThermokarstStage::ActiveDevelopment);
    }

    #[test]
    fn test_slow_growing_lens_is_stabilizing() {
        let mut lens = ThermokarstLens::new(3.0, 10.0, 50);
        lens.growth_rate = 0.5; // Медленный рост

        let stage = StabilityAnalyzer::determine_stage(&lens);

        assert_eq!(stage, ThermokarstStage::Stabilization);
    }

    #[test]
    fn test_shrinking_lens_is_degrading() {
        let mut lens = ThermokarstLens::new(2.0, 8.0, 80);
        lens.growth_rate = -1.0; // Уменьшается

        let stage = StabilityAnalyzer::determine_stage(&lens);

        assert_eq!(stage, ThermokarstStage::Degradation);
    }

    #[test]
    fn test_deep_narrow_lens_has_high_collapse_risk() {
        let lens = ThermokarstLens::new(5.0, 8.0, 10);
        let risk = StabilityAnalyzer::collapse_risk(&lens);

        assert!(risk > 0.5);
    }
}
