import { useEffect, useRef, useState } from 'react';
import L from 'leaflet';
import 'leaflet/dist/leaflet.css';
import { MapPin, Ruler, Circle } from 'lucide-react';

interface LocationPickerProps {
  mode: 'forward' | 'inverse';
  onLocationSelected: (lat: number, lon: number, diameter?: number) => void;
}

export function LocationPicker({ mode, onLocationSelected }: LocationPickerProps) {
  const mapRef = useRef<L.Map | null>(null);
  const [selectedLocation, setSelectedLocation] = useState<{ lat: number; lon: number } | null>(null);
  const [measuringMode, setMeasuringMode] = useState(false);
  const [measuredDiameter, setMeasuredDiameter] = useState<number | null>(null);
  const circleRef = useRef<L.Circle | null>(null);
  const markerRef = useRef<L.Marker | null>(null);

  useEffect(() => {
    if (!mapRef.current) {
      const map = L.map('location-map').setView([62.5, 129.3], 6);

      const satellite = L.tileLayer('https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}', {
        attribution: 'Tiles © Esri',
        maxZoom: 19,
      }).addTo(map);

      const street = L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
        attribution: '© OpenStreetMap',
        maxZoom: 19,
      });

      const baseMaps = {
        'Спутник': satellite,
        'Карта': street,
      };

      L.control.layers(baseMaps).addTo(map);

      map.on('click', (e: L.LeafletMouseEvent) => {
        if (measuringMode) return;

        const { lat, lng } = e.latlng;
        setSelectedLocation({ lat, lon: lng });

        if (markerRef.current) {
          markerRef.current.remove();
        }

        const marker = L.marker([lat, lng], {
          icon: L.divIcon({
            className: 'location-marker',
            html: '<div style="width: 30px; height: 30px; background: #667eea; border: 4px solid white; border-radius: 50%; box-shadow: 0 2px 8px rgba(0,0,0,0.3);"></div>',
            iconSize: [30, 30],
            iconAnchor: [15, 15],
          }),
        }).addTo(map);

        marker.bindPopup(`
          <div style="font-family: sans-serif; text-align: center;">
            <h3 style="margin: 0 0 8px 0; font-size: 14px;">📍 Выбранная точка</h3>
            <p style="margin: 4px 0; font-size: 12px;">${lat.toFixed(4)}°N, ${lng.toFixed(4)}°E</p>
          </div>
        `).openPopup();

        markerRef.current = marker;

        if (circleRef.current) {
          circleRef.current.remove();
          circleRef.current = null;
          setMeasuredDiameter(null);
        }
      });

      mapRef.current = map;
    }
  }, [measuringMode]);

  const startMeasuring = () => {
    if (!selectedLocation) {
      alert('Сначала выберите точку на карте!');
      return;
    }

    setMeasuringMode(true);
    const map = mapRef.current!;

    if (circleRef.current) {
      circleRef.current.remove();
    }

    const circle = L.circle([selectedLocation.lat, selectedLocation.lon], {
      radius: 10,
      color: '#48bb78',
      fillColor: '#48bb78',
      fillOpacity: 0.3,
      weight: 3,
    }).addTo(map);

    circleRef.current = circle;

    const RadiusControl = L.Control.extend({
      onAdd: function() {
        const div = L.DomUtil.create('div', 'leaflet-bar leaflet-control');
        div.style.background = 'white';
        div.style.padding = '10px';
        div.style.borderRadius = '8px';
        div.style.boxShadow = '0 2px 8px rgba(0,0,0,0.2)';

        div.innerHTML = `
          <div style="font-family: sans-serif;">
            <div style="font-weight: bold; margin-bottom: 8px;">Измерение водоема</div>
            <div style="margin-bottom: 8px;">
              <label style="display: block; font-size: 12px; margin-bottom: 4px;">Диаметр: <span id="diameter-value">20</span> м</label>
              <input type="range" id="diameter-slider" min="2" max="100" value="10" step="0.5" style="width: 200px;" />
            </div>
            <button id="confirm-measurement" style="
              width: 100%;
              padding: 8px;
              background: #48bb78;
              color: white;
              border: none;
              border-radius: 4px;
              cursor: pointer;
              font-weight: 600;
            ">✓ Подтвердить</button>
            <button id="cancel-measurement" style="
              width: 100%;
              padding: 8px;
              background: #e53e3e;
              color: white;
              border: none;
              border-radius: 4px;
              cursor: pointer;
              font-weight: 600;
              margin-top: 4px;
            ">✗ Отмена</button>
          </div>
        `;

        const slider = div.querySelector('#diameter-slider') as HTMLInputElement;
        const valueSpan = div.querySelector('#diameter-value') as HTMLSpanElement;
        const confirmBtn = div.querySelector('#confirm-measurement') as HTMLButtonElement;
        const cancelBtn = div.querySelector('#cancel-measurement') as HTMLButtonElement;

        L.DomEvent.disableClickPropagation(div);
        L.DomEvent.disableScrollPropagation(div);

        slider.addEventListener('input', (e) => {
          const radius = parseFloat((e.target as HTMLInputElement).value);
          const diameter = radius * 2;
          valueSpan.textContent = diameter.toFixed(1);
          circle.setRadius(radius);
        });

        confirmBtn.addEventListener('click', () => {
          const radius = parseFloat(slider.value);
          const diameter = radius * 2;
          setMeasuredDiameter(diameter);
          map.removeControl(radiusControl);
          setTimeout(() => setMeasuringMode(false), 100);
        });

        cancelBtn.addEventListener('click', () => {
          if (circleRef.current) {
            circleRef.current.remove();
            circleRef.current = null;
          }
          setMeasuredDiameter(null);
          map.removeControl(radiusControl);
          setTimeout(() => setMeasuringMode(false), 100);
        });

        return div;
      }
    });

    const radiusControl = new RadiusControl({ position: 'topright' });
    radiusControl.addTo(map);
  };

  const handleConfirm = () => {
    if (!selectedLocation) {
      alert('Выберите точку на карте!');
      return;
    }

    if (mode === 'inverse' && !measuredDiameter) {
      const proceed = window.confirm('Вы не измерили водоем. Продолжить с параметрами по умолчанию?');
      if (!proceed) return;
    }

    onLocationSelected(selectedLocation.lat, selectedLocation.lon, measuredDiameter || undefined);
  };

  return (
    <div className="panel" style={{ height: 'calc(100vh - 250px)' }}>
      <h2><MapPin size={20} /> Выбор местоположения</h2>

      <div style={{ marginBottom: '1rem' }}>
        {mode === 'forward' ? (
          <p style={{ fontSize: '0.9rem', color: '#4a5568' }}>
            Кликните на карту чтобы выбрать точку для прогноза развития термокарста.
          </p>
        ) : (
          <p style={{ fontSize: '0.9rem', color: '#4a5568' }}>
            1. Кликните на карту чтобы выбрать центр водоема<br/>
            2. Используйте инструмент измерения чтобы обвести водоем<br/>
            3. Подтвердите выбор
          </p>
        )}
      </div>

      {selectedLocation && (
        <div style={{
          padding: '1rem',
          background: '#f0f9ff',
          borderRadius: '8px',
          marginBottom: '1rem',
        }}>
          <div style={{ fontWeight: 600, marginBottom: '0.5rem' }}>✓ Точка выбрана:</div>
          <div style={{ fontSize: '0.9rem' }}>
            {selectedLocation.lat.toFixed(4)}°N, {selectedLocation.lon.toFixed(4)}°E
          </div>
          {measuredDiameter && (
            <div style={{ marginTop: '0.5rem', color: '#48bb78', fontWeight: 600 }}>
              ✓ Диаметр измерен: {measuredDiameter.toFixed(1)} м
            </div>
          )}
        </div>
      )}

      <div style={{ display: 'flex', gap: '0.5rem', marginBottom: '1rem' }}>
        {mode === 'inverse' && selectedLocation && !measuringMode && (
          <button
            onClick={startMeasuring}
            style={{
              flex: 1,
              padding: '0.75rem',
              background: '#48bb78',
              color: 'white',
              border: 'none',
              borderRadius: '8px',
              cursor: 'pointer',
              fontWeight: 600,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              gap: '0.5rem',
            }}
          >
            <Ruler size={16} />
            Измерить водоем
          </button>
        )}

        {selectedLocation && (
          <button
            onClick={handleConfirm}
            disabled={measuringMode}
            style={{
              flex: 1,
              padding: '0.75rem',
              background: measuringMode ? '#cbd5e0' : '#667eea',
              color: 'white',
              border: 'none',
              borderRadius: '8px',
              cursor: measuringMode ? 'not-allowed' : 'pointer',
              fontWeight: 600,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              gap: '0.5rem',
            }}
          >
            <Circle size={16} />
            Подтвердить
          </button>
        )}
      </div>

      <div id="location-map" style={{ height: 'calc(100% - 200px)', borderRadius: '12px' }}></div>
    </div>
  );
}
