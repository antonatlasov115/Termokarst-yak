import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { MapPin, Play, Info, Settings, Download, ArrowRight, ArrowLeft } from 'lucide-react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';
import { LocationPicker } from './LocationPicker';
import { ThermokarstMap } from './ThermokarstMap';
import './index.css';

type SimulationMode = 'forward' | 'inverse';
type AppStep = 'location' | 'simulation';

interface SimulationParams {
  region: string;
  years: number;
  temperature: number;
  ice_content: number;
  vegetation: number;
  latitude: number;
  longitude: number;
}

interface InverseParams {
  current_depth: number;
  current_diameter: number;
  observation_year: number;
  latitude: number;
  longitude: number;
  ice_content: number;
  vegetation: number;
}

interface SimulationResult {
  year: number;
  depth: number;
  diameter: number;
  volume: number;
  stability: number;
}

interface InverseResult {
  estimated_age: number;
  start_year: number;
  confidence: number;
  results: SimulationResult[];
}

function App() {
  const [step, setStep] = useState<AppStep>('location');
  const [mode, setMode] = useState<SimulationMode>('forward');
  const [coordinates, setCoordinates] = useState({ lat: 62.5, lon: 129.3 });
  const [measuredDiameter, setMeasuredDiameter] = useState<number | null>(null);
  const [showInfo, setShowInfo] = useState(false);
  const [isRunning, setIsRunning] = useState(false);
  const [results, setResults] = useState<SimulationResult[]>([]);
  const [inverseResult, setInverseResult] = useState<InverseResult | null>(null);
  const [currentYear, setCurrentYear] = useState(0);

  const [params, setParams] = useState<SimulationParams>({
    region: 'central',
    years: 50,
    temperature: 2.5,
    ice_content: 0.4,
    vegetation: 0.6,
    latitude: 62.5,
    longitude: 129.3,
  });

  const [inverseParams, setInverseParams] = useState<InverseParams>({
    current_depth: 3.5,
    current_diameter: 15.0,
    observation_year: 2025,
    latitude: 62.5,
    longitude: 129.3,
    ice_content: 0.4,
    vegetation: 0.6,
  });

  const handleLocationSelected = (lat: number, lon: number, diameter?: number) => {
    setCoordinates({ lat, lon });
    setParams({ ...params, latitude: lat, longitude: lon });
    setInverseParams({ ...inverseParams, latitude: lat, longitude: lon });
    
    if (diameter) {
      setMeasuredDiameter(diameter);
      setInverseParams({ ...inverseParams, current_diameter: diameter, latitude: lat, longitude: lon });
    }
    setStep('simulation');
  };

  const runForwardSimulation = async () => {
    try {
      setIsRunning(true);
      const result = await invoke<SimulationResult[]>('run_forward_simulation', { params });
      setResults(result);
      setCurrentYear(params.years);
      setInverseResult(null);
    } catch (error) {
      console.error('Ошибка симуляции:', error);
      alert('Ошибка: ' + error);
    } finally {
      setIsRunning(false);
    }
  };

  const runInverseSimulation = async () => {
    try {
      setIsRunning(true);
      const result = await invoke<InverseResult>('run_inverse_simulation', { params: inverseParams });
      setResults(result.results);
      setCurrentYear(inverseParams.observation_year);
      setInverseResult(result);
    } catch (error) {
      console.error('Ошибка симуляции:', error);
      alert('Ошибка: ' + error);
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
    const blob = new Blob([dataStr], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
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
            <h2>Выберите режим симуляции</h2>
            <div style={{ display: 'flex', gap: '1rem', marginTop: '1rem' }}>
              <button
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

          <LocationPicker mode={mode} onLocationSelected={handleLocationSelected} />
        </div>
      </div>
    );
  }

  return (
    <div className="app">
      <header className="header">
        <div className="header-content">
          <h1>🏔️ Термокарст Якутии</h1>
          <p>Симуляция термокарстовых образований v0.3.0</p>
        </div>
        <div style={{ display: 'flex', gap: '0.5rem' }}>
          <button
            className="info-btn"
            onClick={() => {
              setStep('location');
              setResults([]);
              setInverseResult(null);
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
          <p>Desktop приложение для симуляции термокарстовых образований с Rust бэкендом.</p>
          <ul>
            <li>✅ Прямая симуляция - прогноз развития</li>
            <li>✅ Обратная симуляция - определение возраста</li>
            <li>✅ Интерактивная карта</li>
            <li>✅ Newton solver интеграция</li>
          </ul>
        </div>
      )}

      <div className="container">
        <div className="sidebar">
          <div className="panel">
            <div style={{ display: 'flex', gap: '0.5rem', marginBottom: '1.5rem' }}>
              <button
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
                }}
              >
                Прямая
              </button>
              <button
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
                }}
              >
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
                
                <div className="form-group">
                  <label>Регион</label>
                  <select
                    value={params.region}
                    onChange={(e) => setParams({...params, region: e.target.value})}
                  >
                    <option value="north">Северная Якутия</option>
                    <option value="central">Центральная Якутия</option>
                    <option value="south">Южная Якутия</option>
                  </select>
                </div>

                <div className="form-group">
                  <label>Температура (+°C): {params.temperature}</label>
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
                  <label>Льдистость: {(params.ice_content * 100).toFixed(0)}%</label>
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.05"
                    value={params.ice_content}
                    onChange={(e) => setParams({...params, ice_content: parseFloat(e.target.value)})}
                  />
                </div>

                <div className="form-group">
                  <label>Растительность: {(params.vegetation * 100).toFixed(0)}%</label>
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
                  {isRunning ? 'Симуляция...' : 'Запустить'}
                </button>
              </>
            ) : (
              <>
                <h2><Settings size={20} /> Обратная симуляция</h2>

                <div className="form-group">
                  <label>Глубина (м): {inverseParams.current_depth}</label>
                  <input
                    type="range"
                    min="0.5"
                    max="10"
                    step="0.1"
                    value={inverseParams.current_depth}
                    onChange={(e) => setInverseParams({...inverseParams, current_depth: parseFloat(e.target.value)})}
                  />
                </div>

                <div className="form-group">
                  <label>Диаметр (м): {measuredDiameter?.toFixed(1) || inverseParams.current_diameter}</label>
                  <input
                    type="range"
                    min="2"
                    max="50"
                    step="0.5"
                    value={measuredDiameter || inverseParams.current_diameter}
                    onChange={(e) => {
                      const val = parseFloat(e.target.value);
                      setInverseParams({...inverseParams, current_diameter: val});
                      if (measuredDiameter) setMeasuredDiameter(val);
                    }}
                  />
                </div>

                <div className="form-group">
                  <label>Год наблюдения: {inverseParams.observation_year}</label>
                  <input
                    type="range"
                    min="2000"
                    max="2026"
                    step="1"
                    value={inverseParams.observation_year}
                    onChange={(e) => setInverseParams({...inverseParams, observation_year: parseInt(e.target.value)})}
                  />
                </div>

                <button
                  className="btn btn-primary"
                  onClick={runInverseSimulation}
                  disabled={isRunning}
                >
                  <Play size={20} />
                  {isRunning ? 'Анализ...' : 'Запустить'}
                </button>
              </>
            )}

            {results.length > 0 && (
              <button className="btn btn-secondary" onClick={exportResults}>
                <Download size={20} />
                Экспорт
              </button>
            )}
          </div>

          {inverseResult && (
            <div className="panel results-summary">
              <h3>🔄 Результаты</h3>
              <div className="stat">
                <span className="stat-label">Возраст:</span>
                <span className="stat-value">{inverseResult.estimated_age} лет</span>
              </div>
              <div className="stat">
                <span className="stat-label">Год начала:</span>
                <span className="stat-value">~{inverseResult.start_year}</span>
              </div>
              <div className="stat">
                <span className="stat-label">Уверенность:</span>
                <span className="stat-value">{(inverseResult.confidence * 100).toFixed(0)}%</span>
              </div>
            </div>
          )}

          {mode === 'forward' && finalResult && (
            <div className="panel results-summary">
              <h3>📊 Финал (год {params.years})</h3>
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
            </div>
          )}
        </div>

        <div className="main-content">
          {results.length === 0 ? (
            <div className="empty-state">
              <h2>👈 Запустите симуляцию</h2>
              <p>Выберите параметры и нажмите кнопку запуска</p>
            </div>
          ) : (
            <>
              <div className="panel" style={{ height: '500px' }}>
                <h2>🗺️ Карта роста</h2>
                <div style={{ marginBottom: '1rem' }}>
                  <label>Год: {currentYear}</label>
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
                    <XAxis dataKey="year" />
                    <YAxis />
                    <Tooltip />
                    <Legend />
                    <Line type="monotone" dataKey="depth" stroke="#8884d8" strokeWidth={2} name="Глубина (м)" />
                  </LineChart>
                </ResponsiveContainer>
              </div>

              <div className="panel">
                <h2>Диаметр</h2>
                <ResponsiveContainer width="100%" height={250}>
                  <LineChart data={results}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="year" />
                    <YAxis />
                    <Tooltip />
                    <Legend />
                    <Line type="monotone" dataKey="diameter" stroke="#82ca9d" strokeWidth={2} name="Диаметр (м)" />
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
