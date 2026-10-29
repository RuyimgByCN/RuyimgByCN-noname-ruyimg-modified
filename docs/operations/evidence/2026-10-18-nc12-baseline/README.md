# NC12 效能基线：原生线启动/建局、结算尾延迟、复制/模拟、存读档、AI 与长局内存（2026-10-18）

- 日期：2026-10-18（UTC 10:22–10:24 采集；北京时间 18:22–18:24）
- 工作包：**NC12 效能验收与旧线退出**的**基线测量切片**（[方案 §NC12](../../../architecture/native-convergence-plan.md#nc12)）；口径遵循[性能架构 §11](../../../../noname-performance-architecture.md)与[测试与性能证据](../../phase17-test-perf-infrastructure.md)
- 源码快照：`HEAD = 4cf05dd34ff85bbccc264a304b7d16fadf7d5275`（`feat(engine-native): NC05 第二类案例（战役/跨局档案 + 升级恢复）`），`git status --short | wc -l = 261`（工作区有他人未提交改动；本轮**只新增**测量工具与证据，未改产品源码）
- 测量二进制：`nc12_bench`（release）`sha256 = 27dfa5693c058dd26566657e3db8c54b00e61361615f866c9a36e3d5e119b5d5`；工具源码[`crates/engine-native/examples/nc12_bench.rs`](../../../../crates/engine-native/examples/nc12_bench.rs) `sha256 = 3f293b3435b0847932effe295704d7c5ec87c9140bc8ce5efae4606654c9df8c`。**二进制在整套测量前冻结到临时目录**，避免共享 `target/` 被并发重建替换。
- 一句话结论：**无扩展档的六项指标已取得可复跑基线；单扩展档（swd）的「存读档」因读档校验拒绝而失败、扩展档的 variant-1 AI 会触发宿主断言 panic（含一次权威结算路径 panic），因此扩展档基线只取得部分覆盖**，无任何优化前后收益结论。

> 本页只记录本次快照的实测数字。**不同设备/负载不得相除算收益**；同一台机器上并发有其他 agent 的构建与测试（load average 36.85–44.65），见 §2 与 §8。

## 1 设备与工具链

| 项 | 值 |
|---|---|
| 机型 | Mac15,10（Apple M3 Max），10 性能核 + 4 能效核，内存 38,654,705,664 B（36 GiB） |
| 系统 | macOS 27.0.1（Build 26A434），Darwin 27.0.0 arm64（`RELEASE_ARM64_T6031`） |
| 工具链 | rustc 1.95.0 (59807616e 2026-04-14) / cargo 1.95.0 / `stable-aarch64-apple-darwin` |
| 构建 | `cargo build --offline --release -p engine-native --example nc12_bench`（workspace release profile，含 LTO/单 codegen unit） |
| 温控 | `pmset -g therm`：无温控/性能告警记录 |
| 并发负载 | 整套测量期间 `uptime` load average **36.85–44.65**（14 逻辑核，同机其他 agent 的 cargo/测试并发）。**p95/max 受此污染**，p50 相对稳健 |

原始环境输出见 [01-env.txt](01-env.txt)（含 build 前 `HEAD`、dirty 计数、`git diff --stat`、工具哈希、load）。

## 2 测量口径（可复跑）

### 2.1 三档负载（不得跨档算收益）

| 档位 | 来源目录（`packages/extensions/`） | package_id | 目录规模（卡牌/技能/扩展武将） |
|---|---|---|---|
| `none` 无扩展 | —— | —— | 124 / 215 / 0 |
| `swd` 单扩展 | `shahai-swd-upstream-noname` | `shahai-swd` | 214 / 429 / 79 |
| `huanle` 单扩展 | `huanle-upstream-noname` | `huanle-cards` | 131 / 220 / 0 |
| `combo` 常用组合 | `huanle-upstream-noname` + `shahai-swd-upstream-noname` + `shahai-gujian-upstream-noname` | `huanle-cards`+`shahai-swd`+`shahai-gujian` | 261 / 452 / 79 |

装载走**真实安装路径**：`import_extensions`（全量 drop-in 根，41 个目录）→ 内容锁 → `bridge_pack(<包>/assets/cards.json)` → `Engine::new_with_extensions`。注意 `packages/extensions/` 41 个目录中只有 19 个提供 `assets/cards.json`（可用 `bridge_pack` 装载）；本轮三档都在其中。

### 2.2 样本与统计

- 微基准（建局、save/clone/plan/归档编解码）：**每组 30 个样本**（性能架构建议「热态微基准至少 30 组」）。
- 整局驱动：**每组 3 局**，种子 `42007/42008/42009`，4 座、`Engine::new*`（Prototype 模式，存活 ≤1 终局）；每组样本数=总步数（数百条命令）。
- 分位数为 **nearest-rank**：`排序后 v[ceil(p/100·n)−1]`；`plan`=AI 克隆模拟耗时，`submit`=权威规则结算耗时（`apply_plan`→`submit`），`cmd`=两者之和（端到端命令延迟）。
- **降级（degraded）**：扩展档 variant-1 规划会触发产品源码 panic，进程直接中止。为此另有 `catch_panics=1` 一轮：捕获 `plan_action` 内 panic，记录 `plan_panics`，以「按 `min` 取首选项/推进阶段」的兜底动作续行。**降级轮只用于补齐覆盖，其数字不作延迟口径、不作收益计算**（工具在每行输出 `degraded=true`）。
- 内存：进程峰值 RSS 用外部 `/usr/bin/time -l` 的 `maximum resident set size`；进程内 RSS 采样不可用（本机沙箱禁止 `ps`/`top`，`Operation not permitted`），改用**进程内计数分配器**（`NC12_ALLOC=1`）统计窗口内堆分配/释放字节、分配次数与峰值活字节。开启计数后计时含原子操作开销，该轮延迟数字不作口径。
- 失败分类：**环境失败**＝包导入/桥接/目录缺失；**测量失败**＝对局未收敛、命令被拒、读档被拒、进程 panic。两者在日志中分别标注，首次失败原样保留。
- 原始样本：每次运行可用 `csv=<path>` 导出逐样本 ns；本轮为控制证据体积未落盘逐样本 CSV（分位与 total/n 已在日志中），如需重跑见 §9。

### 2.3 未测量项（不得外推）

1. **真实浏览器/WASM 档位**：未构建 `engine-native-wasm`、未在真实浏览器测启动/帧/内存。
2. **移动设备与 Windows/Linux 原生档位**：本轮仅 macOS arm64 单机。
3. **动画帧、输入响应、GPU 提交与端到端显示延迟**：需要真实客户端与显示测量；本轮为无界面内核测量。
4. **联机 RTT/丢包档位与服务端处理耗时**：需要真实网络档位。
5. **≥30 分钟持续负载、温控/降频/耗电曲线**：本轮为短时测量。
6. **进程冷启动「创建到可操作」**：本轮只测 `Engine::new*`/目录构建 API 级耗时（首次安装 vs 稳定安装的对照未做）。
7. **其余 38 个扩展包逐包性能**：本轮只覆盖 3 档来源（4 个包）。
8. **其它模式与座数**：本轮全部为 4 座 Prototype；身份局/Boss/KOF/国战与 5–8 座未测。
9. **旧线对照**：未测 `engine-core`/`engine-ai`/`engine-wasm`，因此**不给任何提速倍数**。
10. **分配/火焰图剖析**：未做 plan_action 内部热点与分配点归因。
11. **逐样本原始 CSV、DVFS/频率采样**：未落盘。
12. **`restore` 失败后的恢复语义**（swd）：已知失败，未做「何种状态可恢复」的边界刻画。

## 3 结果表

### 表 A 三档装载与目录规模（`probe samples=5`，2026-10-18 快照）

| 档位 | 来源目录 | package_id | 全量导入 import_ns | 请求包桥接 bridge_ns | 卡牌 | 技能 | 扩展武将 | ruleset_version | content_lock | 开局归档字节 |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| none | （基础目录） | -（未加载扩展） | - | - | 124 | 215 | 0 | 321 | 0 | 1888 |
| swd | shahai-swd-upstream-noname | shahai-swd | 51794000 | 665042 | 214 | 429 | 79 | 18039999305147582814 | 18039999305147582495 | 2504 |
| huanle | huanle-upstream-noname | huanle-cards | 42944750 | 198375 | 131 | 220 | 0 | 11753449270702291366 | 2530077233847515367 | 1916 |
| combo | huanle-upstream-noname+shahai-swd-upstream-noname+shahai-gujian-upstream-noname | huanle-cards+shahai-swd+shahai-gujian | 41803834 | 710542 | 261 | 452 | 79 | 17444453271599010434 | 8221081234744234947 | 2724 |

### 表 B 建局耗时（`samples=30`，4 座，种子 42007+i，ns）

| 档位 | 标准目录构建 p50 | 建局 p50 | 建局 p95 | 建局 max | 全量导入 import | 请求包桥接 bridge | 卡牌 | 技能 | 开局归档字节 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| none | 4542 | 4750 | 6125 | 14750 | - | - | 124 | 215 | 1888 |
| swd | 4542 | 72500 | 194500 | 223208 | 51581833 | 475500 | 214 | 429 | 2504 |
| huanle | 4542 | 15958 | 23333 | 58584 | 61810708 | 73125 | 131 | 220 | 1916 |
| combo | 4542 | 76333 | 160542 | 168250 | 56279541 | 539916 | 261 | 452 | 2724 |

### 表 C 整局规划驱动：每命令延迟（4 座，3 局/组，种子 42007–42009）

分位单位 ns；`plan`=AI 克隆模拟，`submit`=权威规则结算，`cmd`=plan+submit。降级=该组含规划 panic（catch_panics=1 捕获后以兜底动作续行）。

| 轮次 | 档位 | 预算 | 变体 | 局数 | 步数 | 收官局 | wall/步 | plan p50 | plan p95 | plan max | submit p50 | submit p95 | submit max | cmd p50 | cmd p95 | cmd max | 共识节点/秒 | 规划 panic | 降级 |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|:--:|
| 干净 | combo | 256 | 0 | 3 | 1301 | 3/3 | 51000 | 29417 | 97291 | 394708 | 12458 | 18292 | 85083 | 41958 | 111958 | 479791 | 51935 | 0 | false |
| 干净 | huanle | 256 | 0 | 3 | 612 | 3/3 | 33863 | 17375 | 64916 | 166458 | 8125 | 20083 | 39666 | 25541 | 77166 | 188166 | 82860 | 0 | false |
| 干净 | none | 256 | 0 | 3 | 631 | 3/3 | 37058 | 18959 | 67666 | 176916 | 8208 | 19750 | 44333 | 27250 | 83042 | 200708 | 75679 | 0 | false |
| 干净 | swd | 256 | 0 | 3 | 809 | 3/3 | 43540 | 23500 | 81667 | 256875 | 11584 | 20166 | 44875 | 35709 | 97834 | 289750 | 62827 | 0 | false |
| 干净 | combo | 256 | 1 | 3 | 1008 | 3/3 | 118302 | 38334 | 391125 | 651917 | 12958 | 23292 | 134542 | 51666 | 408083 | 688042 | 51385 | 2 | true |
| 干净 | huanle | 256 | 1 | 3 | 8210 | 1/3 | 401940 | 228709 | 1470417 | 12025375 | 8459 | 16333 | 264833 | 236583 | 1481541 | 12053292 | 10720 | 0 | false |
| 干净 | none | 256 | 1 | 3 | 1025 | 3/3 | 73700 | 25708 | 206583 | 416625 | 8292 | 15333 | 75917 | 34416 | 217500 | 431875 | 78967 | 0 | false |
| 干净 | combo | 64 | 0 | 3 | 1301 | 3/3 | 50792 | 29584 | 95166 | 208958 | 12417 | 18333 | 69584 | 42250 | 111250 | 243500 | 52224 | 0 | false |
| 干净 | huanle | 64 | 0 | 3 | 612 | 3/3 | 35265 | 18208 | 65625 | 177166 | 8167 | 20208 | 63041 | 26666 | 82625 | 199416 | 79363 | 0 | false |
| 干净 | none | 64 | 0 | 3 | 631 | 3/3 | 36028 | 18875 | 67292 | 189541 | 8125 | 19584 | 28708 | 26917 | 82625 | 213375 | 77582 | 0 | false |
| 干净 | swd | 64 | 0 | 3 | 809 | 3/3 | 43263 | 23459 | 79500 | 248125 | 11458 | 19167 | 43958 | 35124 | 96625 | 282375 | 63034 | 0 | false |
| 干净 | combo | 64 | 1 | 3 | 1008 | 3/3 | 131768 | 38833 | 397458 | 3154375 | 12750 | 23625 | 2764875 | 51959 | 417208 | 5589625 | 46687 | 2 | true |
| 干净 | huanle | 64 | 1 | 3 | 8210 | 1/3 | 402125 | 226417 | 1484166 | 7986167 | 8458 | 16750 | 1920291 | 234625 | 1493333 | 8008625 | 10734 | 0 | false |
| 干净 | none | 64 | 1 | 3 | 1025 | 3/3 | 74686 | 25875 | 209417 | 439167 | 8375 | 17000 | 50250 | 34583 | 219126 | 456459 | 78027 | 0 | false |
| 干净 | combo | 8 | 0 | 3 | 1301 | 3/3 | 50916 | 29125 | 95875 | 226500 | 12542 | 17500 | 55959 | 41792 | 113625 | 261208 | 51979 | 0 | false |
| 干净 | huanle | 8 | 0 | 3 | 612 | 3/3 | 34178 | 17750 | 64959 | 170708 | 8125 | 20042 | 36041 | 25958 | 81543 | 192375 | 81851 | 0 | false |
| 干净 | none | 8 | 0 | 3 | 631 | 3/3 | 39465 | 19125 | 71917 | 871041 | 8250 | 19958 | 40375 | 27500 | 87584 | 893458 | 69225 | 0 | false |
| 干净 | swd | 8 | 0 | 3 | 809 | 3/3 | 43569 | 23792 | 79875 | 303084 | 11542 | 19042 | 64875 | 35667 | 93875 | 340418 | 62563 | 0 | false |
| 干净 | combo | 8 | 1 | 3 | 498 | 3/3 | 82044 | 30917 | 168916 | 301500 | 13708 | 30542 | 193709 | 46166 | 196917 | 337916 | 56301 | 0 | false |
| 干净 | none | 8 | 1 | 3 | 696 | 3/3 | 55287 | 21875 | 117750 | 223334 | 8500 | 16125 | 49083 | 30333 | 133709 | 228709 | 78998 | 0 | false |
| 干净 | swd | 8 | 1 | 3 | 646 | 3/3 | 75447 | 29958 | 154583 | 295792 | 12167 | 22875 | 77209 | 42583 | 175125 | 333333 | 59650 | 0 | false |
| 兜底 | combo | 256 | 1 | 3 | 1008 | 3/3 | 131516 | 38583 | 405709 | 2697584 | 12917 | 25708 | 77458 | 52041 | 428583 | 2720334 | 45750 | 2 | true |
| 兜底 | huanle | 256 | 1 | 3 | 8210 | 1/3 | 414152 | 230584 | 1453875 | 31149250 | 8667 | 17084 | 303167 | 239001 | 1464751 | 31186250 | 10406 | 0 | false |
| 兜底 | none | 256 | 1 | 3 | 1025 | 3/3 | 73354 | 26084 | 206375 | 380000 | 8333 | 15458 | 60792 | 34667 | 217124 | 395291 | 79298 | 0 | false |
| 兜底 | combo | 64 | 1 | 3 | 1008 | 3/3 | 117170 | 37958 | 371667 | 628084 | 12917 | 23542 | 64958 | 51666 | 398583 | 663459 | 51895 | 2 | true |
| 兜底 | huanle | 64 | 1 | 3 | 8210 | 1/3 | 419727 | 231125 | 1471792 | 30532375 | 8708 | 17416 | 801083 | 239500 | 1480792 | 30537750 | 10270 | 0 | false |
| 兜底 | none | 64 | 1 | 3 | 1025 | 3/3 | 73927 | 25500 | 211792 | 408708 | 8333 | 15584 | 45583 | 34125 | 221541 | 427666 | 78680 | 0 | false |
| 兜底 | combo | 8 | 1 | 3 | 498 | 3/3 | 79331 | 30167 | 162333 | 354042 | 13500 | 29000 | 50125 | 45209 | 187749 | 393876 | 58051 | 0 | false |
| 兜底 | huanle | 8 | 1 | 3 | 932 | 3/3 | 63364 | 30833 | 147250 | 231958 | 9166 | 17625 | 77000 | 40250 | 159458 | 243750 | 65691 | 2 | true |
| 兜底 | none | 8 | 1 | 3 | 696 | 3/3 | 54567 | 21875 | 113959 | 190791 | 8459 | 16708 | 27834 | 30666 | 127334 | 213999 | 80116 | 0 | false |
| 兜底 | swd | 8 | 1 | 3 | 646 | 3/3 | 76722 | 30667 | 160000 | 286250 | 12500 | 23542 | 70542 | 42667 | 178917 | 311208 | 58747 | 0 | false |

### 表 D 状态复制 / AI 模拟 / 归档编码（`budget=64`，`samples=30`，ns）

| 档位 | 阶段 | 步数 | save p50 | Engine::clone p50 | plan_action p50 | plan_action p95 | encode_archive p50 | 归档字节 | 规划节点 p50/max | 状态形状 |
|---|---|---:|---:|---:|---:|---:|---:|---:|---|---|
| none | open | 0 | 292 | 292 | 9500 | 12791 | 2500 | 1888 | 1/1 | players=4 frames=0 parks=0 skills=0 vm_marks=0 vm_piles=0 vm_once=0 vm_flags=0 deck=108 discard=0 table=0 hand=16 equips=0 judge=0 off_court=0 lianyaohu_store=0 zhucheng=0 log=0 revision=0 |
| none | mid | 60 | 291 | 1375 | 13708 | 53792 | 2625 | 1913 | 1/1 | players=4 frames=0 parks=0 skills=0 vm_marks=0 vm_piles=0 vm_once=0 vm_flags=0 deck=90 discard=21 table=0 hand=13 equips=0 judge=0 off_court=0 lianyaohu_store=0 zhucheng=0 log=153 revision=60 |
| swd | open | 0 | 375 | 416 | 17250 | 56875 | 3625 | 2504 | 1/1 | players=4 frames=0 parks=0 skills=32 vm_marks=0 vm_piles=0 vm_once=0 vm_flags=0 deck=198 discard=0 table=0 hand=16 equips=0 judge=0 off_court=0 lianyaohu_store=0 zhucheng=0 log=0 revision=0 |
| swd | mid | 60 | 375 | 1500 | 20875 | 23084 | 3542 | 2533 | 1/1 | players=4 frames=0 parks=0 skills=32 vm_marks=0 vm_piles=0 vm_once=0 vm_flags=0 deck=178 discard=26 table=0 hand=11 equips=0 judge=0 off_court=0 lianyaohu_store=0 zhucheng=0 log=170 revision=60 |
| combo | open | 0 | 416 | 416 | 17666 | 23667 | 3666 | 2724 | 1/1 | players=4 frames=0 parks=0 skills=36 vm_marks=0 vm_piles=0 vm_once=0 vm_flags=0 deck=245 discard=0 table=0 hand=16 equips=0 judge=0 off_court=0 lianyaohu_store=0 zhucheng=0 log=0 revision=0 |
| combo | mid | 60 | 333 | 1292 | 19875 | 20167 | 3583 | 2749 | 1/1 | players=4 frames=0 parks=0 skills=36 vm_marks=0 vm_piles=0 vm_once=0 vm_flags=0 deck=225 discard=23 table=0 hand=13 equips=0 judge=0 off_court=0 lianyaohu_store=0 zhucheng=0 log=157 revision=60 |

**状态规模随对局推进（growth 采样：帧数/技能数/牌区/日志/归档字节）**

| 档位 | 步 | revision | frames | parks | skills | deck | discard | hand | log | 归档字节 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| none | 0 | 0 | 0 | 0 | 0 | 108 | 0 | 16 | 0 | 1888 |
| none | 20 | 20 | 0 | 0 | 0 | 102 | 7 | 15 | 49 | 1903 |
| none | 40 | 40 | 0 | 0 | 0 | 96 | 16 | 12 | 102 | 1908 |
| none | 60 | 60 | 0 | 0 | 0 | 90 | 21 | 13 | 153 | 1913 |
| none | 80 | 80 | 0 | 0 | 0 | 84 | 29 | 11 | 205 | 1918 |
| none | 100 | 100 | 0 | 0 | 0 | 78 | 36 | 10 | 253 | 1928 |
| none | 120 | 120 | 0 | 0 | 0 | 72 | 42 | 10 | 303 | 1938 |
| none | 140 | 140 | 0 | 0 | 0 | 66 | 50 | 8 | 356 | 1948 |
| none | 160 | 160 | 0 | 0 | 0 | 60 | 57 | 7 | 406 | 1958 |
| none | 180 | 180 | 0 | 0 | 0 | 54 | 65 | 5 | 459 | 1968 |
| none | 200 | 200 | 2 | 1 | 0 | 48 | 71 | 5 | 509 | 2122 |
| none | 220 | 220 | 0 | 0 | 0 | 42 | 77 | 5 | 557 | 1988 |
| none | 240 | 240 | 2 | 1 | 0 | 36 | 84 | 4 | 610 | 2137 |
| swd | 0 | 0 | 0 | 0 | 32 | 198 | 0 | 16 | 0 | 2504 |
| swd | 20 | 20 | 0 | 0 | 32 | 191 | 9 | 15 | 58 | 2523 |
| swd | 40 | 40 | 0 | 0 | 32 | 184 | 20 | 11 | 117 | 2533 |
| swd | 60 | 60 | 0 | 0 | 32 | 178 | 26 | 11 | 170 | 2533 |
| swd | 80 | 80 | 0 | 0 | 32 | 170 | 32 | 13 | 223 | 2533 |
| swd | 100 | 100 | 0 | 0 | 32 | 164 | 38 | 13 | 273 | 2543 |
| swd | 120 | 120 | 3 | 1 | 24 | 158 | 48 | 9 | 337 | 2688 |
| swd | 140 | 140 | 0 | 0 | 16 | 152 | 55 | 8 | 396 | 2430 |
| swd | 160 | 160 | 0 | 0 | 16 | 145 | 64 | 6 | 450 | 2435 |
| swd | 180 | 180 | 0 | 0 | 16 | 139 | 71 | 5 | 502 | 2440 |
| swd | 200 | 200 | 0 | 0 | 16 | 133 | 77 | 5 | 555 | 2440 |
| swd | 220 | 220 | 0 | 0 | 16 | 125 | 83 | 7 | 608 | 2440 |
| swd | 240 | 240 | 0 | 0 | 16 | 119 | 89 | 7 | 658 | 2445 |
| combo | 0 | 0 | 0 | 0 | 36 | 245 | 0 | 16 | 0 | 2724 |
| combo | 20 | 20 | 0 | 0 | 36 | 239 | 7 | 15 | 49 | 2739 |
| combo | 40 | 40 | 0 | 0 | 36 | 233 | 16 | 12 | 102 | 2749 |
| combo | 60 | 60 | 0 | 0 | 36 | 225 | 23 | 13 | 157 | 2749 |
| combo | 80 | 80 | 2 | 1 | 36 | 219 | 29 | 13 | 210 | 2898 |
| combo | 100 | 100 | 2 | 1 | 36 | 213 | 36 | 12 | 261 | 2908 |
| combo | 120 | 120 | 0 | 0 | 36 | 207 | 46 | 8 | 315 | 2764 |
| combo | 140 | 140 | 0 | 0 | 36 | 201 | 52 | 8 | 368 | 2764 |
| combo | 160 | 160 | 0 | 0 | 27 | 195 | 59 | 7 | 430 | 2697 |
| combo | 180 | 180 | 0 | 0 | 27 | 187 | 65 | 9 | 483 | 2697 |
| combo | 200 | 200 | 0 | 0 | 27 | 181 | 73 | 7 | 537 | 2697 |
| combo | 220 | 220 | 0 | 0 | 27 | 175 | 79 | 7 | 590 | 2697 |
| combo | 240 | 240 | 0 | 0 | 27 | 167 | 85 | 9 | 643 | 2697 |
| combo | 260 | 260 | 0 | 0 | 18 | 163 | 92 | 6 | 703 | 2635 |
| combo | 280 | 280 | 0 | 0 | 18 | 155 | 100 | 6 | 757 | 2640 |
| combo | 300 | 300 | 2 | 1 | 18 | 149 | 106 | 6 | 810 | 2784 |
| combo | 320 | 320 | 0 | 0 | 18 | 143 | 114 | 4 | 863 | 2640 |
| combo | 340 | 340 | 0 | 0 | 18 | 137 | 120 | 4 | 915 | 2645 |
| combo | 360 | 360 | 0 | 0 | 18 | 129 | 127 | 5 | 969 | 2645 |
| combo | 380 | 380 | 0 | 0 | 18 | 123 | 135 | 3 | 1023 | 2645 |
| combo | 400 | 400 | 0 | 0 | 18 | 117 | 142 | 2 | 1075 | 2650 |

### 表 E 存读档（`samples=30`，`steps=80`，ns）

| 档位 | 阶段 | 步数 | save p50 | encode p50 | decode p50 | restore p50 | restore p95 | restore max | 归档字节 | 往返失败 | 状态哈希不一致 | 结果 |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|:--:|
| none | open | 0 | 292 | 2291 | 3334 | 4959 | 10250 | 29083 | 1888 | 0 | 0 | OK |
| none | played | 80 | 292 | 2292 | 3416 | 4791 | 5667 | 8750 | 1918 | 0 | 0 | OK |
| swd | open | 0 | 375 | 3208 | 4542 | 64250 | 106000 | 113208 | 2504 | 0 | 0 | OK |
| swd | played | 80 | 500 | 3250 | 4792 | 65708 | 77334 | 89292 | 2533 | 30 | 0 | FAIL |
| huanle | open | 0 | 333 | 2417 | 3542 | 16792 | 27458 | 41208 | 1916 | 0 | 0 | OK |
| huanle | played | 80 | 292 | 2375 | 3542 | 16458 | 31500 | 47792 | 1946 | 0 | 0 | OK |
| combo | open | 0 | 417 | 3375 | 5083 | 73583 | 97333 | 118833 | 2724 | 0 | 0 | OK |
| combo | played | 80 | 458 | 3542 | 5375 | 67417 | 74708 | 82750 | 2898 | 0 | 0 | OK |

**读档被拒诊断**（同一快照重复 30 次的同一诊断，去重后）

| 档位 | 阶段 | what | card | card_name | 计入牌区 | 未计入特殊区 | 挂起机器 parks |
|---|---|---|---:|---|---|---|---:|
| swd | played | card_in_multiple_zones | 13 | sha | [discard=2] uncounted=[declared_kinds] parks=0 card_name=sha | - | - |

### 表 F AI 策略矩阵（同种子 42007，`cmds=200`，ns）

兜底=该组含规划 panic（combo 档 variant1 在 step 5 崩，见 §5）。`与 v0 决策不同步`=同预算下与 lookahead1/variant0 逐步比较的不同决策数。

| 档位 | 预算 | lookahead | variant | 步数 | accepted | plan p50 | plan p95 | plan max | 节点 total | 节点 p50/max | 节点/秒 | 与 v0 不同步 | 收官 | 规划 panic |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|---:|---:|:--:|---:|
| none | 8 | 1 | 0 | 200 | 200 | 19334 | 73542 | 180333 | 433 | 1/8 | 72965 | 0 | false | 0 |
| none | 8 | 1 | 1 | 200 | 200 | 19084 | 107417 | 147541 | 725 | 1/8 | 86578 | 104 | false | 0 |
| none | 8 | 2 | 0 | 166 | 166 | 18834 | 81667 | 179208 | 442 | 1/8 | 89148 | 37 | true | 0 |
| none | 8 | 2 | 1 | 118 | 118 | 28250 | 112000 | 147375 | 526 | 3/8 | 100428 | 70 | true | 0 |
| none | 64 | 1 | 0 | 200 | 200 | 18458 | 75417 | 146292 | 433 | 1/8 | 80335 | 0 | false | 0 |
| none | 64 | 1 | 1 | 200 | 200 | 19000 | 207708 | 379042 | 1176 | 1/25 | 93003 | 94 | false | 0 |
| none | 64 | 2 | 0 | 200 | 200 | 18709 | 101000 | 149375 | 530 | 1/17 | 85281 | 0 | false | 0 |
| none | 64 | 2 | 1 | 200 | 200 | 27792 | 240125 | 300625 | 1421 | 2/33 | 98107 | 89 | false | 0 |
| none | 256 | 1 | 0 | 200 | 200 | 16458 | 63750 | 134458 | 433 | 1/8 | 85814 | 0 | false | 0 |
| none | 256 | 1 | 1 | 200 | 200 | 18375 | 226375 | 339083 | 1176 | 1/25 | 95057 | 94 | false | 0 |
| none | 256 | 2 | 0 | 200 | 200 | 20667 | 92917 | 183792 | 530 | 1/17 | 85369 | 0 | false | 0 |
| none | 256 | 2 | 1 | 200 | 200 | 27209 | 222167 | 300333 | 1421 | 2/33 | 101961 | 89 | false | 0 |
| combo | 64 | 1 | 0 | 200 | 200 | 26458 | 81583 | 214166 | 406 | 1/8 | 58748 | 0 | false | 0 |
| combo | 64 | 1 | 1 | 200 | 200 | 27084 | 376250 | 506125 | 1342 | 1/32 | 62672 | 92 | false | 1 |
| combo | 64 | 2 | 0 | 200 | 200 | 24542 | 79625 | 221083 | 469 | 1/13 | 67217 | 0 | false | 0 |
| combo | 64 | 2 | 1 | 107 | 107 | 43416 | 517208 | 698334 | 1023 | 3/53 | 73350 | 58 | true | 1 |

### 表 G 长局内存（`budget=8`，`min_cmds=800`，`NC12_ALLOC=1` 计数分配器）

堆字节为「打开计数后」窗口内的累计量；峰值活字节为 alloc−free 的运行峰值。RSS 由外部 `/usr/bin/time -l` 取 maximum resident set size（沙箱禁止 `ps`/`top`）。

| 档位 | 局 | 步数 | 收官 | 规划 panic | 窗口堆分配 (B) | 窗口堆释放 (B) | 局末活字节 (B) |
|---|---:|---:|:--:|---:|---:|---:|---:|
| none | 0 | 259 | true | 0 | 32939176 | 32823201 | 115975 |
| none | 1 | 144 | true | 0 | 45899159 | 45825583 | 73576 |
| none | 2 | 228 | true | 0 | 73221231 | 73122103 | 99128 |
| none | 3 | 219 | true | 0 | 96836103 | 96737184 | 98919 |
| combo | 0 | 404 | true | 0 | 76993782 | 76732586 | 261196 |
| combo | 1 | 521 | true | 0 | 199296114 | 199000611 | 295503 |

| 档位 | 局数 | 总步数 | 窗口分配 (B) | 窗口释放 (B) | 活字节增量 (B) | 分配次数 | 峰值活字节 (B) | 进程峰值 RSS (B) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| none | 4 | 850 | 97058709 | 97057279 | 1430 | 413863 | 262112 | 5619712 |
| combo | 2 | 925 | 199793279 | 199736030 | 57249 | 861852 | 592389 | 7897088 |

### 表 H 失败与边界

**H1a 非零退出清单（未捕获的 panic 直接中止进程；exit=101）**

| 日志 | 档位 | 预算 | lookahead | variant | 种子 | 额外参数 | 退出码 |
|---|---|---:|---:|---:|---:|---|---:|
| 04 | swd | 64 | 1 | 1 | 42007 | - | 101 |
| 04 | swd | 256 | 1 | 1 | 42007 | - | 101 |
| 04 | huanle | 8 | 1 | 1 | 42007 | - | 101 |
| 04b | swd | 64 | 1 | 1 | 42007 | catch_panics=1 | 101 |
| 04b | swd | 256 | 1 | 1 | 42007 | catch_panics=1 | 101 |
| 09 | combo | 64 | 1 | 1 | 42007 | - | 101 |
| 09b | swd | 64 | 1 | 1 | 42007 | catch_panics=1 | 101 |
| 10 | combo | 64 | 1 | 1 | 42007 | - | 101 |
| 10 | combo | 256 | 1 | 1 | 42007 | - | 101 |
| 10 | combo | 64 | 2 | 1 | 42007 | - | 101 |

**H1b panic 现场（位置与消息，按日志顺序）**

| 日志 | # | panic 位置 | 消息 |
|---|---:|---|---|
| 04 | 1 | crates/engine-native-kernel/src/machine.rs:408:32 | card 204 not in hand of 0 |
| 04 | 2 | crates/engine-native-kernel/src/machine.rs:408:32 | card 204 not in hand of 0 |
| 04 | 3 | crates/engine-native-kernel/src/kernel/monkey.rs:73:26 | 被拦截的桃应在使用者手牌 |
| 04 | 4 | crates/engine-native-kernel/src/machine.rs:408:32 | card 90 not in hand of 0 |
| 04 | 5 | crates/engine-native-kernel/src/kernel/monkey.rs:73:26 | 被拦截的桃应在使用者手牌 |
| 04 | 6 | crates/engine-native-kernel/src/machine.rs:408:32 | card 90 not in hand of 0 |
| 04 | 7 | crates/engine-native-kernel/src/kernel/monkey.rs:73:26 | 被拦截的桃应在使用者手牌 |
| 04b | 1 | crates/engine-native-kernel/src/machine.rs:408:32 | card 204 not in hand of 0 |
| 04b | 2 | crates/engine-native-kernel/src/machine.rs:408:32 | card 204 not in hand of 0 |
| 04b | 3 | crates/engine-native-kernel/src/kernel/monkey.rs:73:26 | 被拦截的桃应在使用者手牌 |
| 04b | 4 | crates/engine-native-kernel/src/kernel/monkey.rs:73:26 | 被拦截的桃应在使用者手牌 |
| 04b | 5 | crates/engine-native-kernel/src/machine.rs:408:32 | card 90 not in hand of 0 |
| 04b | 6 | crates/engine-native-kernel/src/kernel/monkey.rs:73:26 | 被拦截的桃应在使用者手牌 |
| 04b | 7 | crates/engine-native-kernel/src/machine.rs:408:32 | card 90 not in hand of 0 |
| 04b | 8 | crates/engine-native-kernel/src/kernel/monkey.rs:73:26 | 被拦截的桃应在使用者手牌 |
| 09 | 1 | crates/engine-native-kernel/src/machine.rs:408:32 | card 90 not in hand of 0 |
| 09b | 1 | crates/engine-native-kernel/src/machine.rs:408:32 | card 204 not in hand of 0 |
| 10 | 1 | crates/engine-native-kernel/src/machine.rs:408:32 | card 90 not in hand of 0 |
| 10 | 2 | crates/engine-native-kernel/src/machine.rs:408:32 | card 90 not in hand of 0 |
| 10 | 3 | crates/engine-native-kernel/src/machine.rs:408:32 | card 90 not in hand of 0 |

**H2 读档失败边界（archive：swd 单扩展）**

| 档位 | 步数 | 往返失败/样本 | 结果 | 诊断 |
|---|---:|---:|:--:|---|
| none | 5 | 0/3 | OK | - |
| none | 20 | 0/3 | OK | - |
| none | 40 | 0/3 | OK | - |
| huanle | 5 | 0/3 | OK | - |
| huanle | 20 | 0/3 | OK | - |
| huanle | 40 | 0/3 | OK | - |
| swd | 5 | 0/3 | OK | - |
| swd | 20 | 3/3 | FAIL | card_in_multiple_zones card=13(sha) [discard=2] uncounted=[declared_kinds] parks=0 card_name=sha |
| swd | 40 | 3/3 | FAIL | card_in_multiple_zones card=13(sha) [discard=2] uncounted=[declared_kinds] parks=0 card_name=sha |
| combo | 5 | 0/3 | OK | - |
| combo | 20 | 0/3 | OK | - |
| combo | 40 | 0/3 | OK | - |


## 4 关键数字摘要（本机本快照，load 36.9–44.7）

- **建局 API**：基础目录构建 p50 **4.54 µs**；`Engine::new(seed,4)` p50 **4.75 µs**。带扩展建局 p50：swd **72.5 µs**、huanle **15.96 µs**、combo **76.3 µs**（p95 分别 194.5/23.3/160.5 µs）。全量 drop-in 导入（41 目录，一次性环境成本）**42–62 ms**，请求包桥接 **0.07–0.71 ms**。
- **规则结算（submit，权威路径）** p50：none **8.2 µs** / swd **11.5 µs** / huanle **8.2 µs** / combo **12.4 µs**（budget=64 variant=0）；p95 均 ≈ **1.9–2.0×p50**，max 29–70 µs。**这是不含 AI 的规则尾延迟口径**。
- **AI 规划（plan_action，克隆模拟）** p50：none **18.9 µs** / swd **23.5 µs** / huanle **18.2 µs** / combo **29.6 µs**（budget=64 variant=0）。variant=1（全手牌候选）同档同预算 p50：none 18.9→**25.9 µs（1.37×）**、combo 29.6→**38.8 µs（1.31×）**、swd（budget=8 口径）23.8→**30.0 µs（1.26×）**，节点总数 1.7–2.7×；**huanle variant=1 为 226 µs，但该组未收敛（§5.4），不作为变体代价口径**。
- **端到端每命令（cmd=plan+submit）** p50：none **26.9 µs** / swd **35.1 µs** / huanle **26.7 µs** / combo **42.3 µs**（budget=64 variant=0）。
- **整局耗时**（3 局合计/每局，variant=0）：none 21.9 ms/3 局（631 步，**7.3 ms/局**）、swd 34.2 ms（809 步，11.4 ms/局）、huanle 21.1 ms（612 步，7.0 ms/局）、combo 63.2 ms（1301 步，21.1 ms/局）。整局步数本身随内容与策略变化，**不是吞吐指标**。
- **预算是否起作用**：variant=0 候选极少（3 局总节点 1284–2597），预算 8/64/256 给出**逐值相同**的轨迹与指标——即该策略下预算不是约束；variant=1 时 budget=8 会提前耗尽（none 档出现 48 步 `budget_exhausted`），64 与 256 结果一致（节点 5252，未触顶）。
- **状态复制**：`Engine::save()` p50 **0.29–0.42 µs**；`Engine::clone()` p50 **0.29–1.50 µs**（同为 4 座，随技能实例数与牌区增大而增大：none open 0.29 µs → swd mid 1.50 µs）；`encode_archive` p50 **2.29–3.67 µs**。归档字节随对局推进小幅增长（none 1888→~1950 B、swd 2504→2543 B、combo 2724→2898 B）；帧/机器挂起时（frames=2/parks=1）会跳到 2122–2908 B。
- **状态规模增长**（growth 采样，见 §3 表 D）：`players` 恒 4；`frames/parks` 大多数步为 0，挂起时最多 3 帧/1 park；`skills` 随死亡回收（swd 32→16）；`deck` 单调下降、`discard` 单调上升、`log` 线性增长（240 步 ≈ 610 条事件）；归档字节与 `log` 长度弱相关。
- **存读档**（30 次往返）：`save` p50 0.29–0.50 µs、`encode` 2.29–3.54 µs、`decode` 3.33–5.38 µs、`restore` p50 none **4.79–4.96 µs** / huanle **16.5–16.8 µs** / combo **67.4–73.6 µs**（含扩展目录重建），`state_hash` 往返全部一致（30/30）。**swd 档 30/30 次读档被拒**（见 §5.1），无有效 restore 数字。
- **AI 矩阵**（同种子 42007、200 步）：节点吞吐 **7.3–10.2 万节点/秒**；`lookahead=2` 相对 1 的 p50 变化：variant=0 在噪声级（none 18.5→18.7 µs、combo 26.5→24.5 µs），variant=1 为 **+46%（none 19.0→27.8 µs）/+60%（combo 27.1→43.4 µs）**，节点数 1.21–1.22×（none 档）。`variant=1` 相对 0 的决策差异 **58–104 步**（`cmds=200`；不同预算/档位下不同，轨迹分叉后不可逐步比较）。
- **长局内存**（budget=8，≥800 步）：none 4 局 850 步 → 窗口分配 **97.06 MB**、释放 **97.06 MB**、活字节增量 **+1430 B**、分配 413,863 次、峰值活字节 **262 KB**、进程峰值 RSS **5.62 MB**；combo 2 局 925 步 → 活字节增量 **+57.2 KB**、峰值活字节 **592 KB**、峰值 RSS **7.90 MB**。**未观察到随局数异常累积**（活字节增量在百 KB 量级、与更高内容量同向）。
- **扩展代价方向**（同机同负载方向性观察，非收益）：目录越大 → 建局越贵（124→261 张牌 p50 4.75→76.3 µs）；技能实例越多 → clone/save 越贵（0→36 个实例 clone p50 0.29→1.29 µs）；`restore` 对扩展目录重建敏感（4.96→67.4 µs）。

## 5 失败记录（环境失败与测量失败分开）

### 5.1 测量失败 1：swd 单扩展的**读档被拒**（30/30 样本）

```
cargo run --offline --release -p engine-native --example nc12_bench -- archive tier=swd seats=4 samples=30 steps=80
→ restore 失败: IdOutOfBounds { what: "card_in_multiple_zones", got: 13 }（30/30）
   诊断：card 13（sha）在**弃牌堆出现 2 次**（counted=[discard=2]），parks=0
```

- 边界：`steps=5` 通过；`steps=20` 与 `steps=40` 均 **3/3 失败**；`none`/`huanle`/`combo` 同条件下全部通过（[11-restore-failure-matrix.log](11-restore-failure-matrix.log)，§3 表 H2）。
- 判定：`Engine::save()` → `encode_archive` → `decode_archive` 都能成功，**只有 `restore` 拒绝**，即引擎允许的活状态被自己的所有权校验（`snap/validation.rs` 的 `mark()` 对同一牌 id 只允许出现一次）判为非法。**这是测量失败而非环境失败**；修复前 swd 档「存读档字节大小」可得，**「读档耗时」不可得**。
- 未做：根因定位（哪条 swd 内容路径把同一张牌二次置入弃牌堆）与最小复现用例——本轮只测量，不改产品源码。

### 5.2 测量失败 2：规划内的宿主断言 panic（扩展档 + variant=1）

| 现场 | 消息 | 触发 | 路径 |
|---|---|---|---|
| `crates/engine-native-kernel/src/machine.rs:408:32`（`take_hand_card`） | `card 90 not in hand of 0` | combo，seed 42007，budget 64，variant=1，**step 5 seat 1**（该步有开放选择、`parks=1`） | `plan_action` 克隆内的模拟 `submit`（backtrace 见 [09-first-failure-panic.log](09-first-failure-panic.log)） |
| `crates/engine-native-kernel/src/kernel/monkey.rs:73:26` | `被拦截的桃应在使用者手牌` | combo，seed 42007，budget 64，variant=1，**step 236 seat 0**；huanle 单档同类 | 同上（模拟内） |

- 证据：[09-first-failure-panic.log](09-first-failure-panic.log)（不捕获，`RUST_BACKTRACE=full`，exit 101 + 完整栈）。
- 现象：**AI 模拟候选能触发宿主断言 panic，进程直接中止**（不是「拒绝」）。这一路径把 3 个 drive 组打成部分覆盖：swd@budget64/256 variant1、huanle@budget8 variant1（[04-drive.log](04-drive.log)）。
- 已复现的边界（[10-panic-boundary-matrix.log](10-panic-boundary-matrix.log)）：combo variant=1 在 budget=64/256 与 lookahead=2 下 **seed 42007 崩溃、seed 42008 不崩**；budget=8（候选更早耗尽）不崩；variant=0 不崩；`none` 档在所有预算/变体下均不崩。
- 根因**未验证**（本轮只测量）。可复现的观察：崩溃步都发生在**规划克隆上**，而 `plan_action` 会对克隆执行 `redact_opponent_hands`（未知牌确定性重分配）——重分配后，此前挂起机器/选择规格里记录的实体牌 id 可能与新牌区不一致。该假设需要产品侧单测确认，本页不当作结论。

### 5.3 测量失败 3：swd 单扩展的**权威结算路径 panic**（比 5.2 更严重）

```
cargo run --offline --release -p engine-native --example nc12_bench -- drive tier=swd seats=4 budget=64 lookahead=1 variant=1 games=3 seed=42007 catch_panics=1
→ 第 3 局（seed 42009）step 68 权威提交时 panic：
   machine.rs:408:32 "card 204 not in hand of 0"
   backtrace: plan_action? 否 —— Engine::submit → finish_segment → kernel::commit_frame
```

- 证据：[09b-first-failure-swd-real-trajectory-panic.log](09b-first-failure-swd-real-trajectory-panic.log)（`RUST_BACKTRACE=1` + 逐步 TRACE；`PANIC caught` 计数为 **0**，说明不是被捕获的模拟内 panic，而是 `apply_plan` 的权威提交）。
- 影响：swd 档 variant=1 的**干净长局基线不可得**（该组在 swd@64/256 两轮均中止）；同时它是产品侧健壮性问题（规则内核在扩展内容 + AI 选择下可达断言）。
- 未做：根因与最小复现（只测量）。

### 5.4 测量失败 4：huanle 单扩展 variant=1 **不收敛**

- `huanle` + variant=1 + budget=64/256：3 局共 8210 步，**仅 1 局终局**，2 局跑满 `max_steps=4000` 仍未结束（`step_capped=true`，无 panic、无拒绝）。对应的 4000 步步长使该组 wall/步 ≈ 0.40 ms、plan max 达 12 ms 量级——**这是「未收敛」的产物，不是稳态延迟**，不参与延迟比较。
- 结论：huanle 档 variant=1 的整局耗时基线**不成立**；variant=0 与其它档正常收敛（3/3）。

### 5.5 环境失败

- 无包导入/桥接失败；41 个目录全部导入成功（`skipped=0`）。
- 进程内 RSS 采样失败：沙箱禁止 `ps`（`Operation not permitted`），已在工具内降级为 `rss_kb=NA` 并改用计数分配器 + 外部 `/usr/bin/time -l`（§2.2）。这是**环境限制**，不是被测代码问题。

## 6 与旧线的关系

本轮**未测**旧线（`engine-core`/`engine-ai`/`engine-wasm`），也未测上游原版。因此：

- 不写任何提速倍数、不写「原生比旧线快 N 倍」；
- 旧线历史数字见[测试与性能证据](../../phase17-test-perf-infrastructure.md)与[原生基准](../../phase19-native-baseline.md)，其负载/构建/计数口径与本页不同，**不可相除**。

## 7 复跑

```bash
cd noname-next
cargo build --offline --release -p engine-native --example nc12_bench
bash docs/operations/evidence/2026-10-18-nc12-baseline/run-all.sh     # 复跑全部 13 个日志（串行，含 load 记录）
python3 -B docs/operations/evidence/2026-10-18-nc12-baseline/extract.py          # 由日志重生成结果表
python3 -B docs/operations/evidence/2026-10-18-nc12-baseline/extract.py --check  # 日志一致性（命令数/非零退出）
```

单独复现某一项（`tier ∈ {none,swd,huanle,combo}`）：

```bash
B=./target/release/examples/nc12_bench
$B probe samples=5
$B build tiers=swd samples=30 seats=4 seed=42007
$B drive tier=none seats=4 budget=64 lookahead=1 variant=0 games=3 seed=42007
$B copy  tier=swd budget=64 steps=60 samples=30 growth_every=20
$B archive tier=swd samples=30 steps=80
$B ai tier=none budgets=8,64,256 configs=1/0,1/1,2/0,2/1 cmds=200 seed=42007
/usr/bin/time -l $B long tier=none seats=4 budget=8 min_cmds=800 max_games=4 sample_every=25 alloc=1
# 崩溃现场（首次失败原样保留）：
RUST_BACKTRACE=full $B drive tier=combo seats=4 budget=64 lookahead=1 variant=1 seed=42007 trace=1
RUST_BACKTRACE=1    $B drive tier=swd   seats=4 budget=64 lookahead=1 variant=1 games=3 seed=42007 catch_panics=1 trace=1
```

注：`long` 的完整参数见 [run-all.sh](run-all.sh)（`budget=8 min_cmds=800 max_games=4 sample_every=25 alloc=1`）。

## 8 限制

1. **并发负载**：整套测量在 load average **36.85–44.65** 下完成（14 逻辑核；同机其他 agent 并行构建/测试）。p95/max 明显受污染，**p50 才是本次可用的比较量**。所有日志逐条记录了 `load_before`/`load_after`。
2. **单设备单快照**：结论只适用于本机 macOS arm64 + 上述 HEAD/二进制；工作区有 261 项他人未提交改动，源码可能在测量后继续变化。
3. **样本规模**：微基准 30 样本，整局每组 3 局。扩展档部分组因 panic/未收敛只取得部分覆盖（§5）。
4. **降级轮不可混算**：`catch_panics=1` 的组（表 C 标「兜底」）走的是兜底动作续行，轨迹与干净轮不同。
5. **内存口径**：计数分配器只统计打开计数后的窗口；窗口前存量的释放在窗口内会让「活字节增量」偏小，故同时给出原始分配/释放字节。峰值 RSS 由外部 `/usr/bin/time -l` 给出（含进程启动与导入 41 个包的一次性驻留）。
6. **未测量项**见 §2.3；这些档位不得用本页数字外推。

## 9 证据文件

| 文件 | 内容 |
|---|---|
| [01-env.txt](01-env.txt) | 设备/系统/工具链/HEAD/dirty/哈希/load（run-all.sh 自动生成） |
| [02-probe.log](02-probe.log) | 三档装载与目录规模、单次规划冒烟 |
| [03-build.log](03-build.log) | 4 档建局 30 样本 |
| [04-drive.log](04-drive.log) | 干净轮整局驱动 24 组（含 3 组 panic 中止的首次失败记录） |
| [04b-drive-degraded-variant1.log](04b-drive-degraded-variant1.log) | variant=1 兜底轮 12 组（补齐覆盖，标注 degraded） |
| [05-copy.log](05-copy.log) | save/clone/plan/encode 与状态规模增长 |
| [06-archive.log](06-archive.log) | 存读档 4 档 × 30 样本（含 swd 读档拒绝诊断） |
| [07-ai.log](07-ai.log) | AI 策略矩阵（none 档 3 预算 × 4 配置） |
| [07b-ai-combo-degraded.log](07b-ai-combo-degraded.log) | AI 策略矩阵（combo 档兜底轮） |
| [08-long.log](08-long.log) | 长局内存（none 4 局 / combo 2 局，含 `/usr/bin/time -l`） |
| [09-first-failure-panic.log](09-first-failure-panic.log) | **首次失败**：combo variant1 规划内 panic + 完整 backtrace |
| [09b-first-failure-swd-real-trajectory-panic.log](09b-first-failure-swd-real-trajectory-panic.log) | **首次失败**：swd variant1 权威结算 panic + backtrace + 逐步 TRACE |
| [10-panic-boundary-matrix.log](10-panic-boundary-matrix.log) | 崩溃边界矩阵 |
| [11-restore-failure-matrix.log](11-restore-failure-matrix.log) | 读档失败边界（4 档 × 3 步数） |
| [run-all.sh](run-all.sh) | 一键复跑脚本（串行 + load 记录 + `/usr/bin/time -l`） |
| [extract.py](extract.py) | 由原始日志重生成结果表 / 一致性检查 |
| [SHA256SUMS](SHA256SUMS) | 本目录全部证据文件校验和 |

校验：`cd docs/operations/evidence/2026-10-18-nc12-baseline && shasum -a 256 -c SHA256SUMS`
