<script lang="ts">
  import AppShell from "$lib/components/AppShell.svelte";
  import Button from "$lib/components/Button.svelte";
  import Input from "$lib/components/Input.svelte";
  import Panel from "$lib/components/Panel.svelte";
  import {
    pgConnect,
    pgListTables,
    pgGridPage,
    pgGridUpdate,
  } from "$lib/ipc/postgres";
  import type {
    ConnectionConfig,
    GridPage,
    GridPageRequest,
    GridUpdateRequest,
  } from "$lib/types/pg";

  // --- Connection state ---
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

  // --- Table / grid state ---
  let schema = $state("public");
  let table = $state("");
  let limit = $state(50);
  let offset = $state(0);
  let grid = $state<GridPage | null>(null);
  let tableList = $state<string[]>([]);
  let busy = $state(false);

  // --- IPC actions ---
  async function connect() {
    error = null;
    try {
      const r = await pgConnect(cfg);
      runtimeId = r.runtime_id;
      connectedDb = cfg.database;
    } catch (e) {
      error = String(e);
    }
  }

  async function listTables() {
    error = null;
    tableList = [];
    if (!runtimeId) {
      error = "Not connected";
      return;
    }
    busy = true;
    try {
      tableList = await pgListTables(runtimeId, schema.trim() || "public");
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
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
      const req: GridPageRequest = {
        table: { schema: schema.trim(), name: table.trim() },
        limit,
        offset,
        order_by: null,
      };
      grid = await pgGridPage(runtimeId, req);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  function pkForRow(row: unknown[]): [string, unknown][] {
    if (!grid) return [];
    return grid.pk_columns.map((pk) => {
      const idx = grid!.columns.findIndex((c) => c.name === pk);
      return [pk, idx >= 0 ? row[idx] : null];
    });
  }

  async function updateCell(row: unknown[], colName: string, newValue: string) {
    if (!grid || !runtimeId) return;
    error = null;
    if (grid.pk_columns.length === 0) {
      error = "This table has no primary key. Editing is disabled.";
      return;
    }
    if (grid.pk_columns.includes(colName)) {
      error = "Editing primary key columns is disabled.";
      return;
    }
    const req: GridUpdateRequest = {
      table: { schema: schema.trim(), name: table.trim() },
      pk: pkForRow(row),
      column: colName,
      value: newValue === "" ? null : newValue,
    };
    try {
      await pgGridUpdate(runtimeId, req);
      await loadGrid();
    } catch (e) {
      error = String(e);
    }
  }
</script>

<AppShell>
  {#snippet sidebar()}
    <!-- ===== SIDEBAR ===== -->
    <div class="sidebar-inner">
      <div class="sidebar-section">
        <div class="sidebar-label">Connection</div>
        <div class="field-grid">
          <label for="cfg-host" class="field-label">Host</label>
          <Input id="cfg-host" bind:value={cfg.host} placeholder="127.0.0.1" />

          <label for="cfg-port" class="field-label">Port</label>
          <Input id="cfg-port" type="number" bind:value={cfg.port} />

          <label for="cfg-db" class="field-label">DB</label>
          <Input id="cfg-db" bind:value={cfg.database} placeholder="postgres" />

          <label for="cfg-user" class="field-label">User</label>
          <Input id="cfg-user" bind:value={cfg.username} placeholder="postgres" />

          <label for="cfg-pass" class="field-label">Pass</label>
          <Input id="cfg-pass" type="password" bind:value={cfg.password} />
        </div>

        <Button variant="primary" onclick={connect}>Connect</Button>

        {#if runtimeId}
          <div class="conn-status conn-ok">● {connectedDb}</div>
        {:else}
          <div class="conn-status conn-none">Not connected</div>
        {/if}
      </div>

      <div class="sidebar-divider"></div>

      <div class="sidebar-section">
        <div class="sidebar-label">Table</div>
        <div class="field-grid">
          <label for="in-schema" class="field-label">Schema</label>
          <Input id="in-schema" bind:value={schema} placeholder="public" />

          <label for="in-table" class="field-label">Table</label>
          <Input id="in-table" bind:value={table} placeholder="users" />

          <label for="in-limit" class="field-label">Limit</label>
          <Input id="in-limit" type="number" bind:value={limit} />

          <label for="in-offset" class="field-label">Offset</label>
          <Input id="in-offset" type="number" bind:value={offset} />
        </div>

        <div class="btn-row">
          <Button onclick={loadGrid} disabled={!runtimeId || busy}>Load</Button>
          <Button onclick={listTables} disabled={!runtimeId || busy}>List</Button>
        </div>

        {#if tableList.length > 0}
          <div class="table-list">
            {#each tableList as t}
              <button class="table-list-item" onclick={() => (table = t)}>{t}</button>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {/snippet}

  <!-- ===== MAIN CONTENT ===== -->
  <div class="main-inner">
    <!-- Tab bar -->
    <div class="tabbar">
      <span class="tab tab-active">
        {grid ? `${schema}.${table}` : "Grid"}
      </span>
      {#if grid}
        <span class="tab-meta">
          {grid.row_count} rows
          {#if grid.pk_columns.length > 0}
            · PK: {grid.pk_columns.join(", ")}
          {:else}
            · no PK (read-only)
          {/if}
        </span>
      {/if}
    </div>

    <!-- Error banner -->
    {#if error}
      <div class="error-banner">
        <span>⚠ {error}</span>
        <button class="error-close" onclick={() => (error = null)}>✕</button>
      </div>
    {/if}

    <!-- No PK warning -->
    {#if grid && grid.pk_columns.length === 0}
      <div class="warn-banner">
        This table has no primary key — cells are read-only.
      </div>
    {/if}

    <!-- Grid -->
    {#if grid}
      <div class="grid-scroll">
        <table class="data-table">
          <thead>
            <tr>
              {#each grid.columns as c}
                <th class:pk-col={grid.pk_columns.includes(c.name)}>
                  {c.name}
                  <span class="col-type">{c.type_name}</span>
                </th>
              {/each}
            </tr>
          </thead>
          <tbody>
            {#each grid.rows as row}
              <tr>
                {#each grid.columns as c, i}
                  <td class:pk-cell={grid.pk_columns.includes(c.name)}>
                    {#if grid.pk_columns.length > 0 && !grid.pk_columns.includes(c.name)}
                      <input
                        class="cell-input"
                        value={row[i] === null ? "" : String(row[i])}
                        onblur={(e) =>
                          updateCell(
                            row,
                            c.name,
                            (e.currentTarget as HTMLInputElement).value,
                          )}
                      />
                    {:else}
                      <span class="cell-value" data-selectable>
                        {row[i] === null ? "NULL" : String(row[i])}
                      </span>
                    {/if}
                  </td>
                {/each}
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {:else}
      <div class="empty-state">
        {#if !runtimeId}
          Connect to a Postgres database to get started.
        {:else}
          Enter a schema and table name, then click <strong>Load</strong>.
        {/if}
      </div>
    {/if}
  </div>
</AppShell>

<style>
  /* ---- Sidebar ---- */
  .sidebar-inner {
    display: flex;
    flex-direction: column;
    gap: 0;
    height: 100%;
    padding-top: var(--space-2);
  }

  .sidebar-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-3);
  }

  .sidebar-label {
    font-size: var(--text-xs);
    font-weight: 600;
    color: var(--color-text-faint);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .sidebar-divider {
    height: 1px;
    background: var(--color-divider);
    margin: var(--space-1) 0;
  }

  .field-grid {
    display: grid;
    grid-template-columns: 48px 1fr;
    align-items: center;
    gap: 5px var(--space-2);
  }

  .field-label {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    text-align: right;
  }

  .btn-row {
    display: flex;
    gap: var(--space-2);
  }

  .conn-status {
    font-size: var(--text-xs);
    padding: 2px 0;
  }

  .conn-ok {
    color: var(--color-ok-text);
    font-weight: 500;
  }

  .conn-none {
    color: var(--color-text-faint);
  }

  .table-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-height: 200px;
    overflow-y: auto;
    background: var(--color-bg);
    border: var(--border);
    border-radius: var(--radius-sm);
    padding: var(--space-1);
  }

  .table-list-item {
    display: block;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    font-size: var(--text-xs);
    color: var(--color-text);
    padding: 3px var(--space-2);
    border-radius: 3px;
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .table-list-item:hover {
    background: var(--color-accent-subtle);
    color: var(--color-accent);
  }

  /* ---- Main ---- */
  .main-inner {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .tabbar {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    height: var(--topbar-height);
    min-height: var(--topbar-height);
    padding: 0 var(--space-4);
    border-bottom: var(--border);
    background: var(--color-bg-elevated);
  }

  .tab {
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--color-text-muted);
    padding: 2px var(--space-3);
    border-radius: var(--radius-sm);
  }

  .tab-active {
    background: var(--color-accent-subtle);
    color: var(--color-accent);
  }

  .tab-meta {
    font-size: var(--text-xs);
    color: var(--color-text-faint);
  }

  /* ---- Banners ---- */
  .error-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-2) var(--space-4);
    background: var(--color-error-bg);
    color: var(--color-error-text);
    font-size: var(--text-sm);
    border-bottom: 1px solid var(--color-error-text);
  }

  .error-close {
    background: none;
    border: none;
    color: inherit;
    font-size: var(--text-base);
    cursor: pointer;
    opacity: 0.7;
    padding: 0 var(--space-1);
    line-height: 1;
  }

  .warn-banner {
    padding: var(--space-2) var(--space-4);
    background: var(--color-warn-bg);
    color: var(--color-text-muted);
    font-size: var(--text-sm);
    border-bottom: 1px solid var(--color-warn-border);
  }

  /* ---- Data grid ---- */
  .grid-scroll {
    flex: 1;
    overflow: auto;
  }

  .data-table {
    border-collapse: collapse;
    width: 100%;
    font-size: var(--text-sm);
  }

  .data-table thead {
    position: sticky;
    top: 0;
    z-index: 1;
  }

  .data-table th {
    background: var(--color-th-bg);
    border-bottom: 1px solid var(--color-divider);
    padding: var(--space-2) var(--space-3);
    text-align: left;
    font-weight: 500;
    white-space: nowrap;
    color: var(--color-text);
  }

  .data-table th.pk-col {
    color: var(--color-accent);
  }

  .col-type {
    display: block;
    font-size: var(--text-xs);
    color: var(--color-text-faint);
    font-weight: 400;
  }

  .data-table td {
    border-bottom: 1px solid var(--color-row-border);
    padding: 0 var(--space-1);
    vertical-align: middle;
  }

  .data-table tbody tr:hover td {
    background: var(--color-row-hover);
  }

  .pk-cell {
    color: var(--color-text-muted);
  }

  .cell-value {
    display: block;
    padding: 5px var(--space-2);
    white-space: pre;
  }

  .cell-input {
    width: 100%;
    padding: 4px var(--space-2);
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    font-size: var(--text-sm);
    font-family: var(--font-sans);
    color: var(--color-text);
    outline: none;
    box-sizing: border-box;
    transition: border-color 0.1s, background 0.1s;
  }

  .cell-input:focus {
    background: var(--color-input-bg);
    border-color: var(--color-accent);
    box-shadow: 0 0 0 2px var(--color-accent-subtle);
  }

  /* ---- Empty state ---- */
  .empty-state {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-faint);
    font-size: var(--text-base);
    text-align: center;
    padding: var(--space-8);
  }
</style>
