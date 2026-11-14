# Báo cáo Kiểm thử Danh mục 33,3%×3 (DCA Tĩnh) — Khung 20 Năm

> [!NOTE]
> Mô phỏng bằng **Native Rust CLI (`-C target-cpu=native`, 16 luồng OS)** với tham số chuẩn hóa: `time_years = 20.0`, `steps = 5.040 ngày giao dịch`, `num_sims = 35.000`, `seed = 42`, phân phối **Đuôi béo Student-t ($\nu = 4$)**, ngưỡng gãy `-20%` tổng vốn góp lũy kế (`ruin_drawdown_pct = 0.20`).
> - **Chế độ**: **DCA Tĩnh** — Chỉ mua dồn theo tỷ trọng mục tiêu, không bán tái cân bằng (`0` lần bán, `0` phí).

---

## 1. Cấu hình Danh mục & Lịch Nạp Vốn 20 Năm (3 Tài sản, Không Tiền gửi)

| Tài sản | Tỷ trọng (`weight`) | Lợi suất kỳ vọng (`mu`/năm) | Biến động chuẩn (`sigma`/năm) | Biến động Gấp đôi (`2x sigma`) |
| :--- | :---: | :---: | :---: | :---: |
| **Crypto (BTC / ETH)** | **33,33%** | `25,0%` | `60,0%` | `120,0%` |
| **Vàng** | **33,33%** | `9,5%` | `17,0%` | `34,0%` |
| **Cổ phiếu** | **33,33%** | `10,5%` | `18,0%` | `36,0%` |

- **Ma trận tương quan `3×3`**: Crypto–Vàng (`+0,20`), Crypto–Cổ phiếu (`+0,30`), Vàng–Cổ phiếu (`+0,05`).
- **Tổng vốn thực nạp 20 năm**: Vốn đầu `100` + `10` mỗi `21 ngày` (`240` kỳ) $\rightarrow$ **`total_contributed = 2.500`**.

---

## 2. Kết quả Chi tiết 4 Kịch bản DCA Tĩnh 20 Năm (Tổng vốn góp = `2.500`)

| Chỉ số | 1. Thông thường (Thực tế) | 2. Thông thường (Tìm điểm gãy) | 3. Rủi ro 2x (Thực tế) | 4. Rủi ro 2x (Tìm điểm gãy) |
| :--- | :---: | :---: | :---: | :---: |
| **Thời gian chạy (16 luồng)** | `8.918 ms` | `7.686 ms` | `7.461 ms` | `7.860 ms` |
| **P10 (Xấu 10%)** | `3.480,3` | `1.745,3` *(Stress)* | `1.143,1` | `481,6` *(Stress)* |
| **Trung vị P50 (`median_terminal_nav`)** | **`8.100,0`** | `3.388,8` *(Stress)* | **`3.545,7`** | `1.144,9` *(Stress)* |
| **P90 (Tốt 10%)** | **`41.002,9`** | `8.791,0` | `13.654,1` | `3.353,5` |
| **Bội số vốn P50 (`moic_p50`)** | **`3,24x`** | `1,36x` | **`1,42x`** | `0,46x` |
| **Tỷ lệ có lãi (`win_rate`)** | **`96,94%`** | `71,57%` *(Stress)* | **`64,63%`** | `16,87%` *(Stress)* |
| **Sharpe Ratio** | `0,19` | `0,23` | `0,11` | `-0,24` |
| **CVaR 95% lỗ ròng** | `175,8` | `1.263,0` | `1.833,9` | `2.176,6` |
| **Tỷ lệ gãy (`breach_rate`)** | **`44,06%`** | **`43,71%`** *(IS quy đổi)* | **`97,06%`** | **`96,57%`** *(IS quy đổi)* |
| **Số lần bán / Tổng phí TB** | `0` / `0` | `0` / `0` | `0` / `0` | `0` / `0` |

---

## 3. Đóng góp Sụt giảm tại Thời điểm Gãy (`asset_drawdowns_at_ruin`) & Nhận xét

| Chế độ (Thực tế) | Crypto (`33,3%`) | Vàng (`33,3%`) | Cổ phiếu (`33,3%`) |
| :--- | :---: | :---: | :---: |
| **1x – DCA Tĩnh** | `19,2%` | `3,9%` | `4,3%` |

- **Rủi ro chuẩn (1x):** DCA Tĩnh đạt P50 `8.100,0` (`3,24x`), Win Rate `96,94%` và P90 đạt tới `41.002,9`, với CVaR 95% lỗ ròng rất thấp (`175,8`).
- **Rủi ro gấp đôi (2x):** Không bị rút Vàng/Cổ phiếu sang cứu Crypto (`sigma = 120%`), DCA Tĩnh vẫn giữ được P50 **`3.545,7` (`1,42x`)**, P10 `1.143,1` và Win Rate **`64,63%`**.

