using UnityEngine;
using ThermokarstSimulation.Core;

namespace ThermokarstSimulation.Simulation
{
    /// <summary>
    /// Движок симуляции термокарста (формула Атласова v0.3.0)
    /// </summary>
    public class ThermokarstEngine
    {
        private readonly EnvironmentParams parameters;

        public ThermokarstEngine(EnvironmentParams parameters)
        {
            this.parameters = parameters;
        }

        /// <summary>
        /// Симуляция развития термокарста за один год
        /// </summary>
        public void SimulateYear(ThermokarstLens lens)
        {
            // 1. Расчет вертикального протаивания (формула Атласова)
            float depthIncrease = CalculateDepthIncrease();

            // 2. Расчет латерального расширения
            float diameterIncrease = CalculateLateralExpansion(lens.diameter);

            // 3. Обновление линзы
            lens.Update(lens.depth + depthIncrease, lens.diameter + diameterIncrease);
            lens.age++;
        }

        /// <summary>
        /// Расчет вертикального протаивания по формуле Атласова v0.3.0
        /// ξ = √(2λₜ·DDT / (L·ρw·w^0.7)) · exp(0.30·(1-V)) · (1 + 0.12·ln(ΔT/40))
        /// </summary>
        private float CalculateDepthIncrease()
        {
            // Теплопроводность по модели Йоханзена
            float thermalConductivity = parameters.soilType.ThermalConductivity(parameters.soilSaturationRatio);

            // DDT в секундах (критично!)
            float ddtAnnualSeconds = parameters.airTemp * parameters.warmSeasonDays * 86400f;

            const float LATENT_HEAT = 334000f; // Дж/кг
            const float WATER_DENSITY = 1000f; // кг/м³
            float iceContent = Mathf.Max(parameters.iceContent, 0.01f);

            // Факторы
            float vegetationFactor = Mathf.Exp(0.30f * (1f - parameters.vegetationCover));
            float tempFactor = 1f + 0.12f * Mathf.Log(parameters.temperatureAmplitude / 40f);

            // Расчет ALT (сезонное протаивание)
            float innerSqrt = (2f * thermalConductivity * ddtAnnualSeconds) /
                             (LATENT_HEAT * WATER_DENSITY * Mathf.Pow(iceContent, 0.7f));

            float xiAlt = Mathf.Sqrt(innerSqrt) * vegetationFactor * tempFactor;

            return xiAlt;
        }

        /// <summary>
        /// Расчет латерального расширения
        /// D(t) = D₀ + k·ln(1 + t)
        /// </summary>
        private float CalculateLateralExpansion(float currentDiameter)
        {
            float k = 2f * (1f + parameters.iceContent * 0.5f);

            // Скорость расширения уменьшается с размером
            float expansionRate = k / (1f + currentDiameter * 0.1f);

            return expansionRate * Time.deltaTime;
        }

        /// <summary>
        /// Обратное моделирование - оценка возраста по глубине
        /// </summary>
        public float EstimateAgeFromDepth(float depth)
        {
            float xiAlt = CalculateDepthIncrease();

            if (xiAlt <= 0f)
                return 0f;

            // t = (depth / xi_alt)²
            float estimatedYears = Mathf.Pow(depth / xiAlt, 2f);
            return Mathf.Max(estimatedYears, 1f);
        }

        /// <summary>
        /// Оценка возраста по диаметру
        /// </summary>
        public float EstimateAgeFromDiameter(float diameter)
        {
            const float INITIAL_DIAMETER = 2f;
            float k = 2f * (1f + parameters.iceContent * 0.5f);

            // t = exp((D - D₀)/k) - 1
            float estimatedYears = Mathf.Exp((diameter - INITIAL_DIAMETER) / k) - 1f;
            return Mathf.Max(estimatedYears, 1f);
        }
    }
}
