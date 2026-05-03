<script lang="ts">
	import favicon from "$lib/assets/favicon.svg";
	import "../app.css";

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>Loka — Database Management</title>
	<meta name="description" content="Loka is a modern database management tool" />
</svelte:head>

{@render children()}
