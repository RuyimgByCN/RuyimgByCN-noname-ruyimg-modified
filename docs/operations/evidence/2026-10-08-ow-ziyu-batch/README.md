# U05 ow 纵切批三：自愈（禅雅塔）（规则 v120，2026-10-08）

ow 纵切批三：**自愈**（ziyu，禅雅塔单技能）映射 `host:wudian_ziyu`——
每 4 个阶段（全局阶段计数 %4==0），你可以摸一张牌或回复 1 点体力（可
放弃；满血时无回复选项）。新状态：`TurnState.phase_number: u32`（上游
game.phaseNumber 同口径，每阶段开始 +1，入存档/哈希）。前置：
[ow 纵切批二](../2026-10-08-ow-guangshu-batch/README.md)。

## 上游语义核验（源码为准，…/character/ow/skill.js ziyu）

- `ziyu`：trigger global phaseEnd、direct、filter
  `game.phaseNumber % 4 === 0`、content
  `chooseDrawRecover(prompt)`（可选摸 1 或回 1，可放弃）。

## 改动（分开陈述）

- **代码已实现**：`WudianZiyu`（tag 112）——TriggerMeta PhaseBegin
  （**End 阶段开始帧**≈上游 phaseEnd 语义，登记差异：原生锁定 End 阶
  段开始，上游逐阶段 global）+ filter（phase_number>0 && %4==0）+
  OptionalGate 确认 + 两段机（Tag0 摸 1 / Tag1 回 1——满血仅摸牌选
  项，可取消=放弃）。**新状态**：`TurnState.phase_number: u32`（每阶
  段开始 +1，入存档/哈希全套）。包 cards.json：禅雅塔 ziyu mapping
  （skills 段登记），三哈希重算。
- **登记差异**：上游 phaseEnd 逐阶段 global 触发——原生锁定 End 阶段
  开始（Phase{End,begin:true} 帧存在且语义=「结束阶段」时点）；
  phaseNumber 从 0 计（%4 门控含 0——phase_number>0 防开局触发，登记
  微差）。
- **转换包**：cards.json 增 禅雅塔（generals）+ ziyu skills 条目，三
  哈希重算。

## 验收口径

- **回归已加入**：`chanyata_ziyu_every_four_phases`——12 回合内 %4 门
  控至少触发一次；触发时选回复 1（受伤态）验证 2→3。
- **全量通过**：engine-native **438 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 119→120 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（自愈不在标准场景，纯版本位移）。
- **剩余**：ow 其余 21 武将、xianjian 36 武将逐武将纵切；U07
  duoqi/youzhan 暂缓；zhenfa 五种阵法。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（17 文件，相对 noname-next 根）。
