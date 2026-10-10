# P2/N08 模式入口批一：对决 versus 2v2 基线（规则 v80，2026-10-08）

主稿 P2/N08「模式入口扩充」第一批：上游 `apps/core/mode/versus.js`
（9k+ 行 UI/流程脚本）中引擎可接的核心=队伍表+胜负+奖惩，以 **2v2
基线**接入引擎。前置：[逐技对照批五](../2026-10-08-n08-skill-parity-5/README.md)。

## 上游语义核验（源码为准）

`versus.js` help 块 2v2 规则与 death 处理（draw(3) 调用点）：
- 胜负：消灭所有敌人。
- 奖惩：**杀死敌方武将摸三张牌，杀死友方武将弃置所有牌**。
- 未建模（登记）：替补武将上场、4v4 主公制/号位置换、间断/四国变体
  （均为 UI/流程面）。

## 改动（分开陈述）

- **代码已实现**：
  - `GameMode::Versus { teams: Vec<Vec<PlayerId>> }`（两队互斥、并集=
    全部座位）+ `Winner::Versus(usize)` + `versus_teams()/versus_team_of()`
    辅助 + `Engine::new_versus` 构造器。
  - DeathMachine 奖惩（先于胜负判定）：跨队击杀摸三；同队击杀弃掉
    **杀手自己**全部手牌与装备（同主公杀忠臣路径，含装备技能撤销）。
  - 胜负：任一队伍全灭 → 对方队伍胜。
  - 接线：存档 enc/dec（mode tag 3；winner tag 5）、canonical_hash
    （mode 段 tag 4；winner tag 6——Traitor 已占 5，三套 tag 编号独立）、
    快照结构检查（队数=2/成员互斥/覆盖全部座位/outcome-模式一致性）、
    presentation 终局文案。
- **登记未建模**：替补、4v4 主公制、间断/四国变体（见上）。

## 验收口径

- **回归已加入**：`versus_mode_team_victory_and_rewards`——杀敌摸三
  （未终局）、中途存档往返（模式表一致）、杀友弃全部（清奖惩摸牌防
  桃干扰，身份局测试同款口径）、全灭终局 `Winner::Versus(1)`。
- **全量通过**：engine-native **383 passed / 0 failed / 1 ignored**
  （唯一 ignore=写轮眼规模诊断探针），单线程
  `cargo test --offline -p engine-native -- --test-threads=1`；
  依赖包 content-runtime + engine-ai + content-native-loader
  **62 passed / 0 failed**，DEPS_EXIT=0。明细见 [test-logs.txt](test-logs.txt)。
- **golden**：版本 79→80 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（标准场景无 versus 局，纯版本位移）。
- **上游等价边界**：2v2 基线规则等价；替补/4v4/变体未建模（登记）；
  KOF 仍在示例会话层、doudizhu/boss 未接入（主稿待做）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（11 文件，相对 noname-next 根）。
