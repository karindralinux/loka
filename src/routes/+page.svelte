<script lang="ts">
  type EngineType = "Postgres";
  type ConnectionConfig = {
    engine: EngineType;
    host: string;
    port: number;
    database: string;
    username: string;
    password: string;
    ssl_mode: string;
  };

  type TableId = { schema: string; name: string };
  type GridPageRequest = {
    table: TableId;
    limit: number;
    offset: number;
    order_by: string[] | null;
  };
  type GridPage = {
    columns: { name: string; type_name: string }[];
    pk_columns: string[];
    rows: any[][];
    row_count: number;
  };

  type GridUpdateRequest = {
    table: TableId;
    pk: [string, any][];
    column: string;
    value: any;
  };

  let cfg = $state<ConnectionConfig>({
    engine: "Postgres",
    host: "127.0.0.1",
    port: 5432,
    database: "postgres",
    username: "postgres",
    password: "",
    ssl_mode: "disable",
  });

  let runtimeId = $state<string | null>(null);
  let connectedDb = $state<string | null>(null);
  let error = $state<string | null>(null);

  let schema = $state("public");
  let table = $state("");
  let limit = $state(50);
  let offset = $state(0);

  let grid = $state<GridPage | null>(null);
  let tableList = $state<string[]>([]);
  let busy = $state(false);

  async function listTables() {
    error = null;
    tableList = [];
    if (!runtimeId) {
      error = "Not connected";
      return;
    }
    busy = true;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      tableList = await invoke<string[]>("pg_list_tables", {
        runtimeId,
        schema: schema.trim() || "public",
      });
    } catch (e) {
      error = String(e);
      console.error(e);
    } finally {
      busy = false;
    }
  }

  async function connect() {
    error = null;
    const { invoke } = await import("@tauri-apps/api/core");
    const r = await invoke<{ runtime_id: string }>("pg_connect", { cfg });
    runtimeId = r.runtime_id;
    connectedDb = cfg.database;
  }

  async function loadGrid() {
    error = null;
    grid = null;
    if (!runtimeId) {
      error = "Not connected";
      return;
    }
    if (!schema.trim() || !table.trim()) {
      error = "Schema and table are required";
      return;
    }

    busy = true;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const req: GridPageRequest = {
        table: { schema: schema.trim(), name: table.trim() },
        limit,
        offset,
        order_by: null,
      };
      grid = await invoke<GridPage>("pg_grid_page", { runtimeId, req });
    } catch (e) {
      error = String(e);
      console.error(e);
    } finally {
      busy = false;
    }
  }

  function pkForRow(row: any[]): [string, any][] {
    if (!grid) return [];
    const out: [string, any][] = [];
    for (const pk of grid.pk_columns) {
      const idx = grid.columns.findIndex((c) => c.name === pk);
      out.push([pk, idx >= 0 ? row[idx] : null]);
    }
    return out;
  }

  async function updateCell(row: any[], colName: string, newValue: string) {
    if (!grid || !runtimeId) return;
    error = null;

    if (grid.pk_columns.length === 0) {
      error = "This table has no primary key. Editing is disabled (MVP).";
      return;
    }
    if (grid.pk_columns.includes(colName)) {
      error = "Editing primary key columns is disabled (MVP).";
      return;
    }

    const req: GridUpdateRequest = {
      table: { schema: schema.trim(), name: table.trim() },
      pk: pkForRow(row),
      column: colName,
      value: newValue === "" ? null : newValue,
    };

    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke<number>("pg_grid_update", { runtimeId, req });
      await loadGrid();
    } catch (e) {
      error = String(e);
      console.error(e);
    }
  }
</script>

