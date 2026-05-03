/* ═══════════════════════════════════════════════════════
   Loka — Connections Store (localStorage-backed)
   ═══════════════════════════════════════════════════════ */

import type { SavedConnection, ConnectionConfig } from './types.js';
import { CONNECTION_COLORS } from './types.js';

const STORAGE_KEY = 'loka_connections';

function generateId(): string {
  return Date.now().toString(36) + Math.random().toString(36).slice(2, 8);
}

function loadConnections(): SavedConnection[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function saveConnections(connections: SavedConnection[]): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(connections));
}

export function getConnections(): SavedConnection[] {
  return loadConnections();
}

export function addConnection(name: string, config: ConnectionConfig): SavedConnection {
  const connections = loadConnections();
  const colorIdx = connections.length % CONNECTION_COLORS.length;
  const conn: SavedConnection = {
    id: generateId(),
    name: name || `${config.host}:${config.port}/${config.database}`,
    config: { ...config },
    color: CONNECTION_COLORS[colorIdx],
    createdAt: Date.now(),
    lastUsedAt: null,
  };
  connections.push(conn);
  saveConnections(connections);
  return conn;
}

export function removeConnection(id: string): void {
  const connections = loadConnections().filter(c => c.id !== id);
  saveConnections(connections);
}

export function touchConnection(id: string): void {
  const connections = loadConnections();
  const conn = connections.find(c => c.id === id);
  if (conn) {
    conn.lastUsedAt = Date.now();
    saveConnections(connections);
  }
}

export function updateConnection(id: string, name: string, config: ConnectionConfig): void {
  const connections = loadConnections();
  const conn = connections.find(c => c.id === id);
  if (conn) {
    conn.name = name;
    conn.config = { ...config };
    saveConnections(connections);
  }
}
