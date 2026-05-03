<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    sidebar: Snippet;
    children: Snippet;
  }

  let { sidebar, children }: Props = $props();
</script>

<div class="shell">
  <aside class="sidebar">
    {@render sidebar()}
  </aside>

  <main class="main">
    {@render children()}
  </main>
</div>

<style>
  .shell {
    display: flex;
    height: 100%;
    width: 100%;
    overflow: hidden;
    background: var(--color-bg);
  }

  .sidebar {
    width: var(--sidebar-width);
    min-width: var(--sidebar-width);
    max-width: var(--sidebar-width);
    display: flex;
    flex-direction: column;
    background: var(--color-sidebar);
    border-right: var(--border);
    overflow-y: auto;
    overflow-x: hidden;
  }

  .main {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--color-bg);
  }
</style>
