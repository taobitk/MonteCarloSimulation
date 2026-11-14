# Báo cáo Kiểm thử Danh mục 33,3%×3 (DCA Động) — Khung 20 Năm

> [!NOTE]
> Mô phỏng bằng **Native Rust CLI (`-C target-cpu=native`, 16 luồng OS)** với tham số chuẩn hóa: `time_years = 20.0`, `steps = 5.040 ngày giao dịch`, `num_sims = 35.000`, `seed = 42`, phân phối **Đuôi béo Student-t ($\nu = 4$)**, ngưỡng gãy `-20%` tổng vốn góp lũy kế (`ruin_drawdown_pct = 0.20`).
> - **Chế độ**: **DCA Động** — Tái cân bằng khi lệch biên `10%`, phí bán `sell_fee_pct = 2,0%`.

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

## 2. Kết quả Chi tiết 4 Kịch bản DCA Động 20 Năm (Tổng vốn góp = `2.500`)

| Chỉ số | 1. Thông thường (Thực tế) | 2. Thông thường (Tìm điểm gãy) | 3. Rủi ro 2x (Thực tế) | 4. Rủi ro 2x (Tìm điểm gãy) |
| :--- | :---: | :---: | :---: | :---: |
| **Thời gian chạy (16 luồng)** | `8.136 ms` | `8.275 ms` | `8.220 ms` | `8.212 ms` |
| **P10 (Xấu 10%)** | `3.579,0` | `1.474,4` *(Stress)* | `553,0` | `273,0` *(Stress)* |
| **Trung vị P50 (`median_terminal_nav`)** | **`10.557,0`** | `3.748,0` *(Stress)* | **`2.568,4`** | `763,9` *(Stress)* |
| **P90 (Tốt 10%)** | `32.885,3` | `10.396,9` | `21.183,5` | `3.385,6` |
| **Bội số vốn P50 (`moic_p50`)** | **`4,22x`** | `1,50x` | **`1,03x`** | `0,31x` |
| **Tỷ lệ có lãi (`win_rate`)** | **`95,83%`** | `70,44%` *(Stress)* | `50,69%` | `14,59%` *(Stress)* |
| **Sharpe Ratio** | **`0,76`** | `0,54` | `0,18` | `-0,18` |
| **CVaR 95% lỗ ròng** | `469,6` | `1.578,2` | `2.196,6` | `2.322,6` |
| **Tỷ lệ gãy (`breach_rate`)** | **`49,60%`** | **`49,18%`** *(IS quy đổi)* | **`97,71%`** | **`96,78%`** *(IS quy đổi)* |
| **Số lần bán / Tổng phí TB** | `13,09 lần` / `198,1` | `10,72 lần` / `75,8` | `34,83 lần` / `486,1` | `28,05 lần` / `109,7` |

---

## 3. Đóng góp Sụt giảm tại Thời điểm Gãy (`asset_drawdowns_at_ruin`) & Nhận xét

| Chế độ (Thực tế) | Crypto (`33,3%`) | Vàng (`33,3%`) | Cổ phiếu (`33,3%`) |
| :--- | :---: | :---: | :---: |
| **1x – DCA Động** | `14,1%` | `5,8%` | `6,1%` |

- **Rủi ro chuẩn (1x):** DCA Động bứt phá mạnh mẽ ở P50 (`10.557,0`, `4,22x`), Sharpe `0,76` và Win Rate `95,83%`, dù gần nửa số kịch bản (`49,60%`) từng thủng ngưỡng `-20%` giữa đường trước khi hồi phục.
- **Rủi ro gấp đôi (2x):** DCA Động mất `486,1` phí qua `34,83` lần bán tái cân bằng để dồn vào Crypto (`sigma = 120%`), khiến P50 chỉ hòa vốn nhẹ (`2.568,4`, `1,03x`) và Win Rate giảm còn `50,69%`.

