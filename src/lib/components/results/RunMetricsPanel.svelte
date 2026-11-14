<script lang="ts">
	import type { CliRunOutput } from '../../types/simulation';
	import InfoTip from '../ui/InfoTip.svelte';

	interface Props {
		run: CliRunOutput;
	}

	let { run }: Props = $props();
	let r = $derived(run.result);
	let lastIdx = $derived(Math.max(0, r.bands.p50.length - 1));
	const fmt = (v: number) => `$${v.toLocaleString('en-US', { maximumFractionDigits: 1 })}`;
</script>

<div class="space-y-4">
	<!-- Hàng thẻ chỉ số chính -->
	<div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
		<div class="rounded-xl border border-slate-800 bg-slate-900/70 p-3.5">
			<div class="flex items-center justify-between text-xs text-slate-400">
				<span>Trung vị P50 (MOIC)</span>
				<InfoTip text="Giá trị tài sản cuối kỳ tại phân vị thứ 50 và bội số trên tổng vốn thực nạp." />
			</div>
			<div class="mt-1 font-mono text-xl font-bold text-emerald-400">{fmt(r.median_terminal_nav)}</div>
			<div class="mt-0.5 text-xs text-slate-400">
				Vốn nạp: <span class="font-mono text-amber-300">{fmt(r.total_contributed)}</span>
				{#if r.avg_actual_contributed && Math.abs(r.avg_actual_contributed - r.total_contributed) > 1}
					· Thực TB: <span class="font-mono text-sky-300">{fmt(r.avg_actual_contributed)}</span>
				{/if}
				· <span class="font-bold text-emerald-300">{r.moic_p50.toFixed(2)}x</span>
			</div>
		</div>

		<div class="rounded-xl border border-slate-800 bg-slate-900/70 p-3.5">
			<div class="flex items-center justify-between text-xs text-slate-400">
				<span>Dải P10 — P90</span>
				<InfoTip text="Khoảng tin cậy 80% của tài sản cuối kỳ từ kịch bản xấu (P10) đến lạc quan (P90)." />
			</div>
			<div class="mt-1 font-mono text-lg font-bold text-sky-300">{fmt(r.bands.p10[lastIdx] ?? 0)} — {fmt(r.bands.p90[lastIdx] ?? 0)}</div>
			<div class="mt-0.5 text-xs text-slate-400">Win Rate: <span class="font-mono font-bold text-slate-200">{(r.win_rate * 100).toFixed(2)}%</span> · Sharpe: <span class="font-mono font-bold text-sky-300">{r.sharpe_ratio.toFixed(2)}</span></div>
		</div>

		<div class="rounded-xl border border-slate-800 bg-slate-900/70 p-3.5">
			<div class="flex items-center justify-between text-xs text-slate-400">
				<span>Rủi ro Sụp Đổ (Breach)</span>
				<InfoTip text="Tỷ lệ đường đi chạm ngưỡng lỗ phá sản so với vốn nạp (Ruin Drawdown Floor)." />
			</div>
			<div class="mt-1 font-mono text-xl font-bold {r.breach_rate > 0.05 ? 'text-rose-400' : 'text-slate-100'}">{(r.breach_rate * 100).toFixed(2)}%</div>
			<div class="mt-0.5 text-xs text-slate-400">
				Số lần: <span class="font-mono text-slate-200">{r.breach_count}/{r.total_sims}</span>
				{#if r.breach_count > 0}· TB ngày {r.avg_breach_day.toFixed(0)}{/if}
			</div>
		</div>

		<div class="rounded-xl border border-slate-800 bg-slate-900/70 p-3.5">
			<div class="flex items-center justify-between text-xs text-slate-400">
				<span>Rủi ro Đuôi &amp; Chaos</span>
				<InfoTip text="VaR/CVaR 95% lỗ ròng và số sự kiện Chaos Monkey kích hoạt trung bình mỗi kịch bản." />
			</div>
			<div class="mt-1 font-mono text-lg font-bold text-violet-300">{r.avg_chaos_events.toFixed(2)} <span class="text-xs font-normal text-slate-400">events/path</span></div>
			<div class="mt-0.5 text-xs text-slate-400">CVaR95: <span class="font-mono text-rose-300">{fmt(r.cvar_95_net_loss)}</span> · Phí: <span class="font-mono text-amber-300">${r.avg_total_fees.toFixed(1)}</span></div>
		</div>
	</div>

	<!-- Chi tiết phân bổ tài sản & Drawdown khi vỡ trận -->
	<div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
		<div class="rounded-xl border border-slate-800 bg-slate-900/70 p-4">
			<h3 class="mb-2.5 text-xs font-bold tracking-wider text-slate-300 uppercase">Cấu hình Danh mục &amp; Tham số</h3>
			<div class="mb-3 flex flex-wrap gap-2 text-xs">
				<span class="rounded-md bg-slate-800 px-2 py-1 text-slate-300">Chiến lược: <strong class="text-sky-300">{run.is_dynamic_dca ? `Dynamic (±${(run.tolerance_band_pct * 100).toFixed(0)}%)` : 'Static DCA'}</strong></span>
				<span class="rounded-md bg-slate-800 px-2 py-1 text-slate-300">Phí bán: <strong class="text-amber-300">{(run.sell_fee_pct * 100).toFixed(2)}%</strong></span>
				<span class="rounded-md bg-slate-800 px-2 py-1 text-slate-300">Sàn gãy: <strong class="text-rose-300">-{(run.ruin_drawdown_pct * 100).toFixed(0)}%</strong></span>
				<span class="rounded-md bg-slate-800 px-2 py-1 text-slate-300">Thời gian CLI: <strong class="text-emerald-300">{run.elapsed_ms} ms</strong></span>
			</div>
			<div class="grid grid-cols-2 gap-2 sm:grid-cols-4">
				{#each run.assets as asset}
					<div class="rounded-lg border border-slate-800 bg-slate-950/70 p-2 text-xs">
						<div class="font-bold text-slate-200">{asset.name}</div>
						<div class="font-mono text-sky-400">{(asset.weight * 100).toFixed(1)}%</div>
						<div class="text-[11px] text-slate-400">μ={(asset.mu * 100).toFixed(1)}% · σ={(asset.sigma * 100).toFixed(1)}%</div>
					</div>
				{/each}
			</div>
		</div>

		<div class="rounded-xl border border-slate-800 bg-slate-900/70 p-4">
			<h3 class="mb-2.5 text-xs font-bold tracking-wider text-slate-300 uppercase">Phân Rã Sụt Giảm Lúc Chạm Đáy (Drawdown at Ruin)</h3>
			{#if r.breach_count === 0}
				<div class="flex h-24 items-center justify-center rounded-lg border border-dashed border-slate-800 text-xs text-emerald-400">
					Không có kịch bản nào chạm ngưỡng sụp đổ trong {r.total_sims.toLocaleString()} đường mô phỏng.
				</div>
			{:else}
				<div class="space-y-2">
					{#each run.assets as asset, idx}
						{@const dd = (r.asset_drawdowns_at_ruin[idx] ?? 0) * 100}
						<div class="text-xs">
							<div class="mb-1 flex justify-between">
								<span class="font-medium text-slate-300">{asset.name}</span>
								<span class="font-mono font-bold text-rose-400">-{dd.toFixed(1)}%</span>
							</div>
							<div class="h-1.5 w-full overflow-hidden rounded-full bg-slate-800">
								<div class="h-full rounded-full bg-rose-500" style="width: {Math.min(100, Math.max(0, dd))}%"></div>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</div>
	</div>
</div>

