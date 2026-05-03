import { invoke } from "@tauri-apps/api/core";
import type {
  ConnectionConfig,
  ConnectResult,
  GridPage,
  GridPageRequest,
  GridUpdateRequest,
  SavedConnection,
  SavedConnectionId,
  SavedConnectionInput,
  SavedConnectionUpdate,
} from "$lib/types/pg";

export async function connectionsList(): Promise<SavedConnection[]> {
  return invoke<SavedConnection[]>("connections_list");
}

export async function connectionsGet(id: SavedConnectionId): Promise<SavedConnection> {
  return invoke<SavedConnection>("connections_get", { id });
}

export async function connectionsSave(
  input: SavedConnectionInput,
): Promise<SavedConnection> {
  return invoke<SavedConnection>("connections_save", { input });
}

export async function connectionsUpdate(
  input: SavedConnectionUpdate,
): Promise<SavedConnection> {
  return invoke<SavedConnection>("connections_update", { input });
}

export async function connectionsDelete(id: SavedConnectionId): Promise<boolean> {
  return invoke<boolean>("connections_delete", { id });
}

export async function connectionsTest(
  input: ConnectionConfig,
  id?: SavedConnectionId,
): Promise<void> {
  await invoke<void>("connections_test", { input, id });
}

export async function connectionsConnect(
  id: SavedConnectionId,
): Promise<ConnectResult> {
  return invoke<ConnectResult>("connections_connect", { id });
}

export async function pgTestConnection(cfg: ConnectionConfig): Promise<void> {
  await invoke<void>("pg_test_connection", { cfg });
}

export async function pgConnect(cfg: ConnectionConfig): Promise<ConnectResult> {
  return invoke<ConnectResult>("pg_connect", { cfg });
}

export async function pgListSchemas(runtimeId: string): Promise<string[]> {
  return invoke<string[]>("pg_list_schemas", { runtimeId });
}

export async function pgListTables(
  runtimeId: string,
  schema: string,
): Promise<string[]> {
  return invoke<string[]>("pg_list_tables", { runtimeId, schema });
}

export async function pgGridPage(
  runtimeId: string,
  req: GridPageRequest,
): Promise<GridPage> {
  return invoke<GridPage>("pg_grid_page", { runtimeId, req });
}

export async function pgGridUpdate(
  runtimeId: string,
  req: GridUpdateRequest,
): Promise<number> {
  return invoke<number>("pg_grid_update", { runtimeId, req });
}
