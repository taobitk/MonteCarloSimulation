export interface AssetSpec {
	name: string;
	weight: number;
	mu: number;
	sigma: number;
}

export interface QuantileBand {
	p10: number[];
	p25: number[];
	p50: number[];
	p75: number[];
	p90: number[];
}

export interface DcaSimResult {
	total_sims: number;
	bands: QuantileBand;
	contributed_band: number[];
	floor_band: number[];
	worst_breach_paths: number[][];
	total_contributed: number;
	avg_actual_contributed?: number;
	breach_count: number;
	breach_rate: number;
	avg_breach_day: number;
	median_terminal_nav: number;
	net_profit_p50: number;
	moic_p50: number;
	var_95_net_loss: number;
	cvar_95_net_loss: number;
	win_rate: number;
	sharpe_ratio: number;
	avg_sell_count: number;
	avg_total_fees: number;
	avg_chaos_events: number;
	asset_drawdowns_at_ruin: number[];
	psd_repaired: boolean;
}

export interface CliRunOutput {
	id: string;
	portfolio_name: string;
	sweep_name: string;
	years: number;
	horizon_days: number;
	is_dynamic_dca: boolean;
	tolerance_band_pct: number;
	sell_fee_pct: number;
	ruin_drawdown_pct: number;
	initial_nav: number;
	periodic_cash: number;
	assets: AssetSpec[];
	elapsed_ms: number;
	result: DcaSimResult;
}

export interface CliSuiteOutput {
	config_path: string;
	chaos_enabled: boolean;
	chaos_events_count: number;
	total_elapsed_ms: number;
	runs: CliRunOutput[];
}
