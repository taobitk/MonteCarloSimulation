<script lang="ts">
	import { Info } from 'lucide-svelte';

	interface Props {
		text: string;
		title?: string;
	}

	let { text, title }: Props = $props();
	let isOpen = $state(false);
	let btnEl = $state<HTMLButtonElement | null>(null);
	let coords = $state({ top: 0, left: 0 });

	function showTooltip() {
		if (!btnEl) return;
		const r = btnEl.getBoundingClientRect();
		const w = 260;
		let left = r.right + 8;
		let top = r.top - 10;
		if (left + w > window.innerWidth - 12) {
			left = r.left - w - 8;
		}
		if (top + 120 > window.innerHeight) {
			top = window.innerHeight - 130;
		}
		coords = { top: Math.max(8, top), left: Math.max(8, left) };
		isOpen = true;
	}

	function hideTooltip() {
		isOpen = false;
	}
</script>

<div class="inline-flex items-center">
	<button
		bind:this={btnEl}
		type="button"
		class="text-slate-400 hover:text-sky-400 transition-colors p-0.5 rounded focus:outline-none"
		aria-label="Thông tin giải thích"
		onmouseenter={showTooltip}
		onmouseleave={hideTooltip}
		onclick={() => (isOpen ? hideTooltip() : showTooltip())}
	>
		<Info size={14} />
	</button>

	{#if isOpen}
		<div
			role="tooltip"
			class="fixed z-[9999] w-64 p-2.5 rounded-lg bg-slate-900 border border-slate-700 shadow-2xl text-xs text-slate-200 pointer-events-none"
			style="top: {coords.top}px; left: {coords.left}px;"
		>
			{#if title}
				<div class="font-semibold text-sky-400 mb-1">{title}</div>
			{/if}
			<div class="leading-relaxed">{text}</div>
		</div>
	{/if}
</div>
