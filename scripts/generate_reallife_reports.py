#!/usr/bin/env python3
"""Generate normalized Real-Life (32 VN Chaos events) Markdown reports in data/realLife/."""

from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LAB_DIR = ROOT / "data" / "lab"
OUT_DIR = ROOT / "data" / "realLife"
CANONICAL_JSON = ROOT / "static" / "results" / "latest_suite.json"
EXTENDED_JSON = ROOT / "static" / "results" / "reallife_vn_extended_suite.json"

PORTFOLIO_META = {
    "DYN_10_10_40_40": ("reallife_dynamic_{y}y_report.md", "dca_dynamic_{y}y_report.md", "10/10/40/40 (DCA Động)"),
    "STATIC_25x4": ("reallife_static_{y}y_report.md", "dca_static_{y}y_report.md", "25%×4 Đều (DCA Tĩnh)"),
    "DYN_10_20_35_35": ("reallife_10_20_35_35_dynamic_{y}y_report.md", "dca_10_20_35_35_dynamic_{y}y_report.md", "10/20/35/35 (DCA Động)"),
    "STATIC_10_20_35_35": ("reallife_10_20_35_35_static_{y}y_report.md", "dca_10_20_35_35_static_{y}y_report.md", "10/20/35/35 (DCA Tĩnh)"),
    "DYN_33x3": ("reallife_33x3_dynamic_{y}y_report.md", "dca_33x3_dynamic_{y}y_report.md", "33,3%×3 Không Tiền gửi (DCA Động)"),
    "STATIC_33x3": ("reallife_33x3_static_{y}y_report.md", "dca_33x3_static_{y}y_report.md", "33,3%×3 Không Tiền gửi (DCA Tĩnh)"),
    "STATIC_10_10_40_40": ("", "", "10/10/40/40 (DCA Tĩnh đối xứng)"),
    "DYN_25x4": ("", "", "25%×4 Đều (DCA Động đối xứng)"),
    "VN_SAFE_DYN": ("", "", "VN Thận trọng 35/5/30/30 (DCA Động)"),
    "VN_TRADITIONAL_DYN": ("", "", "VN Truyền thống 20/40/40 Không Crypto (DCA Động)"),
    "VN_TRADITIONAL_STATIC": ("", "", "VN Truyền thống 20/40/40 Không Crypto (DCA Tĩnh)"),
    "VN_GROWTH_BAND15": ("", "", "VN GenZ Tăng trưởng 5/30/20/45 (Biên ±15%)"),
    "VN_STARTER_CASHFLOW": ("", "", "VN Khởi nghiệp Vốn đầu nhỏ 20 + Nạp 15/kỳ"),
}

SWEEPS = ["1x_Real", "1x_Stress", "2x_Real", "2x_Stress"]
SWEEP_LABELS = {
    "1x_Real": "1. Thông thường (Thực tế)",
    "1x_Stress": "2. Thông thường (Tìm điểm gãy)",
    "2x_Real": "3. Rủi ro 2x (Thực tế)",
    "2x_Stress": "4. Rủi ro 2x (Tìm điểm gãy)",
}


def fmt_int(val: int) -> str:
    """Format integer with dot thousands separators."""
    return f"{val:,}".replace(",", ".")


def fmt_vn(val: float, decimals: int = 1) -> str:
    """Format float in Vietnamese locale style (dot thousands, comma decimal)."""
    s = f"{val:,.{decimals}f}"
    return s.replace(",", "_").replace(".", ",").replace("_", ".")


def fmt_signed_vn(val: float, decimals: int = 1) -> str:
    """Format signed float in Vietnamese locale style."""
    prefix = "+" if val > 0 else ""
    return f"{prefix}{fmt_vn(val, decimals)}"


def parse_vn_float(cell: str) -> float | None:
    """Extract the first Vietnamese-formatted number from a markdown table cell."""
    cleaned = re.sub(r"\*\([^)]*\)\*", "", cell)
    m = re.search(r"[-+]?\d[\d.]*(?:,\d+)?", cleaned)
    if not m:
        return None
    raw = m.group(0).replace(".", "").replace(",", ".")
    try:
        return float(raw)
    except ValueError:
        return None


