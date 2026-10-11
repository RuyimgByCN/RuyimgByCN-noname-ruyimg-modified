# 2026-10-11 T12 机关凤切片（规则 v195）

任务来源：[进度主稿 T12](../../../architecture/native-progress.md)（领取范围：**机关凤单卡切片**）；开工 HEAD `ec38ca2d`（领取锁）。上游：`archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:951`（转换源仓）、`list.js:87-88`（♣3/♠4 张数权威）。

## 上游语义与原生实现

| 面 | 上游 | 原生 |
|---|---|---|
| 目标 | filterTarget target≠player（单体） | PlayCard 校验恰 1 名其他存活角色（engine.rs Jiguan 分支新增） |
| 响应 | `chooseToRespond({name:'shan'})`，autochoose | 目标手牌闪 SelectCards（min 0/可取消）；**无闪→自动未响应**（上游 chooseToRespond 零候选同果） |
| 未响应 | `addTempSkill("fengyin",{player:'phaseBegin'})` + `damage()` | 全部技能实例禁用 + 授予 `SwdFengyinState`（tag 222，其任一阶段开始 forced 解除）+ 1 点伤害（封印先于伤害，同上游顺序） |
| 封印语义 | 全局 fengyin=`library/skill.js` skillBlocker：**非锁定技失效** | **登记差异**：原生无锁定技区分=全部技能失效（gwent 封印 v60 同口径，沿用已登记差异） |
| 本体 | — | 结算完入弃堆（卡牌机器惯例）；无懈窗包装共享（wrap_nullify_window） |

## 改动内容

- `content/host.rs`：`SwdJiguanfengCard`(221)/`SwdFengyinState`(222)（枚举、tag、from_tag、key、ALL 220→222）；`trigger_meta`（封印=PhaseBegin forced）/filter（owner 任一阶段 begin）/kind 级 def 注册（Jiguan 追加封印状态 def）；机器臂（Act 询问→SelectCard 应答分派）+ `seal_and_damage` 辅助（固有 impl）。
- `kernel.rs`：Jiguan 工厂臂按名分流 `jiguanfeng`（target=首目标）。
- `engine.rs`：Jiguan PlayCard 校验分支（恰 1 名其他角色）+ 测试夹具补新字段。
- `tests/a0_tag_registry.rs`：tag 表穷尽守卫补 221/222。
- 包数据：`assets/cards.json` +2 张（swd_jiguanfeng1 ♣3/swd_jiguanfeng2 ♠4）→ `rehash_cards_asset` 重算 source_hash + 临时 example 重算 manifest.asset_hash（跑完删除）。
- `content/standard.rs`：194→**195**；`tests/replay_scenarios.rs` 14 golden 重录。
- `tests/g1_shahai_swd.rs`：目录计数 30→32（3 处）+ 新增回归 2 支。

## 机关牌池勘误（台账同步）

权威（转换源 `swd/list.js`）实为 **8 种 17 张**：机关术×2/机关妖术×2/**机关怨×2/机关凤×2**/机关桶×2/勾魂锣×2/指路小虎×2/木甲人×3——此前登记"6 种 13 张"漏计机关凤×2 与机关怨×2。已切 5 种 11 张；**余机关术×2/机关妖术×2/机关怨×2**（机关怨含武将牌上置牌+阶段收益+目标侧技能，机关术含 createCard 类型池[hslingjian/jiqi 原生未转换]，均为后续切片）。

## 验证

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p engine-native --test g1_shahai_swd` | **16/16**（14 既有+2 新增） |
| 版本回退探针（195→194 复跑 `replay_scenarios`） | 2/2 全绿（恢复 195 后重录 14 哈希） |
| `cargo test --offline -p engine-native --no-fail-fast` | **531 通过 / 0 失败 / 1 ignore**（529+2，exit 0）——[完整日志](engine-native-tests.log) |
| `RUSTFLAGS="-D warnings" cargo check --offline -p engine-native --lib`（独立 target 目录，用完删） | exit 0 |

## 边界

- 封印语义差异（全部失效 vs 非锁定技失效）沿用 v60 已登记差异，未新增登记。
- 机关凤的响应为手牌闪直接弃置口径；`chooseToRespond` 的装备闪/技能转化响应面未建模（与杀的闪询问同口径）。
- 同批不切 机关术/机关妖术/机关怨（各自结构性依赖见上）。
