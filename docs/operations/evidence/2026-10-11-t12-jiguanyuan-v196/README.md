# 2026-10-11 T12 机关怨切片（规则 v196，存档 v43）

任务来源：[进度主稿 T12](../../../architecture/native-progress.md)（领取范围：**机关怨单卡切片**）；开工 HEAD `5e33a84b`（领取锁）。上游：`archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:975`（卡）与 `skill.js:498`（伴生技，包内定义）。

## 上游语义与原生实现

| 面 | 上游 | 原生 |
|---|---|---|
| 目标 | target≠player 且 !hasSkill("jiguanyuan") | PlayCard：恰 1 名其他存活角色 + 目标 `jiguanyuan_cards` 为空（同口径） |
| enable | hs>1 ‖ (hs==1 且 hs[0]≠本体) | PlayCard：手牌非空且非"仅本体"（同口径） |
| 置牌 | 本体+所选 1 张手牌置目标武将牌上（ui.special/storage） | 两张移入目标**离场区**，`PlayerState.jiguanyuan_cards` 记录实体（本体不进弃堆） |
| 获得 | 目标 jiguanyuan 技 phaseEnd forced：gain storage → 移除 | `SwdJiguanyuanGain`（tag 224，phaseEnd forced，kind 级 def 注册）：离场区待获得牌移入手牌+状态移除 |
| 使用者 | `player.draw()` | `draw_cards(owner,1)` |
| 无懈 | wuxieable | wrap_nullify_window 共享 |

## 改动内容

- `state.rs`：`PlayerState.jiguanyuan_cards: Vec<u32>`（待获得实体，离场区中）。
- `archive.rs`：玩家编解码尾部追加 `jiguanyuan_cards`（len+u32 序列）；**`ARCHIVE_FORMAT_VERSION` 42→43**（v42 字节被版本门禁拒绝，无生产迁移数据）。
- `content/host.rs`：`SwdJiguanyuanCard`(223)/`SwdJiguanyuanGain`(224)（枚举/tag/from_tag/key/ALL 222→224/meta/filter/kind 级 def）；机器臂两支（置牌选牌→离场区+授状态+摸 1；PhaseEnd 强制获得）。
- `kernel.rs`：Jiguan 工厂臂 `jiguanyuan` 分流；`engine.rs`：PlayCard 校验分支。
- `content/standard.rs`：195→**196**；`tests/replay_scenarios.rs` 14 golden 重录。
- 包数据：cards.json +2 张（swd_jiguanyuan1 ♣3/swd_jiguanyuan2 ♦2）→ source_hash/asset_hash 重算。
- `tests/g1_shahai_swd.rs`：目录计数 32→34（3 处）+ 新增回归 2 支。

## 验证

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p engine-native --test g1_shahai_swd jiguanyuan` | **2/2**（置牌→Judge 收束获得 2 张全链；同目标待获得期间再指定拒/换目标放行） |
| 版本回退探针（196→195 复跑 `replay_scenarios`） | 2/2 全绿（恢复 196 后重录 14 哈希） |
| `cargo test --offline -p engine-native --no-fail-fast` | **533 通过 / 0 失败 / 1 ignore**（531+2，exit 0）——[完整日志](engine-native-tests.log) |
| `RUSTFLAGS="-D warnings" cargo check --offline -p engine-native --lib`（独立 target 目录，用完删） | exit 0 |

## 边界

- 置牌进离场区复用既有区；与其他离场区占用（护符类）并存时 `jiguanyuan_cards` 按实体 ID 精确取回，不误取他卡。
- 目标死亡时离场区走既有死亡清理口径；待获得状态随死亡清理由既有清理路径覆盖（与其他机关牌一致）。
- 机关牌剩余：机关术×2（需 hslingjian/jiqi 类型池，原生未转换）、机关妖术×2（需装备池实体发现）——各自为独立后续切片。
