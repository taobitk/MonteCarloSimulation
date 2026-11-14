<script lang="ts">
	import type { CliRunOutput } from '../../types/simulation';

	interface Props {
		runs: CliRunOutput[];
		selectedId: string;
		onSelect: (id: string) => void;
	}

	let { runs, selectedId, onSelect }: Props = $props();
	let yearFilter = $state<number | 'all'>('all');
	let strategyFilter = $state<'all' | 'dynamic' | 'static'>('all');

	let years = $derived(Array.from(new Set(runs.map((r) => r.years))).sort((a, b) => a - b));
	let filteredRuns = $derived(
		runs.filter((r) => {
			if (yearFilter !== 'all' && r.years !== yearFilter) return false;
			if (strategyFilter === 'dynamic' && !r.is_dynamic_dca) return false;
			if (strategyFilter === 'static' && r.is_dynamic_dca) return false;
			return true;
		})
	);

	const fmtMoney = (v: number) => (v >= 1000 ? `$${(v / 1000).toFixed(1)}k` : `$${v.toFixed(0)}`);
	const fmtPct = (v: number) => `${(v * 100).toFixed(2)}%`;
</script>

<div class="rounded-xl border border-slate-800 bg-slate-900/70 p-4 shadow-lg">
	<div class="mb-3 flex flex-wrap items-center justify-between gap-3">
		<div>
			<h2 class="text-sm font-bold tracking-wide text-slate-100 uppercase">Bảng So Sánh Kịch Bản ({filteredRuns.length}/{runs.length})</h2>
			<p class="text-xs text-slate-400">Chọn một dòng để xem biểu đồ Fan Chart &amp; phân tích rủi ro chi tiết</p>
		</div>
		<div class="flex flex-wrap items-center gap-2 text-xs">
			<select bind:value={yearFilter} class="rounded-lg border border-slate-700 bg-slate-950 px-2.5 py-1.5 text-slate-200 focus:border-sky-500 focus:outline-none">
				<option value="all">Tất cả kỳ hạn</option>
				{#each years as y}
					<option value={y}>{y} năm</option>
				{/each}
			</select>
			<select bind:value={strategyFilter} class="rounded-lg border border-slate-700 bg-slate-950 px-2.5 py-1.5 text-slate-200 focus:border-sky-500 focus:outline-none">
				<option value="all">Dynamic &amp; Static</option>
				<option value="dynamic">Chỉ Dynamic DCA</option>
				<option value="static">Chỉ Static DCA</option>
			</select>
		</div>
	</div>

	<div class="max-h-[340px] overflow-auto rounded-lg border border-slate-800">
		<table class="w-full text-left text-xs">
			<thead class="sticky top-0 z-10 bg-slate-950 text-[11px] font-semibold tracking-wider text-slate-400 uppercase">
				<tr>
					<th class="px-3 py-2.5">Kỳ</th>
					<th class="px-3 py-2.5">Danh mục</th>
					<th class="px-3 py-2.5">Chế độ</th>
					<th class="px-3 py-2.5 text-right">Vốn nạp</th>
					<th class="px-3 py-2.5 text-right">P10</th>
					<th class="px-3 py-2.5 text-right">P50 (MOIC)</th>
					<th class="px-3 py-2.5 text-right">P90</th>
					<th class="px-3 py-2.5 text-right">Win</th>
					<th class="px-3 py-2.5 text-right">Sharpe</th>
					<th class="px-3 py-2.5 text-right">Breach</th>
					<th class="px-3 py-2.5 text-right">Phí / Bán</th>
					<th class="px-3 py-2.5 text-right">Chaos</th>
				</tr>
			</thead>
			<tbody class="divide-y divide-slate-800/70">
				{#each filteredRuns as run (run.id)}
					{@const active = run.id === selectedId}
					{@const lastIdx = Math.max(0, run.result.bands.p10.length - 1)}
					<tr
						onclick={() => onSelect(run.id)}
						class="cursor-pointer transition-colors {active ? 'bg-sky-950/60 text-sky-100' : 'bg-slate-900/30 text-slate-300 hover:bg-slate-800/60'}"
					>
						<td class="px-3 py-2 font-mono font-bold text-slate-200">{run.years}y</td>
						<td class="px-3 py-2 font-medium">
							<div class="flex items-center gap-1.5">
								{#if active}<span class="h-2 w-2 rounded-full bg-sky-400"></span>{/if}
								<span>{run.portfolio_name}</span>
							</div>
						</td>
						<td class="px-3 py-2">
							<span class="rounded px-1.5 py-0.5 text-[10px] font-semibold {run.sweep_name === 'Stress' ? 'bg-rose-500/20 text-rose-300' : 'bg-emerald-500/20 text-emerald-300'}">
								{run.sweep_name}
							</span>
						</td>
						<td class="px-3 py-2 text-right font-mono text-amber-300">{fmtMoney(run.result.total_contributed)}</td>
						<td class="px-3 py-2 text-right font-mono text-slate-300">{fmtMoney(run.result.bands.p10[lastIdx] ?? 0)}</td>
						<td class="px-3 py-2 text-right font-mono font-bold text-emerald-400">
							{fmtMoney(run.result.median_terminal_nav)}
							<span class="ml-1 text-[10px] text-slate-400">({run.result.moic_p50.toFixed(2)}x)</span>
						</td>
						<td class="px-3 py-2 text-right font-mono text-sky-300">{fmtMoney(run.result.bands.p90[lastIdx] ?? 0)}</td>
						<td class="px-3 py-2 text-right font-mono">{fmtPct(run.result.win_rate)}</td>
						<td class="px-3 py-2 text-right font-mono">{run.result.sharpe_ratio.toFixed(2)}</td>
						<td class="px-3 py-2 text-right font-mono {run.result.breach_rate > 0.05 ? 'text-rose-400 font-bold' : 'text-slate-300'}">
							{fmtPct(run.result.breach_rate)}
						</td>
						<td class="px-3 py-2 text-right font-mono text-slate-400">
							${run.result.avg_total_fees.toFixed(1)} <span class="text-[10px]">({run.result.avg_sell_count.toFixed(1)}x)</span>
						</td>
						<td class="px-3 py-2 text-right font-mono text-violet-300">{run.result.avg_chaos_events.toFixed(2)}</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
</div>

