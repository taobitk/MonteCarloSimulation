# Báo cáo Kiểm thử Danh mục 33,3%×3 (DCA Động) — Khung 40 Năm

> [!NOTE]
> Mô phỏng bằng **Native Rust CLI (`-C target-cpu=native`, 16 luồng OS)** với tham số chuẩn hóa: `time_years = 40.0`, `steps = 10.080 ngày giao dịch`, `num_sims = 35.000`, `seed = 42`, phân phối **Đuôi béo Student-t ($\nu = 4$)**, ngưỡng gãy `-20%` tổng vốn góp lũy kế (`ruin_drawdown_pct = 0.20`).
> - **Chế độ**: **DCA Động** — Tái cân bằng khi lệch biên `10%`, phí bán `sell_fee_pct = 2,0%`.

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

## 2. Kết quả Chi tiết 4 Kịch bản DCA Động 40 Năm (Tổng vốn góp = `4.900`)

| Chỉ số | 1. Thông thường (Thực tế) | 2. Thông thường (Tìm điểm gãy) | 3. Rủi ro 2x (Thực tế) | 4. Rủi ro 2x (Tìm điểm gãy) |
| :--- | :---: | :---: | :---: | :---: |
| **Thời gian chạy (16 luồng)** | `16.521 ms` | `14.901 ms` | `16.003 ms` | `16.516 ms` |
| **P10 (Xấu 10%)** | `17.683,7` | `4.067,9` *(Stress)* | `688,0` | `332,8` *(Stress)* |
| **Trung vị P50 (`median_terminal_nav`)** | **`105.928,0`** | `19.018,3` *(Stress)* | **`4.699,5`** | `1.111,8` *(Stress)* |
| **P90 (Tốt 10%)** | `656.437,1` | `102.472,2` | `93.967,8` | `7.668,0` |
| **Bội số vốn P50 (`moic_p50`)** | **`21,62x`** | `3,88x` | **`0,96x`** | `0,23x` |
| **Tỷ lệ có lãi (`win_rate`)** | **`98,94%`** | `86,55%` *(Stress)* | `49,13%` | `15,10%` *(Stress)* |
| **Sharpe Ratio** | **`0,44`** | `0,44` | `0,10` | `0,00` |
| **CVaR 95% lỗ ròng** | `0,0` | `2.896,4` | `4.554,3` | `4.693,7` |
| **Tỷ lệ gãy (`breach_rate`)** | **`51,14%`** | **`51,22%`** *(IS quy đổi)* | **`99,02%`** | **`98,61%`** *(IS quy đổi)* |
| **Số lần bán / Tổng phí TB** | `30,11 lần` / `4.565,5` | `26,59 lần` / `989,5` | **`73,90 lần`** / **`7.301,1`** | `62,92 lần` / `657,3` |

---

## 3. Đóng góp Sụt giảm tại Thời điểm Gãy (`asset_drawdowns_at_ruin`) & Nhận xét

| Chế độ (Thực tế) | Crypto (`33,3%`) | Vàng (`33,3%`) | Cổ phiếu (`33,3%`) |
| :--- | :---: | :---: | :---: |
| **1x – DCA Động** | `14,7%` | `6,4%` | `6,7%` |

- **Rủi ro chuẩn (1x):** DCA Động tạo ra *Rebalancing Bonus* khổng lồ sau 40 năm: P50 đạt **`105.928,0` (`21,62x`)**, Win Rate `98,94%` và Sharpe `0,44`, dù trả `4.565,5` phí qua `30,11` lần bán.
- **Rủi ro gấp đôi (2x):** DCA Động biến thành "bắt dao rơi" cực nặng: bán Vàng/Cổ phiếu `73,90` lần và đốt `7.301,1` phí để bơm vào Crypto (`sigma = 120%`, log-drift âm `-47%/năm`), khiến P50 tụt xuống **`4.699,5` (`0,96x` — lỗ nhẹ sau 40 năm)**.

