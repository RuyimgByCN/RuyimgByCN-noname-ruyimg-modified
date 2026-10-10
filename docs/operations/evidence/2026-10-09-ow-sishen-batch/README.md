# U05 ow 纵切批二十三：死神闪弹（规则 v140，2026-10-09）

ow 纵切批二十三：**死神**（ow_sishen，shu 4 男）——闪弹（xiandan，
ShaResolve 内联钩）+xiandan2（黑牌分支伤害 +1，随本杀一次性）单技闭
环；**yihun 暂缓**（虚拟 useCard——useCard({name:sha}) 于目标回合，
youzhan/huoyu 同款原语缺口）+ **shouge 暂缓**（createCard 治疗波——
牌种跨包依赖炉石）。上游 25 将已闭环第 22 将。前置：
[ow 纵切批二十二](../2026-10-09-ow-heiying-batch/README.md)。

## 上游语义核验（源码为准，…/character/ow/skill.js xiandan/xiandan2）

- `xiandan`：shaBegin direct——chooseToDiscard（1 张，可取消）→红
  （get.color==="red"）=trigger.directHit=true（直击）；黑=addTempSkill
  xiandan2 至 shaAfter（本杀内一次性）。
- `xiandan2`：damageBegin forced——杀非传导 num++。

## 改动（分开陈述）

- **代码已实现（内联钩）**：`ShaResolveMachine` 三字段
  （asking_xiandan/xiandan_done/xiandan_bonus；入 MachineSnapshot::Sha
  编解码+canonical_hash——存档 **v13→v14**）+ `!asked` 分支哨卫后插
  钩（持技判定 WUDIAN_XIANDAN_KEY+来源/目标存活+he 非空；每杀至多一
  次）→ 来源弃牌询问（he，min0 可取消）→ 应答臂（step 顶）：红=直中
  （emit_sha_damage 哨卫同款路径）；黑=置 bonus；取消=照常——**asked
  复位**落回 !asked 分支重开出闪询问（青龙再杀 reboot 同款，否则闪弹
  应答被门控应答区误读为闪）；**bonus 消费**：emit_sha_damage 伤害式
  加项（luoyi/jiu 同列）+发完即清。
- **目录标记技**：`WudianXiandan`（tag 139，无机器无触发——仅作目录
  技能实例供内联钩持技判定；mapping host:wudian_xiandan）。
- **登记差异**：xiandan2 的 damageBegin 触发形态折算为本杀 emit 时直
  接加项（同一杀伤害事件，观察等价）；xiandan 触发点=ShaResolve 防具
  /闪询问前（上游 shaBegin 每杀事件同口径）。
- **登记暂缓**：yihun（虚拟 useCard）、shouge（createCard）。
- **转换包**：cards.json 增 死神（shu 4 男，xiandan mapping+双暂缓
  reason）+ npack 资产哈希重算。
- **版本勘误**：首版全量绿但 golden 零位移——检查发现规则版本漏 bump
  （Sha 机器字段不进终态哈希，位移只能来自版本）——补 139→140 重录。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`sishen_xiandan_red_direct_black_
  bonus`（红弃直击 4→3/黑弃 +1 3→1/取消照常 2→1）。
- **全量通过**：engine-native **461 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v139 基线 460 +1）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 139→140 入 canonical_hash——14 场景全 mismatch，
  重录后 replay_scenarios 2/2。
- **测试坑（登记）**：ShaResolve 机器字段五处同步（struct/new/snapshot
  /archive 编解码/a0 样例+engine 哈希 M::Sha+standard new-call——第一
  轮半途失败致后三处未落，编译期全数暴露）；闪弹应答后必须复位 asked
  （青龙 reboot 先例）。

## 剩余

- ow 其余 2 武将（猎空/D.Va——区域换牌/机甲血条依赖重）逐武将纵切；
  xianjian 36 武将；开局事件基建、虚拟 useCard 原语、U07 duoqi、
  zhenfa 五种阵法长期暂缓。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（17 文件，相对 noname-next 根）。
