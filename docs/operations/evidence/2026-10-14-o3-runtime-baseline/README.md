# O3 旧线运行时基线测量（2026-10-14）

## 任务来源与口径

[进度主稿 O3](../../../architecture/native-progress.md)：复用 benchmark-runner 记录 query 命中率、分配/耗时、project_observer 扇出、AI 节点成本；只对已证明的热点局部优化；**不增加跳过校验的 submit_sim**。

环境：git `3cdacc46` 基线 + 本批 benchmark 扩展；rustc 1.95.0 release；本机 Apple Silicon。benchmark-runner 扩展（apps/benchmark-runner/src/main.rs）：计数分配器（GlobalAlloc 包装，进程内 alloc/bytes 计数）+ `query-cache` 场景（QueryContext::with_cache + cache_hits）+ `ai-nodes` 场景（engine-ai BeamSearch::select on StateFork）+ trigger-dispatch/distance-query/observer-projection 的 allocs_delta/bytes_delta/allocs_per_op/bytes_per_op 与 fanout_targets。

## 基线（[baseline-all.txt](baseline-all.txt)，release，完整 all 场景）

| 指标 | 数值 |
|---|---|
| query 缓存命中率（distance 同键 100k 查询） | **100%**（cold 75ns / warm 66ns）——缓存本身有效 |
| distance-query 分配 | 9 allocs / 250B 每查询（240ns） |
| trigger-dispatch 分配 | **135 allocs / 24.9KB 每提交**（p50 43µs；2 events/提交） |
| observer-projection | 扇出 **14 目标/次**（8 人局 seats+known_cards）、133 allocs / 15KB 每投影（20µs） |
| snapshot-restore | 47µs/次 |
| AI 节点成本（engine-ai BeamSearch，10k 节点预算） | **≈120µs/node**（120ns × 1000；256k 节点/31s） |
| identity-match（4 人，AI 驱动 2000 步上限） | p50 3ms/步 |

## 热点结论（按 O3 决策规则）

- **query 命中率/距离查询**：缓存 100% 命中、240ns/查询、9 次小分配——无热点，不需优化。
- **observer 投影**：每投影 133 次分配/15KB（8 人局）——若客户端高频拉取（每帧），投影分配是可优化点（对象池/复用）；当前无产品侧高频调用证据，**不立项**，登记为产品接入（T13 帧率延迟实测）后的复核点。
- **trigger-dispatch 135 allocs/24.9KB 每提交**：命令主通道保留完整校验（不引入 submit_sim）；分配面主要为事件与状态克隆——如需优化须以同负载前后分位数可复查为前提，**本基线即“前”数据**，登记供后续对照。
- **AI 节点成本 120µs/node**：束搜索单节点含克隆+提交+评估；节点预算默认 64 时单决策 ≈8ms——符合“有界”定位，无立项动作。
- **结论：无已证明热点需要局部优化**；O3 测量闭环，基线可复查（benchmark-runner 全场景 key=value 输出+本证据目录）。
