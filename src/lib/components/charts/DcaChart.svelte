<script lang="ts">
	import type { QuantileBand } from '../../types/simulation';

	interface Props {
		bands?: QuantileBand | null;
		contributedBand?: number[];
		floorBand?: number[];
		worstPaths?: number[][];
		initialNav: number;
		horizonDays?: number;
	}

	let { bands = null, contributedBand = [], floorBand = [], worstPaths = [], initialNav = 100, horizonDays = 252 }: Props = $props();
	let container = $state<HTMLDivElement | null>(null);
	let canvas = $state<HTMLCanvasElement | null>(null);
	let hoverInfo = $state<{ x: number; y: number; step: number; p10: number; p50: number; p90: number; contrib: number; floor: number; breach?: number } | null>(null);

	function drawChart() {
		if (!canvas) return;
		const ctx = canvas.getContext('2d');
		if (!ctx) return;
		const dpr = window.devicePixelRatio || 1;
		const rect = canvas.getBoundingClientRect();
		canvas.width = rect.width * dpr;
		canvas.height = rect.height * dpr;
		ctx.scale(dpr, dpr);
		const { width: w, height: h } = rect;
		ctx.clearRect(0, 0, w, h);

		if (!bands || !bands.p50 || bands.p50.length === 0) {
			ctx.fillStyle = '#94a3b8';
			ctx.font = '13px sans-serif';
			ctx.textAlign = 'center';
			ctx.fillText('Chưa có dữ liệu CLI — Hãy chạy "npm run cli:dca" hoặc tải file JSON kết quả', w / 2, h / 2);
			return;
		}

		const nSteps = bands.p50.length;
		let minVal = initialNav * 0.5;
		let maxVal = initialNav * 1.5;
		for (let i = 0; i < nSteps; i++) {
			if (bands.p10[i] < minVal) minVal = bands.p10[i];
			if (bands.p90[i] > maxVal) maxVal = bands.p90[i];
			if (contributedBand[i] && contributedBand[i] * 1.2 > maxVal) maxVal = contributedBand[i] * 1.2;
			if (floorBand[i] && floorBand[i] < minVal) minVal = floorBand[i];
		}
		for (const wp of worstPaths) { for (const v of wp) if (v < minVal) minVal = v; }

		const padX = 48; const padY = 24;
		const plotW = w - padX - 16; const plotH = h - padY * 2;
		const getX = (step: number) => padX + (step / (nSteps - 1)) * plotW;
		const getY = (val: number) => padY + plotH * (1 - (val - minVal) / Math.max(1e-6, maxVal - minVal));

		// Lưới ngang
		ctx.strokeStyle = '#1e293b'; ctx.lineWidth = 1; ctx.beginPath();
		for (let i = 0; i <= 4; i++) {
			const y = padY + (plotH * i) / 4;
			ctx.moveTo(padX, y); ctx.lineTo(w - 16, y);
			ctx.fillStyle = '#94a3b8'; ctx.font = 'bold 11px monospace'; ctx.textAlign = 'right';
			ctx.fillText(`$${(maxVal - ((maxVal - minVal) * i) / 4).toFixed(0)}`, padX - 8, y + 4);
		}
		ctx.stroke();

		// Trục thời gian
		const qStep = Math.floor((nSteps - 1) / 4);
		[qStep, qStep * 2, qStep * 3, nSteps - 1].forEach((stepIdx, qIdx) => {
			const day = Math.round((stepIdx / (nSteps - 1)) * horizonDays);
			ctx.fillStyle = '#94a3b8'; ctx.font = 'bold 11px monospace'; ctx.textAlign = 'center';
			ctx.fillText(`T${qIdx + 1} (${day}d)`, getX(stepIdx), h - 8);
		});

		// Dải mây 10% - 90% & 25% - 75%
		ctx.beginPath();
		for (let i = 0; i < nSteps; i++) { const x = getX(i), y = getY(bands.p90[i]); if (i === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y); }
		for (let i = nSteps - 1; i >= 0; i--) ctx.lineTo(getX(i), getY(bands.p10[i]));
		ctx.closePath(); ctx.fillStyle = 'rgba(14, 165, 233, 0.14)'; ctx.fill();

		ctx.beginPath();
		for (let i = 0; i < nSteps; i++) { const x = getX(i), y = getY(bands.p75[i]); if (i === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y); }
		for (let i = nSteps - 1; i >= 0; i--) ctx.lineTo(getX(i), getY(bands.p25[i]));
		ctx.closePath(); ctx.fillStyle = 'rgba(14, 165, 233, 0.25)'; ctx.fill();

		// Đường Trung vị P50 (Xanh biển)
		ctx.beginPath();
		for (let i = 0; i < nSteps; i++) { const x = getX(i), y = getY(bands.p50[i]); if (i === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y); }
		ctx.strokeStyle = '#38bdf8'; ctx.lineWidth = 2.5; ctx.stroke();

		// Đường Tổng vốn thực nạp bậc thang (Vàng đứt nét)
		if (contributedBand.length === nSteps) {
			ctx.beginPath(); ctx.strokeStyle = '#fbbf24'; ctx.lineWidth = 2; ctx.setLineDash([4, 4]);
			for (let i = 0; i < nSteps; i++) { const x = getX(i), y = getY(contributedBand[i]); if (i === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y); }
			ctx.stroke(); ctx.setLineDash([]);
		}

		// Đường Vạch Gãy Động (Đỏ hồng đứt nét)
		if (floorBand.length === nSteps) {
			ctx.beginPath(); ctx.strokeStyle = 'rgba(244, 63, 94, 0.85)'; ctx.lineWidth = 1.5; ctx.setLineDash([3, 3]);
			for (let i = 0; i < nSteps; i++) { const x = getX(i), y = getY(floorBand[i]); if (i === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y); }
			ctx.stroke(); ctx.setLineDash([]);
		}

		// Kịch bản gãy xấu nhất
		for (const wp of worstPaths) {
			ctx.beginPath(); ctx.strokeStyle = '#f43f5e'; ctx.lineWidth = 2;
			wp.forEach((val, step) => { const x = getX(step), y = getY(val); if (step === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y); });
			ctx.stroke();
		}

		if (hoverInfo) {
			const hX = getX(hoverInfo.step);
			ctx.strokeStyle = 'rgba(255, 255, 255, 0.6)'; ctx.lineWidth = 1.2; ctx.setLineDash([3, 3]);
			ctx.beginPath(); ctx.moveTo(hX, padY); ctx.lineTo(hX, h - padY); ctx.stroke(); ctx.setLineDash([]);
		}
	}

	function handleMouseMove(e: MouseEvent) {
		if (!canvas || !bands || !bands.p50?.length) return;
		const r = canvas.getBoundingClientRect();
		const padX = 48; const plotW = r.width - padX - 16;
		const nSteps = bands.p50.length;
		const mouseX = e.clientX - r.left;
		const step = Math.round(((mouseX - padX) / plotW) * (nSteps - 1));
		if (step >= 0 && step < nSteps) {
			const bVal = worstPaths[0] && worstPaths[0][step] !== undefined ? worstPaths[0][step] : undefined;
			hoverInfo = {
				x: e.clientX - r.left, y: e.clientY - r.top, step,
				p10: bands.p10[step], p50: bands.p50[step], p90: bands.p90[step],
				contrib: contributedBand[step] || initialNav, floor: floorBand[step] || initialNav * 0.8, breach: bVal
			};
			requestAnimationFrame(drawChart);
		}
	}

	function handleMouseLeave() { hoverInfo = null; requestAnimationFrame(drawChart); }

	$effect(() => {
		if (!container) return;
		const ro = new ResizeObserver(() => drawChart());
		ro.observe(container);
		return () => ro.disconnect();
	});
	$effect(() => { drawChart(); });
