# Báo cáo Kiểm thử DCA Tĩnh 40 Năm (`25%` Đều 4 Tài sản, 35.000 Kịch bản)

> [!NOTE]
> Mô phỏng bằng WebAssembly CLI (`simulate_dca`) trên 16 luồng CPU với tham số lịch sử chuẩn hóa: `time_years = 40.0`, `steps = 10.080 ngày giao dịch`, `num_sims = 35.000`, `seed = 42`, **DCA Tĩnh (`is_dynamic_dca = false`)**, **Đuôi béo Student-t ($\nu = 4$)**.

---

## 1. Cấu hình Danh mục & Lịch Nạp Vốn 40 Năm

| Tài sản | Tỷ trọng (`weight`) | Lợi suất kỳ vọng (`mu`/năm) | Biến động chuẩn (`sigma`/năm) | Biến động Gấp đôi (`2x sigma`) |
| :--- | :---: | :---: | :---: | :---: |
| **Tiền gửi tiết kiệm** | **25%** | `5,5%` | `1,0%` | `2,0%` |
| **Crypto (BTC / ETH)** | **25%** | `25,0%` | `60,0%` | `120,0%` |
| **Vàng** | **25%** | `9,5%` | `17,0%` | `34,0%` |
| **Cổ phiếu** | **25%** | `10,5%` | `18,0%` | `36,0%` |

- **Tổng vốn thực nạp 40 năm**: Vốn đầu `100` + `10` mỗi `21 ngày` (`480` kỳ) $\rightarrow$ **`total_contributed = 4.900`**.
- **Số lần bán / Phí bán**: `0` lần / `0` phí.

---

## 2. Tốc độ Thực thi (16 Luồng × 35.000 Kịch bản × 10.080 Ngày)

| Kịch bản kiểm thử | Thời gian chạy (16 Workers) |
| :--- | :---: |
| **1. Thông thường (Thực tế)** | `72.408 ms` (~72,4s) |
| **2. Thông thường (Tìm điểm gãy)** | `72.809 ms` (~72,8s) |
| **3. Rủi ro 2x (Thực tế)** | `63.739 ms` (~63,7s) |
| **4. Rủi ro 2x (Tìm điểm gãy)** | `73.170 ms` (~73,2s) |

---

## 3. Kết quả Chi tiết 4 Kịch bản (Tổng vốn góp = `4.900`)

| Chỉ số | 1. Thông thường (Thực tế) | 2. Thông thường (Tìm điểm gãy) | 3. Rủi ro 2x (Thực tế) | 4. Rủi ro 2x (Tìm điểm gãy) |
| :--- | :---: | :---: | :---: | :---: |
| **Kịch bản P10 (Xấu 10%)** | `16.845,30` | `9.098,09` *(Stress)* | **`6.273,55`** | `4.674,87` *(Stress)* |
| **Kịch bản P25** | `25.690,05` | `11.753,22` | `8.518,94` | `5.309,50` |
| **Trung vị P50 (`median_terminal_nav`)** | **`47.953,00`** | `16.796,92` *(Stress)* | **`14.366,12`** | `6.474,41` *(Stress)* |
| **Kịch bản P75** | `132.621,12` | `27.538,66` | `30.637,81` | `8.938,11` |
| **Kịch bản P90 (Tốt 10%)** | **`640.102,90`** | `54.144,29` | `71.651,91` | `14.234,16` |
| **Lãi ròng P50 (`net_profit_p50`)** | **`+43.053,00`** | `+11.896,92` | **`+9.466,12`** | `+1.574,41` |
| **Bội số vốn P50 (`moic_p50`)** | **`9,79x`** | `3,43x` | **`2,93x`** | `1,32x` |
| **Tỷ lệ có lãi (`win_rate`)** | **`100,00%`** | `100,00%` *(Stress)* | **`98,12%`** | `85,13%` *(Stress)* |
| **Sharpe Ratio** | **`0,08`** | `0,11` | **`0,21`** | `0,32` |
| **VaR 95% lỗ ròng** | `0,00` | `0,00` | **`0,00`** | `505,55` |
| **CVaR 95% lỗ ròng** | `0,00` | `0,00` | **`9,45`** | `758,01` |
| **Số mẫu gãy thu thập (`breach_count`)** | `3.372 / 35.000` | **`8.350 / 35.000`** *(2,48x)* | `31.512 / 35.000` | **`34.413 / 35.000`** |
| **Tỷ lệ gãy thực quy đổi (`breach_rate`)** | **`9,63%`** | **`9,73%`** *(IS quy đổi)* | **`90,03%`** | **`90,20%`** *(IS quy đổi)* |
| **Ngày gãy TB (`avg_breach_day`)** | Ngày `1.162,8` (~4,6 năm) | Ngày `1.487,6` (~5,9 năm) | Ngày `695,3` (~2,8 năm) | Ngày `581,6` (~2,3 năm) |
| **Số lần bán / Tổng phí bán** | `0` / `0` | `0` / `0` | `0` / `0` | `0` / `0` |

