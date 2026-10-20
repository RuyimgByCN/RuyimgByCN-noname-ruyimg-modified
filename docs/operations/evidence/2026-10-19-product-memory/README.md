# 多盘连续 + **堆内存**观测（产品级长程的内存面，2026-10-19）

- 工作包：**NC08 / 产品级长程**；规则版本 v322（工作树）/ 存档格式 v105
- 用例：`apps/game-server/tests/long_run_memory.rs`（**已跟踪**）
- 原始输出：[run.txt](run.txt)

## 0 方法（**为什么不用 RSS**）

| 方案 | 结果 |
|---|---|
| `ps -o rss= -p <pid>` | ❌ **`/bin/ps: Operation not permitted`**（本环境沙箱禁止）|
| `libc::getrusage` | ❌ 工区**未依赖 `libc`**（不引入新依赖）|
| **计数式全局分配器** | ✅ **本用例采用**：`#[global_allocator]` + `std::alloc::System`，用原子量统计**当前/峰值堆字节** |

**口径**：测的是**本进程 Rust 堆**（经全局分配器的申请），**不是 RSS**（不含栈、mmap 直映射、代码段）。
结论只对「**是否存在随盘数增长的堆泄漏**」负责，**不宣称内存占用绝对值**。

## 1 结果（**6 盘连续**）

```
RESULT mem_match idx=0 finished=true outcome=LoyalistWin steps=69 rev=71 heap_bytes=312902
RESULT mem_match idx=1 finished=true outcome=LoyalistWin steps=69 rev=71 heap_bytes=314587
RESULT mem_match idx=2 finished=true outcome=RenegadeWin { player: PlayerId(2) } steps=5 rev=7 heap_bytes=238848
RESULT mem_match idx=3 finished=true outcome=LoyalistWin steps=26 rev=28 heap_bytes=258136
RESULT mem_match idx=4 finished=true outcome=LoyalistWin steps=30 rev=32 heap_bytes=257883
RESULT mem_match idx=5 finished=true outcome=LoyalistWin steps=69 rev=71 heap_bytes=313104
RESULT mem_summary base_bytes=72800 after_first_bytes=312902 end_bytes=312879 growth_bytes=0
       peak_bytes=474448 slack_bytes=33554432 elapsed_ms=751
```

| 判据 | 结果 |
|---|---|
| **每盘都跑到终局** | ✅ **6/6**（5 次 `LoyalistWin`、1 次 `RenegadeWin{player:2}`）|
| 堆占用曲线 | 312 → 314 → **239** → 258 → 258 → 313 KB（**随对局起伏、无阶跃**）|
| **首盘后 vs 末盘后增长** | **0 字节**（312902 → 312879，**甚至略降**）|
| 峰值 | **474 KB** |
| 阈值 | 宽松 **32 MB**（只抓**阶跃式**泄漏）|

⇒ **连续 6 盘未观察到堆增长**（在本口径内）。

## 2 边界（**务必注意**）

1. **不是 RSS**：栈/映射文件/代码段不在统计内；
2. **6 盘、751 ms** 只够抓**明显**泄漏；「整夜跑」需更长时间与**分位数**分析；
3. 阈值 32 MB **很宽松**——它只用于**防止漏检阶跃**，**不能**据此声称「无任何泄漏」；
4. 未覆盖：**真实网络连接**、跨进程持久化、浏览器端。

## 3 复现

```bash
cd noname-next
cargo test --offline -p game-server --test long_run_memory -- --nocapture
```

---

## 4 扩到 **30 盘** + 趋势判据（同日第二轮）

### 4.1 结果

```
RESULT mem_summary matches=30 base_bytes=72806 after_first_bytes=312908 end_bytes=546868
       growth_bytes=233960 peak_bytes=858256 slack_bytes=33554432 elapsed_ms=6151
RESULT mem_trend head5_avg=276477 tail5_avg=324309 delta=47832 steps_total=1959
       outcomes={"LoyalistWin": 27, "RenegadeWin { player: PlayerId(2) }": 3}
```

| 判据 | 结果 |
|---|---|
| **30 盘全部跑到终局** | ✅ **30/30** |
| 结局分布 | **`LoyalistWin` 27** / **`RenegadeWin{player:2}` 3** |
| 总步数 / 耗时 | **1959 步** / **6.15 s** |
| **首盘后 → 末盘后** | 312908 → 546868（**+233960 B ≈ 228 KB**）|
| **前 5 盘均值 → 后 5 盘均值** | 276477 → 324309（**+47832 B ≈ 47 KB**，约 **1.6 KB/盘**）|
| 峰值 | **858256 B ≈ 838 KB** |
| 阈值 | 32 MB（**仅防阶跃**）|

### 4.2 判读（**如实，不夸大**）

1. **无阶跃**：曲线随对局起伏，**没有**每盘固定 +N 的线性增长证据；
2. **但非零**：30 盘累计 +228 KB、后 5 盘均值 +47 KB ⇒ 可能是**稳定缓存/任务池**，**也可能是缓慢泄漏**；
3. **30 盘、6 秒不足以区分**这两者 ⇒ **不宣称「无泄漏」**，只宣称「**未见阶跃式泄漏**」；
4. 若要定论：需 **≥ 500 盘** 或**固定工作量下的稳态检测**（如连续 100 盘后均值是否仍上升）。

### 4.3 附带观察（**不是规则结论**）

随机走步下 **`LoyalistWin` 27/30** —— 这只反映**固定优先级机器人的行为偏差**，
**不能**用于评价身份局平衡性（选步非强 AI）。
