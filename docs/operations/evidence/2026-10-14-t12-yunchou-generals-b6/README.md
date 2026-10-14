# T12 运筹武将面批 6：厉焰转化 + 厉焰·链杀（陆抗转正，2026-10-14）

## 任务来源

[T12 运筹武将面](../../migration/upstream-extension-reconstruction.md)批 5 续批：
厉焰（luweiyan）为批 5 勘察定案的「可立项」项（viewAs 转化循国色先例）。

## 承载

| 技能 | 上游 | 原生承载 |
|---|---|---|
| 厉焰 luweiyan（usable:1 主动 viewAs） | 非基本牌当【水攻】使用 | **PlayCard 门转化分支**（engine.rs，国色/武圣/奇袭/探囊/酒池同族第 6 例——skill_by_name("ycnluweiyan") + 非基本判定 → kind=Shuigong）+ 目录锚行为 `YunchouLuweiyan`（tag 250，trigger_meta None） |
| 厉焰·链杀 luweiyan2（group 子技同授） | 厉焰的水攻用牌后对目标询问视为用杀 | `HostBehavior::YunchouLuweiyan2`（tag 251）：CardUsed 触发 + **结算种判定**（`u.use_as` 优先——转化牌目录种非水攻）+ optional 确认门 + 座位序首目标 `use_virtual_card(Sha)`（登记差异：上游限厉焰转化的水攻+玩家选目标+可取消；原生按持有者全部水攻+自动首目标） |

谦逊 qianxun 仍为内置引用缺口（mapping null）。

## 关键语义点（回归驱动）

- **转化牌的 CardUsed kind 必须查结算种**：`u.use_as.unwrap_or(目录种)`——
  厉焰转化的水攻其目录种是无中生有，filter 首版查目录种恒否（回归抓漏）。
- **链在 submit 结算链内完成**：确认门→SkillTriggered→虚拟杀→响应窗→伤害
  →无懈窗全在一次 submit；断言基准必须读在 PlayCard 前（事件流水实证：
  trigger_confirm→SkillTriggered→CardPlayed(虚拟杀)→respond_shan(pass)→
  Damage→nullify_window 全链）。

## 实测记录

| 项 | 结果 |
|---|---|
| 反例实证 | filter 桩化+转化分支禁用 → 批 6 测试 **FAILED**（0/1） |
| 定向 | g1_shahai_yunchou **19/19** |
| engine-native 全量 | **733/0/0**（golden 不变——PlayCard 分支仅扩展包技能持有者可达）——[日志](engine-native-tests.log) |
| 装载/包管理/服务端 | **84/0** |
| Gate 1b | exit 0（拦提取臂未用变量+helper 死代码，已合并）——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- 运筹武将面：**10/10 名入册、11/21 条目转正**（陆抗 2/3——谦逊内置引用
  暂缓；勘正：条目 21=此前 20+厉焰·链杀独立条目）勘正二：上游条目 20 不变
  （luweiyan2 为 group 子技），**转正 10/20、暂缓 10**。
- 剩余暂缓全部挂机制缺口：出牌主动技面（隙焰/野升）、内置引用（悲歌/谦逊/
  护驾）、濒死主动技面（归汉）、max_hp 减面（蝉缘）、骰子+多步主动技（巧思）、
  外部包依赖（精械）——均按需立项。
