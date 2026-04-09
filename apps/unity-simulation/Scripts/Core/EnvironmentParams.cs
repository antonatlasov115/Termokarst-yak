using UnityEngine;

namespace ThermokarstSimulation.Core
{
    /// <summary>
    /// Параметры окружающей среды для симуляции термокарста
    /// </summary>
    [System.Serializable]
    public class EnvironmentParams
    {
        [Header("Температура")]
        [Tooltip("Средняя температура воздуха теплого сезона (°C)")]
        [Range(-10f, 20f)]
        public float airTemp = 5f;

        [Tooltip("Температура многолетнемерзлых пород (°C)")]
        [Range(-15f, 0f)]
        public float permafrostTemp = -2f;

        [Tooltip("Годовая амплитуда температур (°C)")]
        [Range(40f, 120f)]
        public float temperatureAmplitude = 88f;

        [Header("Грунт")]
        [Tooltip("Тип грунта")]
        public SoilType soilType = SoilType.Loam;

        [Tooltip("Льдистость грунта (0-1)")]
        [Range(0f, 1f)]
        public float iceContent = 0.7f;

        [Tooltip("Степень насыщения грунта водой (0-1)")]
        [Range(0f, 1f)]
        public float soilSaturationRatio = 0.5f;

        [Tooltip("Глубина залегания мерзлоты (м)")]
        [Range(0.1f, 5f)]
        public float permafrostDepth = 1.5f;

        [Header("Растительность")]
        [Tooltip("Покрытие растительностью (0-1)")]
        [Range(0f, 1f)]
        public float vegetationCover = 0.4f;

        [Header("Сезонность")]
        [Tooltip("Продолжительность теплого сезона (дни)")]
        [Range(60, 180)]
        public int warmSeasonDays = 120;

        /// <summary>
        /// Создать параметры для северной Якутии
        /// </summary>
        public static EnvironmentParams NorthernYakutia()
        {
            return new EnvironmentParams
            {
                airTemp = 3f,
                permafrostTemp = -5f,
                iceContent = 0.85f,
                soilType = SoilType.Peat,
                vegetationCover = 0.6f,
                soilSaturationRatio = 0.7f,
                permafrostDepth = 0.8f,
                warmSeasonDays = 90,
                temperatureAmplitude = 95f
            };
        }

        /// <summary>
        /// Создать параметры для центральной Якутии
        /// </summary>
        public static EnvironmentParams CentralYakutia()
        {
            return new EnvironmentParams
            {
                airTemp = 5f,
                permafrostTemp = -2f,
                iceContent = 0.7f,
                soilType = SoilType.Loam,
                vegetationCover = 0.4f,
                soilSaturationRatio = 0.5f,
                permafrostDepth = 1.5f,
                warmSeasonDays = 120,
                temperatureAmplitude = 88f
            };
        }

        /// <summary>
        /// Создать параметры для южной Якутии
        /// </summary>
        public static EnvironmentParams SouthernYakutia()
        {
            return new EnvironmentParams
            {
                airTemp = 7f,
                permafrostTemp = -1f,
                iceContent = 0.5f,
                soilType = SoilType.Loam,
                vegetationCover = 0.3f,
                soilSaturationRatio = 0.4f,
                permafrostDepth = 2f,
                warmSeasonDays = 140,
                temperatureAmplitude = 75f
            };
        }
    }
}
