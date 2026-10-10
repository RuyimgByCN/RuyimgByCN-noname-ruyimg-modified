# ADR 0721：N09 runner 判定负载跑通与计量口径修正

日期：2026-09-16
状态：已实施（engine-native/examples/n09_runner.rs）
关联：[0717](0717-native-runner-baseline.md)、[0720](0720-native-judgement-order-equip-qualification-v6.md)、
[phase19 基线记录](../operations/phase19-native-baseline.md)、进度文档 N09

## 背景与决定

上轮 runner（2026-09-20 记录）在判定负载首次遇到正常 `Pending(gui_dao_rewrite)` 即 panic，
整次退出 101；[0717](0717-native-runner-baseline.md) 列出的计量口径问题一并修正。

### 1. 判定负载跑通（退出 0）

- 统一 drive：NextPhase 返回的 Pending（判定阶段进入时打开的改判询问）不再是异常，
  与段内链式选择一样按策略排干；仅对真实异常（有界未收敛）panic。
- 主动负载策略（不再全 Pass）：`gui_dao_rewrite` 主动打出第一张黑牌改判；
  `tian_du_gain` Confirm；`discard_to_limit` 按 min 弃牌；`nullify_window` 一律 Pass；
  其余取首选项。FFA/身份负载在无技能局下行为不变。
- 布阵健壮性：闪电未命中会转移到下家判定区，同名锦囊滞留他区会使
  `debug_place_judge` 无牌可回收（seed 9000 的 panic 根因）。新增
  `debug_sweep_judge_zones`（全部判定区移入弃牌堆），每轮布阵前清场；
  布阵前抬体力（8）避免闪电必中导致提前终局。
- 判定专项逐局输出 `rewrites/finalized/gained` 事件计数并断言真实发生
  （本轮 10 局均为 `rewrites=6 finalized=6 gained=2`），不再以「选项存在」冒充改判负载。

### 2. 计量与统计口径（0717 第 2–6 项）

- 计时改为逐命令 `Instant` 纳秒原值：P50/P95/P99 为真实 ns，不再 as_micros 截断 ×1000；
  `wall_us` 明确为命令耗时之和（不含建局/打印）。
- 拒绝原因分布经统一 `Metrics::merge` 并入各负载 aggregate；未终局局数显式输出
  （finished=18，seed 10004/10013 为被动脚本合法长局，有界跳过非失败）。
- 判定负载 `m.events` 取日志数；三类负载统一走 merge，不再遗漏 rejected/reasons。
- 分配统计移至全部负载结束后输出（进程累计，含 runner 自身），各负载 aggregate 附
  `alloc_delta_bytes` 增量；峰值/存活内存与关闭计数器对照仍待做。
- `save` 微基准加 `std::hint::black_box` 观测屏障，补 `restore` 平均成本；
  单命令延迟本身已包含段内工作副本与检查点总成本，文档注明两者口径不同。
- 原始数据留存：`N09_SAMPLES` 环境变量指定 CSV 路径，逐命令样本
  （load,game,seq,ns）落盘；本轮 26,552 行随日志一并存档于
  [evidence/2026-09-16-native-n09](../operations/evidence/2026-09-16-native-n09/runner.log)。

### 3. 结果（规则版本 6，release，20 局 seed 9000）

- 退出 0；FFA 20/20 终局（commands=5499，P50=1917ns）；身份 18/20 收敛
  （commands=20567，P50=2042ns）；判定专项 10 局零拒绝（commands=486，P50=2083ns）。
- 不同负载间不作提速比较；旧线对照需等价负载（N06/N12），本批不构成跨目标验证。

## 影响

- runner 不再依赖选项首位是 Pass 的隐含行为；策略函数 `selection_for` 为后续
  等价负载与跨目标对照提供单一口径入口。
- `debug_sweep_judge_zones` 为负载/测试布阵 API，不改变内核语义；规则版本不变（6）。
- N09 剩余：峰值/存活内存、多目标无懈/保存恢复负载扩展、等价负载跨目标对照、设备信息补全。
