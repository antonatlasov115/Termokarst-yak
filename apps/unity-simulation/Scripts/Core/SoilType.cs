using UnityEngine;

namespace ThermokarstSimulation.Core
{
    /// <summary>
    /// Тип грунта
    /// </summary>
    public enum SoilType
    {
        Clay,   // Глина
        Sand,   // Песок
        Peat,   // Торф
        Loam,   // Суглинок
        Silt    // Ил
    }

    /// <summary>
    /// Расширения для работы с типами грунта
    /// </summary>
    public static class SoilTypeExtensions
    {
        /// <summary>
        /// Теплопроводность сухого грунта (Вт/(м·К))
        /// </summary>
        public static float LambdaDry(this SoilType soilType)
        {
            return soilType switch
            {
                SoilType.Clay => 0.4f,
                SoilType.Sand => 0.3f,
                SoilType.Peat => 0.06f,
                SoilType.Loam => 0.5f,
                SoilType.Silt => 0.5f,
                _ => 0.4f
            };
        }

        /// <summary>
        /// Теплопроводность насыщенного грунта (Вт/(м·К))
        /// </summary>
        public static float LambdaSat(this SoilType soilType)
        {
            return soilType switch
            {
                SoilType.Clay => 1.6f,
                SoilType.Sand => 2.2f,
                SoilType.Peat => 0.5f,
                SoilType.Loam => 1.8f,
                SoilType.Silt => 1.8f,
                _ => 1.6f
            };
        }

        /// <summary>
        /// Теплопроводность талого грунта с учетом влажности по модели Йоханзена (1975)
        /// λₜ(Sr) = λdry + (λsat - λdry) · Sr^0.7
        /// </summary>
        /// <param name="saturationRatio">Степень насыщения (0-1)</param>
        public static float ThermalConductivity(this SoilType soilType, float saturationRatio)
        {
            float lambdaDry = soilType.LambdaDry();
            float lambdaSat = soilType.LambdaSat();
            return lambdaDry + (lambdaSat - lambdaDry) * Mathf.Pow(saturationRatio, 0.7f);
        }

        /// <summary>
        /// Пористость грунта (0-1)
        /// </summary>
        public static float Porosity(this SoilType soilType)
        {
            return soilType switch
            {
                SoilType.Clay => 0.45f,
                SoilType.Sand => 0.35f,
                SoilType.Peat => 0.80f,
                SoilType.Loam => 0.50f,
                SoilType.Silt => 0.55f,
                _ => 0.45f
            };
        }

        /// <summary>
        /// Коэффициент сжимаемости при оттаивании
        /// </summary>
        public static float CompressionCoefficient(this SoilType soilType)
        {
            return soilType switch
            {
                SoilType.Clay => 0.15f,
                SoilType.Sand => 0.05f,
                SoilType.Peat => 0.40f,
                SoilType.Loam => 0.20f,
                SoilType.Silt => 0.25f,
                _ => 0.15f
            };
        }
    }
}
