# Báo cáo Kiểm thử Mô phỏng DCA Động (10 Năm × 35.000 Kịch bản — Đã Chuẩn Hóa)

> [!NOTE]
> Báo cáo được chạy lại trực tiếp trên bản WebAssembly đã cập nhật chuẩn hóa Girsanov Importance Sampling (`src/lib/wasm/wasm_engine.js` + `wasm_engine_bg.wasm`, hàm `simulate_dca`) qua Node.js CLI ở cả chế độ **đơn luồng (1 thread)** và **đa luồng (16 worker threads)**.

---

## 1. Cấu hình Danh mục & Tham số Tham khảo Lịch sử

### 1.1. Tỷ trọng & Tham số Cơ sở (Bộ "Thông thường" – 1x Rủi ro)

Dựa trên dữ liệu thống kê 10 năm (2015–2025) và mặt bằng lãi suất thị trường Việt Nam/Quốc tế:

| Tài sản | Tỷ trọng (`weight`) | Lợi suất kỳ vọng (`mu`/năm) | Biến động chuẩn (`sigma`/năm) | Biến động Gấp đôi (`2x sigma`) | Cơ sở dữ liệu tham khảo |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **Tiền gửi tiết kiệm** | **10%** | `5.5%` (`0.055`) | `1.0%` (`0.01`) | `2.0%` (`0.02`) | Lãi suất tiết kiệm 12 tháng tại VN giai đoạn 2024–2025 (~4.7%–6.0%/năm). |
| **Crypto (BTC / ETH)** | **10%** | `25.0%` (`0.250`) | `60.0%` (`0.60`) | `120.0%` (`1.20`) | Volatility 10 năm: BTC ~46%–67%, ETH ~72%–90%. `mu = 25%` phản ánh kỳ vọng dài hạn thực tế thay vì CAGR 2015–2025 (~71%–94% từ đáy $250). |
| **Vàng** | **40%** | `9.5%` (`0.095`) | `17.0%` (`0.17`) | `34.0%` (`0.34`) | Volatility lịch sử 10 năm của Vàng (~15%–19.6%/năm); CAGR dài hạn cân bằng ~9%–10% (2015–2025 đạt ~13.8%–15.2%). |
| **Cổ phiếu** | **40%** | `10.5%` (`0.105`) | `18.0%` (`0.18`) | `36.0%` (`0.36`) | Pha trộn VN-Index (CAGR ~6%–9%, `sigma` ~17%–23%) và S&P 500 (CAGR ~11.9%–13.6%, `sigma` ~15.3%). |

### 1.2. Ma trận Tương quan 4×4 (`corr_matrix`)

Kiểm định Cholesky: **`psd_repaired = false`** (ma trận hợp lệ tuyệt đối, không cần sửa PSD).

| | Tiền gửi | Crypto (BTC/ETH) | Vàng | Cổ phiếu |
| :--- | :---: | :---: | :---: | :---: |
| **Tiền gửi** | `1.00` | `0.00` | `+0.10` | `-0.15` |
| **Crypto (BTC/ETH)** | `0.00` | `1.00` | `+0.20` | `+0.30` |
| **Vàng** | `+0.10` | `+0.20` | `1.00` | `+0.05` |
| **Cổ phiếu** | `-0.15` | `+0.30` | `+0.05` | `1.00` |

### 1.3. Tham số DCA Động & Môi trường Mô phỏng

- **Thời gian & Số kịch bản**: `10 năm` (`time_years = 10.0`, `steps = 2.520 ngày giao dịch`), `num_sims = 35.000` (**88,2 triệu bước mô phỏng** mỗi trường hợp).
- **Lịch nạp vốn (Quy ước đơn vị chuẩn)**: Vốn ban đầu `100`, nạp định kỳ `10` mỗi `21 ngày` (120 kỳ trong 10 năm) $\rightarrow$ **Tổng vốn góp `total_contributed = 1.300`**.
- **Cơ chế DCA Động**: `is_dynamic_dca = true`, **độ lệch bán `tolerance_band_pct = 10%` (`0.10`)**, **phí bán `sell_fee_pct = 2.0%` (`0.02`)** *(đã cập nhật từ 0,5% lên 2,0% để sát chi phí giao dịch/spread thực tế)*.
- **Ngưỡng Điểm gãy & Đuôi béo**: `ruin_drawdown_pct = 20%` (gãy nếu tài sản $\le 80\%$ tổng vốn đã góp lũy kế), `is_fat_tail = true` (Student-t $\nu = 4$), `crisis_corr = 0.85`, `crisis_threshold_pct = 0.15`.
- **Thuật toán Tìm điểm gãy (`is_stress_test = true`)**: Đã chuẩn hóa $\theta = -2,0 / \sqrt{\text{steps} \times n_{\text{assets}}}$ (`half_theta_sq = 0.5 * theta^2`) và dừng tích lũy Radon-Nikodym ngay tại thời điểm chạm sàn (`if !breached`).

---

## 2. Hiệu năng & Tốc độ Thực thi (WASM CLI)