<div style="padding:16px; font-family: system-ui; display:grid; gap:12px;">
  <h2>Loka — Editable Grid (MVP)</h2>

  <fieldset style="display:grid; gap:8px; max-width: 720px;">
    <legend>Connection</legend>
    <div style="display:grid; grid-template-columns: 120px 1fr; gap:8px;">
      <label for="cfg-host">Host</label><input id="cfg-host" bind:value={cfg.host} />
      <label for="cfg-port">Port</label><input id="cfg-port" type="number" bind:value={cfg.port} />
      <label for="cfg-db">DB</label><input id="cfg-db" bind:value={cfg.database} />
      <label for="cfg-user">User</label><input id="cfg-user" bind:value={cfg.username} />
      <label for="cfg-pass">Pass</label><input id="cfg-pass" type="password" bind:value={cfg.password} />
    </div>
    <div style="display:flex; gap:8px; align-items:center;">
      <button onclick={connect}>Connect</button>
      {#if runtimeId}
        <span style="color: green; font-weight: 500;">Connected to: {connectedDb}</span>
        <span style="font-size: 11px; opacity: 0.6;">(id: {runtimeId})</span>
      {:else}
        <span style="color: #666;">Not connected</span>
      {/if}
    </div>
  </fieldset>

  <fieldset style="display:grid; gap:8px; max-width: 720px;">
    <legend>Table</legend>
    <div style="display:grid; grid-template-columns: 120px 1fr; gap:8px;">
      <label for="in-schema">Schema</label><input id="in-schema" bind:value={schema} placeholder="public" />
      <label for="in-table">Table</label><input id="in-table" bind:value={table} placeholder="users" />
      <label for="in-limit">Limit</label><input id="in-limit" type="number" bind:value={limit} />
      <label for="in-offset">Offset</label><input id="in-offset" type="number" bind:value={offset} />
    </div>
    <div style="display:flex; gap:8px; align-items:center;">
      <button onclick={loadGrid} disabled={!runtimeId || busy}>Load</button>
      <button onclick={listTables} disabled={!runtimeId || busy}>List Tables</button>
      {#if grid}
        <span
          >Rows: {grid.row_count} | PK: {grid.pk_columns.length
            ? grid.pk_columns.join(", ")
            : "none"}</span
        >
      {/if}
    </div>
    {#if tableList.length > 0}
      <div style="font-size:12px; display:flex; flex-wrap:wrap; gap:8px; background:#f9f9f9; padding:8px; border:1px solid #eee;">
        <b>Available:</b>
        {#each tableList as t}
          <button style="padding:2px 6px; font-size:11px;" onclick={() => table = t}>{t}</button>
        {/each}
      </div>
    {/if}
  </fieldset>

  {#if error}
    <pre style="white-space: pre-wrap; color: #b00020;">{error}</pre>
  {/if}

  {#if grid}
    {#if grid.pk_columns.length === 0}
      <div style="padding:8px; background:#fff3cd; border:1px solid #ffeeba;">
        This table has no primary key. Editing is disabled in MVP.
      </div>
    {/if}

    <div style="overflow:auto; border:1px solid #ddd;">
      <table style="border-collapse: collapse; width: 100%; font-size: 13px;">
        <thead>
          <tr>
            {#each grid.columns as c}
              <th
                style="position:sticky; top:0; background:#f6f6f6; border-bottom:1px solid #ddd; padding:8px; text-align:left;"
              >
                {c.name}
                <div style="font-size:11px; opacity:.65">{c.type_name}</div>
              </th>
            {/each}
          </tr>
        </thead>
        <tbody>
          {#each grid.rows as row}
            <tr>
              {#each grid.columns as c, i}
                <td style="border-top:1px solid #eee; padding:6px;">
                  {#if grid.pk_columns.length > 0 && !grid.pk_columns.includes(c.name)}
                    <input
                      value={row[i] === null ? "" : String(row[i])}
                      style="width: 100%; box-sizing: border-box;"
                      onblur={(e) =>
                        updateCell(
                          row,
                          c.name,
                          (e.currentTarget as HTMLInputElement).value,
                        )}
                    />
                  {:else}
                    {row[i] === null ? "NULL" : String(row[i])}
                  {/if}
                </td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
