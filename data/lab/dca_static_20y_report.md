# Báo cáo Kiểm thử DCA Tĩnh 20 Năm (`25%` Đều 4 Tài sản, 35.000 Kịch bản)

> [!NOTE]
> Mô phỏng bằng WebAssembly CLI (`simulate_dca`) trên 16 luồng CPU với tham số lịch sử chuẩn hóa: `time_years = 20.0`, `steps = 5.040 ngày giao dịch`, `num_sims = 35.000`, `seed = 42`, **DCA Tĩnh (`is_dynamic_dca = false`)**, **Đuôi béo Student-t ($\nu = 4$)**.

---

## 1. Cấu hình Danh mục & Lịch Nạp Vốn 20 Năm

| Tài sản | Tỷ trọng (`weight`) | Lợi suất kỳ vọng (`mu`/năm) | Biến động chuẩn (`sigma`/năm) | Biến động Gấp đôi (`2x sigma`) |
| :--- | :---: | :---: | :---: | :---: |
| **Tiền gửi tiết kiệm** | **25%** | `5,5%` | `1,0%` | `2,0%` |
| **Crypto (BTC / ETH)** | **25%** | `25,0%` | `60,0%` | `120,0%` |
| **Vàng** | **25%** | `9,5%` | `17,0%` | `34,0%` |
| **Cổ phiếu** | **25%** | `10,5%` | `18,0%` | `36,0%` |

- **Tổng vốn thực nạp 20 năm**: Vốn đầu `100` + `10` mỗi `21 ngày` (`240` kỳ) $\rightarrow$ **`total_contributed = 2.500`**.
- **Số lần bán / Phí bán**: `0` lần / `0` phí.

---

## 2. Tốc độ Thực thi (16 Luồng × 35.000 Kịch bản × 5.040 Ngày)

| Kịch bản kiểm thử | Thời gian chạy (16 Workers) |
| :--- | :---: |
| **1. Thông thường (Thực tế)** | `37.686 ms` (~37,7s) |
| **2. Thông thường (Tìm điểm gãy)** | `36.302 ms` (~36,3s) |
| **3. Rủi ro 2x (Thực tế)** | `36.276 ms` (~36,3s) |
| **4. Rủi ro 2x (Tìm điểm gãy)** | `29.030 ms` (~29,0s) |

---

## 3. Kết quả Chi tiết 4 Kịch bản (Tổng vốn góp = `2.500`)

| Chỉ số | 1. Thông thường (Thực tế) | 2. Thông thường (Tìm điểm gãy) | 3. Rủi ro 2x (Thực tế) | 4. Rủi ro 2x (Tìm điểm gãy) |
| :--- | :---: | :---: | :---: | :---: |
| **Kịch bản P10 (Xấu 10%)** | `3.865,60` | `2.612,89` *(Stress)* | `2.038,65` | `1.495,85` *(Stress)* |
| **Kịch bản P25** | `4.992,15` | `3.110,00` | `2.640,85` | `1.721,66` |
| **Trung vị P50 (`median_terminal_nav`)** | **`7.197,56`** | `3.913,04` *(Stress)* | **`3.859,78`** | `2.115,35` *(Stress)* |
| **Kịch bản P75** | `12.972,31` | `5.295,83` | `6.388,20` | `2.792,99` |
| **Kịch bản P90 (Tốt 10%)** | **`31.358,66`** | `8.115,24` | `11.180,98` | `3.942,29` |
| **Lãi ròng P50 (`net_profit_p50`)** | **`+4.697,56`** | `+1.413,04` | **`+1.359,78`** | `-384,65` |
| **Bội số vốn P50 (`moic_p50`)** | **`2,88x`** | `1,57x` | **`1,54x`** | `0,85x` |
| **Tỷ lệ có lãi (`win_rate`)** | **`99,47%`** | `92,72%` *(Stress)* | **`78,52%`** | `33,26%` *(Stress)* |
| **Sharpe Ratio** | **`0,19`** | `0,31` | **`0,13`** | `0,04` |
| **VaR 95% lỗ ròng** | `0,00` | `119,34` | `703,87` | `1.105,33` |
| **CVaR 95% lỗ ròng** | `0,00` | `331,85` | **`883,64`** | `1.192,99` |
| **Số mẫu gãy thu thập (`breach_count`)** | `3.391 / 35.000` | **`11.361 / 35.000`** *(3,35x)* | `31.258 / 35.000` | **`34.650 / 35.000`** |
| **Tỷ lệ gãy thực quy đổi (`breach_rate`)** | **`9,69%`** | **`9,58%`** *(IS quy đổi)* | **`89,31%`** | **`88,80%`** *(IS quy đổi)* |
| **Ngày gãy TB (`avg_breach_day`)** | Ngày `1.164,0` (~4,6 năm) | Ngày `1.505,6` (~6,0 năm) | Ngày `633,0` (~2,5 năm) | Ngày `474,7` (~1,9 năm) |
| **Số lần bán / Tổng phí bán** | `0` / `0` | `0` / `0` | `0` / `0` | `0` / `0` |

### Đóng góp mức sụt giảm trọng số tại thời điểm gãy (`asset_drawdowns_at_ruin`)

| Tài sản (25% đều) | 1. Thông thường (Thực tế) | 2. Thông thường (Tìm điểm gãy) | 3. Rủi ro 2x (Thực tế) | 4. Rủi ro 2x (Tìm điểm gãy) |
| :--- | :---: | :---: | :---: | :---: |
| Tiền gửi (25%) | `0,03%` | `0,03%` | `0,08%` | `0,08%` |
| Crypto BTC/ETH (25%) | **`16,47%`** | **`16,99%`** | **`18,22%`** | **`17,33%`** |
| Vàng (25%) | `3,62%` | `4,13%` | `5,58%` | `4,80%` |
| Cổ phiếu (25%) | `4,03%` | `4,60%` | `6,06%` | `5,28%` |

---

## 4. So sánh Nhanh 20 Năm: DCA Động (`10/10/40/40`, phí `2%`) vs DCA Tĩnh (`25%×4`)

- **Ở mức rủi ro chuẩn (1x)**: DCA Động đạt **P50 `8.470,32` (`3,39x`)**, **Sharpe `1,49`**, **tỷ lệ gãy `5,83%`** (nhờ `4,64` lần bán tốn `49,42` phí); còn DCA Tĩnh đạt P50 `7.197,56` (`2,88x`), Sharpe `0,19`, tỷ lệ gãy `9,69%`, nhưng **P90 vọt lên `31.358,66` (`12,5x`)** nếu Crypto chạy mượt suốt 20 năm.
- **Ở mức rủi ro 2x**: DCA Động giữ P50 **`5.168,59`** (cao hơn `3.859,78` của Tĩnh), nhưng **Tĩnh 25%×4 có tấm đệm 25% Tiền gửi** nên đáy **P10 (`2.038,65`) và CVaR (`883,64`) an toàn hơn** so với Động (`P10 = 1.638,32`, `CVaR = 1.580,31`).

