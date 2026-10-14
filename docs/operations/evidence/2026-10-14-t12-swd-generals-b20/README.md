# T12 swd 武将面批 20：分舵——习嫣整将转正（2026-10-14）

## 承载

分舵 jiefen（习嫣第二技，出牌主动技每回合限一次）：**三段 HostMachine**——
JiefenTarget（选交牌目标：手>自己的存活角色）→ JiefenTargetDiscard（目标选
1 张手牌交出）→ JiefenOwnerDiscard（自己交 1 张手牌给手牌最少的其他存活角
色，确定性直发）。HostStage 枚举扩 JiefenTarget/JiefenTargetDiscard/
JiefenOwnerDiscard 三值（tag 7/8/9），limits once_per_turn（上游 usable:1）。

**登记差异**：上游 chooseCardTarget 玩家同时选牌与目标，原生两段询问+确定
性目标（自交段 first 直发）。

## 测试（三问显式段）

q1 目标选择（Player(1)）/q2 目标交牌窗（observer=p1）/效果断言（p1 3-1=
2、p3 +1-1=1、p0 +1=1）。布局勘误两轮：debug_place 本身已 push（去双重
push）、全清手（原只清 p0 致极差≠1）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 | g1_shahai_swd **31/31** |
| engine-native 全量 | **749/0/0**（golden 不变——主动技触发面标准场景无持有者）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0（拦未用 my/give 变量，已修）——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- swd 武将面：**8/79 名、8/241 技转正**（习嫣 2/2 整将；七部分+一整将格局
  不变+习嫣整将=八名有进展）。
- 批 21 候选： continuation——swd 剩余 71 名按将勘察。