def parse_lab_report(path: Path) -> dict[str, dict[str, float]]:
    """Parse 4-sweep metric table from a data/lab/*.md file."""
    out: dict[str, dict[str, float]] = {sw: {} for sw in SWEEPS}
    if not path.exists():
        return out
    lines = path.read_text(encoding="utf-8").splitlines()
    in_tbl = False
    for line in lines:
        if "| Chỉ số |" in line or "| Chỉ tiêu |" in line:
            in_tbl = True
            continue
        if in_tbl:
            if not line.strip().startswith("|"):
                break
            if ":---" in line:
                continue
            cols = [c.strip() for c in line.strip().strip("|").split("|")]
            if len(cols) < 5:
                continue
            label = cols[0].lower()
            key = None
            if "p10" in label:
                key = "p10"
            elif "trung vị p50" in label or "median_terminal_nav" in label:
                key = "p50"
            elif "p90" in label:
                key = "p90"
            elif "moic_p50" in label or "bội số vốn" in label:
                key = "moic"
            elif "win_rate" in label or "tỷ lệ có lãi" in label:
                key = "win"
            elif "sharpe" in label:
                key = "sharpe"
            elif "cvar 95%" in label:
                key = "cvar95"
            elif "breach_rate" in label or "tỷ lệ gãy" in label:
                key = "breach"
            if key:
                for idx, sw in enumerate(SWEEPS):
                    v = parse_vn_float(cols[idx + 1])
                    if v is not None:
                        out[sw][key] = v
    return out


def index_runs(suite_data: dict) -> dict[tuple[str, int, str], dict]:
    """Index CliRunOutput entries by (portfolio_name, years, sweep_name)."""
    idx: dict[tuple[str, int, str], dict] = {}
    for run in suite_data.get("runs", []):
        idx[(run["portfolio_name"], int(run["years"]), run["sweep_name"])] = run
    return idx


def render_assets_section(run: dict) -> list[str]:
    """Render asset table for a run."""
    lines = [
        "| Tài sản | Tỷ trọng (`weight`) | `mu`/năm | `sigma` (1x) | `sigma` (2x) |",
        "| :--- | :---: | :---: | :---: | :---: |",
    ]
    for a in run["assets"]:
        lines.append(
            f"| **{a['name']}** | **{fmt_vn(a['weight'] * 100, 2).rstrip('0').rstrip(',')}%** "
            f"| `{fmt_vn(a['mu'] * 100, 1)}%` | `{fmt_vn(a['sigma'] * 100, 1)}%` | `{fmt_vn(a['sigma'] * 200, 1)}%` |"
        )
    return lines


def render_4sweep_table(runs_by_sw: dict[str, dict], contrib_plan: float) -> list[str]:
    """Render detailed 4-sweep comparison table for one portfolio + horizon."""
    header = "| Chỉ số | " + " | ".join(SWEEP_LABELS[s] for s in SWEEPS) + " |"
    sep = "| :--- | " + " | ".join(":---:" for _ in SWEEPS) + " |"
    rows: list[tuple[str, list[str]]] = []

    def vals(fn) -> list[str]:
        return [fn(runs_by_sw[s]) for s in SWEEPS]

    rows.append(("**Vốn kế hoạch / Thực nạp TB**", vals(lambda r: f"`{fmt_vn(contrib_plan, 0)}` / **`{fmt_vn(r['result']['avg_actual_contributed'], 1)}`**")))
    rows.append(("**Số sự kiện Chaos TB/đời**", vals(lambda r: f"**`{fmt_vn(r['result']['avg_chaos_events'], 2)}`**")))
    rows.append(("**P10 (Xấu 10%)**", vals(lambda r: f"`{fmt_vn(r['result']['bands']['p10'][-1], 1)}`")))
    rows.append(("**P25**", vals(lambda r: f"`{fmt_vn(r['result']['bands']['p25'][-1], 1)}`")))
    rows.append(("**Trung vị P50 (`median_terminal_nav`)**", vals(lambda r: f"**`{fmt_vn(r['result']['median_terminal_nav'], 1)}`**")))
    rows.append(("**P75**", vals(lambda r: f"`{fmt_vn(r['result']['bands']['p75'][-1], 1)}`")))
    rows.append(("**P90 (Tốt 10%)**", vals(lambda r: f"`{fmt_vn(r['result']['bands']['p90'][-1], 1)}`")))
    rows.append(("**Lãi ròng P50 (`net_profit_p50`)**", vals(lambda r: f"`{fmt_signed_vn(r['result']['net_profit_p50'], 1)}`")))
    rows.append(("**Bội số vốn P50 (`moic_p50`)**", vals(lambda r: f"**`{fmt_vn(r['result']['moic_p50'], 2)}x`**")))
    rows.append(("**Tỷ lệ có lãi (`win_rate`)**", vals(lambda r: f"**`{fmt_vn(r['result']['win_rate'] * 100, 2)}%`**")))
    rows.append(("**Sharpe Ratio**", vals(lambda r: f"**`{fmt_vn(r['result']['sharpe_ratio'], 2)}`**")))
    rows.append(("**VaR 95% lỗ ròng**", vals(lambda r: f"`{fmt_vn(r['result']['var_95_net_loss'], 1)}`")))
    rows.append(("**CVaR 95% lỗ ròng**", vals(lambda r: f"`{fmt_vn(r['result']['cvar_95_net_loss'], 1)}`")))
    rows.append(("**Số mẫu gãy (`breach_count`)**", vals(lambda r: f"`{fmt_int(r['result']['breach_count'])} / {fmt_int(r['result']['total_sims'])}`")))
    rows.append(("**Tỷ lệ gãy (`breach_rate`)**", vals(lambda r: f"**`{fmt_vn(r['result']['breach_rate'] * 100, 2)}%`**")))
    rows.append(("**Ngày gãy TB (`avg_breach_day`)**", vals(lambda r: f"Ngày `{fmt_vn(r['result']['avg_breach_day'], 1)}`" if r["result"]["breach_count"] > 0 else "`0`")))
    rows.append(("**Số lần bán / Tổng phí TB**", vals(lambda r: f"`{fmt_vn(r['result']['avg_sell_count'], 2)} lần` / `{fmt_vn(r['result']['avg_total_fees'], 1)}`")))

    out = [header, sep]
    for label, cols in rows:
        out.append(f"| {label} | " + " | ".join(cols) + " |")
    return out


