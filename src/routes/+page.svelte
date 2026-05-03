<script lang="ts">
  import { goto } from '$app/navigation';
  import Icon from '$lib/components/Icon.svelte';
  import type { ConnectionConfig, SavedConnection } from '$lib/types.js';
  import { CONNECTION_COLORS } from '$lib/types.js';
  import {
    getConnections,
    addConnection,
    removeConnection,
    touchConnection,
    updateConnection,
  } from '$lib/connections.js';

  // ── State ──────────────────────────────────────────────
  let connections = $state<SavedConnection[]>([]);
  let showForm = $state(false);
  let editingId = $state<string | null>(null);
  let connecting = $state(false);
  let error = $state<string | null>(null);
  let searchQuery = $state('');

  // Form state
  let formName = $state('');
  let formHost = $state('127.0.0.1');
  let formPort = $state(5432);
  let formDatabase = $state('postgres');
  let formUsername = $state('postgres');
  let formPassword = $state('');
  let formSslMode = $state('disable');
  let formColor = $state(CONNECTION_COLORS[0]);

  // ── Init ───────────────────────────────────────────────
  $effect(() => {
    connections = getConnections();
  });

  // ── Derived ────────────────────────────────────────────
  let filteredConnections = $derived(
    searchQuery.trim()
      ? connections.filter(
          (c) =>
            c.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
            c.config.host.toLowerCase().includes(searchQuery.toLowerCase()) ||
            c.config.database.toLowerCase().includes(searchQuery.toLowerCase()),
        )
      : connections,
  );

  // ── Actions ────────────────────────────────────────────
  function resetForm() {
    formName = '';
    formHost = '127.0.0.1';
    formPort = 5432;
    formDatabase = 'postgres';
    formUsername = 'postgres';
    formPassword = '';
    formSslMode = 'disable';
    formColor = CONNECTION_COLORS[connections.length % CONNECTION_COLORS.length];
    editingId = null;
    error = null;
  }

  function openNewForm() {
    resetForm();
    showForm = true;
  }

  function openEditForm(conn: SavedConnection) {
    formName = conn.name;
    formHost = conn.config.host;
    formPort = conn.config.port;
    formDatabase = conn.config.database;
    formUsername = conn.config.username;
    formPassword = conn.config.password;
    formSslMode = conn.config.ssl_mode;
    formColor = conn.color;
    editingId = conn.id;
    showForm = true;
    error = null;
  }

  function closeForm() {
    showForm = false;
    resetForm();
  }

  function saveConnection() {
    const config: ConnectionConfig = {
      engine: 'Postgres',
      host: formHost,
      port: formPort,
      database: formDatabase,
      username: formUsername,
      password: formPassword,
      ssl_mode: formSslMode,
    };
    const name = formName || `${formHost}:${formPort}/${formDatabase}`;

    if (editingId) {
      updateConnection(editingId, name, config);
    } else {
      addConnection(name, config);
    }
    connections = getConnections();
    closeForm();
  }

  function deleteConnection(id: string) {
    removeConnection(id);
    connections = getConnections();
  }

  async function connectAndOpen(conn: SavedConnection) {
    error = null;
    connecting = true;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const r = await invoke<{ runtime_id: string }>('pg_connect', {
        cfg: conn.config,
      });
      touchConnection(conn.id);
      // Navigate to workspace with runtime id
      goto(`/workspace?rid=${r.runtime_id}&db=${encodeURIComponent(conn.config.database)}&name=${encodeURIComponent(conn.name)}&color=${encodeURIComponent(conn.color)}`);
    } catch (e) {
      error = String(e);
    } finally {
      connecting = false;
    }
  }

  async function quickConnect() {
    const config: ConnectionConfig = {
      engine: 'Postgres',
      host: formHost,
      port: formPort,
      database: formDatabase,
      username: formUsername,
      password: formPassword,
      ssl_mode: formSslMode,
    };

    // Save it first
    const name = formName || `${formHost}:${formPort}/${formDatabase}`;
    const saved = addConnection(name, config);
    connections = getConnections();

    // Then connect
    await connectAndOpen(saved);
  }

  function timeAgo(ts: number | null): string {
    if (!ts) return 'Never';
    const diff = Date.now() - ts;
    const mins = Math.floor(diff / 60000);
    if (mins < 1) return 'Just now';
    if (mins < 60) return `${mins}m ago`;
    const hours = Math.floor(mins / 60);
    if (hours < 24) return `${hours}h ago`;
    const days = Math.floor(hours / 24);
    return `${days}d ago`;
  }
