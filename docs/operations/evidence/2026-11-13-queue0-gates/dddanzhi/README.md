# 2026-11-13：队列 0 暗织修复与定向证据

核验基点为 `3a6e398485db6ef0729abe3013c3c4b7705a5d6d`，工作区 dirty。本目录只记录队列 0 的 `dddanzhi`（上游译名「暗织」，translate.js:123）实现、回归与已观察到的边界；不代替主 agent 的完整门禁及独立复核。规则 333／NGSA 107／content schema 5／ABI 759／WS8；本批不改版本、协议、存档结构或触发元数据。

## 实际命令与原始结果

以下每轮均执行同一条定向命令，未冒充无过滤三 crate 验收：

```bash
CARGO_INCREMENTAL=0 cargo test --offline -p engine-native --test g1 u01_dddanzhi_regression -- --nocapture
```

前两轮另设 `DDDANZHI_CAPTURE_DIR` 为本目录 `before/` 的绝对路径，用于在生产修改前导出真实选择挂起的旧 NGSA。每轮的 `*-command.json` 保存实际 argv、环境变量；原始 stdout/stderr 完整保留在对应 `.log`，未就地覆盖历史失败。

| 原始日志 | 通过／失败／忽略 | 具体结果 |
| --- | --- | --- |
| [red.log](red.log) | 0／5／0 | 旧实现首跑；含回合布阵与重复出杀的夹具问题。 |
| [red-fixture-corrected.log](red-fixture-corrected.log) | 0／5／0 | 纠正夹具后，生产仍未改；五条真实反例全红。 |
| [green.log](green.log) | 7／2／0 | AOE 传目标的夹具不合法；真实五谷挂起缺少机器持牌归属。 |
| [green-rerun.log](green-rerun.log) | 8／2／0 | 普通五谷及黑牌转五谷的开放选择恢复均出现 `OpenChoiceMismatch`。 |
| [green-final-attempt.log](green-final-attempt.log) | 12／6／0 | 扩充后的第一轮；暴露探针时机、可取消追加未应答、原生技能名与无杀不提问等夹具问题。 |
| [green-final-rerun.log](green-final-rerun.log) | 17／3／0 | 追加、装备、多目标决斗历史等已通过；仍见不合法凤泽输入、无杀不提问断言、以及下述真实 CardId 0 既有缺陷。 |
| [green-final-fixtures.log](green-final-fixtures.log) | 19／1／0 | 剩余桃园实体未弃置：回血发出 Recover 子帧后才读取当前 CardUsed，已读不到原载体。 |
| [green-final.log](green-final.log) | **20／0／0** | 最终整组定向通过；另有 221 个非本模块用例被过滤。 |

最终通过后只清理两处空白：去掉 events.rs 新增的 EOF 空行，撤销 equipment.rs 对既有 L28 代码的纯格式变化。首份 [frozen-scope.json](frozen-scope.json)、[source-changes.patch](source-changes.patch) 与首次 [diff-check.log](diff-check.log) 均保留。最终文件哈希、物理／有效行数和精确所有权见 [final-frozen-scope.json](final-frozen-scope.json)，补丁见 [final-source-changes.patch](final-source-changes.patch)；[final-diff-check-command.json](final-diff-check-command.json) 的 `git diff --check` 退出 0。

代码／测试／fixture 已冻结交由主 agent 执行完整门禁。未提交、未推送；未创建或清理共享 target，未创建私有 Cargo 缓存。独立复核及上游实际运行对照不由本实现者作完成判定。

## 最终源码改动

所有权共 13 个生产文件，另有测试入口、独立测试和旧档 fixture；完整路径及 SHA256 见最终清单。

