using UnityEngine;

namespace ThermokarstSimulation.Procedural
{
    /// <summary>
    /// Ячейка террейна с физическими параметрами
    /// </summary>
    [System.Serializable]
    public class TerrainCell
    {
        [Header("Геометрия")]
        public float currentHeight;      // Текущая высота
        public float initialHeight;      // Высота до таяния
        public float waterDepth;         // Глубина воды

        [Header("Физические параметры")]
        public SoilType soilType;
        public float iceContent;         // Льдистость (0-1)
        public float vegetationCover;    // Растительный покров (0-1)
        public float saturationRatio;    // Влажность для Йоханзена (0-1)

        [Header("Состояние таяния")]
        public float cumulativeThawDepth; // Накопленная глубина протаивания ξ_thermo
        public float timeActiveYears;     // Сколько лет тает
        public bool isActive;             // Активна ли ячейка

        public TerrainCell(float height)
        {
            currentHeight = height;
            initialHeight = height;
            waterDepth = 0f;

            soilType = Core.SoilType.Loam;
            iceContent = 0.4f;
            vegetationCover = 0.5f;
            saturationRatio = 0.5f;

            cumulativeThawDepth = 0f;
            timeActiveYears = 0f;
            isActive = false;
        }

        /// <summary>
        /// Просадка грунта при таянии льда
        /// </summary>
        public float CalculateSubsidence(float deltaXi)
        {
            // Просадка = объем растаявшего льда
            return deltaXi * iceContent;
        }

        /// <summary>
        /// Проверка, находится ли ячейка под водой
        /// </summary>
        public bool IsUnderwater(float waterLevel)
        {
            return currentHeight < waterLevel;
        }
    }
}
