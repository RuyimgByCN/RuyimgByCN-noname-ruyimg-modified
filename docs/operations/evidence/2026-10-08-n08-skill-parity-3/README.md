# P2/N08 逐技能语义对照批三：失去体力原语（规则 v77，2026-10-08）

主稿 P2/N08「逐技能语义对照」第三批：**类级修复**——新增引擎
「失去体力」原语，消除 G1 四处「失去体力简化为伤害」登记差异。前置：
[逐技对照批二](../2026-10-08-n08-skill-parity-2/README.md)。

## 上游语义核验（源码为准）

上游 `loseHp`：直接扣减体力——不经护甲（护甲只抵伤害）、无伤害前置
窗口、无铁索连环传导、伤害来源触发技（遗计/反馈类监听 damage 事件）
不响应；体力 ≤0 同样触发濒死。原生此前无此原语，四处 G1 卡以 1 点
普通伤害近似，连带错误地获得护甲抵扣/铁索传导/伤害触发技响应面。

## 改动（分开陈述）

- **代码已实现（原语）**：`EventData::LoseHp { player, amount }` +
  `EventKind::LoseHp` + `LogEvent::HpLost`（表现层 HpLost 事件/文本
  双投影）。`commit_lose_hp`：扣体力→日志→濒死检查（source=None）；
  独立事件种天然不进伤害监听。接线：帧数据 enc/dec（tag 16 表尾追加）、
  事件种 enc 18 / dec 18（**顺修既有缺口**：dec_event_kind 缺
  TurnOverBefore=16/TurnOvered=17 两臂而 enc 侧早已在写）、
  canonical_hash（tag 22）、快照边界检查、intrinsic_machine None 臂。
- **代码已实现（发射点替换×4）**：面里金针（摸 1 后失去 1 点体力）、
  飞镖（取消分支）、瘟疫（无手牌分支）、冰魄通（同色分支）——
  Damage → LoseHp；源码注释登记差异同步注记「规则 v77 原语」。
- **保留登记**：草船借箭/舌战群儒等「失去体力」与其他大差异（自选
  展示/多轮问答链）捆绑的条目——单点替换会半对齐，不属本批；
  随机类登记（瘟疫弃首张/乾坤镖/麻婆豆腐等「随机改确定性」）保留
  （引擎无熵源，确定性近似策略，见覆盖清单行为差异表）。

## 验收口径

- **回归已加入/更新**：`g1_shahai_gujian`（面里金针/飞镖断言改失去
  体力口径 + **「无 DamageDealt 日志 + 有 HpLost 日志」守卫**）、
  `g1_shahai_gwent`（瘟疫同口径）。定向 13/0 + 6/0。
- **全量通过**：engine-native **382 passed / 0 failed / 1 ignored**
  （唯一 ignore=写轮眼规模诊断探针），单线程
  `cargo test --offline -p engine-native -- --test-threads=1`；
  依赖包 content-runtime + engine-ai + content-native-loader
  **62 passed / 0 failed**，DEPS_EXIT=0。明细见 [test-logs.txt](test-logs.txt)。
- **golden**：版本 76→77 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（无场景触及四卡，纯版本位移）。
- **上游等价边界**：本批消除「失去体力简化为伤害」类（四处）；
  随机类与捆绑条目仍登记；不构成 G1 整包规则等价证明。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（15 文件，相对 noname-next 根）。
