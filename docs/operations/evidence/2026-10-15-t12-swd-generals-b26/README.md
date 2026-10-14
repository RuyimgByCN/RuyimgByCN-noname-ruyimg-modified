# T12 swd 武将面批 26：雪煌——芝茵整将收口（limited+虚拟火杀循环）（2026-10-15）

## 承载

**雪煌 xuehuang**（tag 286，键数 285→**286**；字段批）：

- limited 每局一次主动技（ActivateSkill 通道）：手牌全红且非空 → 弃全部
  手牌，逐张视为对（至多）两名其他存活角色使用火杀
  （`use_virtual_card(FireSha)`，牌堆底载体+use_as 声明 v169 原语）。
- **登记差异**：上游随机 2 名敌人（isEnemyOf 阵营面）→ 原生座位序前 2
  名（yinbo 先例）；limited 经 PlayerState bool 持久。
- **字段批三件**：PlayerState.xuehuang_used（存档 v68→**v69**）、规则
  v280→**v281**、golden 14 场重录。

## golden 重录与探针（比既有字段批更强的证明）

- 版本回退探针：**280+字段复跑 14 场与旧 golden 逐字节一致**——golden
  哈希域不含存档字节/新增字段，位移唯一来源=规则版本字段；雪煌不在标准
  场景可达路径，行为面零扰动。已写入 replay_scenarios.rs 重录依据注。
- 14 场新哈希入表（sha_dodge/death_terminal/sancai/yishang/suidun/shouyi/
  discard/rollback/equip_distance/chain_fire/aoe/duel/nullify/judge 全
  表替换）。

## 测试（swd_generals_b26_xuehuang_limited_virtual_firesha，反例 3 项）

| 面 | 断言 |
|---|---|
| 反例 含黑 | activate 被拒（手牌须全红） |
| 正例 | 手 3 红 → 全弃 3 + p1/p2 各吃 3 张火杀（8→5）+ xuehuang_used 置位 |
| 反例 limited | 本回合再发动拒 |
| 反例 跨回合 | 推进 4 个回合后仍拒（bool 持久非回合级） |

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 g1_shahai_swd | **37/37** |
| engine-native 全量 | **756/0/0**（含存档 v69 往返+golden 14 场）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- swd 武将面：**21/79 名、24/241 技转正；整将十五→十六**（芝茵 3/3 新
  晋）。
- 剩余部分转正：壶中仙 2/3（壶境挂卡牌面对账批）、屈娴 1/2（huanxia
  viewAs+子技组）、拓跋玉儿 1/3（poyue/poyue2 ShaResolve 内联直挂+存档
  触点）。
- 挂账不变：swd 卡牌面对账批（92 vs 45）。
- 批 27 候选：huanxia（屈娴整将收口）或 poyue 线（拓跋玉儿，涉
  ShaResolveMachine 快照字段=又一字段批）。
