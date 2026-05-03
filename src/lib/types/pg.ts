// TypeScript types mirroring Rust/loka-core DTOs

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

export type SavedConnectionId = string;

export interface SavedConnection {
  id: SavedConnectionId;
  name: string;
  config: ConnectionConfig;
}

export interface SavedConnectionInput {
  name: string;
  config: ConnectionConfig;
}

export interface SavedConnectionUpdate {
  id: SavedConnectionId;
  name: string;
  config: ConnectionConfig;
}

export interface EngineCapabilities {
  sql: boolean;
  introspection: boolean;
  editable_grid: boolean;
}

export interface ConnectResult {
  runtime_id: string;
  capabilities: EngineCapabilities;
}

export interface SqlColumn {
  name: string;
  type_name: string;
}

export interface SqlQueryResult {
  columns: SqlColumn[];
  rows: unknown[][];
  row_count: number;
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

export interface GridPage {
  columns: SqlColumn[];
  pk_columns: string[];
  rows: unknown[][];
  row_count: number;
}

export interface GridUpdateRequest {
  table: TableId;
  pk: [string, unknown][];
  column: string;
  value: unknown;
}
