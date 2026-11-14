# Báo cáo Kiểm thử DCA Động 20 Năm (`10/10/40/40`, Phí bán `2,0%`, 35.000 Kịch bản)

> [!NOTE]
> Mô phỏng bằng WebAssembly CLI (`simulate_dca`) trên 16 luồng CPU với tham số lịch sử chuẩn hóa: `time_years = 20.0`, `steps = 5.040 ngày giao dịch`, `num_sims = 35.000`, `seed = 42`, **phí bán `sell_fee_pct = 2,0%`**, **độ lệch bán `10%`**, **Đuôi béo Student-t ($\nu = 4$)**.

---

## 1. Cấu hình Danh mục & Lịch Nạp Vốn 20 Năm

| Tài sản | Tỷ trọng (`weight`) | Lợi suất kỳ vọng (`mu`/năm) | Biến động chuẩn (`sigma`/năm) | Biến động Gấp đôi (`2x sigma`) |
| :--- | :---: | :---: | :---: | :---: |
| **Tiền gửi tiết kiệm** | **10%** | `5,5%` | `1,0%` | `2,0%` |
| **Crypto (BTC / ETH)** | **10%** | `25,0%` | `60,0%` | `120,0%` |
| **Vàng** | **40%** | `9,5%` | `17,0%` | `34,0%` |
| **Cổ phiếu** | **40%** | `10,5%` | `18,0%` | `36,0%` |

- **Tổng vốn thực nạp 20 năm**: Vốn đầu `100` + `10` mỗi `21 ngày` (`240` kỳ) $\rightarrow$ **`total_contributed = 2.500`**.
- **Ngưỡng gãy**: Thủng `-20%` so với tổng vốn góp lũy kế (`ruin_drawdown_pct = 0.20`).

---

## 2. Tốc độ Thực thi (16 Luồng × 35.000 Kịch bản × 5.040 Ngày)

| Kịch bản kiểm thử | Thời gian chạy (16 Workers) |
| :--- | :---: |
| **1. Thông thường (Thực tế)** | `36.630 ms` (~36,6s) |
| **2. Thông thường (Tìm điểm gãy)** | `36.761 ms` (~36,8s) |
| **3. Rủi ro 2x (Thực tế)** | `36.935 ms` (~36,9s) |
| **4. Rủi ro 2x (Tìm điểm gãy)** | `36.544 ms` (~36,5s) |

---

## 3. Kết quả Chi tiết 4 Kịch bản (Tổng vốn góp = `2.500`)

| Chỉ số | 1. Thông thường (Thực tế) | 2. Thông thường (Tìm điểm gãy) | 3. Rủi ro 2x (Thực tế) | 4. Rủi ro 2x (Tìm điểm gãy) |
| :--- | :---: | :---: | :---: | :---: |
| **Kịch bản P10 (Xấu 10%)** | `4.846,91` | `2.741,55` *(Stress)* | `1.638,32` | `667,12` *(Stress)* |
| **Kịch bản P25** | `6.284,41` | `3.522,83` | `2.793,09` | `1.004,48` |
| **Trung vị P50 (`median_terminal_nav`)** | **`8.470,32`** | `4.657,72` *(Stress)* | **`5.168,59`** | `1.695,60` *(Stress)* |
| **Kịch bản P75** | `11.481,68` | `6.179,29` | `9.894,00` | `2.991,81` |
| **Kịch bản P90 (Tốt 10%)** | `15.376,66` | `8.074,16` | `17.893,76` | `5.021,01` |
| **Lãi ròng P50 (`net_profit_p50`)** | **`+5.970,32`** | `+2.157,72` | **`+2.668,59`** | `-804,40` |
| **Bội số vốn P50 (`moic_p50`)** | **`3,39x`** | `1,86x` | **`2,07x`** | `0,68x` |
| **Tỷ lệ có lãi (`win_rate`)** | **`99,72%`** | `93,43%` *(Stress)* | **`78,77%`** | `32,14%` *(Stress)* |
| **Sharpe Ratio** | **`1,49`** | `1,12` | **`0,54`** | `-0,01` |
| **VaR 95% lỗ ròng** | `0,00` | `140,25` | `1.277,41` | `1.964,50` |
| **CVaR 95% lỗ ròng** | **`0,00`** | `517,16` | `1.580,31` | `2.066,63` |
| **Số mẫu gãy thu thập (`breach_count`)** | `2.041 / 35.000` | **`9.437 / 35.000`** *(4,62x)* | `28.437 / 35.000` | **`34.266 / 35.000`** |
| **Tỷ lệ gãy thực quy đổi (`breach_rate`)** | **`5,83%`** | **`5,79%`** *(IS quy đổi)* | **`81,25%`** | **`80,23%`** *(IS quy đổi)* |
| **Ngày gãy TB (`avg_breach_day`)** | Ngày `1.301,6` (~5,2 năm) | Ngày `1.807,5` (~7,2 năm) | Ngày `920,9` (~3,7 năm) | Ngày `705,0` (~2,8 năm) |
| **Số lần bán tái cân bằng TB** | `4,64 lần` | `3,49 lần` | `12,34 lần` | `9,53 lần` |
| **Tổng phí bán TB (`avg_total_fees`)** | **`49,42`** | `24,45` | **`127,53`** | `42,97` |

### Đóng góp mức sụt giảm trọng số tại thời điểm gãy (`asset_drawdowns_at_ruin`)

| Tài sản (Trọng số) | 1. Thông thường (Thực tế) | 2. Thông thường (Tìm điểm gãy) | 3. Rủi ro 2x (Thực tế) | 4. Rủi ro 2x (Tìm điểm gãy) |
| :--- | :---: | :---: | :---: | :---: |
| Tiền gửi (10%) | `0,08%` | `0,11%` | `0,49%` | `0,27%` |
| Crypto BTC/ETH (10%) | `4,40%` | `5,03%` | `6,27%` | `5,91%` |
| Vàng (40%) | `6,23%` | `7,59%` | `11,31%` | `9,71%` |
| Cổ phiếu (40%) | `6,64%` | `8,08%` | `11,85%` | `10,25%` |

