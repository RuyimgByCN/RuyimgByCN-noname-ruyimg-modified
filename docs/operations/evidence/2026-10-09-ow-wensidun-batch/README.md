# U05 ow 纵切批二十：温斯顿跳跃（规则 v137，2026-10-09）

ow 纵切批二十：**温斯顿**（ow_wensidun，shu 4 男）——跳跃（feitiao，
出牌阶段主动无次数限制）单技闭环；**dianji 暂缓**（viewAs 惊雷杀——
jingleishan 牌种不在原生目录+距离 2 限定）。上游 25 将已闭环第 19 将。
前置：[ow 纵切批十九](../2026-10-09-ow-orisa-batch/README.md)。

## 上游语义核验（源码为准，…/character/ow/skill.js feitiao/feitiao2）

- `feitiao`：phaseUseBegin direct（无确认）——chooseCardTarget（弃 1
  张 he+选一名其他角色）→ 弃牌+目标 randomDiscard()+自己获 feitiao2
  （与该目标距离视为 1 至回合结束）。无 usable 限制（同回合可多次）。
- `feitiao2`：mod globalFrom -Infinity（与 storage 目标距离无限近）。

## 改动（分开陈述）

- **代码已实现**：`TurnState.feitiao_target: Option<PlayerId>`（回合级
  存量；canonical_hash+存档 **v10→v11**；commit_turn_ended 两分支显式
  复位——v128 纪律）+ `WudianFeitiao`（tag 135，usable 无限制）两段询
  问机（弃 1 张 he 必选→选目标必选→弃置+目标 he 池随机弃 1+置跳跃目
  标）；**距离短路**：computed_distance 开头 `a==当前回合玩家 &&
  feitiao_target==Some(b) → 1`（链钩同款；上游 -Infinity 钳距同口径）。
- **登记差异**：chooseCardTarget 复合询问拆两段（先牌后目标，链钩同
  型）；-Infinity 经距离下限钳 1（路霸同口径）；主技无次数上限同上游。
- **登记暂缓**：dianji（惊雷杀牌种缺失）。
- **转换包**：cards.json 增 温斯顿（shu 4 男，feitiao mapping+dianji
  暂缓 reason）+ npack 资产哈希重算。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`wensidun_feitiao_range_and_random_
  discard`（无跳跃时距离 2 杀被拒/跳跃弃牌+目标随机弃 1/距离恒 1 杀
  可达+命中/无次数限制再发动/换回合解除）。
- **全量通过**：engine-native **458 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v136 基线 457 +1）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 136→137 + TurnState.feitiao_target 零值字段入
  canonical_hash——14 场景全 mismatch，重录后 replay_scenarios 2/2。
- **测试坑（登记）**：两段询问机的目标应答落到空 Act 臂（v128 链钩同
  病二次）——按「挂起时 stage 已前移到应答臂」约定重构（TargetChoice
  收牌→Act 收目标并结算）。

## 剩余

- ow 其余 5 武将（猎空/死神/D.Va/查莉娅/黑影）逐武将纵切；xianjian
  36 武将；开局事件基建、虚拟 useCard 原语、U07 duoqi、zhenfa 五种阵
  法长期暂缓。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（17 文件，相对 noname-next 根）。
