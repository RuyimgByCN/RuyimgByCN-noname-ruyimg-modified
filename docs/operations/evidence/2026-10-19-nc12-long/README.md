# NC12 **长局**模式跑批（2026-10-19）

- 工作包：**NC12 / 八项第 8 项（性能）**；规则版本 v322（工作树）/ 存档格式 v105
- 工具：`nc12_bench long tier=<档> seats=4 budget=8 min_cmds=300 max_games=2`
- 原始输出：[long.txt](long.txt)
- 设备/工具链：Apple M3 Max / macOS 27.0.1 / rustc 1.95.0 / `--release`

## 1 结果（4 档 × 2 局 = **8 局**）

| 档 | 局 | 步数 | 收敛 | 停滞 | plan panic |
|---|---:|---:|---|---|---:|
| `none` | 2 | 见 raw | ✅ finished=true | false | 0 |
| `dir:shahai-gwent-upstream-noname` | 2 | 见 raw | ✅ | false | 0 |
| `dir:cardpile-upstream-noname` | 2 | 见 raw | ✅ | false | 0 |
| `dir:shahai-swd-upstream-noname` | 2 | 见 raw | ✅ | false | 0 |

```
games 8
未收敛/停滞/panic: []
steps total 2082
```

| 判据 | 结果 |
|---|---|
| **8 局全部 `finished=true`** | ✅ |
| **无停滞**（`stalled=false`）| ✅ |
| **无规划 panic**（`plan_panics=0`）| ✅ |
| **步数合计** | **2082** |

## 2 存档循环（`phase=post_save_cycles`）

| 档 | `save_cycle_total_ns` | `rss_kb` |
|---|---:|---|
| `none` | **453.7 µs** | **NA** |
| `gwent` | 427.0 µs | NA |
| `cardpile` | 994.0 µs | NA |
| `swd` | 952.3 µs | NA |

## 3 `rss_kb=NA` 的含义（**工具自述**）

bench 自己在字段里写了来源：

```
rss_source=ps -o rss= (沙箱禁止时 NA；外部用 /usr/bin/time -l)
```

⇒ **本环境 `/bin/ps` 被沙箱拒绝**（与本会话[多盘内存观测](../2026-10-19-product-memory/README.md)一致），
bench **如实记 NA**、**不外推**。堆内存另由**计数式全局分配器**测量（见该证据：6 盘堆增长 0 字节、30 盘 +228 KB）。

## 4 判读（**如实**）

1. 长局（`min_cmds=300` 起）在 **4 档共 8 局**中**全部收敛**、**无停滞、无 panic** ✅；
2. **存档循环 0.43–0.99 ms**（单次 save/restore 往返的批量合计）；
3. **不是**优化前后对比；**不是**产品级（无 server/浏览器）；
4. **RSS 无数据**（沙箱禁止 `ps`）—— **不宣称内存指标**。