</script>

<div bind:this={container} class="relative w-full h-full min-h-[320px] bg-slate-950 rounded-lg border border-slate-800 p-3 overflow-hidden select-none">
	<canvas bind:this={canvas} class="w-full h-full block cursor-crosshair" onmousemove={handleMouseMove} onmouseleave={handleMouseLeave}></canvas>

	<!-- Chú giải Fan Chart DCA -->
	<div class="absolute top-4 right-4 flex items-center space-x-3 text-xs bg-slate-900/95 px-3 py-1.5 rounded-md border border-slate-700/80 shadow-md backdrop-blur-sm pointer-events-none">
		<span class="flex items-center space-x-1.5"><span class="w-3 h-1 bg-amber-400 rounded"></span><span class="text-amber-300 font-bold">Vốn nạp</span></span>
		<span class="flex items-center space-x-1.5"><span class="w-3 h-1 bg-sky-400 rounded"></span><span class="text-sky-300 font-bold">P50</span></span>
		<span class="flex items-center space-x-1.5"><span class="w-3 h-1 bg-rose-500 rounded"></span><span class="text-rose-400 font-bold">Gãy động</span></span>
	</div>

	{#if hoverInfo && canvas}
		{@const isRightHalf = hoverInfo.x > canvas.clientWidth * 0.55}
		{@const curDay = Math.round((hoverInfo.step / (bands && bands.p50.length > 1 ? bands.p50.length - 1 : 1)) * horizonDays)}
		<div class="absolute z-20 pointer-events-none bg-slate-900/95 border border-slate-700 rounded-lg p-2.5 text-xs shadow-2xl text-slate-100 backdrop-blur-md"
			style="left: {isRightHalf ? hoverInfo.x - 210 : hoverInfo.x + 16}px; top: {Math.max(16, hoverInfo.y - 45)}px;">
			<div class="font-bold text-sky-400 border-b border-slate-800 pb-1 mb-1.5">Phiên {curDay}d | Vốn nạp: ${hoverInfo.contrib.toFixed(0)}</div>
			<div class="grid grid-cols-2 gap-x-3 gap-y-1 font-mono">
				<span class="text-slate-400 font-sans">P90:</span><span class="text-sky-300 text-right font-bold">${hoverInfo.p90.toFixed(1)}</span>
				<span class="text-slate-400 font-sans">P50 (Kỳ vọng):</span><span class="text-emerald-400 text-right font-bold">${hoverInfo.p50.toFixed(1)}</span>
				<span class="text-slate-400 font-sans">P10:</span><span class="text-amber-400 text-right font-bold">${hoverInfo.p10.toFixed(1)}</span>
				<span class="text-rose-400 font-sans">Vạch gãy:</span><span class="text-rose-400 text-right font-bold">${hoverInfo.floor.toFixed(1)}</span>
			</div>
		</div>
	{/if}
</div>
