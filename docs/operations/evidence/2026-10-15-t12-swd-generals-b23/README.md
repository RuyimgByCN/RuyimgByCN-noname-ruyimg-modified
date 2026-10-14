# T12 swd 武将面批 23：四将整将（平恤/还灵 + 义从/酒池 builtin）（2026-10-15）

## 承载

**四将全部补成整将**（部分转正 1/2 → 2/2）：

- **平恤 pingxu**（秦叔宝，tag 278）：无触发面标记行为——全局距离字段技，
  `computed_distance` 消费：场上有**任一存活持技者**时，攻击方无武器距离
  -1、防守方无防具距离 +1（谛听/义从同位追加；上游 mod globalFrom/
  globalTo 全局字段同口径）。
- **还灵 huanling**（轩辕剑侍，tag 279）：自己**任一阶段结束**（可选确认
  门）选一名其他角色，自己与其同时翻面（TurnOvered emit×2）；取消=双方
  都不翻。上游 `player: "phaseEnd"` 逐阶段均触发——原生 PhaseEnd 帧同口
  径（蝶舞先例），非仅 End 阶段。
- **义从/酒池 builtin 引用**（张高/程咬金，零新增行为）：builtin 白名单
  `BUILTIN_SKILL_REFS` 2→4（+义从/酒池），cards.json mapping
  `builtin:义从`/`builtin:酒池`——**键须用标准目录中文名**（首版误写拼
  音 `builtin:jiuchi` 被装载器 UnknownBehaviorKey 拦截）。

HostStage 复用 Act/TargetChoice，无存档 bump；键数冻结表 277→**279**。

## 过程勘误（测试侧三处）

1. 平恤第 3 步 `equips.clear()` 把防具牌逐出所有区域——save/restore 守卫
   `card_missing_from_all_zones` 拦截；改为卸牌回弃牌堆。
2. 还灵窗序=可选触发门（trigger_confirm）先于选人窗——首版把确认窗当选
   人窗解析致目标面为空。
3. 弃牌阶段 `discard_to_limit` 询问先于阶段末触发窗——清手规避后再测。

## 测试（swd_generals_b23_pingxu_huanling_builtin_four_full）

| 技 | 正例 | 反例 |
|---|---|---|
| 平恤 | 净持平（-1+1）/守方有防具仅-1/攻方有武器仅+1（卸防具回弃堆） | 撤销持技者回基线 2 |
| 还灵 | Play 末确认+选 p1 → 双翻面 | Discard 末取消不翻；撤销后无窗 |
| 酒池 | 黑桃手牌当酒（jiu_used 置位） | 第二坛拒 |
| 义从 | hp≤2：攻距 2→1、被算 1→2 | —（标准技既有语义，wiring 断言） |

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 g1_shahai_swd | **34/34** |
| engine-native 全量 | **753/0/0**（golden 不变）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- swd 武将面：**19/79 名、19/241 技转正；整将 9→13**（习嫣/韩罗/张高/
  程咬金/秦叔宝/轩辕剑侍 + 原 8 名中已整将者；本批新增 4 名整将）。
- builtin 白名单随标准技实现继续扩充（护驾/谦逊/义从/酒池 → 4 项）。
- 批 24 候选：剩余 60 名按将勘察，逐技回读上游核验。
