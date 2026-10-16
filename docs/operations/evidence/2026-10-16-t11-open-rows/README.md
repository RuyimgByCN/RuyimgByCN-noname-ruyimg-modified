# T11 行为差异表其余开放行清单（2026-10-16）

**核验日期**：2026-10-16　**HEAD（起草时）**：`8c7a9614`  
**范围**：覆盖清单[行为差异表](../../../native-coverage-manifest.md#行为差异迁移对齐前不得当作与旧线等价)在 2026-10-16 多批核对/定案之后，**仍开放、可领取**的残余行与子项；含 T11 完成条件中的「同一命令下事件 / 随机流 / 恢复轨迹对照」。  
**不做**：改 `crates/**`；不改进度主稿 / 两仓台账（并行 worker）；不重开已定案「保留原生」口径。

机器可读副本：[open-rows.json](./open-rows.json)。

## 快照：哪些行已不在本清单

下列行在 2026-10-16 已**核对一致**或**定案保留原生（上游登记为已知差异）**，本清单**不再开执行项**（改判须另立 ADR）：

| 类别 | 行 / 口径 | 证据 |
|---|---|---|
| 一致关闭 | AOE 目标冻结 / 无牌跳过（语义）/ 决斗起手·无杀·伤害来源；传导伤害来源 / 传导结算完整性；N06 铁索无懈逐目标；距离下限 1 / 座距只计存活 | [aoe-duel](../2026-10-16-aoe-duel-nullify-ask-semantics/README.md)、[chain](../2026-10-16-chain-propagation-seat-order/README.md)、[n06](../2026-10-16-n06-chain-per-target-nullify/README.md)、[distance](../2026-10-16-rule-source-distance-basis/README.md) |
| 口径定案（8+） | 无懈名单起点、无懈对无懈结构、传导目标序、解链时点、多目标非 multitarget 定序、判定顺序、濒死轮次、延时锦囊无懈时点、临时技能存续期 | [规则基准「语义选择」](../../../architecture/native-rules-baseline.md)、各 `2026-10-16-*` 证据 |
| 决策收口 | 触发全序（保留确定性全序）、死亡清技能（死亡即删实例）、奖惩主序（保留奖惩先于终局） | [trigger-order](../2026-10-16-trigger-order-upstream/README.md)、[death-reward](../2026-10-16-death-reward-order-and-skill-clearing/README.md) |

进度主稿「窗口/顺序口径定案」边界原文：定案只覆盖上述口径；**其余行（传导复合边界、改判与天妒扩面等）仍按批次验收**——本清单即其可执行展开。

## 开放行（按优先级）

优先级约定：`P0`=可直接审计/补测且阻塞口径关闭；`P1`=需小专项实现或扩面夹具；`P2`=条件立项 / 模式变体 / 长期横切。

| 优先 | ID | 主题 | 基准 / 覆盖清单锚点 | 已有证据（勿重复勘察） | 缺口（实现 / 测试 / 对照轨迹） | 建议批次 | D4 交叉 |
|---|---|---|---|---|---|---|---|
| P0 | `T11-DIFF-REWARD-OVER-VERIFY` | 奖惩与终局：上游 `game.over` 后是否仍执行 `dieAfter2` | 覆盖清单「奖惩与终局判定顺序」待核项；todo `COV-REWARD-OVER-VERIFY` | [death-reward](../2026-10-16-death-reward-order-and-skill-clearing/README.md) | **审计**：上游差分/探针确认 over 后奖惩是否发放 → 关闭待核或改登记；无引擎改动预期 | B-audit-1（规则审计半日） | 无 |
| P0 | `T11-DIFF-JUDGE-REWRITE-EXPAND` | 改判与天妒：新增判定技 / 复合路径扩面 | 覆盖清单「改判与天妒切片」；N08-c/d 已关 | [trigger-order](../2026-10-16-trigger-order-upstream/README.md)（全序背景）；既有 `judgement_event_order_rewrite_then_tiandu` / `pause_roundtrip_rewrite_then_tiandu_combo` | **测试+对照**：新判定技组合、Finalized 后反例、装备资格边界；命令后事件序与挂起恢复增量；随机改判路径若有须锁 RNG 消费序 | B-judge-1 | 无 |
| P0 | `T11-DIFF-CHAIN-COMPOUND` | 传导复合边界（解链已定案后的可达组合） | 进度「传导复合边界」；传导证据 §2 ①② 窗口读 `linked` | [chain-propagation](../2026-10-16-chain-propagation-seat-order/README.md) | **测试**：传导进行中读 `linked` 的技能/钩（上游仍链上、原生已解）；传导中濒死/转移；解链日志数量口径冻结。**不对齐解链时点**（已定案） | B-chain-1 | 无 |
| P1 | `T11-DIFF-EQUIP-TIMING` | 装备即时生效 vs 上游装备时机窗 | 覆盖清单「装备即时生效」；todo `COV-EQUIP-TIMING-WINDOW` | （本批无新上游摘录；沿用清单行） | **立项**：ADR 触发窗口化 **或** 正式永久差异关闭；最小复现=+1 马对结算中杀的回溯场景；若立项则版本次+golden | B-equip-1 | 无 |
| P1 | `T11-DIFF-AOE-EVENT-TRAIL` | AOE 无牌跳过：语义一致、事件轨迹不同 | 覆盖清单「AOE 无牌跳过可见性」事件轨迹登记 | [aoe-duel](../2026-10-16-aoe-duel-nullify-ask-semantics/README.md) | **对照轨迹**：原生不建立即取消的响应事件——跨线对照须用投影后事件/询问次数（已有断言）；补「同命令日志形状」说明或夹具，避免误当缺陷 | B-trail-1（夹具/文档） | 无 |
| P1 | `T11-DIFF-SELECTTARGET-MINUS1` | `selectTarget:-1` 自动目标数组构造起点未逐项确认 | AOE 询问顺序勘正残留；座位序起点族 | [chain §2–3](../2026-10-16-chain-propagation-seat-order/README.md) | **审计**：南蛮/万箭等自动目标数组起点 vs 上游；若仅当使用者=当前回合时重合则登记已知边界，否则补测 | B-audit-2 | 无 |
| P1 | `T11-DIFF-TIESUO-RECAST` | 铁索 `recastable`（重铸）面未建模 | N06 边界；覆盖清单铁索行 | [n06](../2026-10-16-n06-chain-per-target-nullify/README.md) | **实现或正式 deferred**：重铸入口/弃置与摸牌；或 ADR 登记不落 | B-tiesuo-recast | 无 |
| P2 | `T11-DIFF-DUEL-MIDDEATH` | 决斗中途一方死亡（来源字段）不可达边界 | aoe-duel §C | [aoe-duel](../2026-10-16-aoe-duel-nullify-ask-semantics/README.md) | **登记维持**：当前技能集不可达；有致死响应路径内容时再开夹具 | 条件 | 无 |
| P2 | `T11-DIFF-TRIGGER-GETINDEX` | `getIndex` / `firstDo` / `lastDo` 桶 | 覆盖清单「触发全序细节」；todo `COV-TRIGGER-GETINDEX-BUCKETS` | [trigger-order](../2026-10-16-trigger-order-upstream/README.md) | **条件立项**：有内容需求再建模；否则保持开放登记 | 内容驱动 | 无 |
| P2 | `T11-DIFF-VERSUS-REVERSEORDER` | versus 结算方向选择（`reverseOrder`） | aoe-duel §C；规则基准模式余项 | [aoe-duel](../2026-10-16-aoe-duel-nullify-ask-semantics/README.md)；todo `RULE-MODE-REMAINDERS` | **模式变体**：不属标准差异；随 versus 深化批 | 模式批 | 非 D4（versus，非 Boss 关卡）；勿与 Boss 子系统混开 |
| P1 | `T11-EVENT-RNG-RESTORE-PARITY` | 同一命令下事件 / 随机流 / 恢复轨迹对照（横切） | 进度 T11 完成条件末段；todo `T11-EVENT-RNG-RESTORE-PARITY` | ADR 0727；`replay_and_restore_per_command_parity`；边界 `restore_at_every_choice_point_*` / 多条 `pause_roundtrip_*` | 见下方专节：扩场景夹具 + RNG 序断言 + 非 Prototype 模式中间恢复 | B-parity-1（长期，可切片） | 无（横切） |

### 与 D4 Boss 流程面的交叉（等 D4，不重复开行为差异项）

下列在 [todo-register](../2026-10-16-todo-register/README.md) / [D4–D8 决定](../2026-10-16-d4-d8-decisions/README.md) 已挂 **T11 Boss 关卡子系统**，**不是**行为差异表残余行；本清单只标注依赖，避免双开工单：

| ID（既有） | 主题 | 处置 |
|---|---|---|
| `T11-BOSS-STAGE-SUBSYSTEM` | Boss 关卡流程子系统 + ADR | **等 D4** |
| `T11-BOSS-HUANREN` / `HIDECHAIN-8` / `WING-ROTATION` / `BIANSHEN-CHANGEBOSS` | 还人 / 隐藏链 / 三翼 / bianshen 换 Boss | **等 D4**（依赖上项） |
| `T11-BOSS-ZHUGE-LIANG` / `PANGTONG-EXTERNAL` | 诸葛亮 ylyg* / 庞统外部包技 | **等 D5**（非行为差异表） |
| `T11-BOSS-DIZANG-DAYUAN` / `MENGPO-WANGSHI-PLAY` / `SHIKIEIKI-EXCLUDED` | Boss 内容零散余项 / 排除 | 内容批；与差异表无直接锚点 |

`RULE-MODE-REMAINDERS` 中 Boss `loopType==2` 等：**等 D4**；斗地主 online / KOF 宿主循环另轨。

## 计数

| 口径 | 数量 |
|---|---|
| 本清单开放执行行（上表含横切） | **11** |
| 其中与 D4/D5「等裁决执行」交叉、本清单不重开 | **D4 族 5 + D5 族 2**（见上表，另计） |
| 链到既有 2026-10-16 上游语义证据目录（避免重复勘察） | **≥10** 目录（aoe-duel、chain、death-reward、delayed-nullify、dying-save、judge-order、trigger-order、temp-skill、rule-source-distance、n06） |

---

## 专节：同一命令下的事件、随机流和恢复轨迹对照

### 已有基线（勿重做）

| 面 | 已有夹具 / ADR | 覆盖边界 |
|---|---|---|
| 逐命令哈希 + 全量日志 | `replay_and_restore_per_command_parity`（[ADR 0727](../../../adr/0727-native-n06-replay-restore-per-command-parity.md)） | Prototype、seed 307、约 60 命令；半程一次恢复 |
| 选择点恢复 | `restore_at_every_choice_point_continues_identically`；多条 `pause_roundtrip_*`（杀/濒死/决斗/AOE/判定/铁索窗/鬼道/天妒组合等） | 挂起窗口；非任意命令中点 |
| 身份终局逐提交 | `p2_identity_endgame_per_submission_parity` | 身份终局路径切片 |
| RNG 入存档可复放 | 规则基准随机口径；v78 等批均匀抽取 | 标准路径；扩展技 RNG 仍逐技 |

### 仍缺的场景 / 夹具（最小集）

1. **传导复合命令串**：连环伤害 → 传导 →（可选）濒死/救援 → 解链日志；逐命令比对 `state_hash` + 事件增量（代号 `parity_chain_compound`）。
2. **改判→天妒→再判定技** 扩面：在既有 `judgement_event_order_rewrite_then_tiandu` 外，加「中途 save/restore」与「RNG 改判消费位数」断言（代号 `parity_judge_rewrite_rng`）。
3. **AOE 无牌跳过轨迹**：同脚本下原生日志无「立即取消响应事件」——夹具显式允许缺省形状，只锁询问次数与终态（代号 `parity_aoe_skip_trail`）。
4. **装备时机对照脚本**（若 `T11-DIFF-EQUIP-TIMING` 立项）：结算中装备 ±马 前后距离/是否可出杀；上游轨迹采集另轨（代号 `parity_equip_timing`）。
5. **非 Prototype 中间恢复**：Identity / Versus / BossStage 各至少一条「命令中点 save→restore→续跑」与基准逐命令哈希一致（代号 `parity_mode_midrestore`；BossStage 深路径 **等 D4**）。
6. **随机流位数**：对含 `rng.gen_range` 的标准技路径，断言同 seed 下 `rng` 调用次数与首个分歧命令索引（可挂在既有定向测，不必新全量）。

### 最小复现命令草案（不跑全量；可选用独立 `CARGO_TARGET_DIR`）

```bash
# 0) 存在性（--list，秒级；勿占共享 target 全量）
CARGO_TARGET_DIR=/tmp/nn-t11-parity-check \
  cargo test --offline -p engine-native --test native_kernel \
  replay_and_restore_per_command_parity -- --list

CARGO_TARGET_DIR=/tmp/nn-t11-parity-check \
  cargo test --offline -p engine-native --test boundary_snapshot \
  restore_at_every_choice_point_continues_identically -- --list

# 1) 定向短跑（单测；仍建议独立 target）
CARGO_TARGET_DIR=/tmp/nn-t11-parity-check \
  cargo test --offline -p engine-native --test native_kernel \
  replay_and_restore_per_command_parity -- --exact --nocapture

CARGO_TARGET_DIR=/tmp/nn-t11-parity-check \
  cargo test --offline -p engine-native --test native_kernel \
  'judgement_event_order_rewrite_then_tiandu|chain_propagation_preserves_source|propagated_damage_is_full_resolution|savage_attack_response_skip_and_pass|duel_no_sha_immediate_loss|last_rebel_reward_precedes_victory' \
  -- --nocapture

CARGO_TARGET_DIR=/tmp/nn-t11-parity-check \
  cargo test --offline -p engine-native --test boundary_snapshot \
  'pause_roundtrip_rewrite_then_tiandu_combo|pause_roundtrip_chain_target_window|pause_roundtrip_aoe_mid_queue' \
  -- --nocapture

# 2) 清单联动（短）
CARGO_TARGET_DIR=/tmp/nn-t11-parity-check \
  cargo test --offline -p engine-native --test replay_scenarios -- --nocapture

# 用完可删：rm -rf /tmp/nn-t11-parity-check
```

**禁止**：在共享 `target/` 上 `cargo clean` 或无独立目录跑 engine-native 全量；Gate 1b 若需要同样独立 `CARGO_TARGET_DIR`。

### 完成判定（横切项）

- 上表缺夹具 1–3 至少落地定向测 + 证据目录；4–5 按对应差异行立项后补。
- 语义变化 → 递增 `STANDARD_RULESET_VERSION` 并按 AGENTS.md 重录 14 golden；纯夹具/断言冻结可不升版本。
- 跨线对照一律**投影后语义**（可见状态 / 事件轨迹 / 终局），不追求与旧线原始哈希等价（见 death-reward 决策）。

---

## 建议引擎侧首批（3–5 项）

1. **`T11-DIFF-REWARD-OVER-VERIFY`** — 纯上游审计，最快关闭待核。  
2. **`T11-DIFF-CHAIN-COMPOUND`** — 解链定案后的可达组合冻结，防回归误改。  
3. **`T11-DIFF-JUDGE-REWRITE-EXPAND`** — 改判/天妒扩面 + 恢复往返。  
4. **`T11-EVENT-RNG-RESTORE-PARITY` 切片**：先做 `parity_chain_compound` + `parity_aoe_skip_trail` 两夹具。  
5. **`T11-DIFF-EQUIP-TIMING`** — ADR 立项或永久差异二选一（避免长期悬挂）。

Boss 关卡 / 隐藏链 / 还人等：**等 D4 子系统 ADR**，不要塞进首批行为差异修复。