### Đóng góp mức sụt giảm trọng số tại thời điểm gãy (`asset_drawdowns_at_ruin`)

| Tài sản (25% đều) | 1. Thông thường (Thực tế) | 2. Thông thường (Tìm điểm gãy) | 3. Rủi ro 2x (Thực tế) | 4. Rủi ro 2x (Tìm điểm gãy) |
| :--- | :---: | :---: | :---: | :---: |
| Tiền gửi (25%) | `0,03%` | `0,03%` | `0,08%` | `0,08%` |
| Crypto BTC/ETH (25%) | **`16,43%`** | **`17,09%`** | **`18,31%`** | **`17,72%`** |
| Vàng (25%) | `3,63%` | `4,19%` | `5,73%` | `5,18%` |
| Cổ phiếu (25%) | `4,04%` | `4,61%` | `6,21%` | `5,68%` |

---

## 4. Nhận xét Nổi bật trên Khung 40 Năm

1. **Lãi kép DCA Động thắng áp đảo ở P50 & Đáy P10 (Rủi ro 1x)**:
   - Ở 40 năm, **DCA Động `10/10/40/40`** dù chịu `710,91` phí bán (`11,51` lần bán) vẫn đẩy **P50 lên `68.569,23` (`13,99x`)** và **P10 lên `27.437,79`** — cao hơn hẳn **P50 (`47.953,00` / `9,79x`)** và **P10 (`16.845,30`)** của DCA Tĩnh 25%×4!
   - *(Lưu ý: `P90 = 640.102,90` của DCA Tĩnh 40 năm là kết quả toán học khi giả định Crypto giữ `mu = 25%/năm` liên tục suốt 40 năm không giảm tốc — ngoài thực tế khó duy trì tốc độ đó ở quy mô siêu lớn).*
2. **Sức mạnh của "Khiên 25% Tiền gửi" ở Rủi ro 2x trong 40 năm**:
   - Khi biến động nhân đôi suốt 40 năm, **DCA Tĩnh 25%×4** có **Win Rate `98,12%`**, **P10 = `6.273,55`** (vẫn lãi so với `4.900` vốn góp) và **CVaR 95% chỉ `9,45`** — an toàn hơn ở đuôi xấu so với **DCA Động `10/10/40/40`** (`Win Rate = 84,67%`, `P10 = 3.491,62`, `CVaR = 3.321,32`) vì `25%` Tiền gửi tích lũy liên tục trong 40 năm tạo thành mỏ neo vững chắc khi 80% Vàng + Cổ phiếu bị gấp đôi biến động.
3. **Độ chuẩn thuật toán Tìm điểm gãy trên 10.080 bước**: Tỷ lệ gãy quy đổi IS đạt **`5,89%`** vs `6,07%` (Động 1x), **`85,46%`** vs `85,45%` (Động 2x), **`9,73%`** vs `9,63%` (Tĩnh 1x), **`90,20%`** vs `90,03%` (Tĩnh 2x) — sai số dưới **0,2 điểm %** ngay cả ở kỳ hạn 40 năm!

