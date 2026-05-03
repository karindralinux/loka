<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    title?: string;
    children: Snippet;
  }

  let { title, children }: Props = $props();
</script>

<section class="panel">
  {#if title}
    <header class="panel-header">
      <span class="panel-title">{title}</span>
    </header>
  {/if}
  <div class="panel-body">
    {@render children()}
  </div>
</section>

<style>
  .panel {
    background: var(--color-bg-elevated);
    border: var(--border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-sm);
    overflow: hidden;
  }

  .panel-header {
    padding: var(--space-2) var(--space-4);
    border-bottom: var(--border);
    background: var(--color-th-bg);
  }

  .panel-title {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .panel-body {
    padding: var(--space-4);
  }
</style>
