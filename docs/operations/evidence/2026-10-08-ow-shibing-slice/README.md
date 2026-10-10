# U05 ow 纵切首切片：士兵76（规则 v118，2026-10-08）

ow（轩辕剑外传…实为守望先锋联动）武将子目录**纵切首切片**：新转换包
shaai-ow-upstream-noname + 士兵76 双技——**突击**（tuji：锁定技，回合
内每已使用一张牌进攻距离 -1）+ **目镜**（mujing：出牌阶段限？上游无
usable——将一张黑色牌当【杀】使用/打出，原生仅出牌入口）。力场
（lichang）暂缓（下准备阶段子回合时点未建模）。character 管线首例：
general_meta 复用 + mapping 通道（无实体牌）。前置：
[U07 收尾](../2026-10-08-u07-deferral-closeout/README.md)。

## 上游语义核验（源码为准，archive/qita/sanguosha/noname/extension/杀海拾遗/character/ow/）

- `character.js` ow_shibing：group shu、hp 4、skills
  ["tuji","mujing","lichang"]。
- `skill.js tuji`：mod globalFrom——`_status.currentPhase === from` 时
  `distance - from.countUsed()`（回合内每已用牌进攻距离-1）。
- `skill.js mujing`：enable chooseToRespond/chooseToUse、filterCard 黑
  色、viewAs sha——黑色牌当杀使用或打出。
- `lichang`：phaseEnd 弃红牌 → 下个准备阶段令距离 1 内角色回复 1 或摸
  2（子回合时点）——暂缓。

## 改动（分开陈述）

- **代码已实现**：**character 管线首例**——新转换包（无实体牌、仅
  generals mapping：士兵76 tuji/mujing 映射 + lichang 暂缓 reason）+
  `WudianTuji`（tag 104，权限型 Noop）+ `WudianMujing`（tag 105）三段
  机（黑牌 → 目标 → declared-Sha 塔击同构）+ **状态**：
  `TurnState.cards_used_play_phase: u32`（Play begin 清零、
  commit_card_used 非取消 +1；入存档/哈希全套）+ computed_distance 突
  击钩（持有者+当前回合+计数>0 → minus+=计数）。snap 派生两段。
- **登记差异**：mujing 上游 chooseToRespond/chooseToUse 双入口——原生
  仅出牌入口（丈八/塔击同口径）；tuji countUsed=出牌阶段任意牌使用数
  （取消不计）。lichang 暂缓（下准备阶段时点未建模）。
- **转换包**：shaai-ow-upstream-noname（cards 空、generals 1 名——
  纯武将包先例：cards 数组空 + generals mapping）。

## 验收口径

- **回归已加入**：`ow_shibing_tuji_and_mujing`——目录技能断言（tuji/
  mujing；lichang 暂缓未映射不入目录）→ 目镜当杀（黑牌 → 2 号掉血
  4→3）→ 突击计数（用 1 张攻距 2→1）→ 换回合计数清零回基线。
- **全量通过**：engine-native **436 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 117→118 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（ow 不在标准场景，纯版本位移）。
- **剩余**：ow 其余 23 武将/105 技能、xianjian 36 武将逐武将纵切（待
  排）；U07 duoqi/youzhan 暂缓；zhenfa 五种阵法。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（16 文件，相对 noname-next 根）。
