<script lang="ts">
	import { onMount } from "svelte";
	import favicon from "$lib/assets/favicon.svg";
	import "$lib/ui/tokens.css";
	import "$lib/ui/base.css";
	import { initTheme, getTheme, toggleTheme } from "$lib/stores/theme";

	let { children } = $props();

	let theme = $state<"light" | "dark">("light");

	onMount(() => {
		initTheme();
		theme = getTheme();
	});

	function handleToggle() {
		theme = toggleTheme();
	}
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<div class="layout-root">
	<div class="titlebar">
		<span class="app-name">Loka</span>
		<button class="theme-toggle" onclick={handleToggle} title="Toggle theme">
			{theme === "dark" ? "☀︎" : "☾"}
		</button>
	</div>
	{@render children()}
</div>

<style>
	.layout-root {
		display: flex;
		flex-direction: column;
		height: 100vh;
		width: 100vw;
		overflow: hidden;
	}

	.titlebar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		height: var(--topbar-height);
		min-height: var(--topbar-height);
		padding: 0 var(--space-4);
		background: var(--color-sidebar);
		border-bottom: var(--border);
		-webkit-app-region: drag;
	}

	.app-name {
		font-size: var(--text-sm);
		font-weight: 600;
		color: var(--color-text-muted);
		letter-spacing: 0.04em;
		text-transform: uppercase;
	}

	.theme-toggle {
		-webkit-app-region: no-drag;
		background: none;
		border: none;
		color: var(--color-text-muted);
		font-size: var(--text-md);
		cursor: pointer;
		padding: var(--space-1) var(--space-2);
		border-radius: var(--radius-sm);
		line-height: 1;
		transition: background 0.15s;
	}

	.theme-toggle:hover {
		background: var(--color-btn-hover);
	}
</style>
