<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    variant?: "default" | "primary";
    disabled?: boolean;
    type?: "button" | "submit" | "reset";
    onclick?: (e: MouseEvent) => void;
    children: Snippet;
    [key: string]: unknown;
  }

  let {
    variant = "default",
    disabled = false,
    type = "button",
    onclick,
    children,
    ...rest
  }: Props = $props();
</script>

<button
  {type}
  {disabled}
  class="btn btn-{variant}"
  onclick={onclick}
  {...rest}
>
  {@render children()}
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-1);
    padding: 4px var(--space-3);
    font-size: var(--text-sm);
    font-weight: 500;
    border-radius: var(--radius-sm);
    border: var(--border);
    cursor: pointer;
    transition: background 0.1s, opacity 0.1s;
    white-space: nowrap;
    line-height: 1.6;
  }

  .btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .btn-default {
    background: var(--color-btn-bg);
    color: var(--color-btn-text);
    border-color: var(--color-input-border);
  }

  .btn-default:hover:not(:disabled) {
    background: var(--color-btn-hover);
  }

  .btn-primary {
    background: var(--color-btn-primary-bg);
    color: var(--color-btn-primary-text);
    border-color: transparent;
  }

  .btn-primary:hover:not(:disabled) {
    opacity: 0.88;
  }
</style>
