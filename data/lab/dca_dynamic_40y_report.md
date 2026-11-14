# Báo cáo Kiểm thử DCA Động 40 Năm (`10/10/40/40`, Phí bán `2,0%`, 35.000 Kịch bản)

> [!NOTE]
> Mô phỏng bằng WebAssembly CLI (`simulate_dca`) trên 16 luồng CPU với tham số lịch sử chuẩn hóa: `time_years = 40.0`, `steps = 10.080 ngày giao dịch`, `num_sims = 35.000`, `seed = 42`, **phí bán `sell_fee_pct = 2,0%`**, **độ lệch bán `10%`**, **Đuôi béo Student-t ($\nu = 4$)**.

---

## 1. Cấu hình Danh mục & Lịch Nạp Vốn 40 Năm

| Tài sản | Tỷ trọng (`weight`) | Lợi suất kỳ vọng (`mu`/năm) | Biến động chuẩn (`sigma`/năm) | Biến động Gấp đôi (`2x sigma`) |
| :--- | :---: | :---: | :---: | :---: |
| **Tiền gửi tiết kiệm** | **10%** | `5,5%` | `1,0%` | `2,0%` |
| **Crypto (BTC / ETH)** | **10%** | `25,0%` | `60,0%` | `120,0%` |
| **Vàng** | **40%** | `9,5%` | `17,0%` | `34,0%` |
| **Cổ phiếu** | **40%** | `10,5%` | `18,0%` | `36,0%` |

- **Tổng vốn thực nạp 40 năm**: Vốn đầu `100` + `10` mỗi `21 ngày` (`480` kỳ) $\rightarrow$ **`total_contributed = 4.900`**.
- **Ngưỡng gãy**: Thủng `-20%` so với tổng vốn góp lũy kế (`ruin_drawdown_pct = 0.20`).

---

## 2. Tốc độ Thực thi (16 Luồng × 35.000 Kịch bản × 10.080 Ngày)

| Kịch bản kiểm thử | Thời gian chạy (16 Workers) |
| :--- | :---: |
| **1. Thông thường (Thực tế)** | `91.729 ms` (~91,7s) |
| **2. Thông thường (Tìm điểm gãy)** | `93.190 ms` (~93,2s) |
| **3. Rủi ro 2x (Thực tế)** | `75.141 ms` (~75,1s) |
| **4. Rủi ro 2x (Tìm điểm gãy)** | `71.840 ms` (~71,8s) |

*(Mỗi kịch bản 40 năm xử lý **352,8 triệu bước mô phỏng** đa tài sản có Cholesky 4×4 & tái cân bằng động).*

---

## 3. Kết quả Chi tiết 4 Kịch bản (Tổng vốn góp = `4.900`)

| Chỉ số | 1. Thông thường (Thực tế) | 2. Thông thường (Tìm điểm gãy) | 3. Rủi ro 2x (Thực tế) | 4. Rủi ro 2x (Tìm điểm gãy) |
| :--- | :---: | :---: | :---: | :---: |
| **Kịch bản P10 (Xấu 10%)** | **`27.437,79`** | `10.456,55` *(Stress)* | `3.491,62` | `1.009,96` *(Stress)* |
| **Kịch bản P25** | `42.317,20` | `15.970,22` | `7.845,50` | `1.746,18` |
| **Trung vị P50 (`median_terminal_nav`)** | **`68.569,23`** | `25.290,89` *(Stress)* | **`20.404,60`** | `3.599,76` *(Stress)* |
| **Kịch bản P75** | `112.278,22` | `40.585,88` | `57.038,58` | `8.298,47` |
| **Kịch bản P90 (Tốt 10%)** | `176.785,16` | `62.172,97` | `146.969,63` | `18.985,81` |
| **Lãi ròng P50 (`net_profit_p50`)** | **`+63.669,23`** | `+20.390,89` | **`+15.504,60`** | `-1.300,24` |
| **Bội số vốn P50 (`moic_p50`)** | **`13,99x`** | `5,16x` | **`4,16x`** | `0,73x` |
| **Tỷ lệ có lãi (`win_rate`)** | **`99,98%`** | `98,99%` *(Stress)* | **`84,67%`** | `39,86%` *(Stress)* |
| **Sharpe Ratio** | **`1,09`** | `1,04` | **`0,29`** | `0,18` |
| **VaR 95% lỗ ròng** | `0,00` | `0,00` | `2.646,79` | `4.127,74` |
| **CVaR 95% lỗ ròng** | `0,00` | `0,00` | `3.321,32` | `4.296,51` |
| **Số mẫu gãy thu thập (`breach_count`)** | `2.124 / 35.000` | **`7.076 / 35.000`** *(3,33x)* | `29.909 / 35.000` | **`34.381 / 35.000`** |
| **Tỷ lệ gãy thực quy đổi (`breach_rate`)** | **`6,07%`** | **`5,89%`** *(IS quy đổi)* | **`85,45%`** | **`85,46%`** *(IS quy đổi)* |
| **Ngày gãy TB (`avg_breach_day`)** | Ngày `1.431,5` (~5,7 năm) | Ngày `2.292,3` (~9,1 năm) | Ngày `1.248,2` (~5,0 năm) | Ngày `995,4` (~3,9 năm) |
| **Số lần bán tái cân bằng TB** | `11,51 lần` | `9,54 lần` | `24,92 lần` | `19,93 lần` |
| **Tổng phí bán TB (`avg_total_fees`)** | **`710,91`** | `281,71` | **`1.455,16`** | `272,89` |

### Đóng góp mức sụt giảm trọng số tại thời điểm gãy (`asset_drawdowns_at_ruin`)

| Tài sản (Trọng số) | 1. Thông thường (Thực tế) | 2. Thông thường (Tìm điểm gãy) | 3. Rủi ro 2x (Thực tế) | 4. Rủi ro 2x (Tìm điểm gãy) |
| :--- | :---: | :---: | :---: | :---: |
| Tiền gửi (10%) | `0,16%` | `0,46%` | `0,79%` | `0,53%` |
| Crypto BTC/ETH (10%) | `4,52%` | `5,26%` | `6,45%` | `6,19%` |
| Vàng (40%) | `6,64%` | `8,79%` | `12,37%` | `11,00%` |
| Cổ phiếu (40%) | `7,00%` | `9,25%` | `12,88%` | `11,54%` |

