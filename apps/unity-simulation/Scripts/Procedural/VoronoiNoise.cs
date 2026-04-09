using UnityEngine;

namespace ThermokarstSimulation.Procedural
{
    /// <summary>
    /// Генератор шума Вороного для ледяных жил
    /// </summary>
    public static class VoronoiNoise
    {
        /// <summary>
        /// Генерация карты ледяных жил (полигональная структура)
        /// </summary>
        /// <param name="width">Ширина карты</param>
        /// <param name="height">Высота карты</param>
        /// <param name="cellSize">Размер полигональных ячеек</param>
        /// <param name="seed">Seed для генерации</param>
        /// <returns>Карта льдистости (0-1)</returns>
        public static float[,] GenerateIceVeinsMap(int width, int height, float cellSize, int seed)
        {
            float[,] iceMap = new float[width, height];

            // Генерация точек Вороного (центры полигонов)
            System.Random prng = new System.Random(seed);
            int numPoints = Mathf.CeilToInt((width * height) / (cellSize * cellSize));
            Vector2[] voronoiPoints = new Vector2[numPoints];

            for (int i = 0; i < numPoints; i++)
            {
                voronoiPoints[i] = new Vector2(
                    (float)prng.NextDouble() * width,
                    (float)prng.NextDouble() * height
                );
            }

            // Для каждой ячейки найти расстояние до ближайшей и второй ближайшей точки
            for (int y = 0; y < height; y++)
            {
                for (int x = 0; x < width; x++)
                {
                    Vector2 pos = new Vector2(x, y);

                    float minDist1 = float.MaxValue;
                    float minDist2 = float.MaxValue;

                    // Найти две ближайшие точки
                    foreach (Vector2 point in voronoiPoints)
                    {
                        float dist = Vector2.Distance(pos, point);

                        if (dist < minDist1)
                        {
                            minDist2 = minDist1;
                            minDist1 = dist;
                        }
                        else if (dist < minDist2)
                        {
                            minDist2 = dist;
                        }
                    }

                    // Расстояние до границы полигона
                    float edgeDistance = minDist2 - minDist1;

                    // Ледяные жилы находятся на границах (малое edgeDistance)
                    // Центры полигонов имеют меньше льда
                    float iceValue;
                    if (edgeDistance < cellSize * 0.1f)
                    {
                        // Граница - много льда (ледяная жила)
                        iceValue = Mathf.Lerp(0.8f, 0.95f, (float)prng.NextDouble());
                    }
                    else
                    {
                        // Центр полигона - мало льда
                        float t = Mathf.Clamp01(edgeDistance / (cellSize * 0.5f));
                        iceValue = Mathf.Lerp(0.6f, 0.3f, t);
                    }

                    iceMap[x, y] = iceValue;
                }
            }

            return iceMap;
        }

        /// <summary>
        /// Комбинированная карта: Вороной + шум Перлина
        /// </summary>
        public static float[,] GenerateRealisticIceMap(int width, int height, float cellSize, int seed)
        {
            float[,] voronoiMap = GenerateIceVeinsMap(width, height, cellSize, seed);
            float[,] perlinMap = NoiseGenerator.GenerateMoistureMap(width, height, 30f, seed);

            float[,] combinedMap = new float[width, height];

            for (int y = 0; y < height; y++)
            {
                for (int x = 0; x < width; x++)
                {
                    // Комбинируем: структура от Вороного, вариация от Перлина
                    float voronoi = voronoiMap[x, y];
                    float perlin = perlinMap[x, y];

                    // 70% Вороной (структура), 30% Перлин (вариация)
                    combinedMap[x, y] = voronoi * 0.7f + perlin * 0.3f;
                }
            }

            return combinedMap;
        }
    }
}
