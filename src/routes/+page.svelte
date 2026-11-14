<script lang="ts">
	import { onMount } from 'svelte';
	import DcaChart from '#lib/components/charts/DcaChart.svelte';
	import SuiteComparisonTable from '#lib/components/results/SuiteComparisonTable.svelte';
	import RunMetricsPanel from '#lib/components/results/RunMetricsPanel.svelte';
	import type { CliSuiteOutput } from '../lib/types/simulation';

	let suite = $state<CliSuiteOutput | null>(null);
	let selectedId = $state<string>('');
	let sourceLabel = $state<string>('results/latest_suite.json');
	let loading = $state<boolean>(true);
	let errorMsg = $state<string>('');

	let selectedRun = $derived(suite?.runs.find((r) => r.id === selectedId) ?? suite?.runs[0] ?? null);

	async function loadDefaultJson() {
		loading = true;
		errorMsg = '';
		try {
			const res = await fetch(`/results/latest_suite.json?t=${Date.now()}`);
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			const data: CliSuiteOutput = await res.json();
			suite = data;
			selectedId = data.runs[0]?.id ?? '';
			sourceLabel = 'static/results/latest_suite.json';
		} catch {
			suite = null;
			errorMsg = 'Chưa tìm thấy file static/results/latest_suite.json. Hãy chạy "npm run cli:dca" hoặc chọn file JSON từ máy.';
		} finally {
			loading = false;
		}
	}

	function handleFileUpload(event: Event) {
		const input = event.target as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;
		const reader = new FileReader();
		reader.onload = () => {
			try {
				const parsed: CliSuiteOutput = JSON.parse(String(reader.result));
				if (!Array.isArray(parsed.runs)) throw new Error('Thiếu trường runs[]');
				suite = parsed;
				selectedId = parsed.runs[0]?.id ?? '';
				sourceLabel = file.name;
				errorMsg = '';
			} catch (e) {
				errorMsg = `File JSON không hợp lệ: ${(e as Error).message}`;
			}
		};
		reader.readAsText(file);
	}

	onMount(() => {
		loadDefaultJson();
	});
</script>

<svelte:head>
	<title>DCA CLI Results Viewer</title>
</svelte:head>

<div class="min-h-screen bg-slate-950 text-slate-100">
	<header class="border-b border-slate-800 bg-slate-900/80 backdrop-blur-md">
		<div class="mx-auto flex max-w-7xl flex-wrap items-center justify-between gap-4 px-4 py-3.5 sm:px-6">
			<div>
				<h1 class="text-lg font-bold tracking-tight text-white">DCA &amp; Chaos Monkey — CLI Results Viewer</h1>
				<p class="text-xs text-slate-400">
					Nguồn dữ liệu: <span class="font-mono text-sky-300">{sourceLabel}</span>
					{#if suite}
						· Config: <span class="font-mono text-slate-300">{suite.config_path}</span>
						· Thời gian CLI: <span class="font-mono text-emerald-400">{(suite.total_elapsed_ms / 1000).toFixed(2)}s</span>
						· Chaos: <span class="font-semibold {suite.chaos_enabled ? 'text-violet-300' : 'text-slate-400'}">{suite.chaos_enabled ? `BẬT (${suite.chaos_events_count} events)` : 'TẮT'}</span>
					{/if}
				</p>
			</div>

			<div class="flex flex-wrap items-center gap-2.5">
				<button
					type="button"
					onclick={loadDefaultJson}
					class="cursor-pointer rounded-lg border border-sky-500/40 bg-sky-500/15 px-3 py-1.5 text-xs font-semibold text-sky-300 transition hover:bg-sky-500/25"
				>
					Nạp lại từ CLI
				</button>
				<label class="cursor-pointer rounded-lg border border-slate-700 bg-slate-800 px-3 py-1.5 text-xs font-semibold text-slate-200 transition hover:bg-slate-700">
					Mở file JSON...
					<input type="file" accept=".json,application/json" class="hidden" onchange={handleFileUpload} />
				</label>
			</div>
		</div>
	</header>

	<main class="mx-auto max-w-7xl space-y-5 px-4 py-5 sm:px-6">
		{#if loading}
			<div class="flex h-64 items-center justify-center rounded-xl border border-slate-800 bg-slate-900/50 text-sm text-slate-400">
				Đang đọc dữ liệu kết quả từ CLI...
			</div>
		{:else if errorMsg}
			<div class="rounded-xl border border-amber-500/40 bg-amber-500/10 p-6 text-center text-sm text-amber-200">
				<p class="font-semibold">{errorMsg}</p>
				<p class="mt-2 font-mono text-xs text-slate-400">Lệnh mẫu: npm run cli:dca -- --chaos data/configs/chaos_vn_32_events.json</p>
			</div>
		{:else if suite && selectedRun}
			<SuiteComparisonTable runs={suite.runs} selectedId={selectedRun.id} onSelect={(id) => (selectedId = id)} />

			<div class="rounded-xl border border-slate-800 bg-slate-900/70 p-4 shadow-lg">
				<div class="mb-3 flex flex-wrap items-center justify-between gap-2">
					<div>
						<h2 class="text-sm font-bold text-white">
							Biểu Đồ Fan Chart — {selectedRun.years} Năm · {selectedRun.portfolio_name} ({selectedRun.sweep_name})
						</h2>
						<p class="text-xs text-slate-400">
							Dải P10–P90, Trung vị P50, Đường Vốn Thực Nạp &amp; {selectedRun.result.worst_breach_paths.length} Đường Chạm Sàn Tệ Nhất
						</p>
					</div>
					<span class="rounded-md bg-slate-800 px-2.5 py-1 font-mono text-xs text-slate-300">
						{selectedRun.result.total_sims.toLocaleString()} paths · {selectedRun.horizon_days} phiên
					</span>
				</div>
				<div class="h-[380px]">
					<DcaChart
						bands={selectedRun.result.bands}
						contributedBand={selectedRun.result.contributed_band}
						floorBand={selectedRun.result.floor_band}
						worstPaths={selectedRun.result.worst_breach_paths}
						initialNav={selectedRun.initial_nav}
						horizonDays={selectedRun.horizon_days}
					/>
				</div>
			</div>

			<RunMetricsPanel run={selectedRun} />
		{/if}
	</main>
</div>
