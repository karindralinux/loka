<script lang="ts">
  import { onMount } from "svelte";
  import AppShell from "$lib/components/AppShell.svelte";
  import Button from "$lib/components/Button.svelte";
  import Input from "$lib/components/Input.svelte";
  import Panel from "$lib/components/Panel.svelte";
  import {
    connectionsConnect,
    connectionsDelete,
    connectionsList,
    connectionsSave,
    connectionsTest,
    connectionsUpdate,
    pgGridPage,
    pgGridUpdate,
    pgListSchemas,
    pgListTables,
  } from "$lib/ipc/postgres";
  import type {
    ConnectionConfig,
    GridPage,
    GridPageRequest,
    GridUpdateRequest,
    SavedConnection,
    SavedConnectionId,
    SavedConnectionInput,
    SavedConnectionUpdate,
  } from "$lib/types/pg";

  // ---- Connection screen state ----
  let savedConnections = $state<SavedConnection[]>([]);
  let selectedId = $state<SavedConnectionId | null>(null);
  let formName = $state("Local Postgres");
  let cfg = $state<ConnectionConfig>({
    engine: "Postgres",
    host: "127.0.0.1",
    port: 5432,
    database: "postgres",
    username: "postgres",
    password: "",
    ssl_mode: "disable",
  });
  let connectionNotice = $state<string | null>(null);
  let connectionError = $state<string | null>(null);
  let connectionBusy = $state(false);

  // ---- Workspace state ----
  let runtimeId = $state<string | null>(null);
  let activeConnectionName = $state<string | null>(null);
  let schemas = $state<string[]>([]);
  let schemaTables = $state<Record<string, string[]>>({});
  let expandedSchemas = $state<Set<string>>(new Set());
  let activeSchema = $state<string | null>(null);
  let activeTable = $state<string | null>(null);
  let grid = $state<GridPage | null>(null);
  let workspaceError = $state<string | null>(null);
  let workspaceBusy = $state(false);
  let limit = $state(50);
  let offset = $state(0);

  onMount(async () => {
    await refreshConnections();
  });

  async function refreshConnections() {
    connectionBusy = true;
    connectionError = null;
    try {
      savedConnections = await connectionsList();
      if (selectedId && !savedConnections.some((c) => c.id === selectedId)) {
        selectedId = null;
      }
    } catch (e) {
      connectionError = String(e);
    } finally {
      connectionBusy = false;
    }
  }

  function selectConnection(conn: SavedConnection) {
    selectedId = conn.id;
    formName = conn.name;
    cfg = {
      ...conn.config,
      password: "",
    };
    connectionNotice = null;
    connectionError = null;
  }

  function resetForm() {
    selectedId = null;
    formName = "Local Postgres";
    cfg = {
      engine: "Postgres",
      host: "127.0.0.1",
      port: 5432,
      database: "postgres",
      username: "postgres",
      password: "",
      ssl_mode: "disable",
    };
    connectionNotice = null;
    connectionError = null;
  }

  async function handleTest() {
    connectionBusy = true;
    connectionError = null;
    connectionNotice = null;
    try {
      await connectionsTest(cfg, selectedId ?? undefined);
      connectionNotice = "Connection OK";
    } catch (e) {
      connectionError = String(e);
    } finally {
      connectionBusy = false;
    }
  }

  async function handleSave() {
    connectionBusy = true;
    connectionError = null;
    connectionNotice = null;
    try {
      if (!formName.trim()) {
        connectionError = "Connection name is required.";
        return;
      }
      const input: SavedConnectionInput = { name: formName.trim(), config: cfg };
      let saved: SavedConnection;
      if (selectedId) {
        const update: SavedConnectionUpdate = { id: selectedId, ...input };
        saved = await connectionsUpdate(update);
        connectionNotice = "Connection updated.";
      } else {
        saved = await connectionsSave(input);
        connectionNotice = "Connection saved.";
      }
      await refreshConnections();
      selectConnection(saved);
    } catch (e) {
      connectionError = String(e);
    } finally {
      connectionBusy = false;
    }
  }

  async function handleDelete(id: SavedConnectionId) {
    connectionBusy = true;
    connectionError = null;
    connectionNotice = null;
    try {
      await connectionsDelete(id);
      await refreshConnections();
      if (selectedId === id) {
        resetForm();
      }
      connectionNotice = "Connection deleted.";
    } catch (e) {
      connectionError = String(e);
    } finally {
      connectionBusy = false;
    }
  }

  async function handleConnect() {
    connectionBusy = true;
    connectionError = null;
    connectionNotice = null;
    try {
      if (!selectedId) {
        connectionError = "Select a saved connection to connect.";
        return;
      }
      const result = await connectionsConnect(selectedId);
      runtimeId = result.runtime_id;
      activeConnectionName =
        savedConnections.find((c) => c.id === selectedId)?.name ?? null;
      await loadSchemas();
    } catch (e) {
      connectionError = String(e);
    } finally {
      connectionBusy = false;
    }
  }

  async function loadSchemas() {
    if (!runtimeId) return;
    workspaceBusy = true;
    workspaceError = null;
    try {
      schemas = await pgListSchemas(runtimeId);
      if (schemas.length > 0 && !activeSchema) {
        await toggleSchema(schemas[0]);
      }
    } catch (e) {
      workspaceError = String(e);
    } finally {
      workspaceBusy = false;
    }
  }

  async function toggleSchema(schema: string) {
    const next = new Set(expandedSchemas);
    if (next.has(schema)) {
      next.delete(schema);
      expandedSchemas = next;
      return;
    }
    next.add(schema);
    expandedSchemas = next;
    if (!schemaTables[schema] && runtimeId) {
      try {
        const tables = await pgListTables(runtimeId, schema);
        schemaTables = { ...schemaTables, [schema]: tables };
      } catch (e) {
        workspaceError = String(e);
      }
    }
  }

  async function selectTable(schema: string, table: string) {
    activeSchema = schema;
    activeTable = table;
    await loadGrid();
  }

  async function loadGrid() {
    if (!runtimeId || !activeSchema || !activeTable) return;
    workspaceBusy = true;
    workspaceError = null;
    try {
      const req: GridPageRequest = {
        table: { schema: activeSchema, name: activeTable },
        limit,
        offset,
        order_by: null,
      };
      grid = await pgGridPage(runtimeId, req);
    } catch (e) {
      workspaceError = String(e);
    } finally {
      workspaceBusy = false;
    }
  }

  function pkForRow(row: unknown[]): [string, unknown][] {
    if (!grid) return [];
    const currentGrid = grid;
    return currentGrid.pk_columns.map((pk) => {
      const idx = currentGrid.columns.findIndex((c) => c.name === pk);
      return [pk, idx >= 0 ? row[idx] : null];
    });
  }

  async function updateCell(row: unknown[], colName: string, newValue: string) {
    if (!grid || !runtimeId || !activeSchema || !activeTable) return;
    workspaceError = null;
    if (grid.pk_columns.length === 0) {
      workspaceError = "This table has no primary key. Editing is disabled.";
      return;
    }
    if (grid.pk_columns.includes(colName)) {
      workspaceError = "Editing primary key columns is disabled.";
      return;
    }
    const req: GridUpdateRequest = {
      table: { schema: activeSchema, name: activeTable },
      pk: pkForRow(row),
      column: colName,
      value: newValue === "" ? null : newValue,
    };
    try {
      await pgGridUpdate(runtimeId, req);
      await loadGrid();
    } catch (e) {
      workspaceError = String(e);
    }
  }

  function disconnect() {
    runtimeId = null;
    activeConnectionName = null;
    schemas = [];
    schemaTables = {};
    expandedSchemas = new Set();
    activeSchema = null;
    activeTable = null;
    grid = null;
    workspaceError = null;
  }
