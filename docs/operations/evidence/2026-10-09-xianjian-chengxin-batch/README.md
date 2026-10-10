# U05 xianjian 第二十三批（规则 v164，2026-10-09）

**明绣诚信补完**（v160 暂缓消账）：诚信（chengxin，global Dying 确认
每 4 轮冷却）接入 v162 濒死救援通道（第二例）——明绣升**全闭环**。前
置：[小蛮切片](../2026-10-09-xianjian-xiaoman-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js chengxin/chengxin2）

- `chengxin`：round:4 enable chooseToUse（dying）——filterTarget
  target==dying；content target.recover(1-hp)（回复至 1）+addTempSkill
  chengxin2。
- `chengxin2`：damageBefore forced cancel——免伤持续至其阶段后
 （{player:"phaseAfter"}）。

## 改动（分开陈述）

- **状态**：`PlayerState.chengxin_active: bool`（免伤）+
  `chengxin_ready_round: u32`（round:4 冷却=触发后 round+4）。存档
  v29→**v30**。
- **免伤钩（kernel commit_damage 急冻同区）**：免伤状态目标受所有伤害
  无效（DamagePrevented+return，上游 damageBefore cancel 同口径）；
  **清钩**：commit_phase 本人 End 阶段结束（**登记差异**：上游
  chengxin2 存活至最近阶段结束）。
- **宿主行为**：`XianjianChengxin`（tag 181，global Dying 确认门=
  濒死救援通道第二例：确认后冷却置位+濒死者回复至 1（recover 量=
  1-当前体力，负体力一次拉回）+免伤置位）——六表+ALL 180→181。
- **cards.json**：明绣 chengxin mapping null→host:xj_chengxin（暂缓
  消账）+ rehash 两步；**版本** 163→164；a0 追加 181。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`mingxiu_chengxin_rescue_and_
  immunity`（-2 体力濒死→确认→回复至 1+免伤置位+冷却 round+4→免伤
  期受伤无效→本人 End 末清除→清除后照常受伤→冷却中无诚信询问、无桃
  死亡）。
- **全量通过**：engine-native **493 passed / 0 failed / 1 ignored**
  （v163 基线 492 +1）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 163→164 + 双字段入 canonical_hash——14 场景全
  mismatch，重录后 replay_scenarios 2/2。

## 剩余

- xianjian 其余 14/36 将（雪见/猎空离场区已通、柳梦璃 runxin 可做等）；
  changnian 死亡拦截、响应链、createCard、Recover 前置窗口等缺口族与
  长期项不变。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（14 文件，相对 noname-next
根）。