</script>

<div class="connection-page">
  <!-- Background gradient orbs -->
  <div class="bg-orbs">
    <div class="orb orb-1"></div>
    <div class="orb orb-2"></div>
    <div class="orb orb-3"></div>
  </div>

  <div class="connection-content">
    <!-- Header / Branding -->
    <header class="connection-header animate-fade-in-up">
      <div class="logo-mark">
        <Icon name="database" size={28} />
      </div>
      <h1 class="logo-text">Loka</h1>
      <p class="logo-tagline">Modern Database Management</p>
    </header>

    <!-- Main area: Form + Saved Connections side by side (or stacked) -->
    <div class="connection-grid">
      <!-- New Connection Form -->
      <div class="connection-form-card card-glass animate-fade-in-up stagger-1" class:expanded={showForm || connections.length === 0}>
        <div class="form-header">
          <div class="form-title-row">
            <Icon name="plus" size={18} />
            <h3>{editingId ? 'Edit Connection' : 'New Connection'}</h3>
          </div>
          {#if showForm && connections.length > 0}
            <button class="btn btn-ghost btn-sm" onclick={closeForm}>
              <Icon name="x" size={14} />
            </button>
          {/if}
        </div>

        {#if showForm || connections.length === 0}
          <div class="form-body" >
            <!-- Connection name + color -->
            <div class="form-row">
              <div class="form-field" style="flex:1">
                <label class="label" for="conn-name">Connection Name</label>
                <input
                  id="conn-name"
                  class="input"
                  placeholder="My Database"
                  bind:value={formName}
                />
              </div>
              <div class="form-field color-field">
                <label class="label">Color</label>
                <div class="color-picker">
                  {#each CONNECTION_COLORS as c}
                    <button
                      class="color-dot"
                      class:active={formColor === c}
                      style="--dot-color: {c}"
                      onclick={() => (formColor = c)}
                      aria-label="Select color {c}"
                    ></button>
                  {/each}
                </div>
              </div>
            </div>

            <!-- Host / Port -->
            <div class="form-row">
              <div class="form-field" style="flex:2">
                <label class="label" for="conn-host">Host</label>
                <div class="input-icon-wrapper">
                  <Icon name="server" size={14} class="input-icon" />
                  <input
                    id="conn-host"
                    class="input input-with-icon"
                    placeholder="127.0.0.1"
                    bind:value={formHost}
                  />
                </div>
              </div>
              <div class="form-field" style="flex:1; max-width:120px">
                <label class="label" for="conn-port">Port</label>
                <input
                  id="conn-port"
                  class="input"
                  type="number"
                  bind:value={formPort}
                />
              </div>
            </div>

            <!-- Database -->
            <div class="form-row">
              <div class="form-field" style="flex:1">
                <label class="label" for="conn-database">Database</label>
                <div class="input-icon-wrapper">
                  <Icon name="database" size={14} class="input-icon" />
                  <input
                    id="conn-database"
                    class="input input-with-icon"
                    placeholder="postgres"
                    bind:value={formDatabase}
                  />
                </div>
              </div>
            </div>

            <!-- Username / Password -->
            <div class="form-row">
              <div class="form-field" style="flex:1">
                <label class="label" for="conn-user">Username</label>
                <div class="input-icon-wrapper">
                  <Icon name="key" size={14} class="input-icon" />
                  <input
                    id="conn-user"
                    class="input input-with-icon"
                    placeholder="postgres"
                    bind:value={formUsername}
                  />
                </div>
              </div>
              <div class="form-field" style="flex:1">
                <label class="label" for="conn-pass">Password</label>
                <div class="input-icon-wrapper">
                  <Icon name="lock" size={14} class="input-icon" />
                  <input
                    id="conn-pass"
                    class="input input-with-icon"
                    type="password"
                    placeholder="••••••••"
                    bind:value={formPassword}
                  />
                </div>
              </div>
            </div>

            <!-- SSL Mode -->
            <div class="form-row">
              <div class="form-field" style="flex:1">
                <label class="label" for="conn-ssl">SSL Mode</label>
                <select id="conn-ssl" class="select" bind:value={formSslMode}>
                  <option value="disable">Disable</option>
                  <option value="prefer">Prefer</option>
                  <option value="require">Require</option>
                  <option value="verify-ca">Verify CA</option>
                  <option value="verify-full">Verify Full</option>
                </select>
              </div>
            </div>

            {#if error}
              <div class="error-banner animate-fade-in" style="margin-top: var(--space-2)">
                <Icon name="alert-circle" size={14} class="error-banner-icon" />
                <div class="error-banner-content">{error}</div>
                <button class="btn btn-ghost btn-icon btn-sm" onclick={() => (error = null)}>
                  <Icon name="x" size={12} />
                </button>
              </div>
            {/if}

            <!-- Actions -->
            <div class="form-actions">
              {#if editingId}
                <button class="btn btn-secondary" onclick={closeForm}>Cancel</button>
                <button class="btn btn-primary" onclick={saveConnection}>
                  <Icon name="check" size={14} />
                  Save Changes
                </button>
              {:else}
                <button class="btn btn-secondary" onclick={saveConnection}>
                  <Icon name="download" size={14} />
                  Save
                </button>
                <button
                  class="btn btn-primary"
                  onclick={quickConnect}
                  disabled={connecting}
                >
                  {#if connecting}
                    <span class="spinner"></span>
                  {:else}
                    <Icon name="zap" size={14} />
                  {/if}
                  Save & Connect
                </button>
              {/if}
            </div>
          </div>
        {:else}
          <button class="form-expand-btn" onclick={openNewForm}>
            <div class="expand-plus">
              <Icon name="plus" size={20} />
            </div>
            <span>Create new connection</span>
          </button>
        {/if}
      </div>

      <!-- Saved Connections List -->
      {#if connections.length > 0}
        <div class="saved-connections animate-fade-in-up stagger-2">
          <div class="saved-header">
            <h3>Saved Connections</h3>
            <span class="badge badge-neutral">{connections.length}</span>
          </div>

          {#if connections.length > 3}
            <div class="search-bar">
              <Icon name="search" size={14} class="search-icon" />
              <input
                class="input input-sm search-input"
                placeholder="Search connections..."
                bind:value={searchQuery}
              />
            </div>
          {/if}

          <div class="connections-list">
            {#each filteredConnections as conn, idx (conn.id)}
              <div
                class="connection-item animate-fade-in-up"
                style="animation-delay: {idx * 40}ms"
              >
                <button
                  class="connection-item-main"
                  onclick={() => connectAndOpen(conn)}
                  disabled={connecting}
                >
                  <div class="conn-color-bar" style="background: {conn.color}"></div>
                  <div class="conn-info">
                    <div class="conn-name">{conn.name}</div>
                    <div class="conn-details">
                      <span class="conn-detail-item">
                        <Icon name="server" size={11} />
                        {conn.config.host}:{conn.config.port}
                      </span>
                      <span class="conn-detail-item">
                        <Icon name="database" size={11} />
                        {conn.config.database}
                      </span>
                      <span class="conn-detail-item">
                        <Icon name="key" size={11} />
                        {conn.config.username}
                      </span>
                    </div>
                  </div>
                  <div class="conn-meta">
                    <span class="conn-engine badge badge-info">PostgreSQL</span>
                    {#if conn.lastUsedAt}
                      <span class="conn-last-used">
                        <Icon name="clock" size={11} />
                        {timeAgo(conn.lastUsedAt)}
                      </span>
                    {/if}
                  </div>
                </button>
                <div class="conn-actions">
                  <button
                    class="btn btn-ghost btn-icon btn-sm"
                    data-tooltip="Edit"
                    onclick={() => openEditForm(conn)}
                  >
                    <Icon name="edit" size={13} />
                  </button>
                  <button
                    class="btn btn-danger btn-icon btn-sm"
                    data-tooltip="Delete"
                    onclick={() => deleteConnection(conn.id)}
                  >
                    <Icon name="trash" size={13} />
                  </button>
                </div>
              </div>
            {/each}

            {#if filteredConnections.length === 0 && searchQuery}
              <div class="empty-search">
                <Icon name="search" size={20} />
                <p>No connections match "{searchQuery}"</p>
              </div>
            {/if}
          </div>
        </div>
      {/if}
    </div>

    <!-- Footer -->
    <footer class="connection-footer animate-fade-in stagger-5">
      <span>Loka v0.1.0</span>
      <span class="footer-dot">·</span>
      <span>PostgreSQL</span>
    </footer>
  </div>
</div>

<style>
  /* ── Page Layout ─────────────────────────────────────── */
  .connection-page {
    width: 100vw;
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    overflow: hidden;
  }

  /* ── Background Orbs ─────────────────────────────────── */
  .bg-orbs {
    position: absolute;
    inset: 0;
    overflow: hidden;
    pointer-events: none;
  }
  .orb {
    position: absolute;
    border-radius: 50%;
    filter: blur(100px);
    opacity: 0.15;
  }
  .orb-1 {
    width: 500px;
    height: 500px;
    background: var(--brand-primary);
    top: -150px;
    right: -100px;
    animation: float 8s ease-in-out infinite;
  }
  .orb-2 {
    width: 400px;
    height: 400px;
    background: var(--brand-secondary);
    bottom: -100px;
    left: -100px;
    animation: float 10s ease-in-out infinite 1s;
  }
  .orb-3 {
    width: 300px;
    height: 300px;
    background: var(--brand-accent);
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    animation: float 12s ease-in-out infinite 2s;
  }

  /* ── Content ─────────────────────────────────────────── */
  .connection-content {
    position: relative;
    z-index: 1;
    width: 100%;
    max-width: 720px;
    max-height: 100vh;
    overflow-y: auto;
    padding: var(--space-8) var(--space-6);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-6);
  }

  /* ── Header ──────────────────────────────────────────── */
  .connection-header {
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
  }
  .logo-mark {
    width: 56px;
    height: 56px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, var(--brand-primary), #2dd4a8);
    border-radius: var(--radius-xl);
    color: var(--bg-root);
    box-shadow: 0 0 30px rgba(62, 207, 142, 0.25);
    margin-bottom: var(--space-2);
  }
  .logo-text {
    font-size: var(--text-3xl);
    font-weight: 700;
    background: linear-gradient(135deg, var(--text-primary) 0%, var(--text-secondary) 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }
  .logo-tagline {
    font-size: var(--text-sm);
    color: var(--text-tertiary);
    letter-spacing: 0.02em;
  }

  /* ── Grid ────────────────────────────────────────────── */
  .connection-grid {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  /* ── Form Card ───────────────────────────────────────── */
  .connection-form-card {
    padding: var(--space-5);
    width: 100%;
  }
  .form-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-4);
  }
  .form-title-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    color: var(--text-primary);
  }
  .form-title-row h3 {
    font-size: var(--text-md);
    font-weight: 600;
  }
  .form-body {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    animation: fadeIn 200ms ease;
  }
  .form-row {
    display: flex;
    gap: var(--space-3);
  }
  .form-field {
    display: flex;
    flex-direction: column;
  }

  /* Input with icon */
  .input-icon-wrapper {
    position: relative;
  }
  .input-icon-wrapper :global(.input-icon) {
    position: absolute;
    left: var(--space-3);
    top: 50%;
    transform: translateY(-50%);
    color: var(--text-tertiary);
    pointer-events: none;
  }
  .input-with-icon {
    padding-left: var(--space-8) !important;
  }

  /* Color picker */
  .color-field {
    min-width: 120px;
  }
  .color-picker {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    padding-top: 4px;
  }
  .color-dot {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    border: 2px solid transparent;
    background: var(--dot-color);
    cursor: pointer;
    transition: all var(--transition-fast);
    padding: 0;
  }
  .color-dot:hover {
    transform: scale(1.2);
  }
  .color-dot.active {
    border-color: var(--text-primary);
    box-shadow: 0 0 0 2px var(--bg-root), 0 0 0 4px var(--dot-color);
  }

  /* Expand button (collapsed form) */
  .form-expand-btn {
    width: 100%;
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-4);
    background: transparent;
    border: 1px dashed var(--border-default);
    border-radius: var(--radius-lg);
    color: var(--text-tertiary);
    font-size: var(--text-sm);
    cursor: pointer;
    transition: all var(--transition-base);
    font-family: var(--font-sans);
  }
  .form-expand-btn:hover {
    border-color: var(--brand-primary);
    color: var(--brand-primary);
    background: var(--brand-primary-subtle);
  }
  .expand-plus {
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-surface-300);
    border-radius: var(--radius-md);
    transition: all var(--transition-base);
  }
  .form-expand-btn:hover .expand-plus {
    background: var(--brand-primary);
    color: var(--bg-root);
  }

  /* Error */
  .form-error {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-3);
    background: var(--status-error-bg);
    border: 1px solid rgba(244, 63, 94, 0.2);
    border-radius: var(--radius-md);
    color: var(--status-error);
    font-size: var(--text-sm);
  }

  /* Actions */
  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-3);
    padding-top: var(--space-2);
  }

  /* ── Saved Connections ───────────────────────────────── */
  .saved-connections {
    width: 100%;
  }
  .saved-header {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    margin-bottom: var(--space-4);
  }
  .saved-header h3 {
    font-size: var(--text-md);
    font-weight: 600;
    color: var(--text-primary);
  }

  .search-bar {
    position: relative;
    margin-bottom: var(--space-3);
  }
  .search-bar :global(.search-icon) {
    position: absolute;
    left: var(--space-3);
    top: 50%;
    transform: translateY(-50%);
    color: var(--text-tertiary);
    pointer-events: none;
  }
  :global(.search-input) {
    padding-left: var(--space-8) !important;
  }

  .connections-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  /* ── Connection Item ──────────────────────────────────── */
  .connection-item {
    display: flex;
    align-items: center;
    background: var(--bg-surface-200);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-lg);
    overflow: hidden;
    transition: all var(--transition-base);
  }
  .connection-item:hover {
    border-color: var(--border-hover);
    box-shadow: var(--shadow-md);
    transform: translateY(-1px);
  }

  .connection-item-main {
    flex: 1;
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    background: transparent;
    border: none;
    color: var(--text-primary);
    cursor: pointer;
    text-align: left;
    font-family: var(--font-sans);
    transition: background var(--transition-fast);
    min-width: 0;
  }
  .connection-item-main:hover:not(:disabled) {
    background: var(--bg-surface-300);
  }
  .connection-item-main:disabled {
    opacity: 0.6;
    cursor: wait;
  }

  .conn-color-bar {
    width: 4px;
    height: 40px;
    border-radius: var(--radius-full);
    flex-shrink: 0;
  }

  .conn-info {
    flex: 1;
    min-width: 0;
  }
  .conn-name {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .conn-details {
    display: flex;
    gap: var(--space-3);
    margin-top: 2px;
    flex-wrap: wrap;
  }
  .conn-detail-item {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    font-family: var(--font-mono);
  }

  .conn-meta {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 4px;
    flex-shrink: 0;
  }
  .conn-engine {
    font-size: 10px;
    padding: 1px 6px;
  }
  .conn-last-used {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    font-size: 10px;
    color: var(--text-disabled);
  }

  .conn-actions {
    display: flex;
    gap: 2px;
    padding: 0 var(--space-2);
    opacity: 0;
    transition: opacity var(--transition-fast);
  }
  .connection-item:hover .conn-actions {
    opacity: 1;
  }

  /* Empty search */
  .empty-search {
    text-align: center;
    padding: var(--space-8);
    color: var(--text-tertiary);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
  }
  .empty-search p {
    font-size: var(--text-sm);
  }

  /* ── Footer ──────────────────────────────────────────── */
  .connection-footer {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-xs);
    color: var(--text-disabled);
    padding-top: var(--space-4);
  }
  .footer-dot {
    opacity: 0.3;
  }
</style>
