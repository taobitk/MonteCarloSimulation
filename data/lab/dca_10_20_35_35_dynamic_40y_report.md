# Báo cáo Kiểm thử Danh mục `10/20/35/35` (DCA Động) — Khung 40 Năm

> [!NOTE]
> Mô phỏng bằng **Native Rust CLI (`-C target-cpu=native`, 16 luồng OS)** với tham số chuẩn hóa: `time_years = 40.0`, `steps = 10.080 ngày giao dịch`, `num_sims = 35.000`, `seed = 42`, phân phối **Đuôi béo Student-t ($\nu = 4$)**, ngưỡng gãy `-20%` tổng vốn góp lũy kế (`ruin_drawdown_pct = 0.20`).
> - **Chế độ**: **DCA Động** — Tái cân bằng khi lệch biên `10%`, phí bán `sell_fee_pct = 2,0%`.

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

## 2. Kết quả Chi tiết 4 Kịch bản DCA Động 40 Năm (Tổng vốn góp = `4.900`)

| Chỉ số | 1. Thông thường (Thực tế) | 2. Thông thường (Tìm điểm gãy) | 3. Rủi ro 2x (Thực tế) | 4. Rủi ro 2x (Tìm điểm gãy) |
| :--- | :---: | :---: | :---: | :---: |
| **Thời gian chạy (16 luồng)** | `20.119 ms` | `21.416 ms` | `19.070 ms` | `18.749 ms` |
| **P10 (Xấu 10%)** | **`25.165,6`** | `7.628,7` *(Stress)* | `1.692,4` | `619,7` *(Stress)* |
| **Trung vị P50 (`median_terminal_nav`)** | **`85.827,7`** | `24.857,4` *(Stress)* | **`11.932,8`** | `2.113,0` *(Stress)* |
| **P90 (Tốt 10%)** | `297.380,4` | `84.169,9` | `148.311,3` | `13.379,1` |
| **Bội số vốn P50 (`moic_p50`)** | **`17,52x`** | `5,07x` | **`2,44x`** | `0,43x` |
| **Tỷ lệ có lãi (`win_rate`)** | **`99,89%`** | `96,23%` *(Stress)* | `70,00%` | `26,07%` *(Stress)* |
| **Sharpe Ratio** | **`0,76`** | `0,71` | `0,17` | `0,08` |
| **CVaR 95% lỗ ròng** | **`0,0`** | `847,7` | `4.084,1` | `4.515,2` |
| **Số mẫu gãy (`breach_count`)** | `6.578 / 35.000` | `15.454 / 35.000` | `33.172 / 35.000` | `34.860 / 35.000` |
| **Tỷ lệ gãy (`breach_rate`)** | **`18,79%`** | **`19,03%`** *(IS quy đổi)* | **`94,78%`** | **`94,72%`** *(IS quy đổi)* |
| **Số lần bán / Tổng phí TB** | `19,46 lần` / `1.656,7` | `16,90 lần` / `568,2` | **`48,54 lần`** / **`3.588,8`** | `39,67 lần` / `498,8` |

---

## 3. Đóng góp Sụt giảm tại Thời điểm Gãy (`asset_drawdowns_at_ruin`) & Nhận xét

| Chế độ (Thực tế) | Tiền gửi (`10%`) | Crypto (`20%`) | Vàng (`35%`) | Cổ phiếu (`35%`) |
| :--- | :---: | :---: | :---: | :---: |
| **1x – DCA Động** | `0,38%` | `9,0%` | `6,1%` | `6,4%` |
| **2x – DCA Động** | `1,06%` | `12,1%` | `10,0%` | `10,4%` |

- **Rủi ro chuẩn (1x):** Kết quả rất ấn tượng: `P50` đạt **`85.827,7` (`17,52x`)**, `P10` đạt **`25.165,6`**, `Win Rate = 99,89%`, `Sharpe = 0,76`, `CVaR = 0,0`, và `breach_rate` chỉ `18,79%`.
- **Rủi ro gấp đôi (2x):** Bán tái cân bằng `48,54` lần tốn `3.588,8` phí để cứu `20%` Crypto khiến `P50` tụt xuống `11.932,8` (`2,44x`), thấp hơn cả `10/10/40/40 Động` (`20.404,60`) lẫn `10/20/35/35 Tĩnh` (`15.268,9`).

