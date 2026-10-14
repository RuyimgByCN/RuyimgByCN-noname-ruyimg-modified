# T12 运筹批 18：悲歌本体实现——蔡文姬整将转正（2026-10-14）

## 承载（refresh 变体，`YunchouBeige` tag 263）

| 面 | 上游 | 原生承载 |
|---|---|---|
| 触发 | global damageEnd（杀伤害、受伤者在场、持有者有牌） | filter 同面 + optional 确认门 |
| 判定 | 受伤者 judge | `ctx.draw_judge_card()`（炉石智略搏先例） |
| 弃牌 | chooseToDiscard he 1 可取消 | SelectCards min 0 max 1 可取消（自选 ✓ 同面） |
| 四花色 | 红桃受伤者回 1/方块受伤者摸 2/黑桃来源翻面/草花来源弃 2（强制） | Heart recover（已受伤门）/Diamond draw 2/Spade 翻面内联（风雪刃/急冻先例 face_down 切换+TurnOvered）+Club 首 2 张确定性弃置（登记差异：上游强制全弃=同面） |
| 收回奖励 | 弃牌花色=判定花色收判定牌；点数=判定点数收所弃牌 | 弃堆检索收回入持有者手 |

**登记差异**：草花来源弃 2 上游 `chooseToDiscard(2,true)` 强制（AI 自选）——
原生首两张确定性弃置（苦无先例）；addExpose（AI 威胁面）不做。

## 测试要点（回归驱动）

- **判定花色控制**：`debug_set_deck_top("tao")` 按 kind 找牌有花色变体
  （tao 实际为♦12）——**须按谓词压确定花色**（`debug_set_deck_top_by`）。
- 红桃回血断言：hp2-1+1=2（首版期望 3 为算术错）。
- 方块分支对照：压♦顶→受伤者摸 2。
- 负例：无牌持有者不触发。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 | g1_shahai_yunchou **25/25**（红桃回血/方块摸 2/负例） |
| engine-native 全量 | **744/0/0**（golden 不变——global Damage 触发面标准场景无悲歌持有者）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0（拦冗余 mut/out，已修）——[日志](gate1b-all-targets.log) |

## 登记边界

- 运筹武将面：**10/10 名、18/20 条目转正、2 暂缓**（**蔡文姬 2/2 部分转正**
  ——护驾 builtin 已通，曹操 2/3 缺奸雄·变体）。
- builtin 白名单**未扩**：悲歌以运筹包自有 host 行为（host:beige）承载，
  refresh 引用关系原生不建模（登记）。
- addExpose/黑桃翻面的 TurnOvered 帧在扩展触发面不转窗（急冻先例同登记）。
