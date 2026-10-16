# O5 条件立项剖析收口（2026-10-16，D8）

## 任务来源与口径

[进度主稿 O5](../../../architecture/native-progress.md) + [D8](../2026-10-16-d4-d8-decisions/README.md)：保持条件立项；用 benchmark-runner 做一轮剖析——**无触发则记录结论并关闭**；有触发则最小改动且不退化。

候选触发面（来自[优化假设复核](../../phase17-test-perf-infrastructure.md#2026-09-24-优化假设复核)）：JSON 快照/回放、动态派发、大文件拆分、Blake3 工具合并、目录重命名。

环境（detached worktree `noname-next-wt-perf`，独立 target）：git `8c7a96146d417a9060be27122ccc9446006d65d3`；rustc 1.95.0 release；`cargo run --release --offline -p benchmark-runner -- all`。

## 剖析结果（对照 O3 基线）

完整场景行见 [baseline-all.txt](baseline-all.txt)。与 [2026-10-14 O3](../2026-10-14-o3-runtime-baseline/README.md) 同口径摘要：

| 指标 | 本批 | O3（2026-10-14） | 是否指向 O5 候选 |
|---|---|---|---|
| query-cache 命中率 | **100%**（cold 101ns / warm 71ns） | 100%（75/66ns） | 否 |
| distance-query | 9 allocs / 250B，249 ns | 9 / 250B，240 ns | 否 |
| trigger-dispatch | **135 allocs / 24.9KB**，p50 44µs | 135 / 24.9KB，p50 43µs | 否（主通道校验保留；非 JSON/枚举派发证据） |
| observer-projection | 扇出 14，133 allocs / 15KB，21µs | 同 | 否（仍登记 T13 帧率后复核，非本项改造） |
| snapshot-restore | 49µs | 47µs | 否（未显示 JSON 编解码为瓶颈） |
| AI 节点成本 | ≈130µs/node | ≈120µs/node | 否（有界；无格式/目录触发） |
| identity-match | p50 3ms/步 | p50 3ms/步 | 否 |

## 触发判定

- **JSON 快照/回放格式**：snapshot-restore 仍在数十微秒量级，剖析未指向格式更换收益；**不立项**。
- **动态派发（trait object→大枚举等）**：无独立派发热点证据；trigger/projection 分配与 O3 同阶；**不立项**。
- **大文件拆分 / Blake3 工具合并 / 目录重命名**：本轮无维护缺陷或剖析命中；**不立项**。

## 关闭结论

无 O5 触发条件；**零代码改动**。条件立项策略保留作日后入口：仅当新剖析或维护缺陷明确指向上述面时再建最小改动，并保持 API/测试/指标不退化。

**O5 关闭**。
