# 2026-11-13 进度核对扫描（队列第 0–2 项：13 个来源之后）

- 核对人：Claude Code；时间 2026-11-13 19:59–21:10（CST）
- HEAD：`afa72d46b`，工作区干净；规则 333／NGSA 107／content schema 5／ABI 759／WS8（未变）
- 范围：核对 `2e80ee7b3..afa72d46b` 共 15 个提交（第 0 项收拾工作区、第 1 项交互通道打样、第 2 项 13 个来源）；代码改动约 +84,000 行
- 远端 `native-convergence` 仍为 `99d9091bb`：这 15 个提交**全部未发布、无 CI**

## 实跑命令与结果

| 命令 | 结果 | 原始输出 |
| --- | --- | --- |
| `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native --no-fail-fast` | **exit 101；2420 passed / 2 failed / 6 ignored**（18 段） | [three-crates.log](three-crates.log)、[忽略清单](three-crates-ignored.txt) |
| `cargo test --offline --workspace --no-fail-fast` | **exit 101；7815 passed / 3 failed / 8 ignored**（178 段） | [workspace.log](workspace.log) |
| `python3 -B scripts/check-source-lines.py` | 通过：9362 文件 | — |
| `python3 -B scripts/check_doc_line_refs.py` | exit 0，但报告 **越界 23 处**：均在 2026-11-12 的 U06、U07、L10、L27 复核证据里，所引文件后来被拆分，行号已失效（本次新增文档不在其中） | — |
| `python3 -B scripts/check_local_links.py` | 坏链 0 | — |

三个失败（前两个在必跑的三 crate 门禁内，当前 HEAD 为红）：

- `c331_source_order_phase5::phase5_all44_source_tuples_match_the_current_native_bindings`：`all trigger-bearing HostBehavior variants must have source_order: [FzLiegong]`
- `c331_source33_integration::phase7_all_trigger_bearing_host_behaviors_have_bound_source_order`：`left: 300, right: 299`

- `game-server` 的 `native_character_only_installed_package_enters_match`（`apps/game-server/tests/native_product_path.rs:306`）：`left: 64, right: 2`。U01 的 64 个技能已全部迁为原生 host 技能，这条测试仍断言 `d3-select` 包是 2 个 host 技能加 62 个 VM 技能，`5b265bbce`（U01）之后没有同步，也没有跑全工作区测试。

前两个的原因：`87b64c732`（L20 标记补充）把 `HostBehavior::FzLiegong` 的触发元数据从 `None` 改为有触发，却没有登记选序（source_order）。其后的 `b434f0daf`（U08）、`afa72d46b`（L25）都是在门禁为红的状态下提交的。[L25 证据](../2026-11-12-l25-zuan-review/README.md)列出的命令是完整三 crate 测试，实测摘要里跑的却是加了 `--lib` 的版本（25 passed），集成测试没有跑，所以没发现。

未重跑：前端、typecheck、build:native、smoke:native、跨目标、desktop-client、strict Gate 1b／wasm32、conformance、headless Chrome、preflight。

## 本轮确实做到的

这一轮是实质推进，与此前几轮不同。

1. **第 0 项（`c9cf9c7de`）**：未提交工作区已逐项收拾并提交；本人上一轮写在共用文档里的勘正段完好保留；证据文件移入带日期目录；`choice_routing.rs` 不再读取未公开身份。
2. **第 1 项（`c47831ddb`）**：用 `boss_chuanyun`、`boss_tianyun`、`dddxiaxing` 打样，确认既有 `HostBehavior` ＋ `HostStage` ＋ `ChoiceKind` 通道可以在不升版本号的前提下提供真实玩家选择与选择挂起中的存档恢复；独立复核首轮退回 7 处问题后修复（见[打样证据](../2026-11-12-step1-choice-channel-prototype/README.md)）。
3. **第 2 项 13 个来源**：U06、U07、L06、L22、L29、L10、U01、L28、L27、L19、L20、U08、L25 的规则项迁到 `crates/engine-native-content/src/content/host/step/` 下逐技能实现（带 `*_asks.rs` 询问），清单计数为**已复核等价 851／简化近似 3,573**。851 项中技能 460、武将 159、配置 115、界面钩子 98、卡牌 19。
4. **清单如实**：L11（934 项）、U05（1,621 项）及界面类来源仍标「简化近似」；已完成来源里的 CSS／纯显示面也如实留在「简化近似」，注明归第 5 项。

