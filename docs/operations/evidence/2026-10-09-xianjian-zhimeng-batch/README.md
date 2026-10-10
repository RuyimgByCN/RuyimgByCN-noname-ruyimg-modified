# U05 xianjian 第二十六批切片（规则 v167，2026-10-09）

柳梦璃织梦（xjzhimeng）+织梦·梦（xjzhimeng2）闭环——**柳梦璃升全闭
环**（仅 tianxian 仍暂缓）。暗牌=离场区复用（v162 基建第二次消费）。
前置：[柳梦璃润心](../2026-10-09-xianjian-liumengli-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js xjzhimeng/xjzhimeng2）

- `xjzhimeng`：player phaseEnd direct——chooseTarget 一名其他角色 →
  牌堆顶一张暗置其 storage（mark card）+addSkill xjzhimeng2。
- `xjzhimeng2`：target useCardToBegin frequent silent——被使用同类型
  （get.type 三分类）牌时 draw 1（storage 不清除=暗牌保留）。

## 改动（分开陈述）

- **宿主行为**：`XianjianXjzhimeng`（tag 185，PhaseEnd direct 机内取
  消：发问其他存活目标→牌堆顶 pop 暗置目标 off_court+**runtime 授织
  梦·梦实例**（灵泉同款））+ `XianjianXjzhimeng2`（tag 186，CardUsed
  frequent silent 自动：暗牌持有者被使用同类型（yangpijuan_class 三分
  类）牌时摸 1，暗牌保留=上游 storage 不清除同口径；**登记差异**：自
  目标牌（无中类 targets 空）以使用者为隐式目标=上游 target 事件含自
  目标）——六表+ALL 184→186。无新状态字段（off_court 复用）。
- **cards.json**：柳梦璃 xjzhimeng mapping null→host；追加
  xjzhimeng2 常驻实例登记（runtime 授技 def 源）+ rehash 两步；**版
  本** 166→167；a0 追加 185/186。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`liumengli_xjzhimeng_darkmark_and_
  draw`（织梦询问→暗置牌堆顶至 1 号离场区+授梦实例断言→1 号自己使用
  锦囊（无中）→暗牌同类型则摸 1 否则仅无中 2）。
- **全量通过**：engine-native **497 passed / 0 failed / 1 ignored**
  （v166 基线 496 +1）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 166→167 入 canonical_hash——14 场景全 mismatch，
  重录后 replay_scenarios 2/2。

## 剩余

- xianjian 其余 12/36 将（柳梦璃 tianxian 单技暂缓；余将缺口族：虚拟
  用牌、createCard、moveCard/dieBefore、响应链、事件面、mod 系）；长
  期项不变。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（10 文件，相对 noname-next
根）。
