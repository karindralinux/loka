<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import Icon from '$lib/components/Icon.svelte';
  import type { GridPage, GridPageRequest, GridUpdateRequest } from '$lib/types.js';

  // ── URL Params ─────────────────────────────────────────
  let runtimeId = $derived(page.url.searchParams.get('rid') ?? '');
  let dbName = $derived(decodeURIComponent(page.url.searchParams.get('db') ?? 'postgres'));
  let connName = $derived(decodeURIComponent(page.url.searchParams.get('name') ?? 'Connection'));
  let connColor = $derived(decodeURIComponent(page.url.searchParams.get('color') ?? '#3ecf8e'));

  // ── Pending Change ─────────────────────────────────────
  interface PendingChange {
    /** Unique key: `rowIdx:colName` */
    key: string;
    rowIdx: number;
    colName: string;
    oldValue: string;
    newValue: string;
    pk: [string, any][];
  }

  // ── Tab State ──────────────────────────────────────────
  interface TabState {
    id: string;
    schema: string;
    table: string;
    grid: GridPage | null;
    limit: number;
    offset: number;
    busy: boolean;
    error: string | null;
    colWidths: Record<string, number>;
    pending: PendingChange[];
  }

  let tabs = $state<TabState[]>([]);
  let activeTabId = $state<string | null>(null);
  let activeTab = $derived(tabs.find(t => t.id === activeTabId) ?? null);

  // ── Sidebar State ──────────────────────────────────────
  let schemas = $state<string[]>(['public']);
  let selectedSchema = $state('public');
  let tableList = $state<string[]>([]);
  let tableSearch = $state('');
  let sidebarCollapsed = $state(false);
  let sidebarBusy = $state(false);

  // ── Right Panel (changes) ──────────────────────────────
  let changesPanelOpen = $state(false);
  let committing = $state(false);

  // ── Column Resize State ────────────────────────────────
  let resizingCol = $state<string | null>(null);
  let resizeStartX = 0;
  let resizeStartW = 0;

  // ── Derived ────────────────────────────────────────────
  let filteredTables = $derived(
    tableSearch.trim()
      ? tableList.filter(t => t.toLowerCase().includes(tableSearch.toLowerCase()))
      : tableList,
  );
  let totalPages = $derived(activeTab?.grid ? Math.ceil(activeTab.grid.row_count / activeTab.limit) : 0);
  let currentPage = $derived(activeTab ? Math.floor(activeTab.offset / activeTab.limit) + 1 : 1);
  let activePending = $derived(activeTab?.pending ?? []);

  // ── Keyboard shortcut (Cmd+S / Ctrl+S) ─────────────────
  onMount(() => {
    function onKeyDown(e: KeyboardEvent) {
      if ((e.metaKey || e.ctrlKey) && e.key === 's') {
        e.preventDefault();
        if (activePending.length > 0) commitChanges();
      }
    }
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  });

  // ── Init ───────────────────────────────────────────────
  $effect(() => { if (runtimeId) loadSchemas(); });
  $effect(() => { if (runtimeId && selectedSchema) loadTables(); });

  // ── Tab Management ─────────────────────────────────────
  function openTab(schema: string, table: string) {
    const existing = tabs.find(t => t.schema === schema && t.table === table);
    if (existing) { activeTabId = existing.id; return; }
    const tab: TabState = {
      id: `${schema}.${table}.${Date.now()}`,
      schema, table, grid: null, limit: 100, offset: 0,
      busy: false, error: null, colWidths: {}, pending: [],
    };
    tabs = [...tabs, tab];
    activeTabId = tab.id;
    loadGridForTab(tab.id);
  }

  function closeTab(id: string) {
    const idx = tabs.findIndex(t => t.id === id);
    if (idx < 0) return;
    const wasActive = activeTabId === id;
    tabs = tabs.filter(t => t.id !== id);
    if (wasActive) activeTabId = tabs[Math.min(idx, tabs.length - 1)]?.id ?? null;
  }

  function closeTabMiddle(e: MouseEvent, id: string) {
    if (e.button === 1) { e.preventDefault(); closeTab(id); }
  }

  // ── API Calls ──────────────────────────────────────────
  async function loadSchemas() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke<{ rows: any[][] }>('pg_execute_sql', {
        runtimeId,
        sql: "SELECT schema_name FROM information_schema.schemata WHERE schema_name NOT IN ('pg_toast','pg_catalog','information_schema') ORDER BY schema_name",
      });
      schemas = result.rows.map((r: any[]) => String(r[0]));
      if (schemas.length > 0 && !schemas.includes(selectedSchema)) selectedSchema = schemas[0];
    } catch { schemas = ['public']; }
  }

  async function loadTables() {
    tableList = [];
    sidebarBusy = true;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      tableList = await invoke<string[]>('pg_list_tables', { runtimeId, schema: selectedSchema });
    } catch (e) { console.error(e); }
    finally { sidebarBusy = false; }
  }

  async function loadGridForTab(tabId: string) {
    const tab = tabs.find(t => t.id === tabId);
    if (!tab || !runtimeId) return;
    tab.error = null; tab.grid = null; tab.busy = true;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const req: GridPageRequest = {
        table: { schema: tab.schema, name: tab.table },
        limit: tab.limit, offset: tab.offset, order_by: null,
      };
      tab.grid = await invoke<GridPage>('pg_grid_page', { runtimeId, req });
    } catch (e) { tab.error = String(e); }
    finally { tab.busy = false; }
  }

  function refreshActiveTab() { if (activeTabId) loadGridForTab(activeTabId); }

  function goToPage(p: number) {
    if (!activeTab) return;
    activeTab.offset = (p - 1) * activeTab.limit;
    loadGridForTab(activeTab.id);
  }

  function changeLimit(newLimit: number) {
    if (!activeTab) return;
    activeTab.limit = newLimit; activeTab.offset = 0;
    loadGridForTab(activeTab.id);
  }

  function pkForRow(grid: GridPage, row: any[]): [string, any][] {
    return grid.pk_columns.map(pk => {
      const idx = grid.columns.findIndex(c => c.name === pk);
      return [pk, idx >= 0 ? row[idx] : null] as [string, any];
    });
  }

  // ── Pending Changes ────────────────────────────────────
  function stageChange(rowIdx: number, row: any[], colName: string, newValue: string) {
    if (!activeTab?.grid) return;
    const grid = activeTab.grid;
    if (grid.pk_columns.length === 0) { activeTab.error = 'No primary key — editing disabled.'; return; }

    const colIdx = grid.columns.findIndex(c => c.name === colName);
    const oldRaw = row[colIdx];
    const oldValue = oldRaw === null ? '' : String(oldRaw);

    // No actual change
    if (newValue === oldValue) return;

    const key = `${rowIdx}:${colName}`;
    const existing = activeTab.pending.findIndex(p => p.key === key);

    const change: PendingChange = {
      key, rowIdx, colName, oldValue, newValue,
      pk: pkForRow(grid, row),
    };

    if (existing >= 0) {
      // If reverted to original, remove the pending change
      if (newValue === activeTab.pending[existing].oldValue) {
        activeTab.pending = activeTab.pending.filter((_, i) => i !== existing);
      } else {
        activeTab.pending[existing] = change;
      }
    } else {
      activeTab.pending = [...activeTab.pending, change];
    }
  }

  function discardChange(key: string) {
    if (!activeTab) return;
    activeTab.pending = activeTab.pending.filter(p => p.key !== key);
  }

  function discardAllChanges() {
    if (!activeTab) return;
    activeTab.pending = [];
  }

  function getPendingValue(rowIdx: number, colName: string): string | undefined {
    return activeTab?.pending.find(p => p.rowIdx === rowIdx && p.colName === colName)?.newValue;
  }

  function hasPending(rowIdx: number, colName: string): boolean {
    return activeTab?.pending.some(p => p.rowIdx === rowIdx && p.colName === colName) ?? false;
  }

  async function commitChanges() {
    if (!activeTab || activeTab.pending.length === 0 || !runtimeId) return;
    committing = true;
    activeTab.error = null;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      for (const change of activeTab.pending) {
        const req: GridUpdateRequest = {
          table: { schema: activeTab.schema, name: activeTab.table },
          pk: change.pk,
          column: change.colName,
          value: change.newValue === '' ? null : change.newValue,
        };
        await invoke<number>('pg_grid_update', { runtimeId, req });
      }
      activeTab.pending = [];
      await loadGridForTab(activeTab.id);
    } catch (e) { activeTab.error = String(e); }
    finally { committing = false; }
  }

  // ── Column Resize ─────────────────────────────────────
  function startResize(e: MouseEvent, colName: string) {
    e.preventDefault();
    resizingCol = colName;
    resizeStartX = e.clientX;
    resizeStartW = activeTab?.colWidths[colName] ?? 150;
    document.body.classList.add('resizing-columns');
    window.addEventListener('mousemove', onResizeMove);
    window.addEventListener('mouseup', onResizeEnd);
  }

  function onResizeMove(e: MouseEvent) {
    if (!resizingCol || !activeTab) return;
    const diff = e.clientX - resizeStartX;
    activeTab.colWidths[resizingCol] = Math.max(60, resizeStartW + diff);
  }

  function onResizeEnd() {
    resizingCol = null;
    document.body.classList.remove('resizing-columns');
    window.removeEventListener('mousemove', onResizeMove);
    window.removeEventListener('mouseup', onResizeEnd);
  }

  function getColWidth(colName: string): number {
    return activeTab?.colWidths[colName] ?? 150;
  }

  function disconnect() { goto('/'); }
