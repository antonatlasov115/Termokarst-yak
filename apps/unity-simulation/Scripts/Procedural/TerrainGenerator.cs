using UnityEngine;
using ThermokarstSimulation.Core;

namespace ThermokarstSimulation.Procedural
{
    /// <summary>
    /// Процедурная генерация ландшафта Якутии с термокарстом
    /// </summary>
    [RequireComponent(typeof(MeshFilter), typeof(MeshRenderer))]
    public class TerrainGenerator : MonoBehaviour
    {
        [Header("Размеры")]
        [SerializeField] private int width = 256;
        [SerializeField] private int height = 256;
        [SerializeField] private float heightMultiplier = 20f;

        [Header("Шум")]
        [SerializeField] private float noiseScale = 50f;
        [SerializeField] private int octaves = 4;
        [SerializeField] private float persistence = 0.5f;
        [SerializeField] private float lacunarity = 2f;
        [SerializeField] private int seed = 0;

        [Header("Термокарст")]
        [SerializeField] private float thermokarstThreshold = 0.6f;
        [SerializeField] private float thermokarstDensity = 0.3f;
        [SerializeField] private GameObject thermokarstPrefab;

        [Header("Материалы")]
        [SerializeField] private Material terrainMaterial;
        [SerializeField] private Gradient terrainGradient;

        private MeshFilter meshFilter;
        private MeshRenderer meshRenderer;
        private float[,] heightMap;
        private float[,] moistureMap;

        private void Awake()
        {
            meshFilter = GetComponent<MeshFilter>();
            meshRenderer = GetComponent<MeshRenderer>();

            if (seed == 0)
                seed = Random.Range(0, 100000);
        }

        private void Start()
        {
            GenerateTerrain();
        }

        /// <summary>
        /// Генерация ландшафта
        /// </summary>
        public void GenerateTerrain()
        {
            // Генерация карт
            heightMap = NoiseGenerator.GenerateHeightMap(width, height, noiseScale, octaves, persistence, lacunarity, seed);
            moistureMap = NoiseGenerator.GenerateMoistureMap(width, height, noiseScale * 2f, seed);

            // Создание меша
            Mesh mesh = GenerateTerrainMesh();
            meshFilter.mesh = mesh;

            // Применение материала
            if (terrainMaterial != null)
                meshRenderer.material = terrainMaterial;

            // Спавн термокарста
            if (thermokarstPrefab != null)
                SpawnThermokarstsOnTerrain();
        }

        /// <summary>
        /// Генерация меша ландшафта
        /// </summary>
        private Mesh GenerateTerrainMesh()
        {
            Mesh mesh = new Mesh();
            mesh.name = "Procedural Terrain";

            Vector3[] vertices = new Vector3[width * height];
            int[] triangles = new int[(width - 1) * (height - 1) * 6];
            Vector2[] uvs = new Vector2[width * height];
            Color[] colors = new Color[width * height];

            int vertIndex = 0;
            int triIndex = 0;

            for (int y = 0; y < height; y++)
            {
                for (int x = 0; x < width; x++)
                {
                    float heightValue = heightMap[x, y];
                    vertices[vertIndex] = new Vector3(x, heightValue * heightMultiplier, y);
                    uvs[vertIndex] = new Vector2((float)x / width, (float)y / height);

                    // Цвет по высоте
                    if (terrainGradient != null)
                        colors[vertIndex] = terrainGradient.Evaluate(heightValue);
                    else
                        colors[vertIndex] = Color.Lerp(Color.green, Color.white, heightValue);

                    // Треугольники
                    if (x < width - 1 && y < height - 1)
                    {
                        triangles[triIndex] = vertIndex;
                        triangles[triIndex + 1] = vertIndex + width;
                        triangles[triIndex + 2] = vertIndex + width + 1;

                        triangles[triIndex + 3] = vertIndex;
                        triangles[triIndex + 4] = vertIndex + width + 1;
                        triangles[triIndex + 5] = vertIndex + 1;

                        triIndex += 6;
                    }

                    vertIndex++;
                }
            }

            mesh.vertices = vertices;
            mesh.triangles = triangles;
            mesh.uv = uvs;
            mesh.colors = colors;
            mesh.RecalculateNormals();
            mesh.RecalculateBounds();

            return mesh;
        }

        /// <summary>
        /// Спавн термокарстов на ландшафте
        /// </summary>
        private void SpawnThermokarstsOnTerrain()
        {
            System.Random prng = new System.Random(seed);

            for (int y = 0; y < height; y += 10)
            {
                for (int x = 0; x < width; x += 10)
                {
                    float moisture = moistureMap[x, y];
                    float heightValue = heightMap[x, y];

                    // Термокарст формируется в низинах с высокой влажностью
                    if (moisture > thermokarstThreshold && heightValue < 0.5f)
                    {
                        if (prng.NextDouble() < thermokarstDensity)
                        {
                            Vector3 position = new Vector3(x, heightValue * heightMultiplier, y);
                            SpawnThermokarst(position, moisture);
                        }
                    }
                }
            }
        }

        /// <summary>
        /// Создание одного термокарста
        /// </summary>
        private void SpawnThermokarst(Vector3 position, float moisture)
        {
            GameObject thermokarst = Instantiate(thermokarstPrefab, position, Quaternion.identity, transform);

            // Настройка параметров в зависимости от влажности
            var behaviour = thermokarst.GetComponent<ThermokarstSimulation.Integration.ThermokarstBehaviour>();
            if (behaviour != null)
            {
                // Более влажные места = больше льдистость
                // Можно настроить через Inspector или программно
            }
        }

        /// <summary>
        /// Получить высоту в точке
        /// </summary>
        public float GetHeightAt(float x, float z)
        {
            int ix = Mathf.Clamp(Mathf.RoundToInt(x), 0, width - 1);
            int iz = Mathf.Clamp(Mathf.RoundToInt(z), 0, height - 1);
            return heightMap[ix, iz] * heightMultiplier;
        }

        private void OnValidate()
        {
            if (width < 2) width = 2;
            if (height < 2) height = 2;
            if (octaves < 1) octaves = 1;
        }
    }
}