def render_drawdowns_table(runs_by_sw: dict[str, dict]) -> list[str]:
    """Render weighted asset drawdowns at ruin."""
    assets = runs_by_sw["1x_Real"]["assets"]
    lines = [
        "| Chế độ | " + " | ".join(f"{a['name']} (`{fmt_vn(a['weight'] * 100, 1)}%`)" for a in assets) + " |",
        "| :--- | " + " | ".join(":---:" for _ in assets) + " |",
    ]
    for sw in SWEEPS:
        dds = runs_by_sw[sw]["result"]["asset_drawdowns_at_ruin"]
        cols = [f"`{fmt_vn(d * 100, 2)}%`" for d in dds]
        lines.append(f"| **{SWEEP_LABELS[sw]}** | " + " | ".join(cols) + " |")
    return lines


def render_lab_delta_section(runs_by_sw: dict[str, dict], lab_metrics: dict[str, dict[str, float]], lab_filename: str) -> list[str]:
    """Render side-by-side comparison against data/lab/ baseline."""
    lines = [
        f"### Đối chiếu Real-Life (32 Chaos VN) vs. Lab ([{lab_filename}](../lab/{lab_filename}))",
        "",
        "| Kịch bản | P50 Lab | P50 Real-Life | Δ P50 (%) | Win Lab | Win Real | Breach Lab | Breach Real | Δ Breach |",
        "| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |",
    ]
    for sw in SWEEPS:
        r = runs_by_sw[sw]["result"]
        lm = lab_metrics.get(sw, {})
        lp50 = lm.get("p50")
        lwin = lm.get("win")
        lbr = lm.get("breach")
        rp50 = r["median_terminal_nav"]
        rwin = r["win_rate"] * 100.0
        rbr = r["breach_rate"] * 100.0
        dp50 = f"`{fmt_signed_vn((rp50 - lp50) / lp50 * 100.0, 1)}%`" if lp50 else "`N/A`"
        dbr = f"`{fmt_signed_vn(rbr - lbr, 2)} đ%`" if lbr is not None else "`N/A`"
        lines.append(
            f"| **{SWEEP_LABELS[sw]}** | `{fmt_vn(lp50, 1) if lp50 else 'N/A'}` | **`{fmt_vn(rp50, 1)}`** | {dp50} "
            f"| `{fmt_vn(lwin, 2) + '%' if lwin is not None else 'N/A'}` | `{fmt_vn(rwin, 2)}%` "
            f"| `{fmt_vn(lbr, 2) + '%' if lbr is not None else 'N/A'}` | **`{fmt_vn(rbr, 2)}%`** | {dbr} |"
        )
    return lines