| 文件 | 实际改动 |
| --- | --- |
| `engine-native-content/src/content/host/step/u01_batch_d_asks.rs` | 删除「或弃牌堆非空」条件，只读取末牌色。 |
| `engine-native-content/src/content/host/step/u01_batch_d.rs` | 只给选中目标授予一次免疫；旧 CardUsed 追色分支只结束，保留注册以维持内容锁并避免嵌套后回写旧颜色。 |
| `engine-native-kernel/src/kernel/events.rs` | 真实 CardUsed／CardResponded 开始执行时共用记录末牌色；不按帧创建编号排序。共用目标移除／免疫消费助手；内联即时效果保留耗牌、历史和计数。 |
| `engine-native-kernel/src/kernel.rs` | 导出上述末牌色助手给真实装备 PlayCard 入口复用。 |
| `engine-native/src/engine/play_card.rs` | 真实装备使用在前置窗口之前记录历史，并在既有 VmAux 中标明本次 Equipped 属于实际使用；直接移牌不据此冒充使用。 |
| `engine-native-kernel/src/kernel/equipment.rs` | 仅实际使用的黑装备消费自身免疫；新装备弃置，保留旧装备与技能，帧直达 Done，不发伪装备后置效果。 |
| `engine-native-kernel/src/kernel/intrinsic.rs` | 在 AfterTriggers 追加目标完成后、机器固有效果开始前，按最终目标移除免疫者；整牌无目标时仍实际耗牌。 |
| `engine-native-kernel/src/kernel/events/turn.rs` | phaseEnd 内保留保护；phaseEnd Commit 发出 phaseAfter 帧前清两旗，早于 phaseAfter 的 BeforeTriggers。其他回合清理不动。 |
| `engine-native-kernel/src/kernel/tricks.rs` | 只有本次暗织实际移除过目标，五谷／桃园才按剩余名单过滤；未消费免疫的虚拟子集保留旧全局语义。精确收尾当前五谷／桃园的实际载体；桃园在发 Recover 子帧前捕获载体。 |
| `engine-native-kernel/src/resolution.rs` | 保留装备提交主动到 Done；收尾撤销本帧临时标记，涵盖无懈取消未进入群体机器的路径。 |
| `engine-native-kernel/src/snap/validation.rs` | 机器持牌枚举纳入既有 Wugu.cards。 |
| `engine-native-kernel/src/snap/asks.rs` | 由既有 Wugu.cards／pickers 重建实际 `wugu_pick` 询问。 |
| `engine-native-kernel/src/snap/restore.rs` | 用既有 Wugu.owner／cards／pickers 恢复 Init／Pick，并补既有 Taoyuan.owner 的恢复分派；未新增阶段字段。 |

上表文件均位于 `crates/` 下。测试位于 `crates/engine-native/tests/u01_dddanzhi_regression.rs`，已登记 `tests/groups/g1.rs`；旧档 fixture 为 `tests/fixtures/u01-dddanzhi-pending-v107.ngsa`。最大 Rust 有效代码文件为 play_card.rs 的 996 行，均保持 1,000 有效行／2,000 物理行上限。

## 真实事件回归范围

20 条 `#[test]` 通过 PlayCard、ActivateSkill、NextPhase 与 Answer 进入真实结算；debug 入口仅用于布阵／授予技能／牌区准备，不使用 `debug_execute_effect`。技能预期引用上游 `apps/core/extension/3D精选/character/skill.js:5796`。

