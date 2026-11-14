# MonteCarloSimulation

Hệ thống mô phỏng đầu tư tích sản (**DCA — Dollar-Cost Averaging**) theo phương pháp **Monte Carlo** hiệu năng cao viết bằng **Rust / WebAssembly**, tích hợp bộ sự kiện cú sốc vĩ mô **Chaos Monkey (32 sự kiện Việt Nam & toàn cầu)** và giao diện phân tích trực quan bằng **SvelteKit + Tailwind CSS**.

---

## Tính năng chính

- **Engine Rust đa luồng (`dca_cli`) & WASM (`wasm-engine`)**:
  - Hỗ trợ mô phỏng phân phối đuôi dày (**Fat-tail / Student-t**), ma trận tương quan tài sản qua phân rã **Cholesky**.
  - Đối chiếu trực tiếp chiến lược **DCA Tĩnh (Static)** và **DCA Động (Dynamic)**: tái cân bằng theo biên độ lệch (`tolerance_band_pct`), ưu tiên điều hướng dòng tiền nạp mới (Smart Cash Inflow) và tính phí bán (`sell_fee_pct`, mặc định `2%` cho Dynamic).
  - Đo lường đầy đủ rủi ro và hiệu quả: `MOIC P50`, phân vị `P10 / P50 / P90`, `Win Rate`, `Sharpe Ratio`, tỷ lệ chạm ngưỡng sụt giảm `-20%` (`Breach Rate`), `CVaR 95%`, tổng phí giao dịch và tập đường dẫn chạm sàn tệ nhất (`worst_breach_paths`).
- **Chaos Engine — 32 sự kiện thực chiến (`data/configs/chaos_vn_32_events.json`)**:
  - Mô phỏng các cú sốc nối tầng (`cascades`), thời gian hồi (`cooldown_days`), giới hạn kích hoạt (`max_triggers_per_sim`), và biến động tức thời (`Instant`) hoặc kéo dài (`Temporary`) lên giá trị tài sản và kỳ vọng lợi suất (`AssetMu`) cho 4 lớp tài sản: `Tiền gửi`, `Crypto`, `Vàng`, `Cổ phiếu`.
- **Dashboard trực quan (`src/`)**:
  - Bảng so sánh toàn suite (`SuiteComparisonTable`), biểu đồ **Fan Chart** (`P10–P90`, trung vị `P50`, đường vốn thực nạp, các đường chạm sàn tệ nhất) và bảng chỉ số chi tiết (`RunMetricsPanel`).
  - Tự động nạp `static/results/latest_suite.json` hoặc mở trực tiếp file JSON xuất ra từ CLI qua nút **"Mở file JSON..."**.
- **Kho báo cáo đối chiếu Lab vs Real-Life (`data/lab/` & `data/realLife/`)**:
  - Bộ **Lab** (mô phỏng chuẩn không Chaos) và bộ **Real-Life** (`60.000` kịch bản/lượt × `32` sự kiện Chaos trên `13` danh mục × `3` kỳ hạn `10y / 20y / 40y` × `4` sweep = `156` lượt chạy).

---

## Tóm tắt nhanh các chiến lược tiêu biểu (`60.000` kịch bản Real-Life)

| Nhóm chiến lược | Đại diện nổi bật | Điểm mạnh (`1x_Real` 40y) | Sức chịu bão (`2x_Real` 40y) |
| :--- | :--- | :--- | :--- |
| **Tăng trưởng tối đa** | `DYN_33x3` (33,3% Crypto/Vàng/CP) | `MOIC P50` cao nhất **`9,30x`** (`44.565,2`), nhưng Breach `71,49%` | Rủi ro cháy phí tái cân bằng (`0,64x`, lỗ vốn trung vị) |
| **Cân bằng toàn diện** | `DYN_10_10_40_40` (10/10/40/40 Dynamic) | Tăng trưởng **`8,11x`** (`38.884,5`), Sharpe `1,02`, Win `99,78%` | Giữ được **`2,71x`** (`13.001,0`), Win `76,77%` |
| **Phòng thủ / Sống dai** | `VN_SAFE_DYN` (35/5/30/30 Dynamic) | Sharpe đỉnh (**`1,79`** ở 20y, **`1,61`** ở 40y), Breach chỉ `4,84%` | Top 1 sống sót (**`3,90x`**, `18.682,6`, Win `91,72%`) |
| **Cày dòng tiền VN** | `VN_STARTER_CASHFLOW` (Vốn 20, nạp 15/kỳ) | Tổng tiền tích lũy lớn nhất **`61.333,8`** (`8,69x`) | Chịu chung áp lực phí khi biến động 2x (`1,54x`) |