def generate_single_portfolio_report(pname: str, y: int, runs_by_sw: dict[str, dict], lab_metrics: dict[str, dict[str, float]], lab_filename: str, chaos_count: int) -> str:
    """Build full Markdown report for one canonical portfolio + horizon."""
    _, _, label = PORTFOLIO_META[pname]
    r0 = runs_by_sw["1x_Real"]
    res0 = r0["result"]
    sims = res0["total_sims"]
    contrib_plan = res0["total_contributed"]
    mode_str = (
        f"**DCA Động** — Tái cân bằng biên `{fmt_vn(r0['tolerance_band_pct'] * 100, 0)}%`, phí bán `{fmt_vn(r0['sell_fee_pct'] * 100, 1)}%` (có Smart Inflow & khóa bán khi hoảng loạn)"
        if r0["is_dynamic_dca"]
        else "**DCA Tĩnh** — Chỉ mua phân bổ đều dòng tiền mới, không bán tái cân bằng (`0` phí)"
    )

    lines = [
        f"# Báo cáo Real-Life {label} — Khung {y} Năm ({fmt_int(sims)} Kịch bản × {chaos_count} Chaos VN)",
        "",
        "> [!NOTE]",
        f"> Chạy trên **Native Rust CLI (`dca_cli --release`, 16 luồng)** với `chaos_vn_32_events.json` (**{chaos_count} sự kiện VN**), `time_years = {y}.0`, `steps = {fmt_int(r0['horizon_days'])} ngày`, `num_sims = {fmt_int(sims)}`, `seed = 42`, Student-t ($\\nu = 4$), `psd_repaired = {str(res0['psd_repaired']).lower()}`.",
        f"> - **Chiến lược**: {mode_str}.",
        f"> - **Vốn kế hoạch**: Ban đầu `{fmt_vn(r0['initial_nav'], 0)}` + `{fmt_vn(r0['periodic_cash'], 0)}` mỗi `21 ngày` → `{fmt_vn(contrib_plan, 0)}` *(thực nạp TB bị trừ nhẹ do biến cố `p_job_loss`)*. Ngưỡng gãy `-{fmt_vn(r0['ruin_drawdown_pct'] * 100, 0)}%` vốn thực nạp lũy kế.",
        "",
        "---",
        "",
        "## 1. Cấu hình Tài sản Danh mục",
        "",
        *render_assets_section(r0),
        "",
        "---",
        "",
        f"## 2. Kết quả Chi tiết 4 Kịch bản Real-Life ({y} Năm)",
        "",
        *render_4sweep_table(runs_by_sw, contrib_plan),
        "",
        "---",
        "",
        "## 3. Đóng góp Sụt giảm lúc Chạm đáy (`asset_drawdowns_at_ruin`) & Đối chiếu Lab",
        "",
        *render_drawdowns_table(runs_by_sw),
        "",
        *render_lab_delta_section(runs_by_sw, lab_metrics, lab_filename),
        "",
    ]
    return "\n".join(lines)


