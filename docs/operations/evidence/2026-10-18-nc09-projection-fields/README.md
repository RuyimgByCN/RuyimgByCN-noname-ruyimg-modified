# NC09 投影字段真实映射（`character_def`/`identity`/`known_cards`/`outcome`）（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC09**（[方案 §NC09](../../../architecture/native-convergence-plan.md#nc09)）+ **NC03** 投影面
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**4 个此前占位的投影字段已按内核真实数据映射 + 回归通过**；2 处无法表达的模式胜者**明确登记**（不猜测）

## 1 修复内容（此前为占位值）

| 字段 | 修复前 | 修复后 |
|---|---|---|
| `SeatView.character_def` | 恒 `0`（选将后客户端立绘/名字不随动） | 主将槽 def（无将 = 0）：`state_core().player(seat).general_slots[0]` |
| `SeatView.identity` | 恒 `None` | 按**内核可见性同口径**：本人 / 主公（公开）/ 已死亡者（遗言公开）→ `IdentityTag`；其余 `None` |
| `ObserverSnapshot.known_cards` | 恒空 | 观察者**本人手牌**（`CardView{id, def, owner, zone: Hand, kind}`）；他人手牌仅数量、不进入 |
| `ObserverSnapshot.outcome` | 恒 `InProgress` | `LordSide→LoyalistWin`、`Rebels→RebelWin`、`Traitor→RenegadeWin{存活座位}`、`Player(p)→SingleWin`、`Faction(f)→GroupWin{小写势力}`；**未终局 → InProgress** |

**明确未映射（登记，不猜测）**：`Winner::Versus(_)`、`Doudizhu{..}`、`Boss{..}` 在协议
`MatchOutcome` 中**无对应项** → 保持 `InProgress`（避免用错误语义冒充完成）。

**牌种映射边界**：协议 `CardKindTag` 只建模少数公开类别 → 未建模牌种一律
`CardKindTag::Other`（`kind_tag` 显式列出 `Sha/Tao/Shan/Jiu/FireSha/LeiSha`，其余 `Other`）。

## 2 回归（`nc09_native_view_bridge.rs` 新增第 4 项）

| 判据 | 结果 |
|---|---|
| 选将后 `character_def` = 主将槽 def（非 0） | 通过 |
| 本人 `identity` 可见；主公身份对观战者**公开**；存活非主公他人对观战者**不可见** | 通过 |
| `known_cards` = 本人手牌数且全为 `Hand` 区；观战者 `known_cards` 为空 | 通过 |
| 未终局 `outcome` = `InProgress` | 通过 |

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 桥接定向（含既有 3 项） | **4 passed / 0 failed** | [targeted.log](targeted.log) |
| game-server 全量 | **89 passed / 0 failed**（88 → +1） | [game-server-tests.log](game-server-tests.log) |
| 行数门禁 | 8266 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 4 未完成 / 待办（NC09）

- [ ] `MatchOutcome` 对 Versus/斗地主/Boss 的协议表达（需协议扩展或产品侧映射约定）
- [ ] `CardKindTag` 覆盖更多公开类别（当前未建模者 `Other`）
- [ ] `SkillView.enabled/use_count/active/view_as/locked` 仍为占位（需从内核技能实例真实取值）
- [ ] 真实浏览器/桌面端到端与平台档位；online 选将客户端入口；锦标赛面（manifest/分块签名）

## 5 追加：`SkillView` 字段真实取值（2026-10-18 同轮）

| 字段 | 修复前 | 修复后（取自**内核技能实例**，不是「目录里有就算在册」） |
|---|---|---|
| 来源 | 目录里查到名字即投影 | `state_core().skills.get(&(seat, def))` 必须存在；未装配武将的座位**技能列表为空** |
| `enabled` | 恒 `true` | 实例 `enabled` |
| `use_count` | 恒 `0` | 实例 `once_used`（0/1，本回合已发动） |
| `active` | 恒 `false` | `enabled && engine.is_legal_action(seat, ActivateSkill{def})`（**与内核权威校验同源**） |
| `locked` | 恒 `false` | `!active`（保守口径：当前不可主动即视为锁定/被动） |
| `id` / `def` | `def` | 均为目录 def（原生 `ActivateSkill{skill: def}` 以 def 为键，见 `legal_actions_for`） |
| `view_as` | 恒 `false` | **仍恒 false**：原生未建模「视为技」候选面 → **不猜测**，登记待办 |

新增用例 `nc09_bridge_maps_skill_view_from_instances`（第 5 项）：`enabled`/`use_count` 与实例逐项一致、
`active` 与内核权威校验一致、`locked == !active`、`view_as` 恒 false、**未装配座位技能列表为空**。

实测：定向 **5 passed / 0 failed**（[targeted-2.log](targeted-2.log)）；game-server 全量
**90 passed / 0 failed**（[game-server-tests-2.log](game-server-tests-2.log)）；行数门禁 8266 文件
（[source-lines-2.log](source-lines-2.log)）。

### 仍未完成（更新）

- [ ] `view_as`（视为技）候选面未建模 → 客户端无法展示「视为」类主动技
- [ ] `locked` 目前是「当前不可主动」的**保守口径**，未区分「被动技」与「主动技但当前不合法」
      （需目录侧主动形态标记，如 `SkillBehavior::activate` 是否被覆写）
- [ ] Versus/斗地主/Boss 胜者的协议表达；`CardKindTag` 扩展
- [ ] 真实浏览器/桌面端到端与平台档位；online 选将客户端入口；锦标赛面
