# 2026-10-11 T12 机关妖术切片（规则 v197）

任务来源：[进度主稿 T12](../../../architecture/native-progress.md)（领取范围：**机关妖术单卡切片**）；开工 HEAD `26d64d44`（领取锁）。上游：`archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1216`（卡）、`skill.js:1883`（jiguanyaoshu_skill）。

## 上游语义与原生实现

| 面 | 上游 | 原生 |
|---|---|---|
| 目标 | range.global 1，无距离约束；filterTarget 未排除 self | 恰 1 名存活角色（含自身） |
| 空栏 | subtype equip1–5 有空 | 目标 equips 的 EquipSlot 占用集 <5（slot 语义对齐 subtype） |
| 防重 | !hasSkill("jiguanyaoshu_skill") | 无 `SwdJiguanyaoState` 实例（tag 226，运行时授入、持久无时限，同上游 addSkill） |
| isMin | !target.isMin()（手牌数≤体力排除） | 手牌数>体力方可在校验通过 |
| 装备 | `get.inpile('equip')` 按空 subtype 过滤 → createCard → equip | **牌堆+弃堆**装备实体按空 EquipSlot 过滤 → RNG 发现 → 入目标手牌 → emit Equipped（commit_equip 完整装配：同 Kind 替换/装备技授予） |

## 登记差异（规则基准 v197 条目）

1. **发现池**：牌堆+弃堆可见实体 vs 上游全部已定义装备牌凭空造牌；无候选则不装备（上游必有候选）。
2. **补偿触发面暂缓**：上游 `jiguanyaoshu_skill` 为复合技——防重面+补偿触发面（非自己回合失去装备区牌→随机补获等量 hslingjian 灵剑）。补偿面依赖 hslingjian 灵剑类型池（7 张 swd 自造卡，list.js 无、仅机关术/该技生成），**暂缓至机关术批**（机关术本体同样消费该池，同批最经济）。本切片落地防重面。

## 改动内容

- `content/host.rs`：`SwdJiguanyaoshuCard`(225)/`SwdJiguanyaoState`(226)（枚举/tag/from_tag/key/ALL 224→226/meta None×2/filter false×2/kind 级 def 追加防重标记）；机器臂（槽位占用集过滤+洗牌发现+Equipped+授标记+本体弃置）。
- `kernel.rs`：Jiguan 工厂臂 `jiguanyaoshu` 分流；`engine.rs`：PlayCard 校验分支（空栏/防重/isMin）。
- `content/standard.rs`：196→**197**；`tests/replay_scenarios.rs` 14 golden 重录。
- 包数据：cards.json +2 张（swd_jiguanyaoshu1 ♣3/swd_jiguanyaoshu2 ♠3）→ source_hash/asset_hash 重算。
- `tests/g1_shahai_swd.rs`：目录计数 34→36（3 处）+ 新增回归 2 支 + 共享 helper（`jiguan_ids`/`inject_hand`）。

## 验证

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p engine-native --test g1_shahai_swd jiguanyaoshu` | **2/2**（发现装配+防重拒+isMin 拒；5 栏全占拒） |
| 版本回退探针（197→196 复跑 `replay_scenarios`） | 2/2 全绿（恢复 197 后重录 14 哈希） |
| `cargo test --offline -p engine-native --no-fail-fast` | **535 通过 / 0 失败 / 1 ignore**（533+2，exit 0）——[完整日志](engine-native-tests.log) |
| `RUSTFLAGS="-D warnings" cargo check --offline -p engine-native --lib`（独立 target 目录，用完删） | exit 0 |

## 边界

- 补偿触发面（灵剑补偿）暂缓：同目标在标记存续期间永久不可再指定（上游同口径，无时限）。
- 装备池发现为牌堆+弃堆子集；标准牌堆含各栏装备，正常对局可发现候选。
- commit_equip 按 Kind 替换（三尖与武器同为 Weapon 栏但 Kind 不同的既有口径未改动，与本切片校验的 slot 语义并存的边角登记于源码注释）。
