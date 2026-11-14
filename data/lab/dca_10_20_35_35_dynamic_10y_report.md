# Báo cáo Kiểm thử Danh mục `10/20/35/35` (DCA Động) — Khung 10 Năm

> [!NOTE]
> Mô phỏng bằng **Native Rust CLI (`-C target-cpu=native`, 16 luồng OS)** với tham số chuẩn hóa: `time_years = 10.0`, `steps = 2.520 ngày giao dịch`, `num_sims = 35.000`, `seed = 42`, phân phối **Đuôi béo Student-t ($\nu = 4$)**, ngưỡng gãy `-20%` tổng vốn góp lũy kế (`ruin_drawdown_pct = 0.20`).
> - **Chế độ**: **DCA Động** — Tái cân bằng khi lệch biên `10%`, phí bán `sell_fee_pct = 2,0%`.

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

## 2. Kết quả Chi tiết 4 Kịch bản DCA Động 10 Năm (Tổng vốn góp = `1.300`)

| Chỉ số | 1. Thông thường (Thực tế) | 2. Thông thường (Tìm điểm gãy) | 3. Rủi ro 2x (Thực tế) | 4. Rủi ro 2x (Tìm điểm gãy) |
| :--- | :---: | :---: | :---: | :---: |
| **Thời gian chạy (16 luồng)** | `5.117 ms` | `5.799 ms` | `5.700 ms` | `5.011 ms` |
| **P10 (Xấu 10%)** | `1.558,3` | `1.023,5` *(Stress)* | `687,7` | `343,0` *(Stress)* |
| **Trung vị P50 (`median_terminal_nav`)** | **`2.462,0`** | `1.558,3` *(Stress)* | **`1.753,4`** | `736,0` *(Stress)* |
| **P90 (Tốt 10%)** | `4.047,7` | `2.473,6` | `4.976,0` | `1.803,1` |
| **Bội số vốn P50 (`moic_p50`)** | **`1,89x`** | `1,20x` | **`1,35x`** | `0,57x` |
| **Tỷ lệ có lãi (`win_rate`)** | **`96,39%`** | `70,77%` *(Stress)* | `65,05%` | `20,15%` *(Stress)* |
| **Sharpe Ratio** | **`1,27`** | `0,61` | `0,47` | `-0,42` |
| **CVaR 95% lỗ ròng** | `107,2` | `516,8` | `868,3` | `1.058,9` |
| **Số mẫu gãy (`breach_count`)** | `5.573 / 35.000` | `18.466 / 35.000` | `30.013 / 35.000` | `34.547 / 35.000` |
| **Tỷ lệ gãy (`breach_rate`)** | **`15,92%`** | **`15,80%`** *(IS quy đổi)* | **`85,75%`** | **`85,02%`** *(IS quy đổi)* |
| **Số lần bán / Tổng phí TB** | `3,03 lần` / `11,7` | `2,15 lần` / `6,1` | `9,82 lần` / `39,0` | `7,13 lần` / `15,3` |

---

## 3. Đóng góp Sụt giảm tại Thời điểm Gãy (`asset_drawdowns_at_ruin`) & Nhận xét

| Chế độ (Thực tế) | Tiền gửi (`10%`) | Crypto (`20%`) | Vàng (`35%`) | Cổ phiếu (`35%`) |
| :--- | :---: | :---: | :---: | :---: |
| **1x – DCA Động** | `0,06%` | `8,0%` | `4,3%` | `4,7%` |
| **2x – DCA Động** | `0,51%` | `11,5%` | `8,3%` | `8,7%` |

- **Rủi ro chuẩn (1x):** Nâng Crypto từ `10%` lên `20%` giúp P50 tăng lên **`2.462,0` (`1,89x`)** và `Sharpe = 1,27`, trong khi tỷ lệ gãy giữ ở mức vừa phải (`15,92%` — thấp hơn gần 3 lần so với nhóm `33,3%×3`).
- **Rủi ro gấp đôi (2x):** Trong khung 10 năm, DCA Động vẫn giữ P50 **`1.753,4` (`1,35x`)** (cao hơn bản Tĩnh `1.632,3`), nhưng chi phí tái cân bằng bắt đầu tăng lên `9,82` lần bán (`39,0` phí) và `P10` tụt xuống `687,7`.

