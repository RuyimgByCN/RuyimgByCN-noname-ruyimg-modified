# UP37-N01：OL界朱桓「奋励/平寇」与 OL界朱然「胆守」（2026-10-24 / 2026-10-25）

核验基准 `7e28a93799a9c01794085e911a72a4faa5673883` 加未提交工作树；规则 **v322**、存档 **v105**。本批补齐 **OL界朱然/胆守** 四选项状态机；N01 旧线规则面完成。原生 onlyOL 仍受 ABI 757 > host 753 阻塞。

## 完成内容

1. `ol_fenli`（奋励，def `2744558996`）完整复用现有 fenli 原语：
   - `phase_draw`：手牌数为全场最大时可跳过摸牌；
   - `phase_play`：体力为全场最大时可跳过出牌；
   - `phase_discard`：装备数为全场最大且有装备时可跳过弃牌。
2. `ol_pingkou`（平寇，def `2376678765`）：结束阶段按本回合跳过阶段数选择至多 X 名其他角色各 1 伤害；**新增效果** `choose_other_targets_by_skipped_phase_count_damage_draw` 在伤害事件后按实际受伤目标数摸牌（每名角色一次）。ABI 754。
3. `ol_zhuhuan`（OL界朱桓，def `3125343915`）加入 onlyOL 角色表：hp4/群/男，skills `[ol_fenli, ol_pingkou]`。
4. onlyOL npack **0.1.0 → 0.1.4**；ABI **757**；新增两个技能模块与一个角色。
5. **2026-10-25 胆守补齐**：
   - 新 host effect `resolve_ol_jie_danshou`（tag 771，ABI 757）与 continuation 335；
   - 选项 1 弃置目标一张手牌/装备、2 获得其他角色一张、3 造成 1 伤害、4 双方各摸 2；
   - 结算后弃置 X 张（X=选项序号，不足全弃，可取消）；
   - 目标/牌选择走 `discardable_target_cards` / `gainable_target_cards`，尊重弃置/获得保护；
   - 四选项用技能私有 storage 键 1..4 记录，`phase_play` 触发清零；
   - `ol_jie_zhuran` def `2606884808`、hp4/吴/男、技能 `ol_jie_danshou` def `3750129445`；
   - 角色/技能 def 与 catalog 对齐，catalog 状态改为 `ported`，迁移计数同步。
6. **旧包保留**：原始 onlyOL 0.1.2 npack 存为 [onlyOL-0.1.2.npack.json](onlyOL-0.1.2.npack.json)，供旧内容锁恢复；新包 0.1.4 不覆盖旧版本语义。

## 测试调整（语义性，不是掩盖失败）

新增 `original_onlyol_n01`，断言角色/技能/三窗口与平寇 filter；新增 `original_onlyol_n01_danshou` **24 项**：四选项、目标牌移动、序号弃牌、取消、每阶段一次、非法旧选项拒绝、`phase_play` 清零、`ActivateSkill` 实际入口。
本批真实新增 **3 技能、2 角色、5 触发器、6 模块**（含胆守 trigger+active），onlyOL 计数更新为 **193 技能 / 86 角色 / 32 触发器 / 36 模块**；`original_merged_qiaoshui` 同步 192→193。onlyOL 包 ABI 由 754 合法提升到 **757**（新 tag 771），相关断言 754→757 并在注释说明语义。

## 额外修复：非空二元键地图的 Snapshot 编码

`TurnState.damage_pair_counts` / `card_target_counts` 使用 `(PlayerId, PlayerId)` 键，`serde_json` 无法编码元组键，导致**任何有伤害/用牌目标历史的非空 Snapshot 编码失败**。已改为稳定 `"a:b"` 字符串键，并新增 `danshou_storage_survives_snapshot_roundtrip`：胆守造成伤害后捕获 Snapshot、编码、解码、恢复，技能 storage 与已用选项保持。规则/存档版本不变（旧非空快照此前不可生成；空地图仍为 `{}`）。另加 `snapshot::tests::roundtrip_non_empty_pair_keyed_turn_maps` 同时锁定二元键地图、`group_overrides`、`damage_dealt_totals`、`card_action_totals`、`hand_limit_bonus`、`extra_effect_count`、`facility_gained_skills`、`debate_pinned_opinions`、`pending_turn_end_debates`、`content_card_tags` 与回合禁用/获得牌集合的编码/解码/恢复，以及 `danshou_pending_choice_continuation_survives_snapshot`：选项询问挂起时快照、恢复后继续选择目标并造成伤害。

## 结果

| 检查 | 结果 | 证据 |
|---|---|---|
| `original_merged_onlyol` | **69 / 0 / 0** | [日志](onlyol-final17.log)、[退出码](onlyol-final17.exit) |
| `engine-core` 完整 | **3836 / 0 / 1**（174 target） | [日志](engine-core-final20.log)、[退出码](engine-core-final20.exit) |
| `engine-native` 完整 | **1550 / 0 / 71**（316 target） | [日志](engine-native-danshou.log)、[退出码](engine-native-danshou.exit) |
| `game-server` 完整 | **358 / 0 / 0**（25 target） | [日志](game-server-danshou.log)、[退出码](game-server-danshou.exit) |
| `content-compiler` 完整 | **541 / 0 / 0** | [日志](compiler-danshou.log)、[退出码](compiler-danshou.exit) |
| `sync-original-content --verify-only` | **exit 0** | [日志](sync-danshou.log)、[退出码](sync-danshou.exit) |
| 全 workspace | **601 targets / 6588 passed / 0 failed / 72 ignored** | [日志](workspace-final20.log)、[退出码](workspace-final20.exit) |

## 未完成与边界

- **N01 旧线规则完成**：界朱桓/奋励/平寇 + 界朱然/胆守四选项；AI 估值、原生 onlyOL 安装、旧 0.1.2 局恢复未验收。
- 已补奋励跳过摸牌的引擎级回放；平寇两人受伤→摸 2、每目标一次伤害、护驾吸收伤害→不摸牌、零跳过阶段不开窗、选自己为非法目标不变更状态的边界回放。
- 0.1.0 → 0.1.2 未做旧档迁移；按内容锁应拒绝，未做反例。
- 未改规则/存档版本、golden 或人工 DSL 之外的内容。
