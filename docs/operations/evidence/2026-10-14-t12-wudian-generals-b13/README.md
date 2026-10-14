# T12 玩点论杀批 13：坚壁——AOE 目标排除原语首例（2026-10-14）

## 任务来源与承载

[T12 玩点论杀武将面](../../migration/upstream-extension-reconstruction.md)批 12 续批：
坚壁（西真夕红）的「目标排除/转移面」缺口按**通用原语**立项落地——

- **catalog 通用查询** `Catalog::aoe_excludes_target(core, target)`：目标持有的
  任一宿主技能声明 AOE 排除语义（`HostBehavior::aoe_exclude()` 标记，tag 258
  `WudianJianbi`）则 AOE 对其无效——**内核不点名扩展技能**（分层数据驱动）。
- **AOE 目标段跳过链**（kernel AoeResolveMachine）：`!alive || 热闷 || 帷幕 ||
  aoe_excludes_target`——帷幕 weimu_blocks_aoe（v275）同构第 2 例。
- **登记差异**：上游可选确认且覆盖全部多目标锦囊（含五谷类正面），原生锁定
  仅 AOE 面（南蛮/万箭）；五谷类排除归「正面锦囊排除面」后续。

## 过程要点

- 脚本 write-after-assert 事故一次（filter 锚失配致前 6 触点未写盘）——修复
  后改为宽容应用+逐项核验（SKIP 打印）。
- 测试布阵三勘误：目录列表=mapped 面（null 不进列表）；南蛮 kind key=
  "savage"；排除查询直断（grant 后立即 aoe_excludes_target）先于行为断言
  定位链漏。

## 实测记录

| 项 | 结果 |
|---|---|
| 反例实证 | AOE 跳过链桩化（`|| false &&`）→ 批 13 测试 **FAILED**（0/1） |
| 定向 | g1_wudian **120/120**（排除正例/无坚壁负例/目录面） |
| engine-native 全量 | **740/0/0**（golden 不变——查询面仅扩展持有者可达）——[日志](engine-native-tests.log) |
| 装载/包管理/服务端 | **84/0** |
| Gate 1b | exit 0——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- 玩点论杀武将面：**9/9 名、16/22 条目转正、6 暂缓**（六整将+西真夕红 2/3）。
- 剩余暂缓（6）：复仇双技（跨回合存储）、良策（五谷 Remained 子事件）、
  仇渡（deferred 卡依赖）、理断已转出、夺启/佑赞（存量）+坚壁上游全域面
  （正面锦囊排除）。
- 排除原语泛化：后续 AOE 排除类技能循 `aoe_exclude()` 标记+通用查询直接承载。