def generate_group_report(title: str, subtitle: str, pnames: list[str], y: int, idx: dict[tuple[str, int, str], dict], extra_notes: list[str], chaos_count: int) -> str:
    """Generate comparative Markdown report for multiple extended portfolios at one horizon."""
    first_r = idx[(pnames[0], y, "1x_Real")]
    sims = first_r["result"]["total_sims"]
    lines = [
        f"# {title} — Khung {y} Năm ({fmt_int(sims)} Kịch bản × {chaos_count} Chaos VN)",
        "",
        "> [!NOTE]",
        f"> {subtitle}",
        *[f"> - {n}" for n in extra_notes],
        "",
        "---",
        "",
        f"## 1. Bảng Xếp Hạng Tổng Quan ({y} Năm — Kịch bản `1x_Real` & `2x_Real`)",
        "",
        "| Danh mục | Chế độ | Vốn thực nạp TB | Chaos TB | P50 `1x_Real` (MOIC) | Win `1x` | Sharpe `1x` | Breach `1x` | P50 `2x_Real` | Breach `2x` |",
        "| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |",
    ]
    for pname in pnames:
        r1 = idx[(pname, y, "1x_Real")]
        r2 = idx[(pname, y, "2x_Real")]
        res1, res2 = r1["result"], r2["result"]
        mode = f"Động ±{fmt_vn(r1['tolerance_band_pct']*100,0)}%" if r1["is_dynamic_dca"] else "Tĩnh"
        lines.append(
            f"| **{pname}** | {mode} | `{fmt_vn(res1['avg_actual_contributed'], 1)}` | `{fmt_vn(res1['avg_chaos_events'], 2)}` "
            f"| **`{fmt_vn(res1['median_terminal_nav'], 1)}`** (`{fmt_vn(res1['moic_p50'], 2)}x`) "
            f"| `{fmt_vn(res1['win_rate'] * 100, 2)}%` | `{fmt_vn(res1['sharpe_ratio'], 2)}` | **`{fmt_vn(res1['breach_rate'] * 100, 2)}%`** "
            f"| `{fmt_vn(res2['median_terminal_nav'], 1)}` | `{fmt_vn(res2['breach_rate'] * 100, 2)}%` |"
        )

    lines.extend(["", "---", "", "## 2. Chi tiết 4 Kịch bản Từng Danh mục", ""])
    for pname in pnames:
        runs_by_sw = {sw: idx[(pname, y, sw)] for sw in SWEEPS}
        r0 = runs_by_sw["1x_Real"]
        _, _, label = PORTFOLIO_META[pname]
        lines.extend([
            f"### {pname} — {label}",
            "",
            *render_assets_section(r0),
            "",
            *render_4sweep_table(runs_by_sw, r0["result"]["total_contributed"]),
            "",
        ])
    return "\n".join(lines)