- 弃牌非空而没有历史、末次红色使用、红／黑实际决斗响应覆盖相反的此前用牌颜色。
- 选择自己、另两名目标及取消；仅目标获保护、红牌不消费、首张黑牌无效而第二张正常、授予者失去技能仍有效。
- 真实黑色 AOE、显式多目标铁索、酒／闪电隐式自身目标；耗牌、计数和实体位置保留。
- VM 的追加目标明确回答 `rvm_choice`，并断言 `SkillTriggered`；分别验证后来追加免疫者、初始唯一免疫者后追加未保护者。原生 CardUsed 的 event_player 就是使用者（rvm_bridge/context.rs:178），故当前探针 `owner_is_target` 在这类事件真实指使用者。
- 无暗织时，虚拟五谷／桃园传入 `[self]` 仍按原机器全局范围结算，并保留 VirtualCardPlayed 原始目标日志。
- 实际红／黑装备更新末牌色；强制入装备区既不改末牌色也不消费免疫。黑装备无效时不替换旧槽、不授装备技能、不触发劫行 equipAfter 重置；下一张实际黑装备正常。
- 奇佐真实嵌套黑色使用／黑色打出不会被外层红用牌回写；实际 `wn_dutan` 多目标决斗首场黑响应、末场无色用牌按执行顺序记史，不能按帧创建编号取最大值。
- phaseEnd 内黑色副本免疫；phaseAfter 的 BeforeTriggers 中黑色副本已不免疫；下一回合未消费保护也已失效。
- 良策黑牌转五谷、无技能普通五谷的 Pick 挂起恢复，以及无懈前五谷 Init（空 cards／pickers）恢复；取消／正常结算后状态和日志与直接继续一致、临时标记清除、当前载体入弃牌。
- 现有凤泽以黑色酒转桃园，无懈挂起可恢复；只跳过受保护者，其他原生既定受益者回复，载体守恒。桃园既有仅恢复其他人的差异保留，见下文。

## 修复前旧档与内容锁

生产修改前，在真实用牌与 NextPhase 触发 `dddanzhi_target` 尚未应答时保存 [before/pending.ngsa](before/pending.ngsa)，长度 **2,149 字节**，SHA256 为：

```text
3158038e27eb46cffd15b4fc1eb4e378daa381cc9a01e16b96ef8f6656178736
```

旧档与测试 fixture 字节完全一致。原始包 JSON 为 [before/character.json](before/character.json)，建目录涉及的原始源码与哈希见 [before/source-sha256.json](before/source-sha256.json)；原两包指纹和组合锁分别见 [before/pack-fingerprints.txt](before/pack-fingerprints.txt)、[before/extension-content-lock.txt](before/extension-content-lock.txt)。

| 内容锁 | 修复前值／最终测试断言值 |
| --- | --- |
| d3-upstream-noname 单包 | `1553573260256134951` |
| decision34-pack9-closure 单包 | `9097112637251845257` |
| 两包组合 | `11413175519070831972` |

最终测试直接读取该修复前档：decode→encode 与原档逐字节相等；使用原两包恢复后 save，仅将恢复机制故意递增的 `auth_generation` 还原为旧值，编码仍与原档逐字节相等。继续真实 Answer 并出黑牌，验证选中目标免疫、未选中的使用者不获额外保护。本证明包含真实旧输入，不仅是新实现自存自读。

## 保留的差异与已观察到的未修问题

1. 暗织仍按现有 Phase::Prepare 宿主窗口提问。上游为真正的整回合 `phaseBegin`；本批保持触发元数据／内容锁，按主 agent 指示将此时机差异交队列 1。不能据本批绿色测试宣称全技能上游等价。
2. 原生桃园机器只回复使用者之外的存活角色。无免疫虚拟子集与凤泽测试都明确保留这个模型；上游完整目标模型及其他扩展特有隐式目标未在本批扩充。
3. **真实未修缺陷：Duel 的实体 CardId 0 与「无实体」哨兵冲突。** `green-final-rerun.log` 中 phaseAfter 分支使用 CardId 0（♠7 杀）作奇佐决斗载体，血量断言通过、载体入弃牌断言失败。既有 DuelMachine::finish 的 `if card != 0` 跳过该真实实体的收尾；本批没有修改 duel.rs。当前与 HEAD 的该文件逐字节相同，原始源码保存在 [duel-head-source.txt](duel-head-source.txt)，哈希与观察／推断界限见 [known-boundary-duel-card0.json](known-boundary-duel-card0.json)。未单独在纯 HEAD 运行第二份复现。最终暗织嵌套／时限测试改用非零黑色酒载体隔离此既有缺陷；这项绿色不能证明 CardId 0 问题已修。
4. 本批测试在原生路径实际执行；未在上游浏览器实触发，也不代替独立 reviewer 的规则复核。无过滤全量、golden、strict／跨目标和产品门禁由主 agent 另存结果。
