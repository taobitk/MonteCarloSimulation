# Báo cáo Kiểm thử Danh mục `10/20/35/35` (DCA Tĩnh) — Khung 20 Năm

> [!NOTE]
> Mô phỏng bằng **Native Rust CLI (`-C target-cpu=native`, 16 luồng OS)** với tham số chuẩn hóa: `time_years = 20.0`, `steps = 5.040 ngày giao dịch`, `num_sims = 35.000`, `seed = 42`, phân phối **Đuôi béo Student-t ($\nu = 4$)**, ngưỡng gãy `-20%` tổng vốn góp lũy kế (`ruin_drawdown_pct = 0.20`).
> - **Chế độ**: **DCA Tĩnh** — Chỉ mua dồn theo tỷ trọng mục tiêu, không bán tái cân bằng (`0` lần bán, `0` phí).

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

## 2. Kết quả Chi tiết 4 Kịch bản DCA Tĩnh 20 Năm (Tổng vốn góp = `2.500`)

| Chỉ số | 1. Thông thường (Thực tế) | 2. Thông thường (Tìm điểm gãy) | 3. Rủi ro 2x (Thực tế) | 4. Rủi ro 2x (Tìm điểm gãy) |
| :--- | :---: | :---: | :---: | :---: |
| **Thời gian chạy (16 luồng)** | `9.913 ms` | `10.047 ms` | `9.861 ms` | `10.103 ms` |
| **P10 (Xấu 10%)** | `4.013,5` | `2.401,8` *(Stress)* | `1.661,5` | `958,5` *(Stress)* |
| **Trung vị P50 (`median_terminal_nav`)** | **`7.808,0`** | `4.027,3` *(Stress)* | **`4.156,9`** | `1.740,7` *(Stress)* |
| **P90 (Tốt 10%)** | **`28.043,4`** | `8.186,2` | `13.692,5` | `4.159,1` |
| **Bội số vốn P50 (`moic_p50`)** | **`3,12x`** | `1,61x` | **`1,66x`** | `0,70x` |
| **Tỷ lệ có lãi (`win_rate`)** | **`99,11%`** | `87,94%` *(Stress)* | **`74,51%`** | `27,85%` *(Stress)* |
| **Sharpe Ratio** | `0,22` | `0,36` | `0,16` | `-0,06` |
| **CVaR 95% lỗ ròng** | **`0,0`** | `666,5` | `1.364,9` | `1.739,5` |
| **Số mẫu gãy (`breach_count`)** | `4.636 / 35.000` | `15.252 / 35.000` | `31.563 / 35.000` | `34.723 / 35.000` |
| **Tỷ lệ gãy (`breach_rate`)** | **`13,25%`** | **`13,13%`** *(IS quy đổi)* | **`90,18%`** | **`89,63%`** *(IS quy đổi)* |
| **Số lần bán / Tổng phí TB** | `0` / `0` | `0` / `0` | `0` / `0` | `0` / `0` |

---

## 3. Đóng góp Sụt giảm tại Thời điểm Gãy (`asset_drawdowns_at_ruin`) & Nhận xét

| Chế độ (Thực tế) | Tiền gửi (`10%`) | Crypto (`20%`) | Vàng (`35%`) | Cổ phiếu (`35%`) |
| :--- | :---: | :---: | :---: | :---: |
| **1x – DCA Tĩnh** | `0,01%` | `12,9%` | `5,6%` | `6,2%` |
| **2x – DCA Tĩnh** | `0,03%` | `14,3%` | `8,1%` | `8,8%` |

- **Rủi ro chuẩn (1x):** DCA Tĩnh giữ tỷ lệ gãy thấp hơn bản Động (`13,25%` vs `18,39%`), `Win Rate = 99,11%`, `CVaR = 0,0` và đỉnh `P90 = 28.043,4`.
- **Rủi ro gấp đôi (2x):** Bắt đầu vượt lên bản Động ở hầu hết chỉ số sống sót: `P50 = 4.156,9` (`1,66x`), `P10 = 1.661,5`, `Win Rate = 74,51%`, `CVaR = 1.364,9`.

