# 原生保存成本基线（2026-10-24，P2/NC12 第一段）

核验基准 `7e28a93799a9c01794085e911a72a4faa5673883` 加未提交工作树；规则 **v322**、存档 **v105**。本批只测 **engine-native 的 `save()` + archive 编码成本**；不含服务端 WAL／收据／多房间负载，不能当产品 SLA 或全量 NC12 验收。

## 方法

新增 `crates/engine-native/examples/save_cost.rs`：

- 同一设备、release 构建、4 人标准局、seed `20261028`。
- 参数：`players`、`steps`（默认 4、1200）；本批跑 **4 人**与 **8 人**各 1200 步。有选择时取前 `min` 个候选应答，否则当前座位 `NextPhase`。
- 每 100 步做 5 次 `Engine::save()` + `archive::encode_archive`，取中位数；记录编码字节数。
- 输出只含耗时/体积，不做平台外推。

命令：

```bash
cargo run --offline --release -p engine-native --example save_cost
```

## 结果

见[完整输出](save-cost.log)：

4 人（[日志](save-cost-4p.log)）：

```text
RESULT players=4 steps=1200 samples=12 save_p50_us=2.7 save_p95_us=3.9 archive_bytes_max=1888 outcome=None
```

8 人（[日志](save-cost-8p.log)）：

```text
RESULT players=8 steps=1200 samples=12 save_p50_us=4.0 save_p95_us=5.2 archive_bytes_max=2880 outcome=None
```

- 4 人 archive 恒 **1888B**；8 人恒 **2880B**；1200 步内无增长。
- 本机 release 下 4 人 P50 2.7µs/P95 3.9µs；8 人 P50 4.0µs/P95 5.2µs。
- 样本点逐项见日志（step 0…1100）。

## 边界

- **不是服务端持久化**：未测 WAL 追加、收据增长、fsync、多房间并发或 SIGKILL 后恢复成本。
- **不是长局状态增长**：标准 4 人局 1200 步的状态/存档体积基本恒定；含内容私有状态、标记/技能历史、录像或大包时需重测。
- 没有 before/after 优化对照（本批未改保存实现）；不得把数字当收益或 SLA。
- 未改规则/存档版本、golden 或人工 DSL。
