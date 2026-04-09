using UnityEngine;
using ThermokarstSimulation.Core;

namespace ThermokarstSimulation.Procedural
{
    /// <summary>
    /// Продвинутая система эрозии берегов термокарстовых озер
    /// </summary>
    public class ThermokarstErosionSystem
    {
        private readonly EnvironmentParams climate;

        public ThermokarstErosionSystem(EnvironmentParams climate)
        {
            this.climate = climate;
        }

        /// <summary>
        /// Расчет скорости бокового таяния (ниша вытаивания)
        /// </summary>
        /// <param name="waterTemp">Температура воды (°C)</param>
        /// <param name="iceContent">Льдистость берега</param>
        /// <param name="waveAction">Интенсивность волнового воздействия (0-1)</param>
        public float CalculateLateralThawRate(float waterTemp, float iceContent, float waveAction)
        {
            // Базовая скорость таяния зависит от температуры воды
            float baseMeltRate = Mathf.Max(0f, waterTemp) * 0.1f; // м/год при 10°C

            // Льдистость усиливает эффект
            float iceFactor = 1f + iceContent * 2f;

            // Волны ускоряют эрозию
            float waveFactor = 1f + waveAction * 0.5f;

            return baseMeltRate * iceFactor * waveFactor;
        }

        /// <summary>
        /// Термическая абразия (подмыв берега теплой водой)
        /// </summary>
        public float CalculateThermalAbrasion(TerrainCell shoreCell, TerrainCell waterCell, float distance)
        {
            // Температура воды выше температуры мерзлоты
            float tempDifference = climate.airTemp - climate.permafrostTemp;

            // Теплопередача обратно пропорциональна расстоянию
            float heatTransfer = tempDifference / (distance + 1f);

            // Эффективность зависит от льдистости
            float erosionRate = heatTransfer * shoreCell.iceContent * 0.05f;

            return erosionRate;
        }

        /// <summary>
        /// Гравитационное обрушение крутых склонов
        /// </summary>
        public float CalculateSlopeCollapse(float heightDifference, float slopeAngle)
        {
            const float CRITICAL_ANGLE = 35f; // градусы
            const float CRITICAL_HEIGHT = 3f; // метры

            if (slopeAngle < CRITICAL_ANGLE && heightDifference < CRITICAL_HEIGHT)
                return 0f;

            // Скорость обрушения растет экспоненциально
            float angleExcess = Mathf.Max(0f, slopeAngle - CRITICAL_ANGLE);
            float heightExcess = Mathf.Max(0f, heightDifference - CRITICAL_HEIGHT);

            return (angleExcess * 0.1f + heightExcess * 0.2f);
        }

        /// <summary>
        /// Расчет волнового воздействия (зависит от размера озера)
        /// </summary>
        public float CalculateWaveAction(float lakeArea, float windSpeed)
        {
            // Fetch (разгон волны) зависит от площади
            float fetch = Mathf.Sqrt(lakeArea);

            // Высота волны по упрощенной формуле
            float waveHeight = 0.5f * Mathf.Sqrt(fetch) * windSpeed / 10f;

            // Нормализуем к 0-1
            return Mathf.Clamp01(waveHeight / 2f);
        }

        /// <summary>
        /// Сезонная вариация эрозии
        /// </summary>
        public float GetSeasonalMultiplier(float dayOfYear)
        {
            // Максимум эрозии летом (день 180), минимум зимой
            float seasonalPhase = Mathf.Cos((dayOfYear - 180f) / 365f * Mathf.PI * 2f);
            return Mathf.Lerp(0.1f, 1f, (seasonalPhase + 1f) / 2f);
        }
    }
}
