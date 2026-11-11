# 2026-11-11 进度核对扫描（裁决 34 五轮重做与裁决 35–39 之后）

- 核对人：Claude Code；时间 2026-11-11 14:05–15:10（CST）
- HEAD：`245dd0a93`（纯文档；最后代码提交 `99d9091bb`），非文档路径干净；规则 333／NGSA 107／content schema 5／ABI 759／WS8
- 范围：核对 `f38cd5fbe..245dd0a93` 共 28 个提交（裁决 34 清点与五轮重做、裁决 35 净室 Spine 4.0、裁决 37–39 登记）；代码改动 93 个文件、约 +35,300 行

## 实跑命令与结果

| 命令 | 结果 | 原始输出 |
| --- | --- | --- |
| `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native --no-fail-fast` | exit 0；**2229 passed / 0 failed / 6 ignored**（17 段）；6 个 ignore 仍是手工延迟／AI 质量／状态增长／悬挂引用探针 | [three-crates.log](three-crates.log)、[忽略清单](three-crates-ignored.txt) |
| `cargo test --offline --workspace --no-fail-fast` | exit 0；**7622 passed / 0 failed / 8 ignored**（177 段） | [workspace.log](workspace.log) |
| `apps/web-client: pnpm test` | 489 pass / 0 fail（输出未落盘） | — |
| `python3 -B scripts/check-source-lines.py` | 通过：9259 文件 | — |
| `git ls-remote <远端> refs/heads/native-convergence` | `99d9091bb`，与登记的最后代码提交一致 | — |
| `adb devices`；`xcrun devicectl list devices` | 无 Android 设备；物理 iPhone（iPhone17,2）仍为 `unavailable` | — |

未重跑：typecheck、build:native、smoke:native、跨目标、desktop-client、strict Gate 1b／wasm32、conformance、headless Chrome、preflight。

测试全绿只说明代码与它自己的测试一致。下面的抽查看的是实现和测试断言的行为是否等于上游。

## 本轮确实做到的

1. **37 份逐来源清单已建**：`docs/migration/g0/` 37 份主文档加 1 份分册，合计 4,424 项，每项带上游 `file:line`。
2. **自造内容已清除**：`decade_*`、`vir_*`、`ms_*`、`zuan_*`、`shousha_*` 等上游不存在的武将、卡牌、技能，以及 `drawSpineSkeletalCanvas` 示意画法、合成战役步进技能，均已删除。
3. **接入正式包**：各来源经 `attach_formal_package_upstream_rules` 接入 `packages/extensions/*` 正式包，不再只被测试调用。
4. **净室 Spine 4.0 播放器已有实体代码**：`apps/web-client/src/engine/spine40/`（JSON 与二进制 `.skel` 编解码、姿态求解、WebGL 渲染、导入入口与静态立绘回退）共约 4,600 行，11 条测试通过。
5. **发布与 CI**：最后代码提交 `99d9091bb` 已在远端 `native-convergence`，各批 exact-head CI 的首次失败与补跑分开登记。

## 「忠实实现 4,424 / 示意 0 / 缺失 0」不成立

清单里的状态不是比对得出的：`scripts/legacy-harness/decision34_inventory_builder.py` 把「忠实实现」直接写死在脚本里（如第 164–463 行逐来源的固定字符串），重新生成永远是 4,424/0/0。

抽查规则实现（`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34*.rs` 共 18 个文件、约 11,800 行，承载约 1,900 项技能）：

| 技能（来源） | 上游行为 | 原生实现 | 差别 |
| --- | --- | --- | --- |
| `lieqi` 猎奇（U05，`hearth/skill.js:1520`） | 回合开始／结束选一名角色，从三张牌中猜哪张是其手牌，猜中获得 | `u05_exec(…摸 1…)` | 整段逻辑换成摸 1 张 |
| `kekao` 科考（U05，`hearth/skill.js:1927`） | 结束阶段从 3 张随机延时锦囊中选 1 张获得 | `u05_exec(…摸 1…)` | 同上 |
| `txlangxi` 狼袭（L11，`extension.js:4631`） | 准备阶段选一名其他角色，造成 2–4 点随机伤害 | 与另外 10 个技能共用一个分支：对座次上第一个存活的其他角色造成固定 2 点伤害 | 无选择、无随机、多技能合并 |
| `boss_chuanyun` 穿云（L11，`extension_skill.js:3806`） | 结束阶段可对体力比你多的一名角色造成 1 点伤害 | 同上分支，固定打第一个存活的其他角色 | 体力条件丢失、可选变必发 |
| `boss_lingfeng` 灵锋（L11，`extension_skill.js:3843`） | 摸牌阶段改为亮出牌堆顶两张并获得；颜色不同才令一名敌方失去 1 点体力 | 摸 2 张，然后无条件令第一个存活的其他角色失去 1 点体力 | 颜色条件丢失（代码注释写了条件，但没实现）、未替换摸牌阶段 |
| `boss_tianyun` 天陨（L11，`extension_skill.js:4085`） | 结束阶段可选一名敌方：自己失去 1 点体力，造成 2 或 3 点（随机）火焰伤害并弃其装备 | 固定 2 点，目标为第一个存活的其他角色 | 无选择、无随机；测试断言的正是固定 2 点 |
| `leiji` 雷击（L11 跨包，`shenhua.js:7205`） | 使用或打出【闪】时令一名角色判定，黑桃才造成 2 点雷电伤害 | 触发窗口归为「自己使用任意牌」；弃牌堆顶一张（不看花色）后直接对第一个存活的其他角色造成 2 点雷电伤害 | 触发条件与判定都丢失 |
| `buqu` 不屈（L11 跨包，`shenhua.js:7047`） | 濒死时置一张「创」，与已有「创」点数不重复才不死 | 置一张牌后直接回到 1 体力 | 点数重复检查丢失 |
| `dddxiaxing` 侠行（U01，`3D精选/character/skill.js:59`） | 濒死或准备阶段，选一张牌置于牌堆顶，视为对方使用【桃】／【酒】，每轮限一次 | 自动取第一张手牌置顶并设标志 | 无选牌，每轮限一次未见实现 |

