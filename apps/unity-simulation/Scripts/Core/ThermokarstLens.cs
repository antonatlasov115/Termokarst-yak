using UnityEngine;

namespace ThermokarstSimulation.Core
{
    /// <summary>
    /// Термокарстовая линза - основная структура данных
    /// </summary>
    [System.Serializable]
    public class ThermokarstLens
    {
        [Header("Геометрия")]
        [Tooltip("Глубина просадки (м)")]
        public float depth;

        [Tooltip("Диаметр (м)")]
        public float diameter;

        [Tooltip("Объем (м³)")]
        public float volume;

        [Tooltip("Площадь поверхности (м²)")]
        public float surfaceArea;

        [Header("Динамика")]
        [Tooltip("Возраст (годы)")]
        public int age;

        [Tooltip("Скорость роста объема (м³/год)")]
        public float growthRate;

        public ThermokarstLens(float depth, float diameter, int age = 0)
        {
            this.depth = depth;
            this.diameter = diameter;
            this.age = age;
            this.volume = CalculateVolume(depth, diameter);
            this.surfaceArea = CalculateSurfaceArea(diameter);
            this.growthRate = 0f;
        }

        /// <summary>
        /// Рассчитать объем (приближение цилиндром)
        /// </summary>
        private static float CalculateVolume(float depth, float diameter)
        {
            float radius = diameter / 2f;
            return Mathf.PI * radius * radius * depth;
        }

        /// <summary>
        /// Рассчитать площадь поверхности
        /// </summary>
        private static float CalculateSurfaceArea(float diameter)
        {
            float radius = diameter / 2f;
            return Mathf.PI * radius * radius;
        }

        /// <summary>
        /// Обновить параметры линзы
        /// </summary>
        public void Update(float newDepth, float newDiameter)
        {
            float oldVolume = volume;

            depth = newDepth;
            diameter = newDiameter;
            volume = CalculateVolume(newDepth, newDiameter);
            surfaceArea = CalculateSurfaceArea(newDiameter);
            growthRate = volume - oldVolume;
        }

        /// <summary>
        /// Соотношение глубина/диаметр
        /// </summary>
        public float AspectRatio => diameter > 0f ? depth / diameter : 0f;

        /// <summary>
        /// Проверка стабильности
        /// </summary>
        public bool IsStable()
        {
            const float MAX_DEPTH = 15f;
            const float MAX_DIAMETER = 100f;
            const float MIN_ASPECT_RATIO = 0.05f;
            const float MAX_ASPECT_RATIO = 0.6f;

            float aspect = AspectRatio;

            return depth < MAX_DEPTH
                && diameter < MAX_DIAMETER
                && aspect > MIN_ASPECT_RATIO
                && aspect < MAX_ASPECT_RATIO;
        }
    }
}
