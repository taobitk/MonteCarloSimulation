# Báo cáo Kiểm thử Danh mục `10/20/35/35` (DCA Tĩnh) — Khung 40 Năm

> [!NOTE]
> Mô phỏng bằng **Native Rust CLI (`-C target-cpu=native`, 16 luồng OS)** với tham số chuẩn hóa: `time_years = 40.0`, `steps = 10.080 ngày giao dịch`, `num_sims = 35.000`, `seed = 42`, phân phối **Đuôi béo Student-t ($\nu = 4$)**, ngưỡng gãy `-20%` tổng vốn góp lũy kế (`ruin_drawdown_pct = 0.20`).
> - **Chế độ**: **DCA Tĩnh** — Chỉ mua dồn theo tỷ trọng mục tiêu, không bán tái cân bằng (`0` lần bán, `0` phí).

---

## 1. Cấu hình Danh mục & Lịch Nạp Vốn 40 Năm (4 Tài sản)

| Tài sản | Tỷ trọng (`weight`) | Lợi suất kỳ vọng (`mu`/năm) | Biến động chuẩn (`sigma`/năm) | Biến động Gấp đôi (`2x sigma`) |
| :--- | :---: | :---: | :---: | :---: |
| **Tiền gửi tiết kiệm** | **10%** | `5,5%` | `1,0%` | `2,0%` |
| **Crypto (BTC / ETH)** | **20%** | `25,0%` | `60,0%` | `120,0%` |
| **Vàng** | **35%** | `9,5%` | `17,0%` | `34,0%` |
| **Cổ phiếu** | **35%** | `10,5%` | `18,0%` | `36,0%` |

- **Ma trận tương quan `4×4`**: Tiền gửi–Vàng (`+0,10`), Tiền gửi–Cổ phiếu (`-0,15`), Crypto–Vàng (`+0,20`), Crypto–Cổ phiếu (`+0,30`), Vàng–Cổ phiếu (`+0,05`).
- **Tổng vốn thực nạp 40 năm**: Vốn đầu `100` + `10` mỗi `21 ngày` (`480` kỳ) $\rightarrow$ **`total_contributed = 4.900`**.

---

## 2. Kết quả Chi tiết 4 Kịch bản DCA Tĩnh 40 Năm (Tổng vốn góp = `4.900`)

| Chỉ số | 1. Thông thường (Thực tế) | 2. Thông thường (Tìm điểm gãy) | 3. Rủi ro 2x (Thực tế) | 4. Rủi ro 2x (Tìm điểm gãy) |
| :--- | :---: | :---: | :---: | :---: |
| **Thời gian chạy (16 luồng)** | `17.666 ms` | `18.742 ms` | `19.265 ms` | `18.513 ms` |
| **P10 (Xấu 10%)** | `18.517,3` | `8.055,5` *(Stress)* | **`4.286,0`** | `2.407,6` *(Stress)* |
| **Trung vị P50 (`median_terminal_nav`)** | **`55.512,2`** | `18.092,7` *(Stress)* | **`15.268,9`** | `4.439,3` *(Stress)* |
| **P90 (Tốt 10%)** | **`552.465,9`** | `58.550,2` | `95.356,8` | `14.607,5` |
| **Bội số vốn P50 (`moic_p50`)** | **`11,33x`** | `3,69x` | **`3,12x`** | `0,91x` |
| **Tỷ lệ có lãi (`win_rate`)** | **`99,99%`** | `98,82%` *(Stress)* | **`86,58%`** | `44,34%` *(Stress)* |
| **Sharpe Ratio** | `0,08` | `0,13` | `0,19` | `0,18` |
| **CVaR 95% lỗ ròng** | **`0,0`** | `0,0` | `2.085,2` | `2.916,5` |
| **Số mẫu gãy (`breach_count`)** | `4.691 / 35.000` | `12.099 / 35.000` | `32.259 / 35.000` | `34.715 / 35.000` |
| **Tỷ lệ gãy (`breach_rate`)** | **`13,40%`** | **`13,38%`** *(IS quy đổi)* | **`92,17%`** | **`92,29%`** *(IS quy đổi)* |
| **Số lần bán / Tổng phí TB** | `0` / `0` | `0` / `0` | `0` / `0` | `0` / `0` |

---

## 3. Đóng góp Sụt giảm tại Thời điểm Gãy (`asset_drawdowns_at_ruin`) & Nhận xét

| Chế độ (Thực tế) | Tiền gửi (`10%`) | Crypto (`20%`) | Vàng (`35%`) | Cổ phiếu (`35%`) |
| :--- | :---: | :---: | :---: | :---: |
| **1x – DCA Tĩnh** | `0,01%` | `13,0%` | `5,8%` | `6,4%` |
| **2x – DCA Tĩnh** | `0,03%` | `14,4%` | `8,6%` | `9,2%` |

- **Rủi ro chuẩn (1x):** DCA Tĩnh đạt `P50 = 55.512,2` (`11,33x`), `Win Rate = 99,99%`, `P90 = 552.465,9` và tỷ lệ gãy chỉ `13,40%`.
- **Rủi ro gấp đôi (2x):** Vượt trội toàn diện so với bản Động cùng tỷ trọng (`P50 = 15.268,9` vs `11.932,8`; `P10 = 4.286,0` vs `1.692,4`; `Win Rate = 86,58%` vs `70,00%`; `CVaR = 2.085,2` vs `4.084,1`).

