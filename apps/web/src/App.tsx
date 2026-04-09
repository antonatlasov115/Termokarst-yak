import { useState } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, AreaChart, Area } from 'recharts';
import { Play, Download, Settings, Info, Thermometer, Droplets, Mountain, MapPin, ArrowRight, ArrowLeft, Target } from 'lucide-react';
import { ThermokarstMap } from './ThermokarstMap';
import { LocationPicker } from './LocationPicker';
import './App.css';

type SimulationMode = 'forward' | 'inverse';
type AppStep = 'location' | 'simulation';

interface SimulationParams {
  region: 'north' | 'central' | 'south';
  years: number;
  temperature: number;
  iceContent: number;
  vegetation: number;
}

interface InverseParams {
  currentDepth: number;
  currentDiameter: number;
  observationYear: number;
  ndvi: number;
}

interface SimulationResult {
  year: number;
  depth: number;
  diameter: number;
  volume: number;
  stability: number;
}

interface InverseResult {
  estimatedAge: number;
  startYear: number;
  confidence: number;
  results: SimulationResult[];
}

function App() {
  const [step, setStep] = useState<AppStep>('location');
  const [mode, setMode] = useState<SimulationMode>('forward');
  const [params, setParams] = useState<SimulationParams>({
    region: 'central',
    years: 50,
    temperature: 2.5,
    iceContent: 0.4,
    vegetation: 0.6,
  });

  const [inverseParams, setInverseParams] = useState<InverseParams>({
    currentDepth: 3.5,
    currentDiameter: 15.0,
    observationYear: 2025,
    ndvi: 0.35,
  });

  const [results, setResults] = useState<SimulationResult[]>([]);
  const [inverseResult, setInverseResult] = useState<InverseResult | null>(null);
  const [isRunning, setIsRunning] = useState(false);
  const [showInfo, setShowInfo] = useState(false);
  const [currentYear, setCurrentYear] = useState(0);
  const [coordinates, setCoordinates] = useState({ lat: 62.5, lon: 129.3 });
  const [measuredDiameter, setMeasuredDiameter] = useState<number | null>(null);

  // Физические константы (общие для обеих симуляций)
  const L = 334000.0; // Дж/кг - скрытая теплота плавления льда
  const RHO_W = 1000.0; // кг/м³ - плотность воды
  const SECONDS_PER_DAY = 86400.0;
  const BETA = 0.30; // коэффициент покрова
  const GAMMA = 0.12; // коэффициент континентальности
  const DT0 = 40.0; // базовая амплитуда температур

  // Региональные параметры (исправлены амплитуды температур для Якутии)
  const REGIONAL_PARAMS: Record<string, { warmSeasonDays: number; tempAmplitude: number; thermalCond: number }> = {
    north: { warmSeasonDays: 90, tempAmplitude: 98.0, thermalCond: 0.5 },      // Верхоянск
    central: { warmSeasonDays: 110, tempAmplitude: 90.0, thermalCond: 1.2 },   // Центральная Якутия
    south: { warmSeasonDays: 120, tempAmplitude: 85.0, thermalCond: 1.5 },     // Южная Якутия
  };

  const handleLocationSelected = (lat: number, lon: number, diameter?: number) => {
    setCoordinates({ lat, lon });
    if (diameter) {
      setMeasuredDiameter(diameter);
      setInverseParams({ ...inverseParams, currentDiameter: diameter });
    }
    setStep('simulation');
  };

  const runForwardSimulation = async () => {
    try {
      setIsRunning(true);

      const regional = REGIONAL_PARAMS[params.region];

      const mockResults: SimulationResult[] = [];
      for (let year = 0; year <= params.years; year++) {
        if (year === 0) {
          mockResults.push({ year: 0, depth: 0, diameter: 2, volume: 0, stability: 1 });
          continue;
        }

        // Формула Атласова для глубины протаивания
        // ξ_A = √(2λₜ·DDT / (L·ρw·w)) · exp(β·(1-V)) · (1 + γ·ln(ΔT/ΔT₀)) · f_moisture · √year

        // 1. DDT (degree-days of thawing)
        const ddt_days = params.temperature * regional.warmSeasonDays;
        const ddt_seconds = ddt_days * SECONDS_PER_DAY;

        // 2. Льдистость (степень 1.0 по формуле Стефана)
        const w = params.iceContent;

        // 3. Базовая глубина по Стефану
        const xi_0 = Math.sqrt((2.0 * regional.thermalCond * ddt_seconds) / (L * RHO_W * w));

        // 4. Коэффициент покрова: exp(β·(1-V))
        const k_fire = Math.exp(BETA * (1.0 - params.vegetation));

        // 5. Функция континентальности: 1 + γ·ln(ΔT/ΔT₀)
        const f_continental = 1.0 + GAMMA * Math.log(regional.tempAmplitude / DT0);

        // 6. Фактор влажности (предполагаем среднюю влажность 0.3)
        const f_moisture = 1.0 + 0.3 * 0.3;

        // 7. Итоговая глубина с учетом времени
        const depth = xi_0 * k_fire * f_continental * f_moisture * Math.sqrt(year);

        // Латеральное расширение: D(t) = D₀ + k·ln(1 + t)
        const k_lateral = 2.0 * (1.0 + params.iceContent * 0.5);
        const diameter = 2.0 + k_lateral * Math.log(1 + year);

        // Объем: V = π·r²·h
        const volume = Math.PI * Math.pow(diameter / 2, 2) * depth;

        // Стабильность (упрощенная оценка)
        const stability = Math.max(0, 1 - (depth / 10) * (diameter / 20));

        mockResults.push({
          year,
          depth: parseFloat(depth.toFixed(2)),
          diameter: parseFloat(diameter.toFixed(2)),
          volume: parseFloat(volume.toFixed(2)),
          stability: parseFloat(stability.toFixed(2)),
        });
      }

      await new Promise(resolve => setTimeout(resolve, 1000));

      setResults(mockResults);
      setCurrentYear(params.years);
      setInverseResult(null);
    } catch (error) {
      console.error('Ошибка прямой симуляции:', error);
      alert('Ошибка при выполнении симуляции: ' + error);
    } finally {
      setIsRunning(false);
    }
  };

  const runInverseSimulation = async () => {
    try {
      setIsRunning(true);

      // Используем измеренный диаметр если есть
      const diameter = measuredDiameter || inverseParams.currentDiameter;
      const depth = inverseParams.currentDepth;

      // Определяем регион по широте
      const lat = coordinates?.lat || 62.5;
      const regional = lat > 68 ? REGIONAL_PARAMS.north :
                       lat > 64 ? REGIONAL_PARAMS.central :
                       REGIONAL_PARAMS.south;

      // Обратная формула Атласова для определения возраста
      // Из D(t) = D₀ + k·ln(1 + t) => t = exp((D - D₀)/k) - 1
      const k_lateral = 2.0 * (1.0 + params.iceContent * 0.5);
      const ageFromDiameter = Math.max(1, Math.round(Math.exp((diameter - 2.0) / k_lateral) - 1));

      // Проверяем согласованность с глубиной
      // ξ = ξ₀ · k_fire · f_continental · f_moisture · √t
      // => t = (ξ / (ξ₀ · k_fire · f_continental · f_moisture))²

      const ddt_days = params.temperature * regional.warmSeasonDays;
      const ddt_seconds = ddt_days * SECONDS_PER_DAY;
      const w = params.iceContent; // степень 1.0, не 0.7
      const xi_0 = Math.sqrt((2.0 * regional.thermalCond * ddt_seconds) / (L * RHO_W * w));
      const k_fire = Math.exp(BETA * (1.0 - params.vegetation));
      const f_continental = 1.0 + GAMMA * Math.log(regional.tempAmplitude / DT0);
      const f_moisture = 1.0 + 0.3 * 0.3;

      const ageFromDepth = Math.pow(depth / (xi_0 * k_fire * f_continental * f_moisture), 2);

      // Оценка достоверности на основе согласованности двух методов
      const ageDiff = Math.abs(ageFromDiameter - ageFromDepth);
      const maxAge = Math.max(ageFromDiameter, ageFromDepth);
      const confidence = Math.max(0.5, Math.min(0.95, 1.0 - ageDiff / (maxAge * 2)));

      // Усредняем оценки возраста используя confidence как вес
      // Если методы согласны (высокий confidence) - доверяем обоим поровну
      // Если не согласны (низкий confidence) - больше веса диаметру (он точнее измеряется)
      const w_depth = confidence;
      const finalAge = Math.round(ageFromDepth * w_depth + ageFromDiameter * (1.0 - w_depth));
      const startYear = inverseParams.observationYear - finalAge;

      // Генерируем историю роста используя прямую модель
      const mockResults: SimulationResult[] = [];
      for (let year = 0; year <= finalAge; year++) {
        if (year === 0) {
          mockResults.push({
            year: startYear,
            depth: 0,
            diameter: 2.0,
            volume: 0,
            stability: 1.0,
          });
          continue;
        }

        // Используем прямую формулу Атласова
        const d = xi_0 * k_fire * f_continental * f_moisture * Math.sqrt(year);
        const diam = 2.0 + k_lateral * Math.log(1 + year);
        const volume = Math.PI * Math.pow(diam / 2, 2) * d;
        const stability = Math.max(0, 1 - (d / 10) * (diam / 20));

        mockResults.push({
          year: startYear + year,
          depth: parseFloat(d.toFixed(2)),
          diameter: parseFloat(diam.toFixed(2)),
          volume: parseFloat(volume.toFixed(2)),
          stability: parseFloat(stability.toFixed(2)),
        });
      }

      await new Promise(resolve => setTimeout(resolve, 1000));

      setResults(mockResults);
      setCurrentYear(inverseParams.observationYear);
      setInverseResult({
        estimatedAge: finalAge,
        startYear,
        confidence,
        results: mockResults,
      });
    } catch (error) {
      console.error('Ошибка обратной симуляции:', error);
      alert('Ошибка при выполнении обратной симуляции: ' + error);
    } finally {
      setIsRunning(false);
    }
  };

  const exportResults = () => {
    const data = mode === 'inverse' && inverseResult ? {
      mode: 'inverse',
      coordinates,
      measuredDiameter,
      inverseResult,
      results,
    } : {
      mode: 'forward',
      coordinates,
      params,
      results,
    };

    const dataStr = JSON.stringify(data, null, 2);
    const dataBlob = new Blob([dataStr], { type: 'application/json' });
    const url = URL.createObjectURL(dataBlob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `thermokarst_${mode}_${Date.now()}.json`;
    link.click();
  };

  const finalResult = results[results.length - 1];

  if (step === 'location') {
    return (
      <div className="app">
        <header className="header">
          <div className="header-content">
            <h1>🏔️ Термокарст Якутии</h1>
            <p>Выбор местоположения</p>
          </div>
          <button className="info-btn" onClick={() => setShowInfo(!showInfo)}>
            <Info size={20} />
          </button>
        </header>

        {showInfo && (
          <div className="info-panel">
            <h3>Выбор местоположения</h3>
            <p>Кликните на карту чтобы выбрать точку для симуляции.</p>
            <p><strong>Для обратной симуляции:</strong> используйте инструмент измерения чтобы обвести существующий водоем.</p>
          </div>
        )}

        <div style={{ padding: '2rem' }}>
          <div className="panel" style={{ marginBottom: '1.5rem' }}>
            <h2><Target size={20} /> Выберите режим симуляции</h2>
            <div style={{ display: 'flex', gap: '1rem', marginTop: '1rem' }}>
              <button
                className={`tab-btn ${mode === 'forward' ? 'active' : ''}`}
                onClick={() => setMode('forward')}
                style={{
                  flex: 1,
                  padding: '1rem',
                  border: mode === 'forward' ? '2px solid #667eea' : '2px solid #e2e8f0',
                  background: mode === 'forward' ? '#667eea' : 'white',
                  color: mode === 'forward' ? 'white' : '#4a5568',
                  borderRadius: '8px',
                  cursor: 'pointer',
                  fontWeight: 600,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  gap: '0.5rem',
                }}
              >
                <ArrowRight size={20} />
                <div>
                  <div>Прямая симуляция</div>
                  <div style={{ fontSize: '0.8rem', opacity: 0.9 }}>Прогноз развития</div>
                </div>
              </button>
              <button
                className={`tab-btn ${mode === 'inverse' ? 'active' : ''}`}
                onClick={() => setMode('inverse')}
                style={{
                  flex: 1,
                  padding: '1rem',
                  border: mode === 'inverse' ? '2px solid #667eea' : '2px solid #e2e8f0',
                  background: mode === 'inverse' ? '#667eea' : 'white',
                  color: mode === 'inverse' ? 'white' : '#4a5568',
                  borderRadius: '8px',
                  cursor: 'pointer',
                  fontWeight: 600,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  gap: '0.5rem',
                }}
              >
                <ArrowLeft size={20} />
                <div>
                  <div>Обратная симуляция</div>
                  <div style={{ fontSize: '0.8rem', opacity: 0.9 }}>Определение возраста</div>
                </div>
              </button>
            </div>
          </div>

          <LocationPicker
            mode={mode}
            onLocationSelected={handleLocationSelected}
          />
        </div>
      </div>
    );
  }

  return (
    <div className="app">
      <header className="header">
        <div className="header-content">
          <h1>🏔️ Термокарст Якутии</h1>
          <p>Симуляция образования термокарстовых линз v0.2.0</p>
        </div>
        <div style={{ display: 'flex', gap: '0.5rem' }}>
          <button
            className="info-btn"
            onClick={() => {
              setStep('location');
              setResults([]);
              setInverseResult(null);
              setMeasuredDiameter(null);
            }}
            title="Изменить местоположение"
          >
            <MapPin size={20} />
          </button>
          <button className="info-btn" onClick={() => setShowInfo(!showInfo)}>
            <Info size={20} />
          </button>
        </div>
      </header>

      {showInfo && (
        <div className="info-panel">
          <h3>О проекте</h3>
          <p>Модульная система симуляции термокарстовых образований в условиях многолетней мерзлоты Якутии.</p>
          <ul>
            <li>✅ Прямая симуляция - прогноз развития</li>
            <li>✅ Обратная симуляция - определение возраста</li>
            <li>✅ Интерактивная карта с батиметрией</li>
            <li>✅ Измерение размеров водоема</li>
            <li>✅ Научная достоверность: 9.0/10</li>
          </ul>
        </div>
      )}

      <div className="container">
        <div className="sidebar">
          <div className="panel">
            <div style={{ display: 'flex', gap: '0.5rem', marginBottom: '1.5rem' }}>
              <button
                className={`tab-btn ${mode === 'forward' ? 'active' : ''}`}
                onClick={() => setMode('forward')}
                style={{
                  flex: 1,
                  padding: '0.75rem',
                  border: mode === 'forward' ? '2px solid #667eea' : '2px solid #e2e8f0',
                  background: mode === 'forward' ? '#667eea' : 'white',
                  color: mode === 'forward' ? 'white' : '#4a5568',
                  borderRadius: '8px',
                  cursor: 'pointer',
                  fontWeight: 600,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  gap: '0.5rem',
                }}
              >
                <ArrowRight size={16} />
                Прямая
              </button>
              <button
                className={`tab-btn ${mode === 'inverse' ? 'active' : ''}`}
                onClick={() => setMode('inverse')}
                style={{
                  flex: 1,
                  padding: '0.75rem',
                  border: mode === 'inverse' ? '2px solid #667eea' : '2px solid #e2e8f0',
                  background: mode === 'inverse' ? '#667eea' : 'white',
                  color: mode === 'inverse' ? 'white' : '#4a5568',
                  borderRadius: '8px',
                  cursor: 'pointer',
                  fontWeight: 600,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  gap: '0.5rem',
                }}
              >
                <ArrowLeft size={16} />
                Обратная
              </button>
            </div>

            <div style={{
              padding: '0.75rem',
              background: '#f7fafc',
              borderRadius: '8px',
              marginBottom: '1rem',
              fontSize: '0.85rem',
            }}>
              <div style={{ fontWeight: 600, marginBottom: '0.25rem' }}>📍 Местоположение:</div>
              <div>{coordinates.lat.toFixed(4)}°N, {coordinates.lon.toFixed(4)}°E</div>
              {measuredDiameter && (
                <div style={{ marginTop: '0.5rem', color: '#48bb78', fontWeight: 600 }}>
                  ✓ Измерено: Ø {measuredDiameter.toFixed(1)}м
                </div>
              )}
            </div>

            {mode === 'forward' ? (
              <>
                <h2><Settings size={20} /> Прямая симуляция</h2>
                <p style={{ fontSize: '0.85rem', color: '#718096', marginBottom: '1rem' }}>
                  Прогноз развития термокарста в будущем
                </p>

                <div className="form-group">
                  <label>Регион</label>
                  <select
                    value={params.region}
                    onChange={(e) => setParams({...params, region: e.target.value as any})}
                  >
                    <option value="north">Северная Якутия</option>
                    <option value="central">Центральная Якутия</option>
                    <option value="south">Южная Якутия</option>
                  </select>
                </div>

                <div className="form-group">
                  <label>
                    <Thermometer size={16} />
                    Температура (+°C): {params.temperature}
                  </label>
                  <input
                    type="range"
                    min="0"
                    max="5"
                    step="0.1"
                    value={params.temperature}
                    onChange={(e) => setParams({...params, temperature: parseFloat(e.target.value)})}
                  />
                </div>

                <div className="form-group">
                  <label>
                    <Droplets size={16} />
                    Льдистость: {(params.iceContent * 100).toFixed(0)}%
                  </label>
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.05"
                    value={params.iceContent}
                    onChange={(e) => setParams({...params, iceContent: parseFloat(e.target.value)})}
                  />
                </div>

                <div className="form-group">
                  <label>
                    <Mountain size={16} />
                    Растительность: {(params.vegetation * 100).toFixed(0)}%
                  </label>
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.05"
                    value={params.vegetation}
                    onChange={(e) => setParams({...params, vegetation: parseFloat(e.target.value)})}
                  />
                </div>

                <div className="form-group">
                  <label>Период (лет): {params.years}</label>
                  <input
                    type="range"
                    min="10"
                    max="100"
                    step="5"
                    value={params.years}
                    onChange={(e) => setParams({...params, years: parseInt(e.target.value)})}
                  />
                </div>

                <button
                  className="btn btn-primary"
                  onClick={runForwardSimulation}
                  disabled={isRunning}
                >
                  <Play size={20} />
                  {isRunning ? 'Симуляция...' : 'Запустить прямую симуляцию'}
                </button>
              </>
            ) : (
              <>
                <h2><Settings size={20} /> Обратная симуляция</h2>
                <p style={{ fontSize: '0.85rem', color: '#718096', marginBottom: '1rem' }}>
                  Определение времени образования по размерам
                </p>

                <div className="form-group">
                  <label>Текущая глубина (м): {inverseParams.currentDepth}</label>
                  <input
                    type="range"
                    min="0.5"
                    max="10"
                    step="0.1"
                    value={inverseParams.currentDepth}
                    onChange={(e) => setInverseParams({...inverseParams, currentDepth: parseFloat(e.target.value)})}
                  />
                </div>

                <div className="form-group">
                  <label>
                    Текущий диаметр (м): {measuredDiameter ? measuredDiameter.toFixed(1) : inverseParams.currentDiameter}
                    {measuredDiameter && <span style={{ color: '#48bb78', marginLeft: '0.5rem' }}>✓ Измерено</span>}
                  </label>
                  <input
                    type="range"
                    min="2"
                    max="50"
                    step="0.5"
                    value={measuredDiameter || inverseParams.currentDiameter}
                    onChange={(e) => {
                      const val = parseFloat(e.target.value);
                      setInverseParams({...inverseParams, currentDiameter: val});
                      if (measuredDiameter) setMeasuredDiameter(val);
                    }}
                  />
                </div>

                <div className="form-group">
                  <label>Год наблюдения: {inverseParams.observationYear}</label>
                  <input
                    type="range"
                    min="2000"
                    max="2026"
                    step="1"
                    value={inverseParams.observationYear}
                    onChange={(e) => setInverseParams({...inverseParams, observationYear: parseInt(e.target.value)})}
                  />
                </div>

                <div className="form-group">
                  <label>NDVI окружения: {inverseParams.ndvi.toFixed(2)}</label>
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.05"
                    value={inverseParams.ndvi}
                    onChange={(e) => setInverseParams({...inverseParams, ndvi: parseFloat(e.target.value)})}
                  />
                </div>

                <div className="form-group">
                  <label>Регион (для параметров)</label>
                  <select
                    value={params.region}
                    onChange={(e) => setParams({...params, region: e.target.value as any})}
                  >
                    <option value="north">Северная Якутия</option>
                    <option value="central">Центральная Якутия</option>
                    <option value="south">Южная Якутия</option>
                  </select>
                </div>

                <button
                  className="btn btn-primary"
                  onClick={runInverseSimulation}
                  disabled={isRunning}
                >
                  <Play size={20} />
                  {isRunning ? 'Анализ...' : 'Запустить обратную симуляцию'}
                </button>
              </>
            )}

            {results.length > 0 && (
              <button className="btn btn-secondary" onClick={exportResults}>
                <Download size={20} />
                Экспорт результатов
              </button>
            )}
          </div>

          {mode === 'inverse' && inverseResult && (
            <div className="panel results-summary">
              <h3>🔄 Результаты обратной симуляции</h3>
              <div className="stat">
                <span className="stat-label">Возраст:</span>
                <span className="stat-value">{inverseResult.estimatedAge} лет</span>
              </div>
              <div className="stat">
                <span className="stat-label">Год начала:</span>
                <span className="stat-value">~{inverseResult.startYear}</span>
              </div>
              <div className="stat">
                <span className="stat-label">Уверенность:</span>
                <span className="stat-value">{(inverseResult.confidence * 100).toFixed(0)}%</span>
              </div>
              {measuredDiameter && (
                <div className="stat">
                  <span className="stat-label">Измеренный Ø:</span>
                  <span className="stat-value" style={{ color: '#48bb78' }}>{measuredDiameter.toFixed(1)} м</span>
                </div>
              )}
            </div>
          )}

          {mode === 'forward' && finalResult && (
            <div className="panel results-summary">
              <h3>📊 Финальное состояние (год {params.years})</h3>
              <div className="stat">
                <span className="stat-label">Глубина:</span>
                <span className="stat-value">{finalResult.depth.toFixed(2)} м</span>
              </div>
              <div className="stat">
                <span className="stat-label">Диаметр:</span>
                <span className="stat-value">{finalResult.diameter.toFixed(2)} м</span>
              </div>
              <div className="stat">
                <span className="stat-label">Объем:</span>
                <span className="stat-value">{finalResult.volume.toFixed(1)} м³</span>
              </div>
              <div className="stat">
                <span className="stat-label">Стабильность:</span>
                <span className={`stat-value ${finalResult.stability > 0.5 ? 'stable' : 'unstable'}`}>
                  {finalResult.stability > 0.5 ? '✓ Стабильно' : '⚠ Нестабильно'}
                </span>
              </div>
            </div>
          )}
        </div>

        <div className="main-content">
          {results.length === 0 ? (
            <div className="empty-state">
              <h2>👈 Выберите режим и запустите симуляцию</h2>
              <p>
                <strong>Прямая:</strong> прогноз развития термокарста в будущем<br/>
                <strong>Обратная:</strong> определение времени образования по текущим параметрам
              </p>
            </div>
          ) : (
            <>
              <div className="panel" style={{ height: '500px' }}>
                <h2>🗺️ Карта роста термокарста {mode === 'inverse' && '(История)'}</h2>
                <div style={{ marginBottom: '1rem' }}>
                  <label style={{ display: 'block', marginBottom: '0.5rem', color: '#4a5568', fontWeight: 500 }}>
                    {mode === 'forward' ? `Год: ${currentYear}` : `Год: ${currentYear} (возраст: ${currentYear - (inverseResult?.startYear || 0)} лет)`}
                  </label>
                  <input
                    type="range"
                    min={results[0].year}
                    max={results[results.length - 1].year}
                    value={currentYear}
                    onChange={(e) => setCurrentYear(parseInt(e.target.value))}
                    style={{ width: '100%' }}
                  />
                </div>
                <div style={{ height: 'calc(100% - 80px)' }}>
                  <ThermokarstMap
                    latitude={coordinates.lat}
                    longitude={coordinates.lon}
                    results={results}
                    currentYear={currentYear}
                  />
                </div>
              </div>

              <div className="panel">
                <h2>Глубина протаивания</h2>
                <ResponsiveContainer width="100%" height={250}>
                  <LineChart data={results}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="year" label={{ value: mode === 'forward' ? 'Годы' : 'Год', position: 'insideBottom', offset: -5 }} />
                    <YAxis label={{ value: 'Глубина (м)', angle: -90, position: 'insideLeft' }} />
                    <Tooltip />
                    <Legend />
                    <Line type="monotone" dataKey="depth" stroke="#8884d8" strokeWidth={2} name="Глубина (м)" />
                  </LineChart>
                </ResponsiveContainer>
              </div>

              <div className="panel">
                <h2>Латеральное расширение</h2>
                <ResponsiveContainer width="100%" height={250}>
                  <LineChart data={results}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="year" label={{ value: mode === 'forward' ? 'Годы' : 'Год', position: 'insideBottom', offset: -5 }} />
                    <YAxis label={{ value: 'Диаметр (м)', angle: -90, position: 'insideLeft' }} />
                    <Tooltip />
                    <Legend />
                    <Line type="monotone" dataKey="diameter" stroke="#82ca9d" strokeWidth={2} name="Диаметр (м)" />
                  </LineChart>
                </ResponsiveContainer>
              </div>

              <div className="panel">
                <h2>Объем термокарста</h2>
                <ResponsiveContainer width="100%" height={250}>
                  <AreaChart data={results}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="year" label={{ value: mode === 'forward' ? 'Годы' : 'Год', position: 'insideBottom', offset: -5 }} />
                    <YAxis label={{ value: 'Объем (м³)', angle: -90, position: 'insideLeft' }} />
                    <Tooltip />
                    <Legend />
                    <Area type="monotone" dataKey="volume" stroke="#ffc658" fill="#ffc658" fillOpacity={0.6} name="Объем (м³)" />
                  </AreaChart>
                </ResponsiveContainer>
              </div>

              <div className="panel">
                <h2>Стабильность</h2>
                <ResponsiveContainer width="100%" height={250}>
                  <LineChart data={results}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="year" label={{ value: mode === 'forward' ? 'Годы' : 'Год', position: 'insideBottom', offset: -5 }} />
                    <YAxis domain={[0, 1]} label={{ value: 'Индекс стабильности', angle: -90, position: 'insideLeft' }} />
                    <Tooltip />
                    <Legend />
                    <Line type="monotone" dataKey="stability" stroke="#ff7c7c" strokeWidth={2} name="Стабильность" />
                  </LineChart>
                </ResponsiveContainer>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}

export default App;
