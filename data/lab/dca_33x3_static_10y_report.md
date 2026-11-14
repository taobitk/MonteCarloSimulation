# Báo cáo Kiểm thử Danh mục 33,3%×3 (DCA Tĩnh) — Khung 10 Năm

> [!NOTE]
> Mô phỏng bằng **Native Rust CLI (`-C target-cpu=native`, 16 luồng OS)** với tham số chuẩn hóa: `time_years = 10.0`, `steps = 2.520 ngày giao dịch`, `num_sims = 35.000`, `seed = 42`, phân phối **Đuôi béo Student-t ($\nu = 4$)**, ngưỡng gãy `-20%` tổng vốn góp lũy kế (`ruin_drawdown_pct = 0.20`).
> - **Chế độ**: **DCA Tĩnh** — Chỉ mua dồn theo tỷ trọng mục tiêu, không bán tái cân bằng (`0` lần bán, `0` phí).

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

## 2. Kết quả Chi tiết 4 Kịch bản DCA Tĩnh 10 Năm (Tổng vốn góp = `1.300`)

| Chỉ số | 1. Thông thường (Thực tế) | 2. Thông thường (Tìm điểm gãy) | 3. Rủi ro 2x (Thực tế) | 4. Rủi ro 2x (Tìm điểm gãy) |
| :--- | :---: | :---: | :---: | :---: |
| **Thời gian chạy (16 luồng)** | `4.302 ms` | `4.820 ms` | `4.177 ms` | `4.102 ms` |
| **P10 (Xấu 10%)** | `1.332,9` | `835,5` *(Stress)* | `631,2` | `324,6` *(Stress)* |
| **Trung vị P50 (`median_terminal_nav`)** | **`2.320,8`** | `1.302,6` *(Stress)* | **`1.445,1`** | `640,6` *(Stress)* |
| **P90 (Tốt 10%)** | **`6.123,2`** | `2.412,6` | `4.054,6` | `1.412,9` |
| **Bội số vốn P50 (`moic_p50`)** | **`1,79x`** | `1,00x` | **`1,11x`** | `0,49x` |
| **Tỷ lệ có lãi (`win_rate`)** | **`91,08%`** | `50,11%` *(Stress)* | **`56,13%`** | `12,22%` *(Stress)* |
| **Sharpe Ratio** | `0,41` | `0,21` | `0,12` | `-0,44` |
| **CVaR 95% lỗ ròng** | `304,5` | `656,4` | `884,2` | `1.066,4` |
| **Tỷ lệ gãy (`breach_rate`)** | **`39,55%`** | **`39,47%`** *(IS quy đổi)* | **`94,85%`** | **`94,14%`** *(IS quy đổi)* |
| **Số lần bán / Tổng phí TB** | `0` / `0` | `0` / `0` | `0` / `0` | `0` / `0` |

---

## 3. Đóng góp Sụt giảm tại Thời điểm Gãy (`asset_drawdowns_at_ruin`) & Nhận xét

| Chế độ (Thực tế) | Crypto (`33,3%`) | Vàng (`33,3%`) | Cổ phiếu (`33,3%`) |
| :--- | :---: | :---: | :---: |
| **1x – DCA Tĩnh** | `18,1%` | `3,3%` | `3,7%` |
| **2x – DCA Tĩnh** | `20,8%` | `5,3%` | `5,8%` |

- **Rủi ro chuẩn (1x):** DCA Tĩnh giữ P90 rất cao (`6.123,2`) nhờ để Crypto tự chạy nếu tăng mạnh, đạt P50 `2.320,8` (`1,79x`), Win Rate `91,08%` và tỷ lệ gãy `39,55%`.
- **Rủi ro gấp đôi (2x):** Vì không bán Vàng và Cổ phiếu để cứu Crypto (`sigma = 120%`), DCA Tĩnh giữ được P50 `1.445,1` (`1,11x`), P10 `631,2` và Win Rate `56,13%`.

