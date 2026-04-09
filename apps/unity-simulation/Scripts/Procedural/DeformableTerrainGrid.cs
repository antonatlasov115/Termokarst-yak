using UnityEngine;
using ThermokarstSimulation.Core;

namespace ThermokarstSimulation.Procedural
{
    /// <summary>
    /// Деформируемый террейн с клеточным автоматом
    /// </summary>
    [RequireComponent(typeof(MeshFilter), typeof(MeshRenderer))]
    public class DeformableTerrainGrid : MonoBehaviour
    {
        [Header("Размеры")]
        [SerializeField] private int gridWidth = 128;
        [SerializeField] private int gridHeight = 128;
        [SerializeField] private float cellSize = 1f;
        [SerializeField] private float heightScale = 20f;

        [Header("Генерация")]
        [SerializeField] private float noiseScale = 50f;
        [SerializeField] private float voronoiCellSize = 15f;
        [SerializeField] private int seed = 0;

        [Header("Симуляция")]
        [SerializeField] private EnvironmentParams climateParams;
        [SerializeField] private bool autoSimulate = true;
        [SerializeField] private float simulationSpeed = 1f; // лет в секунду
        [SerializeField] private float waterLevel = 5f;

        [Header("Эрозия")]
        [SerializeField] private bool enableLateralErosion = true;
        [SerializeField] private float erosionRate = 0.1f;
        [SerializeField] private float thermalBonusRange = 2f;

        private TerrainCell[,] grid;
        private MeshFilter meshFilter;
        private MeshRenderer meshRenderer;
        private float simulationTime;

        private void Awake()
        {
            meshFilter = GetComponent<MeshFilter>();
            meshRenderer = GetComponent<MeshRenderer>();

            if (seed == 0)
                seed = Random.Range(0, 100000);

            if (climateParams == null)
                climateParams = EnvironmentParams.CentralYakutia();
        }

        private void Start()
        {
            InitializeGrid();
            UpdateMesh();
        }

        private void Update()
        {
            if (!autoSimulate)
                return;

            simulationTime += Time.deltaTime * simulationSpeed;

            if (simulationTime >= 1f)
            {
                float years = Mathf.Floor(simulationTime);
                simulationTime -= years;

                SimulateYears(years);
            }
        }

        /// <summary>
        /// Инициализация сетки террейна
        /// </summary>
        private void InitializeGrid()
        {
            grid = new TerrainCell[gridWidth, gridHeight];

            // Генерация базовой высоты
            float[,] heightMap = NoiseGenerator.GenerateHeightMap(
                gridWidth, gridHeight, noiseScale, 4, 0.5f, 2f, seed
            );

            // Генерация карты льдистости (Вороной + Перлин)
            float[,] iceMap = VoronoiNoise.GenerateRealisticIceMap(
                gridWidth, gridHeight, voronoiCellSize, seed
            );

            // Генерация растительности
            float[,] vegetationMap = NoiseGenerator.GenerateMoistureMap(
                gridWidth, gridHeight, 40f, seed + 100
            );

            // Заполнение ячеек
            for (int y = 0; y < gridHeight; y++)
            {
                for (int x = 0; x < gridWidth; x++)
                {
                    float height = heightMap[x, y] * heightScale;
                    TerrainCell cell = new TerrainCell(height);

                    cell.iceContent = iceMap[x, y];
                    cell.vegetationCover = vegetationMap[x, y];
                    cell.saturationRatio = 0.5f;
                    cell.soilType = SoilType.Loam;

                    grid[x, y] = cell;
                }
            }
        }

        /// <summary>
        /// Симуляция N лет
        /// </summary>
        public void SimulateYears(float years)
        {
            // Шаг 1: Вертикальное протаивание
            SimulateVerticalThaw(years);

            // Шаг 2: Горизонтальная эрозия берегов
            if (enableLateralErosion)
                SimulateLateralErosion(years);

            // Шаг 3: Обновление воды
            UpdateWaterDepth();

            // Шаг 4: Обновление меша
            UpdateMesh();
        }

        /// <summary>
        /// Вертикальное протаивание (формула Атласова v0.3.0)
        /// </summary>
        private void SimulateVerticalThaw(float dtYears)
        {
            for (int y = 0; y < gridHeight; y++)
            {
                for (int x = 0; x < gridWidth; x++)
                {
                    TerrainCell cell = grid[x, y];

                    // Условия начала термокарста
                    bool shouldActivate = cell.vegetationCover < 0.3f || cell.waterDepth > 0f;

                    if (shouldActivate || cell.isActive)
                    {
                        cell.isActive = true;
                        cell.timeActiveYears += dtYears;

                        // Расчет сезонного ALT
                        float xiAlt = CalculateXiAlt(cell);

                        // Кумулятивное таяние: ξ_thermo = ξ_ALT * √t
                        float newXiThermo = xiAlt * Mathf.Sqrt(cell.timeActiveYears);
                        float deltaXi = newXiThermo - cell.cumulativeThawDepth;
                        cell.cumulativeThawDepth = newXiThermo;

                        // Просадка грунта
                        float subsidence = cell.CalculateSubsidence(deltaXi);
                        cell.currentHeight -= subsidence;
                    }
                }
            }
        }