统计：

- `decision34*.rs` 中 `first_other_alive`（座次上第一个存活的其他角色，可能是队友）出现 185 次，`.first()` 自动取牌 107 次，`set_flag` 1,265 次；**向玩家发起选择的调用为 0**。
- U05 的 `decision34_u05_*.rs` 里有 807 处单行 `u05_exec(...)` 模板调用，其中 141 处只是「摸 N 张」，142 处只设一个 `ModFlag`。上游这些技能各有 10–70 行 JS。
- L11 的 `classify_l11_npc_skill_trigger` 把 164 个跨包技能归入 7 个粗粒度触发窗口，未列名的一律按「阶段开始」。
- 测试以 `debug_commit_effect` 直接分派技能并断言上述原生行为（如 `decision34_round4_l11_taixu.rs` 对 `boss_tianyun` 断言「扣 2 点」），不是按上游语义写的预期。L11 的 374 项本包技只有 3 条测试函数。

所以现状是：实体名单、数值和接线是真的，大部分技能体是按上游大意写的**简化近似**，比 11-09 的「一律摸 1」前进了一步，但不能记为忠实实现。

## 裁决 39 的实际影响

[裁决 39](../2026-10-28-user-decisions/README.md) 由执行 agent 依据用户「第三节第 4 项 按你推荐的进行」登记：不升版本号，上游技能里的多步交互子询问「按上游优先级规则确定性结算」。核对发现：

1. 实际结算不是上游优先级（上游 AI 按 `get.damageEffect`、`get.attitude` 等评估目标），而是「座次上第一个存活的其他角色」「第一张手牌」。
2. 规则基准只登记了一条笼统差异，没有逐技能列出哪些技能的玩家选择被去掉。
3. 条件判断、随机数、判定、可选发动这些并不需要新的交互阶段，丢失它们不在裁决 39 的范围内。

4. 内核已有不升版本号的玩家选择通道：原生 `HostBehavior` 配合 `HostStage::TargetChoice`／`SelectCard` 与 `ChoiceKind::SelectPlayers`／`SelectCards`／`Confirm`／按牌名选择（`crates/engine-native-content/src/content/host/asks.rs`、`crates/engine-native-kernel/src/choice.rs`）。Batch 24 的苍螭之璧就是经这条通道在规则 333／ABI 759 下实现「选 1–3 名角色」并支持挂起恢复的。「不升号就只能自动结算」这一前提因此并不普遍成立。

裁决 39 的效果是：这些扩展技能里，人类玩家不能自己选目标和选牌。是否接受这一点需要用户明确知道后再确认。

**用户答复（2026-11-11）**：选定「改为真实选择」，登记为[裁决 40](../2026-10-28-user-decisions/README.md)——撤销裁决 39 的自动结算部分，保留不升号部分。

## 界面与 Spine

- 「与上游同场景截图对照」未做到：[上游参考截图](../2026-11-10-round5-browser-surfaces/decision34-legacy-liuli-reference.png)只是琉璃版的启动欢迎页；[原生截图](../2026-11-10-round5-browser-surfaces/decision34-ui-skin-suites.png)是一组文字状态面板和按钮（如「十周年 UI（L08 · 319 钩子 / 43 表面）decadeLayout.css · standard」），不是十周年UI／手杀ui 的牌桌布局。L08 的 319 个界面钩子落在 `decadeUiSurface.ts`（323 行）和一张数据表里。60 FPS 测的是这块面板。
- Spine 4.0：实现是实在的，但二进制 `.skel` 只用自写编码器生成的数据做过往返测试，编码器和解码器若对格式有同样的误读，测试发现不了；尚未用任何真实动皮验证。裁决 35 要求的「每个特性一条测试」目前是 11 条测试合并覆盖，未见未参与实现者的净室复核记录。用户本机截图对照未做。

## 被删除的带日期记录

`ff82883a7` 删除了本人 2026-11-09 写在[签署表](../2026-10-19-source-signoff-32/README.md)标题下的勘正段。本轮已按原文恢复并标为历史快照。进度主稿、剩余台账、扩展台账里的既有扫描记录完好。

## 结论

- 裁决 34：清点、清除自造内容、接入正式包三步已完成；「按上游重做」只达到简化近似，签署表第①、⑧维的 ✅ 不成立，整包八项签署仍为 **0/37**。
- 裁决 35：播放器已实现，待真实资源验证、逐特性测试和净室复核。
- 裁决 36（PA09 关闭）、37、38 无异议。裁决 39 的自动结算部分已由裁决 40 撤销。
- PA08／PA11／NC12 真机、安装包签名公证、第六阶段仍未完成。
