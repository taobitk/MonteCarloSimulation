# Báo cáo Kiểm thử Danh mục 33,3%×3 (DCA Tĩnh) — Khung 40 Năm

> [!NOTE]
> Mô phỏng bằng **Native Rust CLI (`-C target-cpu=native`, 16 luồng OS)** với tham số chuẩn hóa: `time_years = 40.0`, `steps = 10.080 ngày giao dịch`, `num_sims = 35.000`, `seed = 42`, phân phối **Đuôi béo Student-t ($\nu = 4$)**, ngưỡng gãy `-20%` tổng vốn góp lũy kế (`ruin_drawdown_pct = 0.20`).
> - **Chế độ**: **DCA Tĩnh** — Chỉ mua dồn theo tỷ trọng mục tiêu, không bán tái cân bằng (`0` lần bán, `0` phí).

---

## 1. Cấu hình Danh mục & Lịch Nạp Vốn 40 Năm (3 Tài sản, Không Tiền gửi)

| Tài sản | Tỷ trọng (`weight`) | Lợi suất kỳ vọng (`mu`/năm) | Biến động chuẩn (`sigma`/năm) | Biến động Gấp đôi (`2x sigma`) |
| :--- | :---: | :---: | :---: | :---: |
| **Crypto (BTC / ETH)** | **33,33%** | `25,0%` | `60,0%` | `120,0%` |
| **Vàng** | **33,33%** | `9,5%` | `17,0%` | `34,0%` |
| **Cổ phiếu** | **33,33%** | `10,5%` | `18,0%` | `36,0%` |

- **Ma trận tương quan `3×3`**: Crypto–Vàng (`+0,20`), Crypto–Cổ phiếu (`+0,30`), Vàng–Cổ phiếu (`+0,05`).
- **Tổng vốn thực nạp 40 năm**: Vốn đầu `100` + `10` mỗi `21 ngày` (`480` kỳ) $\rightarrow$ **`total_contributed = 4.900`**.

---

## 2. Kết quả Chi tiết 4 Kịch bản DCA Tĩnh 40 Năm (Tổng vốn góp = `4.900`)

| Chỉ số | 1. Thông thường (Thực tế) | 2. Thông thường (Tìm điểm gãy) | 3. Rủi ro 2x (Thực tế) | 4. Rủi ro 2x (Tìm điểm gãy) |
| :--- | :---: | :---: | :---: | :---: |
| **Thời gian chạy (16 luồng)** | `16.311 ms` | `16.313 ms` | `15.998 ms` | `17.335 ms` |
| **P10 (Xấu 10%)** | `15.584,0` | `5.226,5` *(Stress)* | `2.322,2` | `731,0` *(Stress)* |
| **Trung vị P50 (`median_terminal_nav`)** | **`58.517,9`** | `14.682,2` *(Stress)* | **`12.715,6`** | `2.315,7` *(Stress)* |
| **P90 (Tốt 10%)** | **`855.952,5`** | `66.659,8` | `92.879,9` | `10.844,8` |
| **Bội số vốn P50 (`moic_p50`)** | **`11,94x`** | `3,00x` | **`2,60x`** | `0,47x` |
| **Tỷ lệ có lãi (`win_rate`)** | **`99,67%`** | `91,41%` *(Stress)* | **`75,26%`** | `25,78%` *(Stress)* |
| **Sharpe Ratio** | `0,08` | `0,11` | `0,21` | `0,03` |
| **CVaR 95% lỗ ròng** | `0,0` | `1.765,4` | `3.772,9` | `4.450,3` |
| **Tỷ lệ gãy (`breach_rate`)** | **`44,61%`** | **`44,71%`** *(IS quy đổi)* | **`97,88%`** | **`97,90%`** *(IS quy đổi)* |
| **Số lần bán / Tổng phí TB** | `0` / `0` | `0` / `0` | `0` / `0` | `0` / `0` |

---

## 3. Đóng góp Sụt giảm tại Thời điểm Gãy (`asset_drawdowns_at_ruin`) & Nhận xét

| Chế độ (Thực tế) | Crypto (`33,3%`) | Vàng (`33,3%`) | Cổ phiếu (`33,3%`) |
| :--- | :---: | :---: | :---: |
| **1x – DCA Tĩnh** | `19,6%` | `4,2%` | `4,6%` |

- **Rủi ro chuẩn (1x):** DCA Tĩnh đạt P50 `58.517,9` (`11,94x`), Win Rate `99,67%` và P90 vọt lên `855.952,5` nhờ giữ nguyên tài sản thắng lớn qua 40 năm.
- **Rủi ro gấp đôi (2x):** Giữ dòng tiền đều vào Vàng & Cổ phiếu thay vì bán để cứu Crypto (`sigma = 120%`), DCA Tĩnh vẫn đạt P50 **`12.715,6` (`2,60x`)** và Win Rate **`75,26%`**.

