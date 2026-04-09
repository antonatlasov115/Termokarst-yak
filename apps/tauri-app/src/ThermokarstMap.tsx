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
      const map = L.map('map').setView([latitude, longitude], 15);

      const satellite = L.tileLayer('https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}', {
        attribution: 'Tiles © Esri',
        maxZoom: 19,
      }).addTo(map);

      const street = L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
        attribution: '© OpenStreetMap',
        maxZoom: 19,
      });

      L.control.layers({
        'Спутник': satellite,
        'Карта': street,
      }).addTo(map);

      mapRef.current = map;
    }

    circlesRef.current.forEach(circle => circle.remove());
    circlesRef.current = [];

    if (results.length === 0) return;

    const map = mapRef.current;
    const currentData = results.find(r => r.year === currentYear) || results[results.length - 1];

    // История роста
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
    });

    // Текущий год
    const currentRadius = currentData.diameter / 2;
    const currentCircle = L.circle([latitude, longitude], {
      radius: currentRadius,
      color: '#ff6b6b',
      fillColor: '#ff6b6b',
      fillOpacity: 0.3,
      weight: 3,
    }).addTo(map);

    circlesRef.current.push(currentCircle);

    // Батиметрия
    const depthLevels = 5;
    for (let i = 1; i <= depthLevels; i++) {
      const depthRatio = i / depthLevels;
      const depthRadius = currentRadius * (1 - depthRatio * 0.3);
      const depthOpacity = 0.1 + depthRatio * 0.4;
      const blueValue = Math.floor(255 - depthRatio * 100);
      const depthColor = `rgb(100, 150, ${blueValue})`;

      const depthCircle = L.circle([latitude, longitude], {
        radius: depthRadius,
        color: depthColor,
        fillColor: depthColor,
        fillOpacity: depthOpacity,
        weight: 1,
      }).addTo(map);

      circlesRef.current.push(depthCircle);
    }

    // Маркер центра
    const marker = L.marker([latitude, longitude], {
      icon: L.divIcon({
        className: 'center-marker',
        html: `<div style="
          width: 20px;
          height: 20px;
          background: linear-gradient(135deg, #ff6b6b 0%, #4a90e2 100%);
          border: 3px solid white;
          border-radius: 50%;
          box-shadow: 0 2px 6px rgba(0,0,0,0.4);
        "></div>`,
        iconSize: [20, 20],
        iconAnchor: [10, 10],
      }),
    }).addTo(map);

    circlesRef.current.push(marker as any);

    marker.bindPopup(`
      <div style="font-family: sans-serif;">
        <h3 style="margin: 0 0 8px 0; font-size: 14px;">🏔️ Термокарст</h3>
        <p style="margin: 4px 0; font-size: 12px;"><strong>Год:</strong> ${currentData.year}</p>
        <p style="margin: 4px 0; font-size: 12px;"><strong>Диаметр:</strong> ${currentData.diameter.toFixed(2)} м</p>
        <p style="margin: 4px 0; font-size: 12px; color: #4a90e2;"><strong>⬇️ Глубина:</strong> ${currentData.depth.toFixed(2)} м</p>
      </div>
    `).openPopup();

    const maxRadius = Math.max(...results.map(r => r.diameter / 2));
    const bounds = L.latLngBounds(
      [latitude - maxRadius / 111320, longitude - maxRadius / (111320 * Math.cos(latitude * Math.PI / 180))],
      [latitude + maxRadius / 111320, longitude + maxRadius / (111320 * Math.cos(latitude * Math.PI / 180))]
    );
    map.fitBounds(bounds.pad(0.3));

  }, [latitude, longitude, results, currentYear]);

  return <div id="map" style={{ width: '100%', height: '100%', borderRadius: '12px' }}></div>;
}
