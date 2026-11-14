# Báo cáo Kiểm thử Danh mục `10/20/35/35` (DCA Động) — Khung 20 Năm

> [!NOTE]
> Mô phỏng bằng **Native Rust CLI (`-C target-cpu=native`, 16 luồng OS)** với tham số chuẩn hóa: `time_years = 20.0`, `steps = 5.040 ngày giao dịch`, `num_sims = 35.000`, `seed = 42`, phân phối **Đuôi béo Student-t ($\nu = 4$)**, ngưỡng gãy `-20%` tổng vốn góp lũy kế (`ruin_drawdown_pct = 0.20`).
> - **Chế độ**: **DCA Động** — Tái cân bằng khi lệch biên `10%`, phí bán `sell_fee_pct = 2,0%`.

---

## 1. Cấu hình Danh mục & Lịch Nạp Vốn 20 Năm (4 Tài sản)

| Tài sản | Tỷ trọng (`weight`) | Lợi suất kỳ vọng (`mu`/năm) | Biến động chuẩn (`sigma`/năm) | Biến động Gấp đôi (`2x sigma`) |
| :--- | :---: | :---: | :---: | :---: |
| **Tiền gửi tiết kiệm** | **10%** | `5,5%` | `1,0%` | `2,0%` |
| **Crypto (BTC / ETH)** | **20%** | `25,0%` | `60,0%` | `120,0%` |
| **Vàng** | **35%** | `9,5%` | `17,0%` | `34,0%` |
| **Cổ phiếu** | **35%** | `10,5%` | `18,0%` | `36,0%` |

- **Ma trận tương quan `4×4`**: Tiền gửi–Vàng (`+0,10`), Tiền gửi–Cổ phiếu (`-0,15`), Crypto–Vàng (`+0,20`), Crypto–Cổ phiếu (`+0,30`), Vàng–Cổ phiếu (`+0,05`).
- **Tổng vốn thực nạp 20 năm**: Vốn đầu `100` + `10` mỗi `21 ngày` (`240` kỳ) $\rightarrow$ **`total_contributed = 2.500`**.

---

## 2. Kết quả Chi tiết 4 Kịch bản DCA Động 20 Năm (Tổng vốn góp = `2.500`)

| Chỉ số | 1. Thông thường (Thực tế) | 2. Thông thường (Tìm điểm gãy) | 3. Rủi ro 2x (Thực tế) | 4. Rủi ro 2x (Tìm điểm gãy) |
| :--- | :---: | :---: | :---: | :---: |
| **Thời gian chạy (16 luồng)** | `10.138 ms` | `10.053 ms` | `9.508 ms` | `9.743 ms` |
| **P10 (Xấu 10%)** | `4.504,1` | `2.270,2` *(Stress)* | `1.073,8` | `466,9` *(Stress)* |
| **Trung vị P50 (`median_terminal_nav`)** | **`9.390,9`** | `4.544,7` *(Stress)* | **`4.131,9`** | `1.215,5` *(Stress)* |
| **P90 (Tốt 10%)** | `20.349,9` | `9.383,5` | `20.967,2` | `4.426,4` |
| **Bội số vốn P50 (`moic_p50`)** | **`3,76x`** | `1,82x` | **`1,65x`** | `0,49x` |
| **Tỷ lệ có lãi (`win_rate`)** | **`98,94%`** | `86,46%` *(Stress)* | `66,93%` | `22,81%` *(Stress)* |
| **Sharpe Ratio** | **`1,15`** | `0,86` | `0,35` | `-0,11` |
| **CVaR 95% lỗ ròng** | **`0,0`** | `961,9` | `1.904,7` | `2.193,8` |
| **Số mẫu gãy (`breach_count`)** | `6.435 / 35.000` | `17.779 / 35.000` | `32.067 / 35.000` | `34.754 / 35.000` |
| **Tỷ lệ gãy (`breach_rate`)** | **`18,39%`** | **`18,28%`** *(IS quy đổi)* | **`91,62%`** | **`90,94%`** *(IS quy đổi)* |
| **Số lần bán / Tổng phí TB** | `8,03 lần` / `96,5` | `6,41 lần` / `46,4` | `22,42 lần` / `263,8` | `17,34 lần` / `76,6` |

---

## 3. Đóng góp Sụt giảm tại Thời điểm Gãy (`asset_drawdowns_at_ruin`) & Nhận xét

| Chế độ (Thực tế) | Tiền gửi (`10%`) | Crypto (`20%`) | Vàng (`35%`) | Cổ phiếu (`35%`) |
| :--- | :---: | :---: | :---: | :---: |
| **1x – DCA Động** | `0,25%` | `8,8%` | `5,5%` | `5,9%` |
| **2x – DCA Động** | `0,83%` | `11,9%` | `9,2%` | `9,7%` |

- **Rủi ro chuẩn (1x):** DCA Động đạt **`9.390,9` (`3,76x`)**, `Win Rate = 98,94%`, `CVaR = 0,0`, `Sharpe = 1,15` — cân bằng rất tốt giữa `10/10/40/40` (`8.470,32`) và `33,3%×3` (`10.557,0`), trong khi giữ tỷ lệ gãy chỉ `18,39%`.
- **Rủi ro gấp đôi (2x):** Mất `263,8` phí qua `22,42` lần bán tái cân bằng khiến `P50` (`4.131,9`) ngang ngửa bản Tĩnh (`4.156,9`), nhưng `P10` (`1.073,8` vs `1.661,5`) và `CVaR` (`1.904,7` vs `1.364,9`) yếu hơn rõ rệt.

