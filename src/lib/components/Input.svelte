<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";

  interface Props extends HTMLInputAttributes {
    value?: string | number;
    class?: string;
  }

  let {
    value = $bindable(""),
    type = "text",
    placeholder = "",
    id,
    disabled = false,
    readonly = false,
    class: extraClass = "",
    onblur,
    oninput,
    ...rest
  }: Props = $props();
</script>

<input
  {id}
  {type}
  {placeholder}
  {disabled}
  {readonly}
  class="input {extraClass}"
  bind:value
  onblur={onblur}
  oninput={oninput}
  {...rest}
/>

<style>
  .input {
    width: 100%;
    padding: 4px var(--space-2);
    font-size: var(--text-base);
    color: var(--color-text);
    background: var(--color-input-bg);
    border: 1px solid var(--color-input-border);
    border-radius: var(--radius-sm);
    line-height: 1.5;
    transition: border-color 0.15s;
    outline: none;
  }

  .input:focus {
    border-color: var(--color-accent);
    box-shadow: 0 0 0 2px var(--color-accent-subtle);
  }

  .input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