</script>

<div class="workspace">
  <!-- Sidebar -->
  <aside class="sidebar" class:collapsed={sidebarCollapsed}>
    <div class="sidebar-header">
      {#if !sidebarCollapsed}
        <div class="sidebar-conn">
          <div class="conn-dot" style="background:{connColor}"></div>
          <div class="sidebar-conn-info">
            <span class="sidebar-conn-name truncate">{connName}</span>
            <span class="sidebar-conn-db truncate">{dbName}</span>
          </div>
        </div>
      {/if}
      <button class="btn btn-ghost btn-icon btn-sm" onclick={() => (sidebarCollapsed = !sidebarCollapsed)}>
        <Icon name={sidebarCollapsed ? 'chevron-right' : 'chevron-left'} size={14} />
      </button>
    </div>

    {#if !sidebarCollapsed}
      <div class="sidebar-section">
        <label class="label" for="schema-select">Schema</label>
        <select id="schema-select" class="select" bind:value={selectedSchema}>
          {#each schemas as s}<option value={s}>{s}</option>{/each}
        </select>
      </div>
      <hr class="divider" />
      <div class="sidebar-section">
        <div class="table-header">
          <span class="label" style="margin-bottom:0">Tables</span>
          <span class="badge badge-neutral">{tableList.length}</span>
        </div>
        {#if tableList.length > 5}
          <div class="table-search">
            <Icon name="search" size={12} class="table-search-icon" />
            <input class="input input-sm table-search-input" placeholder="Filter tables..." bind:value={tableSearch} />
          </div>
        {/if}
      </div>
      <nav class="table-list">
        {#if sidebarBusy && tableList.length === 0}
          {#each Array(5) as _}<div class="skeleton table-skeleton"></div>{/each}
        {:else}
          {#each filteredTables as t (t)}
            <button class="table-item" class:active={activeTab?.table === t && activeTab?.schema === selectedSchema}
              ondblclick={() => openTab(selectedSchema, t)}
              onclick={() => openTab(selectedSchema, t)}>
              <Icon name="table" size={14} /><span class="truncate">{t}</span>
            </button>
          {/each}
          {#if filteredTables.length === 0}
            <div class="empty-tables"><span>{tableSearch ? 'No match' : 'No tables'}</span></div>
          {/if}
        {/if}
      </nav>
      <div class="sidebar-footer">
        <button class="btn btn-ghost btn-sm" style="width:100%;justify-content:flex-start" onclick={disconnect}>
          <Icon name="arrow-left" size={14} /> Disconnect
        </button>
      </div>
    {:else}
      <div class="collapsed-icons">
        <button class="btn btn-ghost btn-icon btn-sm" onclick={() => (sidebarCollapsed = false)}><Icon name="layers" size={16} /></button>
        <button class="btn btn-ghost btn-icon btn-sm" onclick={disconnect}><Icon name="arrow-left" size={16} /></button>
      </div>
    {/if}
  </aside>

  <!-- Main Content -->
  <main class="workspace-main">
    <!-- Tab Bar -->
    {#if tabs.length > 0}
      <div class="tab-bar">
        {#each tabs as tab (tab.id)}
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div class="tab" class:active={activeTabId === tab.id}
            onclick={() => (activeTabId = tab.id)}
            onmousedown={(e) => closeTabMiddle(e, tab.id)}>
            <Icon name="table" size={12} />
            <span class="tab-label">{tab.table}</span>
            <button class="tab-close" onclick={(e) => { e.stopPropagation(); closeTab(tab.id); }}>
              <Icon name="x" size={10} />
            </button>
          </div>
        {/each}
      </div>
    {/if}

    <!-- Toolbar -->
    <div class="toolbar">
      <div class="toolbar-left">
        {#if activeTab}
          <div class="toolbar-breadcrumb">
            <span class="toolbar-schema">{activeTab.schema}</span>
            <Icon name="chevron-right" size={12} />
            <span class="toolbar-table">{activeTab.table}</span>
          </div>
          {#if activeTab.grid}
            <span class="badge badge-neutral">{activeTab.grid.row_count} row{activeTab.grid.row_count !== 1 ? 's' : ''}</span>
            {#if activeTab.grid.pk_columns.length > 0}
              <span class="badge badge-success"><Icon name="key" size={10} />{activeTab.grid.pk_columns.join(', ')}</span>
            {:else}
              <span class="badge badge-warning">No PK</span>
            {/if}
          {/if}
        {:else}
          <span class="toolbar-hint"><Icon name="chevron-left" size={14} />Select a table from the sidebar</span>
        {/if}
      </div>
      <div class="toolbar-right">
        {#if activeTab}
          <button class="btn btn-ghost btn-sm" onclick={refreshActiveTab} disabled={activeTab.busy}><Icon name="refresh" size={14} /></button>
          {#if activePending.length > 0}
            <button class="btn btn-primary btn-sm" onclick={commitChanges} disabled={committing}>
              {#if committing}<span class="spinner"></span>{:else}<Icon name="check" size={14} />{/if}
              Commit ({activePending.length})
            </button>
          {/if}
          <button class="btn btn-ghost btn-sm" class:active-toggle={changesPanelOpen} onclick={() => (changesPanelOpen = !changesPanelOpen)}>
            <Icon name="edit" size={14} />
            {#if activePending.length > 0}<span class="changes-badge">{activePending.length}</span>{/if}
          </button>
        {/if}
      </div>
    </div>

    <!-- Error Banner -->
    {#if activeTab?.error}
      <div style="padding: var(--space-4) var(--space-4) 0 var(--space-4)">
        <div class="error-banner animate-fade-in">
          <Icon name="alert-circle" size={14} class="error-banner-icon" />
          <div class="error-banner-content">{activeTab.error}</div>
          <button class="btn btn-ghost btn-icon btn-sm" onclick={() => { if (activeTab) activeTab.error = null; }}>
            <Icon name="x" size={12} />
          </button>
        </div>
      </div>
    {/if}

    <!-- Grid Area -->
    <div class="grid-area">
      {#if !activeTab}
        <div class="empty-state animate-fade-in">
          <div class="empty-icon"><Icon name="database" size={40} /></div>
          <h3>Select a table</h3>
          <p>Choose a table from the sidebar to browse and edit its data.</p>
        </div>
      {:else if activeTab.busy && !activeTab.grid}
        <div class="loading-state"><span class="spinner" style="width:24px;height:24px;border-width:3px"></span><p>Loading...</p></div>
      {:else if activeTab.grid}
        {@const grid = activeTab.grid}
        <div class="grid-scroll">
          <table class="data-grid" style="table-layout:fixed;width:max-content;min-width:100%">
            <colgroup>
              <col style="width:44px" />
              {#each grid.columns as col}
                <col style="width:{getColWidth(col.name)}px" />
              {/each}
            </colgroup>
            <thead>
              <tr>
                <th class="row-num-header">#</th>
                {#each grid.columns as col}
                  <th style="position:relative">
                    <div class="col-header-content">
                      {#if grid.pk_columns.includes(col.name)}<Icon name="key" size={10} class="pk-icon" />{/if}
                      {col.name}
                    </div>
                    <span class="col-type">{col.type_name}</span>
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <div class="col-resize-handle" class:resizing={resizingCol === col.name}
                      onmousedown={(e) => startResize(e, col.name)}></div>
                  </th>
                {/each}
              </tr>
            </thead>
            <tbody>
              {#each grid.rows as row, rowIdx}
                <tr>
                  <td class="row-num">{activeTab.offset + rowIdx + 1}</td>
                  {#each grid.columns as col, colIdx}
                    {@const pending = hasPending(rowIdx, col.name)}
                    <td class:cell-modified={pending}>
                      {#if grid.pk_columns.length > 0 && !grid.pk_columns.includes(col.name)}
                        <input class="cell-input" class:cell-dirty={pending}
                          value={getPendingValue(rowIdx, col.name) ?? (row[colIdx] === null ? '' : String(row[colIdx]))}
                          onblur={(e) => stageChange(rowIdx, row, col.name, (e.currentTarget as HTMLInputElement).value)} />
                      {:else}
                        <div class="cell-readonly" class:cell-null={row[colIdx] === null} class:cell-pk={grid.pk_columns.includes(col.name)}>
                          {row[colIdx] === null ? 'NULL' : String(row[colIdx])}
                        </div>
                      {/if}
                    </td>
                  {/each}
                </tr>
              {/each}
            </tbody>
          </table>
        </div>

        <!-- Pagination -->
        <div class="grid-footer">
          <div class="footer-left">
            <span class="footer-info">Showing {activeTab.offset + 1}–{Math.min(activeTab.offset + activeTab.limit, grid.row_count)} of {grid.row_count}</span>
          </div>
          <div class="footer-center">
            <button class="btn btn-ghost btn-sm" disabled={currentPage <= 1} onclick={() => goToPage(currentPage - 1)}><Icon name="chevron-left" size={14} /></button>
            <span class="page-indicator">Page {currentPage} of {totalPages || 1}</span>
            <button class="btn btn-ghost btn-sm" disabled={currentPage >= totalPages} onclick={() => goToPage(currentPage + 1)}><Icon name="chevron-right" size={14} /></button>
          </div>
          <div class="footer-right">
            <label class="footer-limit"><span>Rows:</span>
              <select class="select input-sm" value={activeTab.limit} onchange={(e) => changeLimit(Number((e.currentTarget as HTMLSelectElement).value))}>
                <option value={25}>25</option><option value={50}>50</option><option value={100}>100</option><option value={250}>250</option><option value={500}>500</option>
              </select>
            </label>
          </div>
        </div>
      {/if}
    </div>
  </main>

  <!-- Right Panel: Pending Changes -->
  {#if changesPanelOpen}
    <aside class="changes-panel animate-slide-in-right">
      <div class="changes-panel-header">
        <h4>Pending Changes</h4>
        <button class="btn btn-ghost btn-icon btn-sm" onclick={() => (changesPanelOpen = false)}>
          <Icon name="x" size={14} />
        </button>
      </div>

      {#if activePending.length === 0}
        <div class="changes-empty">
          <Icon name="check" size={20} />
          <p>No pending changes</p>
        </div>
      {:else}
        <div class="changes-list">
          {#each activePending as change (change.key)}
            <div class="change-item">
              <div class="change-header">
                <span class="change-col">{change.colName}</span>
                <span class="change-row">Row {change.rowIdx + 1}</span>
                <button class="btn btn-ghost btn-icon btn-sm" onclick={() => discardChange(change.key)}>
                  <Icon name="x" size={12} />
                </button>
              </div>
              <div class="change-diff">
                <div class="change-old"><span class="diff-label">−</span><span class="diff-value">{change.oldValue || 'NULL'}</span></div>
                <div class="change-new"><span class="diff-label">+</span><span class="diff-value">{change.newValue || 'NULL'}</span></div>
              </div>
            </div>
          {/each}
        </div>

        <div class="changes-panel-footer">
          <button class="btn btn-ghost btn-sm" onclick={discardAllChanges}>
            <Icon name="trash" size={14} /> Discard All
          </button>
          <button class="btn btn-primary btn-sm" onclick={commitChanges} disabled={committing}>
            {#if committing}<span class="spinner"></span>{:else}<Icon name="check" size={14} />{/if}
            Commit All
          </button>
        </div>
      {/if}
    </aside>
  {/if}
</div>

<style>
  .workspace { width:100vw;height:100vh;display:flex;overflow:hidden;background:var(--bg-root) }
  .sidebar { width:var(--sidebar-width);height:100vh;display:flex;flex-direction:column;background:var(--bg-surface-100);border-right:1px solid var(--border-default);transition:width var(--transition-slow);flex-shrink:0;overflow:hidden }
  .sidebar.collapsed { width:44px }
  .sidebar-header { display:flex;align-items:center;justify-content:space-between;padding:var(--space-3);border-bottom:1px solid var(--border-default);min-height:52px }
  .sidebar-conn { display:flex;align-items:center;gap:var(--space-2);min-width:0;flex:1 }
  .conn-dot { width:10px;height:10px;border-radius:50%;flex-shrink:0;box-shadow:0 0 8px currentColor }
  .sidebar-conn-info { display:flex;flex-direction:column;min-width:0 }
  .sidebar-conn-name { font-size:var(--text-sm);font-weight:600;color:var(--text-primary);line-height:1.2 }
  .sidebar-conn-db { font-size:var(--text-xs);color:var(--text-tertiary);font-family:var(--font-mono) }
  .sidebar-section { padding:var(--space-3) }
  .table-header { display:flex;align-items:center;justify-content:space-between;margin-bottom:var(--space-2) }
  .table-search { position:relative }
  .table-search :global(.table-search-icon) { position:absolute;left:var(--space-2);top:50%;transform:translateY(-50%);color:var(--text-tertiary);pointer-events:none }
  :global(.table-search-input) { padding-left:var(--space-6) !important }
  .table-list { flex:1;overflow-y:auto;padding:0 var(--space-2) }
  .table-item { width:100%;display:flex;align-items:center;gap:var(--space-2);padding:var(--space-2) var(--space-3);background:transparent;border:none;border-radius:var(--radius-md);color:var(--text-secondary);font-family:var(--font-sans);font-size:var(--text-sm);cursor:pointer;transition:all var(--transition-fast);text-align:left }
  .table-item:hover { background:var(--bg-surface-300);color:var(--text-primary) }
  .table-item.active { background:var(--brand-primary-subtle);color:var(--brand-primary);font-weight:500 }
  .table-skeleton { height:32px;margin:var(--space-1) var(--space-2) }
  .empty-tables { display:flex;flex-direction:column;align-items:center;gap:var(--space-2);padding:var(--space-6);color:var(--text-disabled);font-size:var(--text-xs);text-align:center }
  .sidebar-footer { padding:var(--space-2);border-top:1px solid var(--border-default) }
  .collapsed-icons { display:flex;flex-direction:column;align-items:center;gap:var(--space-2);padding:var(--space-2);flex:1 }
  .workspace-main { flex:1;display:flex;flex-direction:column;overflow:hidden;min-width:0 }
  .toolbar { display:flex;align-items:center;justify-content:space-between;padding:var(--space-2) var(--space-4);background:var(--bg-surface-100);border-bottom:1px solid var(--border-default);min-height:44px;gap:var(--space-3) }
  .toolbar-left { display:flex;align-items:center;gap:var(--space-3);min-width:0 }
  .toolbar-right { display:flex;align-items:center;gap:var(--space-2) }
  .toolbar-breadcrumb { display:flex;align-items:center;gap:var(--space-1);font-size:var(--text-sm) }
  .toolbar-schema { color:var(--text-tertiary);font-family:var(--font-mono) }
  .toolbar-table { color:var(--text-primary);font-weight:600;font-family:var(--font-mono) }
  .toolbar-hint { display:flex;align-items:center;gap:var(--space-1);color:var(--text-tertiary);font-size:var(--text-sm) }
  .error-bar { display:flex;align-items:center;gap:var(--space-2);padding:var(--space-2) var(--space-4);background:var(--status-error-bg);border-bottom:1px solid rgba(244,63,94,0.2);color:var(--status-error);font-size:var(--text-sm) }
  .error-bar span { flex:1 }
  .grid-area { flex:1;display:flex;flex-direction:column;overflow:hidden }
  .grid-scroll { flex:1;overflow:auto }
  .empty-state { flex:1;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:var(--space-3);color:var(--text-tertiary) }
  .empty-icon { width:80px;height:80px;display:flex;align-items:center;justify-content:center;background:var(--bg-surface-200);border-radius:var(--radius-2xl);color:var(--text-disabled);margin-bottom:var(--space-2) }
  .empty-state h3 { font-size:var(--text-lg);color:var(--text-secondary) }
  .empty-state p { font-size:var(--text-sm);color:var(--text-tertiary);max-width:320px;text-align:center }
  .loading-state { flex:1;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:var(--space-3);color:var(--text-tertiary);font-size:var(--text-sm) }
  .grid-footer { display:flex;align-items:center;justify-content:space-between;padding:var(--space-2) var(--space-4);background:var(--bg-surface-100);border-top:1px solid var(--border-default);min-height:40px;gap:var(--space-4) }
  .footer-left,.footer-right { flex:1 }
  .footer-right { display:flex;justify-content:flex-end }
  .footer-center { display:flex;align-items:center;gap:var(--space-2) }
  .footer-info { font-size:var(--text-xs);color:var(--text-tertiary);font-family:var(--font-mono) }
  .page-indicator { font-size:var(--text-xs);color:var(--text-secondary);white-space:nowrap;min-width:100px;text-align:center }
  .footer-limit { display:flex;align-items:center;gap:var(--space-2);font-size:var(--text-xs);color:var(--text-tertiary) }
  .footer-limit .select { width:auto;min-width:60px;height:26px;font-size:var(--text-xs) }
  .col-header-content { display:flex;align-items:center;gap:4px }
  :global(.pk-icon) { color:var(--status-warning) }
  .row-num-header { width:44px;min-width:44px;text-align:right;padding-right:var(--space-3) !important }

  /* ── Changes badge on toolbar button ─────────────────── */
  .active-toggle { background:var(--bg-surface-300);color:var(--text-primary) }
  .changes-badge { display:inline-flex;align-items:center;justify-content:center;min-width:16px;height:16px;padding:0 4px;font-size:10px;font-weight:700;background:var(--brand-primary);color:var(--text-inverse);border-radius:var(--radius-full);line-height:1 }

  /* ── Modified cell highlight ─────────────────────────── */
  :global(.cell-modified) { background:rgba(62,207,142,0.06) !important }
  :global(.cell-dirty) { color:var(--brand-primary) !important;background:rgba(62,207,142,0.08) !important }

  /* ── Changes Panel (right sidebar) ──────────────────── */
  .changes-panel { width:280px;height:100vh;display:flex;flex-direction:column;background:var(--bg-surface-100);border-left:1px solid var(--border-default);flex-shrink:0;overflow:hidden }
  .changes-panel-header { display:flex;align-items:center;justify-content:space-between;padding:var(--space-3) var(--space-4);border-bottom:1px solid var(--border-default);min-height:52px }
  .changes-panel-header h4 { font-size:var(--text-sm);font-weight:600;color:var(--text-primary) }
  .changes-empty { flex:1;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:var(--space-2);color:var(--text-disabled);font-size:var(--text-sm) }
  .changes-list { flex:1;overflow-y:auto;padding:var(--space-2) }
  .change-item { background:var(--bg-surface-200);border:1px solid var(--border-default);border-radius:var(--radius-md);margin-bottom:var(--space-2);overflow:hidden }
  .change-header { display:flex;align-items:center;gap:var(--space-2);padding:var(--space-2) var(--space-3) }
  .change-col { font-size:var(--text-xs);font-weight:600;color:var(--text-primary);font-family:var(--font-mono) }
  .change-row { font-size:10px;color:var(--text-disabled);flex:1 }
  .change-diff { padding:0 var(--space-3) var(--space-2) }
  .change-old,.change-new { display:flex;align-items:baseline;gap:var(--space-1);font-size:var(--text-xs);font-family:var(--font-mono);padding:2px 0;border-radius:2px }
  .change-old { color:var(--status-error) }
  .change-new { color:var(--status-success) }
  .diff-label { font-weight:700;width:12px;flex-shrink:0 }
  .diff-value { word-break:break-all }
  .changes-panel-footer { display:flex;justify-content:space-between;padding:var(--space-3);border-top:1px solid var(--border-default);gap:var(--space-2) }

  /* slide-in animation */
  .animate-slide-in-right { animation:slideInRight 200ms ease forwards }
  @keyframes slideInRight { from { opacity:0;transform:translateX(16px) } to { opacity:1;transform:translateX(0) } }
</style>
