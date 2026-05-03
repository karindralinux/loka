/* ═══════════════════════════════════════════════════════
   Loka — Shared Type Definitions
   ═══════════════════════════════════════════════════════ */

export type EngineType = "Postgres";

export interface ConnectionConfig {
  engine: EngineType;
  host: string;
  port: number;
  database: string;
  username: string;
  password: string;
  ssl_mode: string;
}

export interface SavedConnection {
  id: string;
  name: string;
  config: ConnectionConfig;
  color: string;
  createdAt: number;
  lastUsedAt: number | null;
}

export interface TableId {
  schema: string;
  name: string;
}

export interface GridPageRequest {
  table: TableId;
  limit: number;
  offset: number;
  order_by: string[] | null;
}

export interface GridColumn {
  name: string;
  type_name: string;
}

export interface GridPage {
  columns: GridColumn[];
  pk_columns: string[];
  rows: any[][];
  row_count: number;
}

export interface GridUpdateRequest {
  table: TableId;
  pk: [string, any][];
  column: string;
  value: any;
}

/** Color palette for connection badges */
export const CONNECTION_COLORS = [
  "#3ecf8e", // green (brand)
  "#6c63ff", // purple
  "#f472b6", // pink
  "#f59e0b", // amber
  "#3b82f6", // blue
  "#ef4444", // red
  "#06b6d4", // cyan
  "#8b5cf6", // violet
  "#ec4899", // hot pink
  "#14b8a6", // teal
];
