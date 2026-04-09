using UnityEngine;
using ThermokarstSimulation.Core;

namespace ThermokarstSimulation.Physics
{
    /// <summary>
    /// Расчет теплопереноса в грунте
    /// </summary>
    public class HeatTransfer
    {
        private readonly EnvironmentParams parameters;

        public HeatTransfer(EnvironmentParams parameters)
        {
            this.parameters = parameters;
        }

        /// <summary>
        /// Расчет теплового потока (Вт/м²)
        /// </summary>
        public float CalculateHeatFlux()
        {
            float k = parameters.soilType.ThermalConductivity(parameters.soilSaturationRatio);
            float tempGradient = (parameters.airTemp - parameters.permafrostTemp) / parameters.permafrostDepth;
            return k * tempGradient;
        }

        /// <summary>
        /// Расчет температуры на глубине (°C)
        /// </summary>
        public float TemperatureAtDepth(float depth)
        {
            if (depth <= 0f)
                return parameters.airTemp;

            if (depth >= parameters.permafrostDepth)
                return parameters.permafrostTemp;

            // Линейная интерполяция
            float ratio = depth / parameters.permafrostDepth;
            return Mathf.Lerp(parameters.airTemp, parameters.permafrostTemp, ratio);
        }

        /// <summary>
        /// Расчет энергии, необходимой для таяния льда (МДж/м³)
        /// </summary>
        public float LatentHeatRequired(float volume)
        {
            const float LATENT_HEAT_ICE = 334f; // МДж/м³
            return volume * parameters.iceContent * LATENT_HEAT_ICE;
        }

        /// <summary>
        /// Эффективная теплоемкость грунта (МДж/(м³·К))
        /// </summary>
        public float EffectiveHeatCapacity()
        {
            const float HEAT_CAPACITY_SOIL = 2f;
            const float HEAT_CAPACITY_ICE = 1.9f;
            const float HEAT_CAPACITY_WATER = 4.2f;

            float porosity = parameters.soilType.Porosity();
            float soilFraction = 1f - porosity;
            float iceFraction = porosity * parameters.iceContent;
            float waterFraction = porosity * (1f - parameters.iceContent);

            return soilFraction * HEAT_CAPACITY_SOIL
                + iceFraction * HEAT_CAPACITY_ICE
                + waterFraction * HEAT_CAPACITY_WATER;
        }
    }
}
