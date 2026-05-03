import { invoke } from "@tauri-apps/api/core";
import type {
  ConnectionConfig,
  ConnectResult,
  GridPage,
  GridPageRequest,
  GridUpdateRequest,
} from "$lib/types/pg";

export async function pgTestConnection(cfg: ConnectionConfig): Promise<void> {
  await invoke<void>("pg_test_connection", { cfg });
}

export async function pgConnect(cfg: ConnectionConfig): Promise<ConnectResult> {
  return invoke<ConnectResult>("pg_connect", { cfg });
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
