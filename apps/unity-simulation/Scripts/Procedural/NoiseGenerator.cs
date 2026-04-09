using UnityEngine;

namespace ThermokarstSimulation.Procedural
{
    /// <summary>
    /// Генератор шума Перлина для процедурной генерации
    /// </summary>
    public static class NoiseGenerator
    {
        /// <summary>
        /// Многослойный шум Перлина (Fractal Brownian Motion)
        /// </summary>
        public static float FractalBrownianMotion(float x, float y, int octaves, float persistence, float lacunarity, float scale)
        {
            float total = 0f;
            float frequency = 1f;
            float amplitude = 1f;
            float maxValue = 0f;

            for (int i = 0; i < octaves; i++)
            {
                total += Mathf.PerlinNoise(x * frequency * scale, y * frequency * scale) * amplitude;

                maxValue += amplitude;
                amplitude *= persistence;
                frequency *= lacunarity;
            }

            return total / maxValue;
        }

        /// <summary>
        /// Генерация карты высот
        /// </summary>
        public static float[,] GenerateHeightMap(int width, int height, float scale, int octaves, float persistence, float lacunarity, int seed)
        {
            float[,] heightMap = new float[width, height];

            System.Random prng = new System.Random(seed);
            Vector2[] octaveOffsets = new Vector2[octaves];

            for (int i = 0; i < octaves; i++)
            {
                float offsetX = prng.Next(-100000, 100000);
                float offsetY = prng.Next(-100000, 100000);
                octaveOffsets[i] = new Vector2(offsetX, offsetY);
            }

            float maxNoiseHeight = float.MinValue;
            float minNoiseHeight = float.MaxValue;

            float halfWidth = width / 2f;
            float halfHeight = height / 2f;

            for (int y = 0; y < height; y++)
            {
                for (int x = 0; x < width; x++)
                {
                    float amplitude = 1f;
                    float frequency = 1f;
                    float noiseHeight = 0f;

                    for (int i = 0; i < octaves; i++)
                    {
                        float sampleX = (x - halfWidth) / scale * frequency + octaveOffsets[i].x;
                        float sampleY = (y - halfHeight) / scale * frequency + octaveOffsets[i].y;

                        float perlinValue = Mathf.PerlinNoise(sampleX, sampleY) * 2f - 1f;
                        noiseHeight += perlinValue * amplitude;

                        amplitude *= persistence;
                        frequency *= lacunarity;
                    }

                    if (noiseHeight > maxNoiseHeight)
                        maxNoiseHeight = noiseHeight;
                    if (noiseHeight < minNoiseHeight)
                        minNoiseHeight = noiseHeight;

                    heightMap[x, y] = noiseHeight;
                }
            }

            // Нормализация
            for (int y = 0; y < height; y++)
            {
                for (int x = 0; x < width; x++)
                {
                    heightMap[x, y] = Mathf.InverseLerp(minNoiseHeight, maxNoiseHeight, heightMap[x, y]);
                }
            }

            return heightMap;
        }

        /// <summary>
        /// Генерация карты влажности (для определения мест термокарста)
        /// </summary>
        public static float[,] GenerateMoistureMap(int width, int height, float scale, int seed)
        {
            float[,] moistureMap = new float[width, height];
            System.Random prng = new System.Random(seed + 1000);

            float offsetX = prng.Next(-100000, 100000);
            float offsetY = prng.Next(-100000, 100000);

            for (int y = 0; y < height; y++)
            {
                for (int x = 0; x < width; x++)
                {
                    float sampleX = x / scale + offsetX;
                    float sampleY = y / scale + offsetY;

                    moistureMap[x, y] = Mathf.PerlinNoise(sampleX, sampleY);
                }
            }

            return moistureMap;
        }
    }
}
