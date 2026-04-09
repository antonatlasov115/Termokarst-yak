using UnityEngine;
using ThermokarstSimulation.Core;
using ThermokarstSimulation.Integration;

namespace ThermokarstSimulation.Procedural
{
    /// <summary>
    /// Спавнер термокарстов на процедурном ландшафте
    /// </summary>
    public class ThermokarstSpawner : MonoBehaviour
    {
        [Header("Спавн")]
        [SerializeField] private GameObject thermokarstPrefab;
        [SerializeField] private int maxThermokarstsCount = 50;
        [SerializeField] private float spawnRadius = 100f;

        [Header("Условия формирования")]
        [Tooltip("Минимальная влажность для формирования")]
        [Range(0f, 1f)]
        [SerializeField] private float minMoisture = 0.6f;

        [Tooltip("Максимальная высота для формирования")]
        [Range(0f, 1f)]
        [SerializeField] private float maxHeight = 0.5f;

        [Header("Параметры среды")]
        [SerializeField] private EnvironmentParams baseParams;

        private TerrainGenerator terrain;

        private void Start()
        {
            terrain = GetComponent<TerrainGenerator>();

            if (baseParams == null)
                baseParams = EnvironmentParams.CentralYakutia();

            if (thermokarstPrefab != null)
                SpawnThermokarstsRandomly();
        }

        /// <summary>
        /// Случайный спавн термокарстов
        /// </summary>
        public void SpawnThermokarstsRandomly()
        {
            for (int i = 0; i < maxThermokarstsCount; i++)
            {
                Vector3 randomPos = Random.insideUnitCircle * spawnRadius;
                Vector3 spawnPos = new Vector3(randomPos.x, 0, randomPos.y);

                // Получить высоту ландшафта
                if (terrain != null)
                {
                    float terrainHeight = terrain.GetHeightAt(spawnPos.x, spawnPos.z);
                    spawnPos.y = terrainHeight;
                }

                SpawnThermokarst(spawnPos);
            }
        }

        /// <summary>
        /// Создать термокарст в точке
        /// </summary>
        private void SpawnThermokarst(Vector3 position)
        {
            GameObject thermokarst = Instantiate(thermokarstPrefab, position, Quaternion.identity, transform);

            // Настройка параметров
            var behaviour = thermokarst.GetComponent<ThermokarstBehaviour>();
            if (behaviour != null)
            {
                // Можно варьировать параметры в зависимости от позиции
                // Например, северные термокарсты более льдистые
            }
        }

        /// <summary>
        /// Очистить все термокарсты
        /// </summary>
        public void ClearAllThermokarstsObjects()
        {
            foreach (Transform child in transform)
            {
                Destroy(child.gameObject);
            }
        }

        private void OnDrawGizmosSelected()
        {
            Gizmos.color = Color.cyan;
            Gizmos.DrawWireSphere(transform.position, spawnRadius);
        }
    }
}
