# U05 swd 第五批：天仙酒（规则 v89，2026-10-08）

swd 子包第五批转换：天仙酒（♥9/♦2 两张，基本牌）——酒的 swd 变体。
前置：[swd 第四批](../2026-10-08-u05-swd-batch4/README.md)。

## 上游语义核验（源码为准，card/swd/card.js）

`tianxianjiu`：type basic、toself、`savable: dying === player`（濒死
自救）、`usable: 1`（每回合一次）；content——濒死时 recover，否则
addTempSkill（下张杀伤害 +1，至 phaseAfter/shaAfter）。与酒同构。

## 改动（分开陈述）

- **代码已实现**：`CardKind::Tianxianjiu`（编码 108：archive enc/dec、
  a0 variant/ALL 同步）；intrinsic/PlayCard/is_tao_like 三处与 Jiu
  **同臂**——濒死自救、下张杀 +1、每回合一次。
- **登记差异**：与酒共享每回合计数（上游 tianxianjiu 独立 usable:1，
  原生 jiu_used 单旗标共享）；如需独立计数须增 per-kind 回合旗标。
- **转换包**：cards.json 增 ♥9/♦2 两张天仙酒 + 哈希重算（现 11 张）。

## 验收口径

- **回归已加入**：`swd_tianxianjiu_ameisha_and_self_rescue`——天仙酒
  后杀 2 伤（6→4，含濒死救援排空）、同回合第二张被拒（CardLimit）。
- **全量通过**：engine-native **404 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**，DEPS_EXIT=0。
- **golden**：版本 88→89 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（纯版本位移）。
- **剩余**：swd 其余 11+ 卡种逐批转换。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
