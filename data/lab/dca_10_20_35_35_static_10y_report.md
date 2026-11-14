# Báo cáo Kiểm thử Danh mục `10/20/35/35` (DCA Tĩnh) — Khung 10 Năm

> [!NOTE]
> Mô phỏng bằng **Native Rust CLI (`-C target-cpu=native`, 16 luồng OS)** với tham số chuẩn hóa: `time_years = 10.0`, `steps = 2.520 ngày giao dịch`, `num_sims = 35.000`, `seed = 42`, phân phối **Đuôi béo Student-t ($\nu = 4$)**, ngưỡng gãy `-20%` tổng vốn góp lũy kế (`ruin_drawdown_pct = 0.20`).
> - **Chế độ**: **DCA Tĩnh** — Chỉ mua dồn theo tỷ trọng mục tiêu, không bán tái cân bằng (`0` lần bán, `0` phí).

---

## 1. Cấu hình Danh mục & Lịch Nạp Vốn 10 Năm (4 Tài sản)

| Tài sản | Tỷ trọng (`weight`) | Lợi suất kỳ vọng (`mu`/năm) | Biến động chuẩn (`sigma`/năm) | Biến động Gấp đôi (`2x sigma`) |
| :--- | :---: | :---: | :---: | :---: |
| **Tiền gửi tiết kiệm** | **10%** | `5,5%` | `1,0%` | `2,0%` |
| **Crypto (BTC / ETH)** | **20%** | `25,0%` | `60,0%` | `120,0%` |
| **Vàng** | **35%** | `9,5%` | `17,0%` | `34,0%` |
| **Cổ phiếu** | **35%** | `10,5%` | `18,0%` | `36,0%` |

- **Ma trận tương quan `4×4`**: Tiền gửi–Vàng (`+0,10`), Tiền gửi–Cổ phiếu (`-0,15`), Crypto–Vàng (`+0,20`), Crypto–Cổ phiếu (`+0,30`), Vàng–Cổ phiếu (`+0,05`).
- **Tổng vốn thực nạp 10 năm**: Vốn đầu `100` + `10` mỗi `21 ngày` (`120` kỳ) $\rightarrow$ **`total_contributed = 1.300`**.

---

## 2. Kết quả Chi tiết 4 Kịch bản DCA Tĩnh 10 Năm (Tổng vốn góp = `1.300`)

| Chỉ số | 1. Thông thường (Thực tế) | 2. Thông thường (Tìm điểm gãy) | 3. Rủi ro 2x (Thực tế) | 4. Rủi ro 2x (Tìm điểm gãy) |
| :--- | :---: | :---: | :---: | :---: |
| **Thời gian chạy (16 luồng)** | `4.823 ms` | `5.018 ms` | `4.952 ms` | `5.198 ms` |
| **P10 (Xấu 10%)** | `1.504,2` | `1.069,5` *(Stress)* | `842,3` | `527,2` *(Stress)* |
| **Trung vị P50 (`median_terminal_nav`)** | **`2.278,0`** | `1.496,1` *(Stress)* | **`1.632,3`** | `872,8` *(Stress)* |
| **P90 (Tốt 10%)** | **`4.643,0`** | `2.336,3` | `3.772,0` | `1.633,1` |
| **Bội số vốn P50 (`moic_p50`)** | **`1,75x`** | `1,15x` | **`1,26x`** | `0,67x` |
| **Tỷ lệ có lãi (`win_rate`)** | **`96,31%`** | `69,95%` *(Stress)* | **`66,21%`** | `19,99%` *(Stress)* |
| **Sharpe Ratio** | `0,51` | `0,43` | `0,17` | `-0,39` |
| **CVaR 95% lỗ ròng** | `93,0` | `420,0` | `690,7` | `882,7` |
| **Số mẫu gãy (`breach_count`)** | `3.926 / 35.000` | `15.989 / 35.000` | `29.854 / 35.000` | `34.573 / 35.000` |
| **Tỷ lệ gãy (`breach_rate`)** | **`11,22%`** | **`11,03%`** *(IS quy đổi)* | **`85,30%`** | **`83,94%`** *(IS quy đổi)* |
| **Số lần bán / Tổng phí TB** | `0` / `0` | `0` / `0` | `0` / `0` | `0` / `0` |

---

## 3. Đóng góp Sụt giảm tại Thời điểm Gãy (`asset_drawdowns_at_ruin`) & Nhận xét

| Chế độ (Thực tế) | Tiền gửi (`10%`) | Crypto (`20%`) | Vàng (`35%`) | Cổ phiếu (`35%`) |
| :--- | :---: | :---: | :---: | :---: |
| **1x – DCA Tĩnh** | `0,01%` | `11,9%` | `4,5%` | `5,0%` |
| **2x – DCA Tĩnh** | `0,03%` | `14,0%` | `7,4%` | `8,1%` |

- **Rủi ro chuẩn (1x):** DCA Tĩnh giữ tỷ lệ gãy thấp hơn bản Động (`11,22%` vs `15,92%`) và đỉnh `P90` cao hơn (`4.643,0` vs `4.047,7`), đổi lại `P50` (`2.278,0`) và `Sharpe` (`0,51`) thấp hơn.
- **Rủi ro gấp đôi (2x):** Bảo vệ đuôi xấu tốt hơn bản Động (`P10 = 842,3` vs `687,7`, `CVaR = 690,7` vs `868,3`, `Win Rate = 66,21%` vs `65,05%`) vì không rút Vàng/Cổ phiếu sang cứu Crypto.

