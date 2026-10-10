# U07 技能批一：刺猬（韩龙）（规则 v111，2026-10-08）

玩点论杀武将技能声明批一：**刺猬**（韩龙）映射 `host:wudian_ciqiu`
——你的杀对满血目标伤害 +1；加后 ≥ 目标体力则目标获本回合濒死即死标
记（跳过救援直接死亡）且本技能移除（一次性）。未映射声明 9→8。前置：
[wudian 生命周期](../2026-10-08-u07-wudian-lifecycle/README.md)。

## 上游语义核验（源码为准，archive/qita/sanguosha/noname/extension/玩点论杀/character/skill.js:75）

- `ciqiu`：trigger source damageBegin1、forced、filter `event.card ==
  "sha" && event.player.isHealthy()`；content `trigger.num++`；若
  `num >= player.hp` → 目标 `addTempSkill("ciqiu_dying")` + 本人
  `removeSkill("ciqiu")`。
- `ciqiu_dying`：trigger dyingBegin firstDo forced silent →
  `player.die()`（濒死即死，不进入救援询问）。

## 改动（分开陈述）

- **代码已实现**：`HostBehavior::WudianCiqiu`（tag 97，权限型——
  activate 返回 NoopMachine；伤害改写走 commit_damage 钩：来源持有
  ciqiu 技能实例 && card==Sha && 目标满血 → amount+1；加后
  amount ≥ 目标体力 → `TurnState.ciqiu_dying` push 目标 + 来源技能实
  例移除）+ **DyingSaveMachine 即死分支**（dying 者标记在位 → 跳过
  救援询问直接 Death 事件，上游 ciqiu_dying firstDo die 同口径）+
  `TurnState.ciqiu_dying: Vec<PlayerId>`（回合级，换回合清空；入存
  档/哈希全套接线）。包 cards.json：韩龙 ciqiu mapping →
  `host:wudian_ciqiu`（skills 段登记），npack 三哈希重算。
- **登记差异**：上游 damageBegin1 时点（目标受伤前体力判定）与原生
  commit 钩一致；濒死即死无专用日志（Death 事件可观察，登记）。
- **键名口径**：将领 mapping 通道的目录技能名取 cards.json skills 的
  `name`（"ciqiu"），非行为键 "wudian_ciqiu"（后者仅为映射路由键）。

## 验收口径

- **回归已加入**：`hanlong_ciqiu_sha_healthy_bonus_and_dying_mark`——
  非满血不加成（3→2、技能保留）→ 满血目标 +1 且伤 3≥体力 2 → 濒死即
  死（跳过救援）→ 技能移除后再杀满血目标不加成（4→3）。
- **全量通过**：engine-native **428 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 110→111 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（刺猬不在标准场景，纯版本位移）。
- **剩余**：U07 未映射 8 条（率言/筑城/夺启/绝道/割据/抗yin/车阵/诱
  战）逐条评估转换。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（17 文件，相对 noname-next 根）。