def generate_master_report(canon_idx: dict, ext_idx: dict, chaos_count: int) -> str:
    """Build cross-project Master Comparison Report (Lab vs Real-Life & Full Rankings)."""
    all_idx = {**canon_idx, **ext_idx}
    all_portfolios = [p for p in PORTFOLIO_META if (p, 10, "1x_Real") in all_idx]
    canon_portfolios = [p for p in PORTFOLIO_META if PORTFOLIO_META[p][0] and (p, 10, "1x_Real") in canon_idx]

    sample_res = canon_idx[("DYN_10_10_40_40", 10, "1x_Real")]["result"]
    sims_canon = sample_res["total_sims"]

    lines = [
        f"# Báo cáo Tổng Kết: Lab (0 Chaos) vs. Real-Life ({chaos_count} Sự kiện Chaos VN)",
        "",
        "> [!IMPORTANT]",
        f"> Báo cáo đối chiếu trực diện **18 kịch bản Lab** (`data/lab/`, 0 Chaos, 35.000 sims) với **bộ dữ liệu Real-Life** ({fmt_int(sims_canon)} sims, bật trọn vẹn `chaos_vn_32_events.json` gồm 32 sự kiện vĩ mô, tài chính VN, tỷ giá, chính sách Vàng SJC, Crypto winter & biến cố mất việc/y tế cá nhân).",
        "",
        "---",
        "",
        "## 1. Tác động Thực tế của 32 Sự kiện Chaos VN (`1x_Real`: Lab vs. Real-Life)",
        "",
        "| Kỳ hạn | Danh mục | P50 Lab | P50 Real-Life | Δ P50 (%) | Vốn thực nạp TB / KH | Breach Lab | Breach Real | Δ Breach | Chaos TB |",
        "| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |",
    ]
    for y in [10, 20, 40]:
        for pname in canon_portfolios:
            _, lab_tpl, _ = PORTFOLIO_META[pname]
            lm = parse_lab_report(LAB_DIR / lab_tpl.format(y=y))["1x_Real"]
            r = canon_idx[(pname, y, "1x_Real")]["result"]
            lp50, lbr = lm.get("p50"), lm.get("breach")
            rp50, rbr = r["median_terminal_nav"], r["breach_rate"] * 100.0
            dp50 = fmt_signed_vn((rp50 - lp50) / lp50 * 100.0, 1) + "%" if lp50 else "N/A"
            dbr = fmt_signed_vn(rbr - lbr, 2) + " đ%" if lbr is not None else "N/A"
            lines.append(
                f"| **{y}y** | `{pname}` | `{fmt_vn(lp50, 1) if lp50 else 'N/A'}` | **`{fmt_vn(rp50, 1)}`** | `{dp50}` "
                f"| `{fmt_vn(r['avg_actual_contributed'], 1)}` / `{fmt_vn(r['total_contributed'], 0)}` "
                f"| `{fmt_vn(lbr, 2) + '%' if lbr is not None else 'N/A'}` | **`{fmt_vn(rbr, 2)}%`** | `{dbr}` | `{fmt_vn(r['avg_chaos_events'], 2)}` |"
            )

    lines.extend([
        "",
        "---",
        "",
        "## 2. Kiểm chứng Chiến lược Cân Bằng Động (Dynamic) vs. Tĩnh (Static) dưới Chaos VN (`1x_Real`)",
        "",
        "| Kỳ hạn | Cặp danh mục | Dynamic P50 (MOIC) | Static P50 (MOIC) | Chênh lệch P50 | Dynamic Breach vs. Static Breach | Dynamic Sharpe vs. Static |",
        "| :---: | :--- | :---: | :---: | :---: | :---: | :---: |",
    ])
    pairs = [
        ("10/10/40/40", "DYN_10_10_40_40", "STATIC_10_10_40_40"),
        ("25%×4 Đều", "DYN_25x4", "STATIC_25x4"),
        ("10/20/35/35", "DYN_10_20_35_35", "STATIC_10_20_35_35"),
        ("33,3%×3 (Không Tiền gửi)", "DYN_33x3", "STATIC_33x3"),
        ("VN Truyền thống 20/40/40 (Không Crypto)", "VN_TRADITIONAL_DYN", "VN_TRADITIONAL_STATIC"),
    ]
    for y in [10, 20, 40]:
        for label, dname, sname in pairs:
            if (dname, y, "1x_Real") not in all_idx or (sname, y, "1x_Real") not in all_idx:
                continue
            dr = all_idx[(dname, y, "1x_Real")]["result"]
            sr = all_idx[(sname, y, "1x_Real")]["result"]
            dp50, sp50 = dr["median_terminal_nav"], sr["median_terminal_nav"]
            diff = (dp50 - sp50) / sp50 * 100.0
            lines.append(
                f"| **{y}y** | {label} | **`{fmt_vn(dp50, 1)}`** (`{fmt_vn(dr['moic_p50'], 2)}x`) "
                f"| `{fmt_vn(sp50, 1)}` (`{fmt_vn(sr['moic_p50'], 2)}x`) | **`{fmt_signed_vn(diff, 1)}%`** "
                f"| `{fmt_vn(dr['breach_rate']*100, 2)}%` vs `{fmt_vn(sr['breach_rate']*100, 2)}%` "
                f"| `{fmt_vn(dr['sharpe_ratio'], 2)}` vs `{fmt_vn(sr['sharpe_ratio'], 2)}` |"
            )

    lines.extend([
        "",
        "---",
        "",
        "## 3. Bảng Xếp Hạng Toàn Bộ Kịch Bản Real-Life (`1x_Real`)",
        "",
        "| Kỳ hạn | Danh mục | MOIC P50 | P50 | P10 (Xấu 10%) | P90 (Tốt 10%) | Win Rate | Sharpe | Breach Rate | CVaR 95% | Phí bán TB |",
        "| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |",
    ])
    for y in [10, 20, 40]:
        ranked = sorted(all_portfolios, key=lambda p: all_idx[(p, y, "1x_Real")]["result"]["moic_p50"], reverse=True)
        for p in ranked:
            r = all_idx[(p, y, "1x_Real")]["result"]
            lines.append(
                f"| **{y}y** | `{p}` | **`{fmt_vn(r['moic_p50'], 2)}x`** | `{fmt_vn(r['median_terminal_nav'], 1)}` "
                f"| `{fmt_vn(r['bands']['p10'][-1], 1)}` | `{fmt_vn(r['bands']['p90'][-1], 1)}` "
                f"| `{fmt_vn(r['win_rate'] * 100, 2)}%` | `{fmt_vn(r['sharpe_ratio'], 2)}` "
                f"| `{fmt_vn(r['breach_rate'] * 100, 2)}%` | `{fmt_vn(r['cvar_95_net_loss'], 1)}` | `{fmt_vn(r['avg_total_fees'], 1)}` |"
            )

    lines.extend(["", "---", "", "## 4. Tổng kết Định lượng Theo Kỳ hạn (`1x_Real`)", ""])
    for y in [10, 20, 40]:
        best_moic = max(all_portfolios, key=lambda p: all_idx[(p, y, "1x_Real")]["result"]["moic_p50"])
        best_sharpe = max(all_portfolios, key=lambda p: all_idx[(p, y, "1x_Real")]["result"]["sharpe_ratio"])
        safest = min(all_portfolios, key=lambda p: all_idx[(p, y, "1x_Real")]["result"]["breach_rate"])
        r_m = all_idx[(best_moic, y, "1x_Real")]["result"]
        r_s = all_idx[(best_sharpe, y, "1x_Real")]["result"]
        r_b = all_idx[(safest, y, "1x_Real")]["result"]
        lines.extend([
            f"- **Khung {y} năm:**",
            f"  - **Tăng trưởng cao nhất (MOIC P50):** `{best_moic}` (`{fmt_vn(r_m['moic_p50'], 2)}x`, P50 `{fmt_vn(r_m['median_terminal_nav'], 1)}`, Breach `{fmt_vn(r_m['breach_rate']*100, 2)}%`).",
            f"  - **Hiệu quả/Rủi ro tốt nhất (Sharpe):** `{best_sharpe}` (`Sharpe = {fmt_vn(r_s['sharpe_ratio'], 2)}`, Win Rate `{fmt_vn(r_s['win_rate']*100, 2)}%`).",
            f"  - **Bền vững nhất (Breach thấp nhất):** `{safest}` (`Breach = {fmt_vn(r_b['breach_rate']*100, 2)}%`, P10 `{fmt_vn(r_b['bands']['p10'][-1], 1)}`).",
        ])
    lines.append("")
    return "\n".join(lines)