## 抽查（固定种子 `random.seed(20261113)`，从「已复核等价」的技能中抽 12 项，细读 6 项）

| 技能（来源） | 上游 | 原生 | 结论 |
| --- | --- | --- | --- |
| `hanbei` 捍北（U08，`skill.js:648`） | 装备坐骑时使用【杀】不可被响应 | `u08_batch_a.rs`：同条件，置不可响应标志 | 一致 |
| `biaoqi` 骠骑（U08，`skill.js:1452`） | 攻击范围大于目标体力则不可响应，小于则摸 1 张 | `u08_batch_b_ext.rs`：同 | 一致 |
| `tspowei` 破围（L20，`extension.js:4366`） | 使命技：有「围」的角色回合开始时二选一（弃牌造成伤害／获得其手牌）或取消，带成功／失败分支 | `l20_batch_b_mouyi.rs`：多阶段状态机，三个询问（选项、选弃牌、选获得牌） | 结构与选择一致 |
| `th_guili` 归离（L28，`extension.js:4475`） | 首个回合开始选一名角色；其每轮首个回合结束时若未造成伤害，自己获得额外回合 | `l28_batch_a_asks.rs`：选目标询问＋逐轮标志＋额外回合 | 一致 |
| `dinghan` 定汉（L27，`extension.js:2562`） | 首次成为某锦囊目标时无效并记录牌名；回合开始可增删一个记录 | `l27_batch_a_asks.rs`：位掩码记录＋回合开始询问 | 一致 |
| `dddanzhi` 安止（U01，`skill.js:5796`） | 任意角色回合开始时，若上一张被使用／打出的牌是黑色，可选一名角色，其本回合首次成为黑色牌目标时无效 | `u01_batch_d.rs`／`u01_batch_d_asks.rs` | **两处不一致**：①触发条件写成「上一张是黑色 **或** 弃牌堆非空」，后半句使条件几乎恒真；②所选目标不是自己时，自己也获得了同样的免疫 |

6 项里 5 项一致、1 项有偏差。实现质量整体可信，但「已复核等价」不等于零缺陷。

## 复核过程的问题

- 提示词要求抽样项在 headless Chrome 的上游无名杀里实际触发后比对。13 份来源复核证据里没有一份这样做，全部是读源码比对。
- [U01 复核表](../2026-11-12-u01-ddd-review/README.md)有 72 行的「核验内容」是同一句模板话（「触发时机、负例守卫、交互选择分支与状态结算」），`dddanzhi` 就在其中，上面两处偏差没有被发现。
- 界面钩子、配置类的「已复核等价」（共 213 项）多为「无规则效果／惰性登记」，含义弱于技能的等价，例如 L25 的 `_emojiRebate1`（上游是丢表情的显示钩子）。

## 仍未完成

- 队列第 2 项剩余：L11 太虚幻境、U05 杀海拾遗，以及其余含规则项的来源（L01 AI优化、U02／L03 boss、U03／L04 cardpile、L15、L21 等）。
- 第 5 项界面、第 6 项 Spine（逐特性测试、净室复核、用户本机对照）、第 8 项签署（整包仍 **0/37**）、第 9 项真机与签名、第 10 项发布、第 11 项第六阶段。

## 结论

- 方向和做法是对的：交互通道可行且不升号，13 个来源的规则项有了逐技能实现和真实选择。
- 需要立即处理：三 crate 门禁为红（`FzLiegong` 选序）、全工作区另有 1 个过期断言失败；15 个提交未发布、无 CI；`dddanzhi` 两处偏差；复核不得用模板行。