> Xem phân tích đầy đủ trong [Báo cáo tổng hợp Lab vs Real-Life](data/realLife/reallife_vs_lab_master_report.md) và [Tài liệu tổng hợp toàn bộ Real-Life](data/realLife/reallife_full_compendium.md).

---

## Cấu trúc thư mục

```text
MonteCarloSimulation/
├── wasm-engine/                  # Core Rust/WASM Monte Carlo & Chaos engine
│   ├── src/
│   │   ├── bin/dca_cli.rs        # CLI đa luồng chạy batch suite
│   │   ├── math/                 # Phân phối, Cholesky, quantile, thống kê DCA
│   │   └── models/               # Mô hình DCA, Chaos runtime/spec, báo cáo CLI
│   └── tests/                    # Unit & integration tests (Rust)
├── src/                          # Ứng dụng web SvelteKit + Tailwind CSS
│   ├── lib/components/           # DcaChart, SuiteComparisonTable, RunMetricsPanel
│   ├── lib/types/                # TypeScript types cho dữ liệu mô phỏng
│   └── routes/                   # Trang chính CLI Results Viewer
├── data/
│   ├── configs/                  # Cấu hình danh mục chuẩn, mở rộng VN & 32 sự kiện Chaos
│   ├── lab/                      # 18 báo cáo Lab (baseline)
│   └── realLife/                 # 26 báo cáo Real-Life + Master Report + Full Compendium
├── scripts/
│   └── generate_reallife_reports.py  # Script sinh toàn bộ báo cáo Markdown từ JSON
├── static/results/               # Kết quả JSON cho dashboard web
└── tests/                        # Playwright E2E tests
```

---

## Cài đặt & Sử dụng

### 1. Yêu cầu môi trường

- **Node.js** + `npm`
- **Rust toolchain** (`cargo`)
- **Python 3** (nếu cần chạy script sinh báo cáo Markdown)

Cài đặt dependencies frontend:

```sh
npm install
```

### 2. Chạy mô phỏng qua Rust CLI (`dca_cli`)

Chạy bộ 6 danh mục chuẩn với 32 sự kiện Chaos (`60.000` kịch bản):

```sh
npm run cli:dca -- --config data/configs/default_dca_suite.json --chaos data/configs/chaos_vn_32_events.json --sims 60000 --out static/results/latest_suite.json
```

Chạy bộ 7 danh mục mở rộng (cặp đối xứng + chân dung nhà đầu tư Việt Nam):

```sh
npm run cli:dca -- --config data/configs/reallife_vn_extended_suite.json --chaos data/configs/chaos_vn_32_events.json --sims 60000 --out static/results/reallife_vn_extended_suite.json
```

Các tham số CLI hỗ trợ:

| Tham số | Mô tả | Mặc định |
| :--- | :--- | :--- |
| `--config <path>` | Đường dẫn file cấu hình suite JSON | `data/configs/default_dca_suite.json` |
| `--chaos <path>` | File JSON chứa danh sách sự kiện Chaos | Tắt (nếu không truyền) |
| `--sims <n>` | Số kịch bản Monte Carlo mỗi lượt | Theo file config |
| `--years <10,20,40>` | Ghi đè danh sách kỳ hạn (năm) | Theo file config |
| `--preset <name\|all>` | Chạy riêng 1 danh mục hoặc tất cả | `all` |
| `--fee <float>` | Ghi đè phí bán tái cân bằng cho Dynamic DCA | Theo file config |
| `--out <path>` | Đường dẫn xuất file JSON kết quả | `static/results/latest_suite.json` |

### 3. Sinh lại báo cáo Markdown

```sh
python scripts/generate_reallife_reports.py
```

### 4. Khởi chạy Dashboard xem kết quả

```sh
# Chế độ phát triển
npm run dev

# Build bản production (Cloudflare Pages) & xem thử
npm run build
npm run preview
```

### 5. Kiểm thử & Kiểm tra chất lượng

```sh
# Chạy unit + integration tests của engine Rust
cargo test --manifest-path wasm-engine/Cargo.toml

# Kiểm tra TypeScript, Svelte và giới hạn kích thước file
npm run check

# Kiểm tra format Prettier
npm run lint
```
