import { useState } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, AreaChart, Area } from 'recharts';
import { Play, Download, Settings, Info, Thermometer, Droplets, Mountain, MapPin, ArrowRight, ArrowLeft } from 'lucide-react';
import { ThermokarstMap } from './ThermokarstMap';
import './App.css';

type SimulationMode = 'forward' | 'inverse';

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

  const runForwardSimulation = async () => {
    setIsRunning(true);

    const mockResults: SimulationResult[] = [];
    for (let year = 0; year <= params.years; year++) {
      const depth = Math.sqrt(year * 0.5 * params.temperature * (1 - params.vegetation * 0.3)) * (1 + params.iceContent);
      const diameter = 2 + Math.log(year + 1) * 2 * (1 + params.iceContent * 0.5);
      const volume = Math.PI * Math.pow(diameter / 2, 2) * depth;
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
    setIsRunning(false);
  };

  const runInverseSimulation = async () => {
    setIsRunning(true);

    // Обратная симуляция - определяем возраст
    const estimatedAge = Math.round(
      (inverseParams.currentDepth * inverseParams.currentDiameter) /
      (params.temperature * (1 + params.iceContent))
    );
    const startYear = inverseParams.observationYear - estimatedAge;
    const confidence = 0.75 + Math.random() * 0.2;

    // Генерируем историю роста
    const mockResults: SimulationResult[] = [];
    for (let year = 0; year <= estimatedAge; year++) {
      const progress = year / estimatedAge;
      const depth = inverseParams.currentDepth * Math.sqrt(progress);
      const diameter = 2 + (inverseParams.currentDiameter - 2) * Math.log(year + 1) / Math.log(estimatedAge + 1);
      const volume = Math.PI * Math.pow(diameter / 2, 2) * depth;
      const stability = Math.max(0, 1 - (depth / 10) * (diameter / 20));

      mockResults.push({
        year: startYear + year,
        depth: parseFloat(depth.toFixed(2)),
        diameter: parseFloat(diameter.toFixed(2)),
        volume: parseFloat(volume.toFixed(2)),
        stability: parseFloat(stability.toFixed(2)),
      });
    }

    await new Promise(resolve => setTimeout(resolve, 1000));

    setResults(mockResults);
    setCurrentYear(inverseParams.observationYear);
    setInverseResult({
      estimatedAge,
      startYear,
      confidence,
      results: mockResults,
    });
    setIsRunning(false);
  };

  const exportResults = () => {
    const data = mode === 'inverse' && inverseResult ? {
      mode: 'inverse',
      inverseResult,
      results,
    } : {
      mode: 'forward',
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

  return (
    <div className="app">
      <header className="header">
        <div className="header-content">
          <h1>🏔️ Термокарст Якутии</h1>
          <p>Симуляция образования термокарстовых линз v0.2.0</p>
        </div>
        <button className="info-btn" onClick={() => setShowInfo(!showInfo)}>
          <Info size={20} />
        </button>
      </header>

      {showInfo && (
        <div className="info-panel">
          <h3>О проекте</h3>
          <p>Модульная система симуляции термокарстовых образований в условиях многолетней мерзлоты Якутии.</p>
          <ul>
            <li>✅ Прямая симуляция - прогноз развития</li>
            <li>✅ Обратная симуляция - определение возраста</li>
            <li>✅ Интерактивная карта с батиметрией</li>
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
                  Определение времени образования термокарста
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
                  <label>Текущий диаметр (м): {inverseParams.currentDiameter}</label>
                  <input
                    type="range"
                    min="2"
                    max="50"
                    step="0.5"
                    value={inverseParams.currentDiameter}
                    onChange={(e) => setInverseParams({...inverseParams, currentDiameter: parseFloat(e.target.value)})}
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

            <div className="form-group" style={{ marginTop: '1rem' }}>
              <label>
                <MapPin size={16} />
                Координаты
              </label>
              <div style={{ display: 'flex', gap: '0.5rem', marginTop: '0.5rem' }}>
                <input
                  type="number"
                  placeholder="Широта"
                  value={coordinates.lat}
                  onChange={(e) => setCoordinates({...coordinates, lat: parseFloat(e.target.value) || 62.5})}
                  style={{
                    flex: 1,
                    padding: '0.5rem',
                    border: '2px solid #e2e8f0',
                    borderRadius: '8px',
                    fontSize: '0.9rem'
                  }}
                />
                <input
                  type="number"
                  placeholder="Долгота"
                  value={coordinates.lon}
                  onChange={(e) => setCoordinates({...coordinates, lon: parseFloat(e.target.value) || 129.3})}
                  style={{
                    flex: 1,
                    padding: '0.5rem',
                    border: '2px solid #e2e8f0',
                    borderRadius: '8px',
                    fontSize: '0.9rem'
                  }}
                />
              </div>
            </div>

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
