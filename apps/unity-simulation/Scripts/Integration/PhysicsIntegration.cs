using UnityEngine;
using ThermokarstSimulation.Core;

namespace ThermokarstSimulation.Integration
{
    /// <summary>
    /// Интеграция с Unity Physics для реалистичной деформации
    /// </summary>
    [RequireComponent(typeof(ThermokarstBehaviour))]
    public class PhysicsIntegration : MonoBehaviour
    {
        [Header("Физика")]
        [SerializeField] private bool enablePhysicsDeformation = true;
        [SerializeField] private LayerMask terrainLayer;
        [SerializeField] private float deformationForce = 10f;

        [Header("Частицы")]
        [SerializeField] private ParticleSystem waterParticles;
        [SerializeField] private ParticleSystem soilParticles;

        private ThermokarstBehaviour thermokarst;
        private Collider[] nearbyColliders = new Collider[10];

        private void Awake()
        {
            thermokarst = GetComponent<ThermokarstBehaviour>();
        }

        private void FixedUpdate()
        {
            if (!enablePhysicsDeformation)
                return;

            ApplySubsidenceForce();
        }

        /// <summary>
        /// Применить силу просадки к окружающим объектам
        /// </summary>
        private void ApplySubsidenceForce()
        {
            ThermokarstLens lens = thermokarst.GetLens();
            float radius = lens.diameter / 2f;

            // Найти объекты в радиусе
            int count = Physics.OverlapSphereNonAlloc(
                transform.position,
                radius,
                nearbyColliders,
                terrainLayer
            );

            for (int i = 0; i < count; i++)
            {
                Rigidbody rb = nearbyColliders[i].GetComponent<Rigidbody>();
                if (rb != null)
                {
                    // Сила направлена к центру и вниз
                    Vector3 direction = (transform.position - rb.position).normalized;
                    direction.y = -1f;

                    float distance = Vector3.Distance(transform.position, rb.position);
                    float forceMagnitude = deformationForce * (1f - distance / radius);

                    rb.AddForce(direction * forceMagnitude, ForceMode.Force);
                }
            }
        }

        /// <summary>
        /// Эффект таяния (частицы воды)
        /// </summary>
        public void PlayMeltingEffect()
        {
            if (waterParticles != null)
                waterParticles.Play();
        }

        /// <summary>
        /// Эффект обрушения грунта
        /// </summary>
        public void PlayCollapseEffect()
        {
            if (soilParticles != null)
                soilParticles.Play();
        }

        private void OnDrawGizmos()
        {
            if (thermokarst == null)
                return;

            ThermokarstLens lens = thermokarst.GetLens();

            // Визуализация зоны влияния
            Gizmos.color = new Color(1f, 0f, 0f, 0.2f);
            Gizmos.DrawWireSphere(transform.position, lens.diameter / 2f);
        }
    }
}