def main() -> None:
    """Entry point: read JSON suite results and generate all data/realLife/*.md reports."""
    assert CANONICAL_JSON.exists(), f"Missing canonical suite JSON: {CANONICAL_JSON}"
    canon_data = json.loads(CANONICAL_JSON.read_text(encoding="utf-8"))
    assert canon_data.get("chaos_enabled"), "Canonical suite must have chaos_enabled=true"
    canon_idx = index_runs(canon_data)
    chaos_count = int(canon_data.get("chaos_events_count", 32))

    ext_idx: dict[tuple[str, int, str], dict] = {}
    if EXTENDED_JSON.exists():
        ext_data = json.loads(EXTENDED_JSON.read_text(encoding="utf-8"))
        assert ext_data.get("chaos_enabled"), "Extended suite must have chaos_enabled=true"
        ext_idx = index_runs(ext_data)

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    generated = 0

    # 1. 18 Canonical Mirrored Reports
    for pname, (out_tpl, lab_tpl, _) in PORTFOLIO_META.items():
        if not out_tpl:
            continue
        for y in [10, 20, 40]:
            runs_by_sw = {sw: canon_idx[(pname, y, sw)] for sw in SWEEPS}
            lab_metrics = parse_lab_report(LAB_DIR / lab_tpl.format(y=y))
            content = generate_single_portfolio_report(pname, y, runs_by_sw, lab_metrics, lab_tpl.format(y=y), chaos_count)
            (OUT_DIR / out_tpl.format(y=y)).write_text(content, encoding="utf-8")
            generated += 1

    # 2. Extended Reports (if extended JSON present)
    if ext_idx:
        for y in [10, 20, 40]:
            sym_content = generate_group_report(
                "Báo cáo Cặp Đối Xứng Bổ Sung (STATIC 10/10/40/40 & DYN 25×4)",
                "Bổ sung 2 chiến lược đối chứng bị thiếu trong `data/lab/` để so sánh công bằng Dynamic vs. Static trên cùng tỷ trọng dưới 32 sự kiện Chaos VN.",
                ["STATIC_10_10_40_40", "DYN_25x4"],
                y,
                ext_idx,
                ["Vốn đầu `100` + nạp `10` mỗi `21 ngày`, phí bán Dynamic `2,0%`, biên lệch `±10%`."],
                chaos_count,
            )
            (OUT_DIR / f"reallife_symmetric_pairs_{y}y_report.md").write_text(sym_content, encoding="utf-8")
            generated += 1

            pers_content = generate_group_report(
                "Báo cáo Chân Dung Nhà Đầu Tư Việt Nam Thực Chiến (5 Personas)",
                "Kiểm thử 5 cấu hình thực tế tại Việt Nam dưới 32 sự kiện Chaos VN (`chaos_vn_32_events.json`), Student-t ($\\nu=4$), Cholesky PSD.",
                ["VN_SAFE_DYN", "VN_TRADITIONAL_DYN", "VN_TRADITIONAL_STATIC", "VN_GROWTH_BAND15", "VN_STARTER_CASHFLOW"],
                y,
                ext_idx,
                [
                    "`VN_SAFE_DYN`: 35% Tiền gửi · 5% Crypto · 30% Vàng · 30% CP (Phòng thủ cao).",
                    "`VN_TRADITIONAL_DYN/STATIC`: 20% Tiền gửi · 40% Vàng · 40% CP (Không Crypto).",
                    "`VN_GROWTH_BAND15`: 5% Tiền gửi · 30% Crypto · 20% Vàng · 45% CP (Nới biên lệch `±15%`).",
                    "`VN_STARTER_CASHFLOW`: Vốn đầu nhỏ `20` + nạp đều mạnh `15`/21 ngày (10/20/35/35).",
                ],
                chaos_count,
            )
            (OUT_DIR / f"reallife_vn_personas_{y}y_report.md").write_text(pers_content, encoding="utf-8")
            generated += 1

    # 3. Master Comparison Report
    master_content = generate_master_report(canon_idx, ext_idx, chaos_count)
    (OUT_DIR / "reallife_vs_lab_master_report.md").write_text(master_content, encoding="utf-8")
    generated += 1

    # 4. Full Consolidated Compendium
    compendium_path = build_compendium(OUT_DIR, chaos_count, canon_idx[("DYN_10_10_40_40", 10, "1x_Real")]["result"]["total_sims"])
    generated += 1

    print(f"OK: Generated {generated} markdown reports in {OUT_DIR} (including {compendium_path.name})")


