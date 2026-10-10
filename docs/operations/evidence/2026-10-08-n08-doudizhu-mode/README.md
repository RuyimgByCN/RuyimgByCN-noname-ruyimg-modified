# P2/N08 模式入口批二：斗地主（规则 v81，2026-10-08）

主稿 P2/N08「模式入口扩充」第二批：上游 `apps/core/mode/doudizhu.js`
（4k 行 UI/流程脚本）引擎可接核心=身份组成+胜负，复用 versus 批一的
接线模式接入。前置：[versus 2v2](../2026-10-08-n08-versus-mode/README.md)。

## 上游语义核验（源码为准）

- **身份组成**：`chooseCharacterKaihei` → `identityList = ["zhu","fan","fan"]`
  （3 人局：地主×1 + 农民×2；更大局经 `get.identityList` 扩展）。
- **无击杀奖惩**：`dieAfter` 仅挂 dieidentity 展示节点 + `checkResult()`
  判负——无任何 draw 奖惩（grep 全文无 draw(3)/摸三 类逻辑）。
- **胜负**：`game.over(me === game.zhu)` 语义——地主死亡 → 农民方胜；
  农民全灭 → 地主胜。

## 改动（分开陈述）

- **代码已实现**：`GameMode::Doudizhu { lord: PlayerId }` +
  `Winner::Doudizhu { lord_won: bool }` + `doudizhu_lord()` 辅助 +
  `Engine::new_doudizhu` 构造器；DeathMachine 胜负臂（地主死 → 农民胜、
  农民全灭 → 地主胜，无奖惩）；接线=存档 enc/dec（mode tag 4、winner
  tag 6）、canonical_hash（mode 段 tag 5、winner tag 7）、快照检查
  （lord 座位越界 + outcome-模式一致性）、presentation 终局文案
  （versus 同款四件套）。
- **登记未建模**：特殊开局摸牌变体（binglin 地主 5/农民 4、按 infoHp
  加成）、开黑/欢乐变体 UI 流程。

## 验收口径

- **回归已加入**：`doudizhu_mode_victory_and_roundtrip`——无奖惩断言
  （击杀后地主手牌数不变）、中途存档往返（模式表一致）、地主死 →
  农民胜（`Doudizhu { lord_won: false }`）、农民全灭 → 地主胜
  （新局，`lord_won: true`）。
- **全量通过**：engine-native **384 passed / 0 failed / 1 ignored**
  （唯一 ignore=写轮眼规模诊断探针），单线程
  `cargo test --offline -p engine-native -- --test-threads=1`；
  依赖包 content-runtime + engine-ai + content-native-loader
  **62 passed / 0 failed**，DEPS_EXIT=0。明细见 [test-logs.txt](test-logs.txt)。
- **golden**：版本 80→81 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（标准场景无斗地主局，纯版本位移）。
- **上游等价边界**：核心胜负规则等价；变体/UI 面未建模（登记）；
  boss 模式与 KOF 入引擎层仍按主稿待做。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（12 文件，相对 noname-next 根）。
