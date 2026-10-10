# P2/N08 模式入口批三-a：Boss 基线 + 重整复活回环（规则 v82，2026-10-08）

主稿 P2/N08「模式入口扩充」批三-a：按[批三评估定案](../../../architecture/native-rules-baseline.md)
执行——Boss 单体对抗基线 + 招牌机制「重整复活回环」完整落地（非贴皮）。
前置：[boss 评估](../2026-10-08-e01-longxue-survey/README.md)同目录
基准节、[doudizhu](../2026-10-08-n08-doudizhu-mode/README.md)。

## 上游语义核验（源码为准，mode/boss.js）

- `bossPhaseLoop`：死亡玩家轮到回合 → `boss_chongzheng` 计数 +1；
  hp < maxHp 则 hp+1，否则手牌 <4 摸 1；计数 ≥ `bossinfo.chongzheng`
  阈值 → `player.revive(player.hp)` 复活。
- `checkResult` = `game.over(!game.boss.isAlive())`——boss 死亡挑战者
  胜、挑战者全灭 boss 胜。
- `chongzheng` 未设（falsy）→ 无复活模式。

## 改动（分开陈述）

- **代码已实现**：
  - `GameMode::Boss { boss, chongzheng }` + `Winner::Boss { boss_won }`
    + `boss_mode()` + `new_boss`；`PlayerState.boss_chongzheng: u8`
    计数字段（PlayerState enc/dec/哈希接线）。
  - 重整回环：`commit_turn_ended` Boss 分支——死亡座位不跳过，轮到
    回合执行重整 tick（计数 +1、hp<上限 +1 否则手牌<4 摸 1、连续死亡
    座位逐个 tick 后落在下一存活座位）；阈值复活（alive 恢复、计数
    清零）；`chongzheng=0` 无复活（上游同口径）。新日志
    BossChongzhengTick/BossRevived（表现层映射 Recovered）。
  - 胜负臂：boss 死 → 挑战者胜；挑战者全灭 → boss 胜；无奖惩。
  - 接线：存档（mode tag 5、winner tag 7）、canonical_hash（mode 段
    tag 6、winner tag 8）、快照检查（boss 座位越界 + outcome 一致性）。
- **登记未建模**：addFellow 中途入队（批三-b）、boss 专属数值/技能包
  （内容面，G1）、bossinfo.loopFirst/loopType 变体。

## 验收口径

- **回归已加入**：`boss_mode_chongzheng_revival_and_victory`——无奖惩
  断言、第一次 tick（计数 1、回血 0→1、回合落 2 号、未达阈值仍死亡）、
  存档往返（模式表+计数一致）、第二次 tick 阈值复活（计数清零）、
  boss 死挑战者胜、chongzheng=0 无复活局挑战者全灭 boss 胜。
- **全量通过**：engine-native **385 passed / 0 failed / 1 ignored**
  （唯一 ignore=写轮眼规模诊断探针），单线程
  `cargo test --offline -p engine-native -- --test-threads=1`；
  依赖包 content-runtime + engine-ai + content-native-loader
  **62 passed / 0 failed**，DEPS_EXIT=0。明细见 [test-logs.txt](test-logs.txt)。
- **golden**：版本 81→82 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（标准场景无 Boss 局，纯版本位移）。
- **上游等价边界**：重整回环/胜负规则等价；addFellow/技能包/loop 变体
  未建模（登记）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
