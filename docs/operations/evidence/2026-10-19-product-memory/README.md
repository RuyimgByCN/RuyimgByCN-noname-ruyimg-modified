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
