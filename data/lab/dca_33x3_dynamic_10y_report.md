# Báo cáo Kiểm thử Danh mục 33,3%×3 (DCA Động) — Khung 10 Năm

> [!NOTE]
> Mô phỏng bằng **Native Rust CLI (`-C target-cpu=native`, 16 luồng OS)** với tham số chuẩn hóa: `time_years = 10.0`, `steps = 2.520 ngày giao dịch`, `num_sims = 35.000`, `seed = 42`, phân phối **Đuôi béo Student-t ($\nu = 4$)**, ngưỡng gãy `-20%` tổng vốn góp lũy kế (`ruin_drawdown_pct = 0.20`).
> - **Chế độ**: **DCA Động** — Tái cân bằng khi lệch biên `10%`, phí bán `sell_fee_pct = 2,0%`.

---

## 1. Cấu hình Danh mục & Lịch Nạp Vốn 10 Năm (3 Tài sản, Không Tiền gửi)

| Tài sản | Tỷ trọng (`weight`) | Lợi suất kỳ vọng (`mu`/năm) | Biến động chuẩn (`sigma`/năm) | Biến động Gấp đôi (`2x sigma`) |
| :--- | :---: | :---: | :---: | :---: |
| **Crypto (BTC / ETH)** | **33,33%** | `25,0%` | `60,0%` | `120,0%` |
| **Vàng** | **33,33%** | `9,5%` | `17,0%` | `34,0%` |
| **Cổ phiếu** | **33,33%** | `10,5%` | `18,0%` | `36,0%` |

- **Ma trận tương quan `3×3`**: Crypto–Vàng (`+0,20`), Crypto–Cổ phiếu (`+0,30`), Vàng–Cổ phiếu (`+0,05`).
- **Tổng vốn thực nạp 10 năm**: Vốn đầu `100` + `10` mỗi `21 ngày` (`120` kỳ) $\rightarrow$ **`total_contributed = 1.300`**.

---

## 2. Kết quả Chi tiết 4 Kịch bản DCA Động 10 Năm (Tổng vốn góp = `1.300`)

| Chỉ số | 1. Thông thường (Thực tế) | 2. Thông thường (Tìm điểm gãy) | 3. Rủi ro 2x (Thực tế) | 4. Rủi ro 2x (Tìm điểm gãy) |
| :--- | :---: | :---: | :---: | :---: |
| **Thời gian chạy (16 luồng)** | `4.172 ms` | `4.635 ms` | `4.717 ms` | `5.150 ms` |
| **P10 (Xấu 10%)** | `1.325,4` | `736,6` *(Stress)* | `416,6` | `215,1` *(Stress)* |
| **Trung vị P50 (`median_terminal_nav`)** | **`2.618,9`** | `1.353,1` *(Stress)* | **`1.357,3`** | `510,6` *(Stress)* |
| **P90 (Tốt 10%)** | `5.425,5` | `2.589,4` | `5.942,6` | `1.571,8` |
| **Bội số vốn P50 (`moic_p50`)** | **`2,01x`** | `1,04x` | **`1,04x`** | `0,39x` |
| **Tỷ lệ có lãi (`win_rate`)** | **`90,64%`** | `53,26%` *(Stress)* | `51,61%` | `13,95%` *(Stress)* |
| **Sharpe Ratio** | **`0,92`** | `0,30` | `0,27` | `-0,46` |
| **CVaR 95% lỗ ròng** | `389,4` | `771,8` | `1.050,9` | `1.152,7` |
| **Tỷ lệ gãy (`breach_rate`)** | **`44,43%`** | **`44,15%`** *(IS quy đổi)* | **`94,99%`** | **`93,87%`** *(IS quy đổi)* |
| **Số lần bán / Tổng phí TB** | `5,08 lần` / `22,3` | `3,71 lần` / `10,2` | `15,77 lần` / `69,5` | `11,75 lần` / `22,9` |

---

## 3. Đóng góp Sụt giảm tại Thời điểm Gãy (`asset_drawdowns_at_ruin`) & Nhận xét

| Chế độ (Thực tế) | Crypto (`33,3%`) | Vàng (`33,3%`) | Cổ phiếu (`33,3%`) |
| :--- | :---: | :---: | :---: |
| **1x – DCA Động** | `13,1%` | `4,6%` | `4,9%` |
| **2x – DCA Động** | `17,8%` | `7,3%` | `7,7%` |

- **Rủi ro chuẩn (1x):** DCA Động đạt hiệu quả rất tốt ở trung vị (`2.618,9`, `2,01x`) và Sharpe (`0,92`), với `5,08` lần bán tái cân bằng (`22,3` phí), nhưng tỷ lệ từng chạm ngưỡng gãy `-20%` giữa đường là `44,43%` do không có đệm Tiền gửi.
- **Rủi ro gấp đôi (2x):** Số lần bán tái cân bằng tăng lên `15,77` lần (`69,5` phí) để dồn vốn sang Crypto (`sigma = 120%`), kéo P50 xuống còn `1.357,3` (`1,04x`) và Win Rate giảm còn `51,61%`.

