import { useEffect, useRef } from 'react';
import L from 'leaflet';
import 'leaflet/dist/leaflet.css';

interface ThermokarstMapProps {
  latitude: number;
  longitude: number;
  results: Array<{
    year: number;
    diameter: number;
    depth: number;
  }>;
  currentYear: number;
}

export function ThermokarstMap({ latitude, longitude, results, currentYear }: ThermokarstMapProps) {
  const mapRef = useRef<L.Map | null>(null);
  const circlesRef = useRef<L.Circle[]>([]);

  useEffect(() => {
    if (!mapRef.current) {
      // Инициализация карты
      const map = L.map('map').setView([latitude, longitude], 15);

      // Добавляем слой OpenStreetMap
      L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
        attribution: '© OpenStreetMap contributors',
        maxZoom: 19,
      }).addTo(map);

      // Добавляем спутниковый слой (опционально)
      const satellite = L.tileLayer('https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}', {
        attribution: 'Tiles © Esri',
        maxZoom: 19,
      });

      // Контроль слоев
      const baseMaps = {
        'Карта': L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
          attribution: '© OpenStreetMap contributors',
        }),
        'Спутник': satellite,
      };

      L.control.layers(baseMaps).addTo(map);

      mapRef.current = map;
    }

    // Очищаем старые круги
    circlesRef.current.forEach(circle => circle.remove());
    circlesRef.current = [];

    if (results.length === 0) return;

    const map = mapRef.current;

    // Находим данные для текущего года
    const currentData = results.find(r => r.year === currentYear) || results[results.length - 1];

    // Рисуем все круги с прозрачностью (история роста)
    results.forEach((data, index) => {
      const radius = data.diameter / 2;
      const opacity = 0.1 + (index / results.length) * 0.3;

      const circle = L.circle([latitude, longitude], {
        radius: radius,
        color: '#667eea',
        fillColor: '#667eea',
        fillOpacity: opacity,
        weight: 1,
      }).addTo(map);

      circlesRef.current.push(circle);

      // Добавляем метку для каждого 10-го года
      if (data.year % 10 === 0 || data.year === results[0].year || data.year === results[results.length - 1].year) {
        const label = L.marker([latitude, longitude], {
          icon: L.divIcon({
            className: 'year-label',
            html: `<div style="
              background: white;
              padding: 2px 6px;
              border-radius: 4px;
              font-size: 10px;
              font-weight: bold;
              box-shadow: 0 2px 4px rgba(0,0,0,0.2);
              white-space: nowrap;
            ">${data.year} (${data.diameter.toFixed(1)}м)</div>`,
            iconSize: [60, 20],
            iconAnchor: [30, 10],
          }),
        }).addTo(map);

        // Позиционируем метку на краю круга
        const angle = (index / results.length) * 360;
        const rad = (angle * Math.PI) / 180;
        const offsetLat = (radius / 111320) * Math.cos(rad);
        const offsetLng = (radius / (111320 * Math.cos(latitude * Math.PI / 180))) * Math.sin(rad);

        label.setLatLng([latitude + offsetLat, longitude + offsetLng]);
        circlesRef.current.push(label as any);
      }
    });

    // Рисуем текущий год ярко
    const currentRadius = currentData.diameter / 2;
    const currentCircle = L.circle([latitude, longitude], {
      radius: currentRadius,
      color: '#ff6b6b',
      fillColor: '#ff6b6b',
      fillOpacity: 0.4,
      weight: 3,
    }).addTo(map);

    circlesRef.current.push(currentCircle);

    // Добавляем маркер центра
    const marker = L.marker([latitude, longitude], {
      icon: L.divIcon({
        className: 'center-marker',
        html: '<div style="width: 10px; height: 10px; background: red; border: 2px solid white; border-radius: 50%; box-shadow: 0 2px 4px rgba(0,0,0,0.3);"></div>',
        iconSize: [10, 10],
        iconAnchor: [5, 5],
      }),
    }).addTo(map);

    circlesRef.current.push(marker as any);

    // Popup с информацией
    marker.bindPopup(`
      <div style="font-family: sans-serif;">
        <h3 style="margin: 0 0 8px 0; font-size: 14px;">Термокарст</h3>
        <p style="margin: 4px 0; font-size: 12px;"><strong>Год:</strong> ${currentData.year}</p>
        <p style="margin: 4px 0; font-size: 12px;"><strong>Диаметр:</strong> ${currentData.diameter.toFixed(2)} м</p>
        <p style="margin: 4px 0; font-size: 12px;"><strong>Глубина:</strong> ${currentData.depth.toFixed(2)} м</p>
        <p style="margin: 4px 0; font-size: 12px;"><strong>Координаты:</strong> ${latitude.toFixed(4)}°N, ${longitude.toFixed(4)}°E</p>
      </div>
    `).openPopup();

    // Подгоняем масштаб под все круги
    const maxRadius = Math.max(...results.map(r => r.diameter / 2));
    const bounds = L.latLngBounds(
      [latitude - maxRadius / 111320, longitude - maxRadius / (111320 * Math.cos(latitude * Math.PI / 180))],
      [latitude + maxRadius / 111320, longitude + maxRadius / (111320 * Math.cos(latitude * Math.PI / 180))]
    );
    map.fitBounds(bounds.pad(0.3));

  }, [latitude, longitude, results, currentYear]);

  return (
    <div style={{ position: 'relative', width: '100%', height: '100%' }}>
      <div id="map" style={{ width: '100%', height: '100%', borderRadius: '12px' }}></div>
      <div style={{
        position: 'absolute',
        top: '10px',
        right: '10px',
        background: 'rgba(255, 255, 255, 0.95)',
        padding: '12px',
        borderRadius: '8px',
        boxShadow: '0 2px 8px rgba(0,0,0,0.2)',
        zIndex: 1000,
        fontSize: '12px',
      }}>
        <div style={{ marginBottom: '8px', fontWeight: 'bold' }}>Легенда:</div>
        <div style={{ display: 'flex', alignItems: 'center', marginBottom: '4px' }}>
          <div style={{ width: '20px', height: '20px', background: '#667eea', opacity: 0.3, borderRadius: '50%', marginRight: '8px' }}></div>
          <span>История роста</span>
        </div>
        <div style={{ display: 'flex', alignItems: 'center' }}>
          <div style={{ width: '20px', height: '20px', background: '#ff6b6b', opacity: 0.4, border: '3px solid #ff6b6b', borderRadius: '50%', marginRight: '8px' }}></div>
          <span>Текущий год</span>
        </div>
      </div>
    </div>
  );
}
