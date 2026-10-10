# U05 xianjian 第三十批（规则 v170，2026-10-09）

**死亡前置窗口基建落地**（台账待建排序②）+ 紫萱长念
（changnian，v162 暂缓消账）+长念·思（changnian2，runtime 授技）闭环
——紫萱升全闭环。前置：
[王小虎虎煞](../2026-10-09-xianjian-husha-batch/README.md)。

## 来源勘误

changnian/changnian2 属**紫萱**（pal_zixuan 第三技，v162 暂缓登记），
此前各轮收尾误记为柳梦璃——本批更正。

## 基建：死亡前置窗口（Death 帧 BeforeTriggers 起步）

上游 dieBegin/dieBefore 在死亡执行前触发。原生 Death 帧原起步
Intrinsic（DeathMachine 直接执行），无前置窗口。本批改造（TurnOverBefore
E03 同款四点）：

1. push_frame：Death 起步 Intrinsic→**BeforeTriggers**；
2. Ctx::emit（机器发射侧）Death 同步改 BeforeTriggers（**根因**：机器
   侧独立 stage 映射，漏改则 DyingSaveMachine 发射的 Death 绕过窗口——
   本批实测踩坑）；
3. next_stage：Death Before→After→Intrinsic→Done（After 沿用静默）；
4. plan_triggers：before 窗口 before_kind 特判 Death；AfterTriggers 对
   Death 静默（防遗言/拦截技重复入队，TurnOverBefore 同款）。

DeathMachine（Intrinsic）不变：cancelled 族（shiying 型）后续经
PlayerState 标记直读跳过；本批 changnian 为遗言型（不取消死亡，
DeathMachine 照常执行）。

## 上游语义核验（源码为准，…/character/xianjian/skill.js changnian/changnian2）

- `changnian`：player dieBegin direct——chooseTarget 一名其他角色 →
  移交 hej 全部牌（全部入手）+addSkill changnian2。
- `changnian2`：player phaseEnd forced——draw 1。

## 改动（分开陈述）

- **基建**：上述四点（resolution.rs push_frame/next_stage/plan_triggers
  + machine.rs Ctx::emit）。
- **宿主行为**：`XianjianChangnian`（tag 190，Death Before 窗口 direct
  机内取消：发问其他存活目标→移交 hand/equips/judge 全部入手（装备
  revoke 后）+runtime 授长念·思实例）+ `XianjianChangnian2`（tag 191，
  PhaseEnd forced：摸 1）——六表+ALL 189→191。
- **cards.json**：紫萱 changnian mapping null→host；追加 changnian2
  常驻实例登记 + rehash 两步；**版本** 169→170；a0 追加 190/191。
- 无新状态字段。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`liumengli_changnian_death_bequest`
  （1 号回合内击杀持技者→遗言询问→移交 2 张+死亡照常+授长念·思实
  例→目标清手后 End 末摸 1）。
- **全量通过**：engine-native **501 passed / 0 failed / 1 ignored**
  （v169 基线 500 +1）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 169→170 入 canonical_hash——14 场景全 mismatch，
  重录后 replay_scenarios 2/2。
- **测试坑（登记）**：①死亡帧机器发射侧 Ctx::emit 独立 stage 映射必
  须与 push_frame 同步改（本批根因）②求桃链 Pass 循环会误应答后续询
  问——按 prompt 过滤（dying_save 才 Pass）③击杀须发生在杀手自己回
  合内（死亡者卡 current 推进）。

## 剩余

- 月琦 shiying（死亡取消型：需 PlayerState 死亡拦截标记+DeathMachine
  直读跳过——窗口已通，单切可做）；xianjian 其余 11/36 将；长期项不变。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（12 文件，相对 noname-next
根）。
