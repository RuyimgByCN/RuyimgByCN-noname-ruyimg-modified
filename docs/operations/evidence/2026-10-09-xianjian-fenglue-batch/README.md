# U05 xianjian 第三十五批切片（规则 v177，2026-10-09）

罗招嫣（pal_luozhaoyan，shu 4 女）凤略（fenglue）单技闭环——真实牌自
动逐张使用（虚拟管线复用真实牌变体首例）；tanhua Recover 前置窗口缺
口登记暂缓。前置：
[探息批](../2026-10-09-xianjian-xtanxi-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js fenglue/tanhua）

- `fenglue`：player phaseUseBefore direct——chooseTarget 一名其他角色
  （可成为手牌目标者）→ 对其依次自动使用手中可指定其的牌
  （useCard 真实牌逐张；multitarget 牌跳过）。
- `tanhua`：recoverBefore forced——回复改摸 2×量+dying 时自移除——
  **Recover 前置窗口缺口暂缓**（Recover 帧无取消语义）。

## 改动（分开陈述）

- **宿主行为**：`XianjianFenglue`（tag 199，Play-begin 确认门 direct
  机内取消：发问目标（可取消=放弃）→收集手牌中可指定其的杀（距离
  内）与决斗→逐张 `emit CardUsed`（**真实牌变体**：commit
  take_hand_card 自取，机器侧不预取；逐张 Yield 后重查存活））——六
  表+ALL 198→199。**登记差异**：仅杀（距离内）与决斗自动使用，其余
  锦囊不自动=上游 targetEnabled2 全量收窄。
- **登记暂缓**：tanhua（Recover 前置窗口缺口）。
- **cards.json**：罗招嫣行（shu 4 女）+ rehash 两步；**版本** 176→
  177；a0 追加 199。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`luozhaoyan_fenglue_auto_use_sha_
  duel`（2 人局选目标→杀+决斗逐张自动使用（各 1 伤）、无中不自动、
  取消=放弃已由其余覆盖）。
- **全量通过**：engine-native **510 passed / 0 failed / 1 ignored**
  （v176 基线 509 +1）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 176→177 入 canonical_hash——14 场景全 mismatch，
  重录后 replay_scenarios 2/2。
- **测试坑（登记）**：①触发技在 Play-begin 的 AfterTriggers——
  advance_loop 停在首回合 Play，触发早已错过：需绕圈一圈带牌回到
  Play-begin ②emit CardUsed 的 commit 自取手牌——机器侧再 take 即双
  取 panic（虚拟用牌真实牌变体：机器不预取）③测试导航：Judge 起收
  束=5 次 NP 再确认。

## 剩余

- 罗招嫣半闭环（fenglue 闭环/tanhua 暂缓）。xianjian 其余 8/36 将按
  缺口族；长期项不变。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（10 文件，相对 noname-next
根）。