</script>

{#if !runtimeId}
  <div class="connection-screen">
    <div class="connection-header">
      <div>
        <h1>Loka Connections</h1>
        <p>Save database connections and connect with one click.</p>
      </div>
      <Button onclick={resetForm}>New</Button>
    </div>

    <div class="connection-body">
      <Panel title="Saved Connections">
        <div class="connections-list">
          {#if savedConnections.length === 0}
            <div class="empty-note">No saved connections yet.</div>
          {:else}
            {#each savedConnections as conn}
              <div class="connection-row">
                <button
                  type="button"
                  class="connection-item {conn.id === selectedId ? 'selected' : ''}"
                  onclick={() => selectConnection(conn)}
                >
                  <div class="connection-name">{conn.name}</div>
                  <div class="connection-meta">
                    {conn.config.username}@{conn.config.host}:{conn.config.port}
                    <span>·</span>
                    {conn.config.database}
                  </div>
                </button>
                <button
                  type="button"
                  class="inline-button"
                  onclick={() => handleDelete(conn.id)}
                >
                  Delete
                </button>
              </div>
            {/each}
          {/if}
        </div>
      </Panel>

      <Panel title="Connection Details">
        <div class="form-grid">
          <label for="conn-name">Name</label>
          <Input id="conn-name" bind:value={formName} placeholder="Local Postgres" />

          <label for="conn-host">Host</label>
          <Input id="conn-host" bind:value={cfg.host} placeholder="127.0.0.1" />

          <label for="conn-port">Port</label>
          <Input id="conn-port" type="number" bind:value={cfg.port} />

          <label for="conn-db">Database</label>
          <Input id="conn-db" bind:value={cfg.database} placeholder="postgres" />

          <label for="conn-user">User</label>
          <Input id="conn-user" bind:value={cfg.username} placeholder="postgres" />

          <label for="conn-pass">Password</label>
          <Input id="conn-pass" type="password" bind:value={cfg.password} placeholder="(stored)" />
        </div>

        <div class="form-actions">
          <Button onclick={handleTest} disabled={connectionBusy}>Test</Button>
          <Button onclick={handleSave} disabled={connectionBusy} variant="primary">
            {selectedId ? "Update" : "Save"}
          </Button>
          <Button onclick={handleConnect} disabled={connectionBusy}>
            Connect
          </Button>
        </div>

        {#if connectionError}
          <div class="status error">⚠ {connectionError}</div>
        {/if}
        {#if connectionNotice}
          <div class="status ok">● {connectionNotice}</div>
        {/if}
      </Panel>
    </div>
  </div>
{:else}
  <AppShell>
    {#snippet sidebar()}
      <div class="workspace-sidebar">
        <div class="sidebar-header">
          <div class="sidebar-title">Workspace</div>
          <div class="sidebar-meta">
            {activeConnectionName ?? "Connected"}
          </div>
          <Button onclick={disconnect}>Disconnect</Button>
        </div>

        <div class="explorer">
          {#if schemas.length === 0}
            <div class="empty-note">No schemas found.</div>
          {:else}
            {#each schemas as schemaName}
              <div class="schema-block">
                <button
                  class="schema-row"
                  onclick={() => toggleSchema(schemaName)}
                >
                  <span>{schemaName}</span>
                  <span class="schema-count">
                    {schemaTables[schemaName]?.length ?? 0}
                  </span>
                </button>
                {#if expandedSchemas.has(schemaName)}
                  <div class="table-list">
                    {#if schemaTables[schemaName]?.length}
                      {#each schemaTables[schemaName] as tableName}
                        <button
                          class="table-row {activeSchema === schemaName && activeTable === tableName ? 'active' : ''}"
                          onclick={() => selectTable(schemaName, tableName)}
                        >
                          {tableName}
                        </button>
                      {/each}
                    {:else}
                      <div class="empty-note small">No tables</div>
                    {/if}
                  </div>
                {/if}
              </div>
            {/each}
          {/if}
        </div>
      </div>
    {/snippet}

    <div class="main-inner">
      <div class="tabbar">
        <span class="tab tab-active">
          {#if activeSchema && activeTable}
            {activeSchema}.{activeTable}
          {:else}
            Grid
          {/if}
        </span>
        {#if grid}
          <span class="tab-meta">
            {grid.row_count} rows
            {#if grid.pk_columns.length > 0}
              · PK: {grid.pk_columns.join(", ")}
            {:else}
              · no PK
            {/if}
          </span>
        {/if}
      </div>

      <div class="workspace-toolbar">
        <div class="toolbar-group">
          <label for="limit-input">Limit</label>
          <Input id="limit-input" type="number" bind:value={limit} />
        </div>
        <div class="toolbar-group">
          <label for="offset-input">Offset</label>
          <Input id="offset-input" type="number" bind:value={offset} />
        </div>
        <Button onclick={loadGrid} disabled={!activeTable || workspaceBusy}>
          Reload
        </Button>
      </div>

      {#if workspaceError}
        <div class="error-banner">
          <span>⚠ {workspaceError}</span>
          <button class="error-close" onclick={() => (workspaceError = null)}>
            ✕
          </button>
        </div>
      {/if}

      {#if grid && grid.pk_columns.length === 0}
        <div class="warn-banner">
          This table has no primary key — cells are read-only.
        </div>
      {/if}

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
          {#if workspaceBusy}
            Loading…
          {:else}
            Select a table in the sidebar to view data.
          {/if}
        </div>
      {/if}
    </div>
  </AppShell>
{/if}

<style>
  .connection-screen {
    padding: var(--space-8);
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    height: calc(100vh - var(--topbar-height));
    overflow: auto;
  }

  .connection-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
  }

  .connection-header h1 {
    font-size: var(--text-lg);
    font-weight: 600;
    margin-bottom: var(--space-1);
  }

  .connection-header p {
    color: var(--color-text-muted);
    font-size: var(--text-sm);
  }

  .connection-body {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-6);
  }

  .connections-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .connection-row {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: var(--space-2);
    align-items: center;
  }

  .connection-item {
    border: 1px solid var(--color-divider);
    border-radius: var(--radius-sm);
    padding: var(--space-2) var(--space-3);
    display: grid;
    gap: 2px;
    cursor: pointer;
    background: var(--color-bg);
    text-align: left;
    width: 100%;
  }

  .connection-item.selected {
    border-color: var(--color-accent);
    background: var(--color-accent-subtle);
  }

  .connection-name {
    font-weight: 600;
    font-size: var(--text-sm);
  }

  .connection-meta {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    display: flex;
    gap: var(--space-2);
    align-items: center;
  }

  .inline-button {
    background: none;
    border: none;
    color: var(--color-text-faint);
    font-size: var(--text-xs);
    cursor: pointer;
  }

  .inline-button:hover {
    color: var(--color-error-text);
  }

  .form-grid {
    display: grid;
    grid-template-columns: 100px 1fr;
    gap: var(--space-2);
    align-items: center;
  }

  .form-actions {
    display: flex;
    gap: var(--space-2);
    margin-top: var(--space-3);
  }

  .status {
    margin-top: var(--space-3);
    font-size: var(--text-sm);
  }

  .status.ok {
    color: var(--color-ok-text);
  }

  .status.error {
    color: var(--color-error-text);
  }

  .empty-note {
    font-size: var(--text-sm);
    color: var(--color-text-faint);
  }

  .empty-note.small {
    font-size: var(--text-xs);
    padding: var(--space-1) 0;
  }

  .workspace-sidebar {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-3);
    height: 100%;
  }

  .sidebar-header {
    display: grid;
    gap: var(--space-1);
  }

  .sidebar-title {
    font-size: var(--text-xs);
    text-transform: uppercase;
    color: var(--color-text-faint);
    letter-spacing: 0.08em;
  }

  .sidebar-meta {
    font-size: var(--text-sm);
    color: var(--color-text-muted);
  }

  .explorer {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    overflow-y: auto;
  }

  .schema-block {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .schema-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    background: none;
    border: none;
    color: var(--color-text);
    font-size: var(--text-sm);
    padding: 2px var(--space-2);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .schema-row:hover {
    background: var(--color-accent-subtle);
    color: var(--color-accent);
  }

  .schema-count {
    font-size: var(--text-xs);
    color: var(--color-text-faint);
  }

  .table-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding-left: var(--space-2);
  }

  .table-row {
    background: none;
    border: none;
    text-align: left;
    padding: 2px var(--space-2);
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .table-row.active,
  .table-row:hover {
    color: var(--color-accent);
    background: var(--color-accent-subtle);
  }

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

  .workspace-toolbar {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-4);
    border-bottom: var(--border);
    background: var(--color-bg);
  }

  .toolbar-group {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .toolbar-group :global(input) {
    width: 80px;
  }

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

  @media (max-width: 1000px) {
    .connection-body {
      grid-template-columns: 1fr;
    }
  }
</style>