Đo trên máy 16 luồng logic (`seed = 42`, 35.000 kịch bản × 2.520 ngày):

| Trường hợp kiểm thử | Đa luồng (16 Workers) | Đơn luồng (1 Thread) | Tăng tốc (Speedup) | Ghi chú |
| :--- | :---: | :---: | :---: | :--- |
| **1. Thông thường – Thực tế** | `18.618 ms` (~18,62s) | `22.473 ms` (~22,47s) | ~1,21x | Lần chạy đầu chịu chi phí khởi tạo & JIT 16 Worker đồng thời. |
| **2. Thông thường – Tìm điểm gãy** | `2.470 ms` (**~2,47s**) | `22.457 ms` (~22,46s) | **9,09x** | Tốc độ thực khi các worker đã nóng. |
| **3. Rủi ro Gấp đôi – Thực tế** | `2.538 ms` (**~2,54s**) | `22.037 ms` (~22,04s) | **8,68x** | Rất ổn định dù bật tái cân bằng bán & Cholesky 4×4. |
| **4. Rủi ro Gấp đôi – Tìm điểm gãy** | `2.525 ms` (**~2,53s**) | `21.873 ms` (~21,87s) | **8,66x** | Xử lý 88,2 triệu bước trong ~2,5 giây. |

---

## 3. Bảng Kết quả Chi tiết 4 Kịch bản (Tổng vốn góp = `1.300`, Phí bán = `2,0%`)

| Chỉ số | 1. Thông thường (Thực tế) | 2. Thông thường (Tìm điểm gãy) | 3. Rủi ro 2x (Thực tế) | 4. Rủi ro 2x (Tìm điểm gãy) |
| :--- | :---: | :---: | :---: | :---: |
| **Kịch bản P10 (Xấu 10%)** | `1.646,22` | `1.146,07` *(Stress)* | `894,62` | `453,49` *(Stress)* |
| **Kịch bản P25** | `1.940,87` | `1.339,11` | `1.286,32` | `624,69` |
| **Trung vị P50 (`median_terminal_nav`)** | **`2.348,37`** | `1.596,75` *(Stress)* | **`1.921,07`** | `909,06` *(Stress)* |
| **Kịch bản P75** | `2.869,59` | `1.919,16` | `2.883,19` | `1.317,92` |
| **Kịch bản P90 (Tốt 10%)** | `3.468,29` | `2.282,58` | `4.230,59` | `1.864,97` |
| **Lãi ròng P50 (`net_profit_p50`)** | **`+1.048,37`** | `+296,75` | **`+621,07`** | `-390,94` |
| **Bội số vốn P50 (`moic_p50`)** | **`1,81x`** | `1,23x` | **`1,48x`** | `0,70x` |
| **Tỷ lệ có lãi (`win_rate`)** | **`98,32%`** | `78,50%` *(trong môi trường Stress)* | **`74,40%`** | `25,83%` *(trong môi trường Stress)* |
| **Sharpe Ratio** | **`1,54`** | `0,78` | **`0,64`** | `-0,35` |
| **VaR 95% lỗ ròng** | `0,00` | `259,22` | `577,11` | `921,49` |
| **CVaR 95% lỗ ròng** | `0,71` | `368,48` | `718,81` | `980,60` |
| **Số mẫu gãy thu thập (`breach_count`)** | `1.747 / 35.000` | **`10.709 / 35.000`** *(tăng 6,13x)* | `25.617 / 35.000` | **`33.821 / 35.000`** |
| **Tỷ lệ gãy thực quy đổi (`breach_rate`)** | **`4,99%`** | **`5,03%`** *(IS quy đổi)* | **`73,19%`** | **`72,91%`** *(IS quy đổi)* |
| **Ngày gãy TB (`avg_breach_day`)** | Ngày `937,4` (~3,7 năm) | Ngày `1.168,1` (~4,6 năm) | Ngày `645,6` (~2,6 năm) | Ngày `500,5` (~2,0 năm) |
| **Số lần bán tái cân bằng TB** | `1,64 lần` | `1,06 lần` | `5,72 lần` | `4,13 lần` |
| **Tổng phí bán TB (`avg_total_fees`)** | **`6,00`** | `2,98` | **`20,73`** | `8,80` |

### Đóng góp mức sụt giảm trọng số tại thời điểm gãy (`asset_drawdowns_at_ruin`)

| Tài sản (Trọng số) | 1. Thông thường (Thực tế) | 2. Thông thường (Tìm điểm gãy) | 3. Rủi ro 2x (Thực tế) | 4. Rủi ro 2x (Tìm điểm gãy) |
| :--- | :---: | :---: | :---: | :---: |
| Tiền gửi (10%) | `0,01%` | `0,01%` | `0,18%` | `0,10%` |
| Crypto BTC/ETH (10%) | `3,96%` | `4,24%` | `6,00%` | `5,60%` |
| Vàng (40%) | `4,85%` | `5,26%` | `9,88%` | `8,40%` |
| Cổ phiếu (40%) | `5,29%` | `5,70%` | `10,44%` | `8,96%` |

---