        /// <summary>
        /// Расчет сезонного ALT по формуле Атласова v0.3.0
        /// </summary>
        private float CalculateXiAlt(TerrainCell cell)
        {
            // Теплопроводность по Йоханзену
            float lambda = cell.soilType.ThermalConductivity(cell.saturationRatio);

            // DDT в секундах
            float ddtSeconds = climateParams.airTemp * climateParams.warmSeasonDays * 86400f;

            const float LATENT_HEAT = 334000f;
            const float WATER_DENSITY = 1000f;
            float iceContent = Mathf.Max(cell.iceContent, 0.01f);

            // Факторы
            float vegFactor = Mathf.Exp(0.30f * (1f - cell.vegetationCover));
            float tempFactor = 1f + 0.12f * Mathf.Log(climateParams.temperatureAmplitude / 40f);

            // ALT
            float innerSqrt = (2f * lambda * ddtSeconds) /
                             (LATENT_HEAT * WATER_DENSITY * Mathf.Pow(iceContent, 0.7f));

            return Mathf.Sqrt(innerSqrt) * vegFactor * tempFactor;
        }

        /// <summary>
        /// Горизонтальная эрозия берегов (клеточный автомат)
        /// </summary>
        private void SimulateLateralErosion(float dtYears)
        {
            TerrainCell[,] newGrid = (TerrainCell[,])grid.Clone();

            for (int y = 1; y < gridHeight - 1; y++)
            {
                for (int x = 1; x < gridWidth - 1; x++)
                {
                    TerrainCell cell = grid[x, y];

                    // Если ячейка под водой, она нагревает соседей
                    if (cell.IsUnderwater(waterLevel))
                    {
                        // Проверяем соседей
                        for (int dy = -1; dy <= 1; dy++)
                        {
                            for (int dx = -1; dx <= 1; dx++)
                            {
                                if (dx == 0 && dy == 0) continue;

                                int nx = x + dx;
                                int ny = y + dy;

                                TerrainCell neighbor = grid[nx, ny];

                                // Если сосед на берегу (не под водой)
                                if (!neighbor.IsUnderwater(waterLevel))
                                {
                                    // Тепловой бонус от воды
                                    float distance = Mathf.Sqrt(dx * dx + dy * dy);
                                    float thermalBonus = erosionRate * dtYears / distance;

                                    // Ускоряем таяние соседа
                                    newGrid[nx, ny].timeActiveYears += thermalBonus;
                                    newGrid[nx, ny].isActive = true;

                                    // Эрозия крутых склонов
                                    float heightDiff = neighbor.currentHeight - cell.currentHeight;
                                    if (heightDiff > 2f)
                                    {
                                        float slopeErosion = heightDiff * 0.05f * dtYears;
                                        newGrid[nx, ny].currentHeight -= slopeErosion;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            grid = newGrid;
        }

        /// <summary>
        /// Обновление глубины воды
        /// </summary>
        private void UpdateWaterDepth()
        {
            for (int y = 0; y < gridHeight; y++)
            {
                for (int x = 0; x < gridWidth; x++)
                {
                    TerrainCell cell = grid[x, y];

                    if (cell.currentHeight < waterLevel)
                    {
                        cell.waterDepth = waterLevel - cell.currentHeight;
                    }
                    else
                    {
                        cell.waterDepth = 0f;
                    }
                }
            }
        }

        /// <summary>
        /// Обновление меша террейна
        /// </summary>
        private void UpdateMesh()
        {
            Mesh mesh = new Mesh();
            mesh.name = "Deformable Terrain";

            Vector3[] vertices = new Vector3[gridWidth * gridHeight];
            int[] triangles = new int[(gridWidth - 1) * (gridHeight - 1) * 6];
            Vector2[] uvs = new Vector2[gridWidth * gridHeight];
            Color[] colors = new Color[gridWidth * gridHeight];

            int vertIndex = 0;
            int triIndex = 0;

            for (int y = 0; y < gridHeight; y++)
            {
                for (int x = 0; x < gridWidth; x++)
                {
                    TerrainCell cell = grid[x, y];

                    vertices[vertIndex] = new Vector3(x * cellSize, cell.currentHeight, y * cellSize);
                    uvs[vertIndex] = new Vector2((float)x / gridWidth, (float)y / gridHeight);

                    // Цвет по состоянию
                    if (cell.waterDepth > 0f)
                        colors[vertIndex] = Color.blue; // Вода
                    else if (cell.isActive)
                        colors[vertIndex] = Color.Lerp(Color.yellow, Color.red, cell.cumulativeThawDepth / 5f); // Таяние
                    else
                        colors[vertIndex] = Color.green; // Нормальный грунт

                    // Треугольники
                    if (x < gridWidth - 1 && y < gridHeight - 1)
                    {
                        triangles[triIndex] = vertIndex;
                        triangles[triIndex + 1] = vertIndex + gridWidth;
                        triangles[triIndex + 2] = vertIndex + gridWidth + 1;

                        triangles[triIndex + 3] = vertIndex;
                        triangles[triIndex + 4] = vertIndex + gridWidth + 1;
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

            meshFilter.mesh = mesh;
        }

        /// <summary>
        /// Получить ячейку по координатам
        /// </summary>
        public TerrainCell GetCell(int x, int y)
        {
            if (x < 0 || x >= gridWidth || y < 0 || y >= gridHeight)
                return null;

            return grid[x, y];
        }

        private void OnValidate()
        {
            if (gridWidth < 2) gridWidth = 2;
            if (gridHeight < 2) gridHeight = 2;
        }
    }
}