def shift_headings(md_text: str, levels: int = 2) -> str:
    """Shift Markdown ATX headings down by `levels` to nest cleanly in a compendium."""
    out = []
    for line in md_text.splitlines():
        if line.startswith("#"):
            out.append("#" * levels + line)
        else:
            out.append(line)
    return "\n".join(out)


def build_compendium(out_dir: Path, chaos_count: int, sims: int) -> Path:
    """Combine all 25 Real-Life reports into one structured compendium file."""
    sections: list[tuple[str, list[str]]] = [
        ("Phần I — Báo cáo Tổng Kết (Lab vs. Real-Life & Bảng Xếp Hạng)", ["reallife_vs_lab_master_report.md"]),
        ("Phần II — Chân Dung Nhà Đầu Tư Việt Nam Thực Chiến (5 Personas)", [f"reallife_vn_personas_{y}y_report.md" for y in (10, 20, 40)]),
        ("Phần III — Cặp Đối Xứng Bổ Sung (STATIC 10/10/40/40 & DYN 25×4)", [f"reallife_symmetric_pairs_{y}y_report.md" for y in (10, 20, 40)]),
        (
            "Phần IV — Bộ 18 Báo cáo Đối Xứng 1–1 với `data/lab/`",
            [
                tpl.format(y=y)
                for pname, (tpl, _, _) in PORTFOLIO_META.items()
                if tpl
                for y in (10, 20, 40)
            ],
        ),
    ]

    lines = [
        f"# Toàn Tập Báo Cáo Real-Life Monte Carlo ({fmt_int(sims)} Kịch bản × {chaos_count} Chaos VN)",
        "",
        "> [!NOTE]",
        f"> Tài liệu tổng hợp đầy đủ **25 báo cáo** trong `data/realLife/` (chạy trên Native Rust CLI `dca_cli --release`, 16 luồng, `seed = 42`, Student-t $\\nu = 4$, `chaos_vn_32_events.json`, `psd_repaired = false`).",
        "",
        "## Mục Lục Nhanh",
        "",
    ]

    for sec_title, files in sections:
        existing = [f for f in files if (out_dir / f).exists()]
        if not existing:
            continue
        lines.append(f"- **{sec_title}** ({len(existing)} báo cáo): " + ", ".join(f"[{f}](./{f})" for f in existing))

    for sec_title, files in sections:
        existing = [f for f in files if (out_dir / f).exists()]
        if not existing:
            continue
        lines.extend(["", "---", "", f"## {sec_title}", ""])
        for fn in existing:
            content = (out_dir / fn).read_text(encoding="utf-8").strip()
            lines.extend([shift_headings(content, levels=2), ""])

    target = out_dir / "reallife_full_compendium.md"
    target.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")
    return target


if __name__ == "__main__":
    main()
