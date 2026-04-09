using UnityEngine;
using ThermokarstSimulation.Core;
using ThermokarstSimulation.Simulation;

namespace ThermokarstSimulation.Integration
{
    /// <summary>
    /// Unity компонент для симуляции термокарста
    /// </summary>
    [RequireComponent(typeof(MeshFilter), typeof(MeshRenderer))]
    public class ThermokarstBehaviour : MonoBehaviour
    {
        [Header("Параметры среды")]
        [SerializeField] private EnvironmentParams environmentParams;

        [Header("Начальные условия")]
        [SerializeField] private float initialDepth = 0.5f;
        [SerializeField] private float initialDiameter = 2f;

        [Header("Симуляция")]
        [SerializeField] private bool autoSimulate = true;
        [SerializeField] private float timeScale = 1f; // 1 секунда = 1 год
        [SerializeField] private bool visualizeDepth = true;

        [Header("Визуализация")]
        [SerializeField] private Material thermokarstMaterial;
        [SerializeField] private Color shallowColor = new Color(0.6f, 0.4f, 0.2f);
        [SerializeField] private Color deepColor = new Color(0.2f, 0.3f, 0.5f);

        private ThermokarstLens lens;
        private ThermokarstEngine engine;
        private MeshFilter meshFilter;
        private MeshRenderer meshRenderer;
        private float simulationTime;

        private void Awake()
        {
            meshFilter = GetComponent<MeshFilter>();
            meshRenderer = GetComponent<MeshRenderer>();

            if (environmentParams == null)
                environmentParams = EnvironmentParams.CentralYakutia();

            lens = new ThermokarstLens(initialDepth, initialDiameter);
            engine = new ThermokarstEngine(environmentParams);

            if (thermokarstMaterial != null)
                meshRenderer.material = thermokarstMaterial;
        }

        private void Start()
        {
            UpdateMesh();
            UpdateVisualization();
        }

        private void Update()
        {
            if (!autoSimulate)
                return;

            simulationTime += Time.deltaTime * timeScale;

            if (simulationTime >= 1f)
            {
                simulationTime -= 1f;
                SimulateYear();
            }
        }

        /// <summary>
        /// Симулировать один год развития
        /// </summary>
        public void SimulateYear()
        {
            engine.SimulateYear(lens);
            UpdateMesh();
            UpdateVisualization();

            Debug.Log($"Thermokarst Age: {lens.age} years, Depth: {lens.depth:F2}m, Diameter: {lens.diameter:F2}m");
        }

        /// <summary>
        /// Обновить меш термокарста
        /// </summary>
        private void UpdateMesh()
        {
            Mesh mesh = GenerateThermokarstMesh(lens.diameter, lens.depth);
            meshFilter.mesh = mesh;
        }

        /// <summary>
        /// Генерация меша термокарста (цилиндрическая депрессия)
        /// </summary>
        private Mesh GenerateThermokarstMesh(float diameter, float depth)
        {
            Mesh mesh = new Mesh();
            mesh.name = "Thermokarst";

            int segments = 32;
            float radius = diameter / 2f;

            Vector3[] vertices = new Vector3[segments + 2];
            int[] triangles = new int[segments * 3];
            Vector2[] uvs = new Vector2[segments + 2];

            // Центральная точка (дно)
            vertices[0] = new Vector3(0, -depth, 0);
            uvs[0] = new Vector2(0.5f, 0.5f);

            // Край термокарста
            for (int i = 0; i <= segments; i++)
            {
                float angle = (float)i / segments * Mathf.PI * 2f;
                float x = Mathf.Cos(angle) * radius;
                float z = Mathf.Sin(angle) * radius;

                vertices[i + 1] = new Vector3(x, 0, z);
                uvs[i + 1] = new Vector2(
                    0.5f + Mathf.Cos(angle) * 0.5f,
                    0.5f + Mathf.Sin(angle) * 0.5f
                );
            }

            // Треугольники
            for (int i = 0; i < segments; i++)
            {
                triangles[i * 3] = 0;
                triangles[i * 3 + 1] = i + 1;
                triangles[i * 3 + 2] = i + 2;
            }

            mesh.vertices = vertices;
            mesh.triangles = triangles;
            mesh.uv = uvs;
            mesh.RecalculateNormals();
            mesh.RecalculateBounds();

            return mesh;
        }

        /// <summary>
        /// Обновить визуализацию (цвет по глубине)
        /// </summary>
        private void UpdateVisualization()
        {
            if (!visualizeDepth || meshRenderer.material == null)
                return;

            float normalizedDepth = Mathf.Clamp01(lens.depth / 10f);
            Color color = Color.Lerp(shallowColor, deepColor, normalizedDepth);
            meshRenderer.material.color = color;
        }

        /// <summary>
        /// Получить текущее состояние линзы
        /// </summary>
        public ThermokarstLens GetLens() => lens;

        /// <summary>
        /// Сбросить симуляцию
        /// </summary>
        public void Reset()
        {
            lens = new ThermokarstLens(initialDepth, initialDiameter);
            simulationTime = 0f;
            UpdateMesh();
            UpdateVisualization();
        }

        private void OnDrawGizmos()
        {
            if (lens == null)
                return;

            // Визуализация границ термокарста
            Gizmos.color = Color.cyan;
            Gizmos.DrawWireSphere(transform.position, lens.diameter / 2f);

            // Визуализация глубины
            Gizmos.color = Color.red;
            Gizmos.DrawLine(
                transform.position,
                transform.position + Vector3.down * lens.depth
            );
        }
    }
}
