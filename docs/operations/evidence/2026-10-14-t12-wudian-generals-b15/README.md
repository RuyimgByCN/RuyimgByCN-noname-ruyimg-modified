# T12 玩点论杀批 15：良策（造之仁君整将转正，2026-10-14）

## 承载

| 面 | 上游 | 原生承载 |
|---|---|---|
| viewAs 转化 | 基本手牌当【五谷丰登】（usable:1） | PlayCard 独立转化分支（**独立于杀转化链**——链头条件 `kind != Sha` 使 sha 牌不可达链内分支，首版误挂链内回归抓漏）；登记差异：不限次 |
| 五谷剩余归获 | wuguRemained 子事件：选目标获得剩余牌 | `WudianLiangceRemain`（tag 261）`wugu_remain_gain()` 标记 + **catalog 通用查询** `wugu_remain_beneficiary`（坚壁排除原语同款模式）+ 五谷机器 `finish_wugu` 剩余牌归持有者（登记差异：上游玩家选目标，原生归持有者本人） |

## 关键根因（回归驱动）

首版转化分支误挂 `kind != Sha && kind != Le` 链内——sha 牌**永不进入**该链
（链头条件即排除 Sha）——转化静默失效后 push 走无目标锦囊 `card_use`（不带
use_as）→ 机器工厂回退**目录种** Sha → "sha has one target" panic。修复两处：
①独立转化分支（转化链入口前）；②无目标锦囊 push 改 `card_use_as` 带 use_as
声明（否则良策转化的五谷目录种是基本牌，工厂回退错误）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 | g1_wudian **122/122**（转化受理+五谷择牌+剩余归获链） |
| engine-native 全量 | **738/0/0**（golden 不变——转化仅扩展持有者可达；剩余归获路径五谷场景标准目录无良策持有者）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0（拦探针残留，已清）——[日志](gate1b-all-targets.log) |

## 登记边界

- 玩点论杀武将面：**9/9 名、19/22 条目转正、3 暂缓**（**八整将**：+造之仁君；
  西真夕红 3/4 缺坚壁全域面、杨仪/田豫/刘孚部分）。
- 良策 usable:1 不限次（厉焰同款登记）；wuguRemained 选目标近似归持有者。
- 仇渡（deferred 卡依赖）、夺启/佑赞（存量）仍暂缓。
