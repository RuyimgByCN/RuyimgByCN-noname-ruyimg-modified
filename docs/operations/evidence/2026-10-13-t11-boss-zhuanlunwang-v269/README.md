# T11 转轮王片（规则 v269，2026-10-13）

## 任务来源与上游依据

[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 fde759ee 延续）。上游依据：`boss_zhuanlunwang`（转轮王·薛礼/male/qun/**hp6**/skills `[boss_modao, boss_lunhui, boss_wangsheng, boss_zlfanshi]`，apps/core/mode/boss.js:1386）；`boss_modao`@8882（共用技）/ `boss_lunhui`@3962 / `boss_wangsheng`@3991 / `boss_zlfanshi`@4006；显示名 boss.js:11068/11306–11311（魔道/轮回/往生/反噬——本片无显示名勘误）。

## 四技语义与原生落法

- **魔道**（共用技，直接复用 `BOSS_MODAO` def——夜叉/罗刹先例）：准备阶段摸 2。零新增代码。
- **轮回**（Noop 查询型，Judge-begin 臂）：准备阶段若体力 ≤2——与体力最高且大于 2 的一名其他角色交换体力值（hp 直写，双向 `min(maxHp)` 封顶同上游；体力最高并列时取座位序首个——`max_by_key` 平局取末位故显式「先求最大值再取首个」；无事件面直写=登记差异）。
- **往生**（Noop 查询型，Play-begin 臂）：出牌阶段开始视为随机使用【南蛮入侵】/【万箭齐发】——`use_virtual_card` 载体管线（乱剑/千方同口径）；randomGet→**恒定取南蛮**确定性近似登记；目标=全部其他存活（机器直发不经距离/目标数校验）。
- **反噬**（Noop 查询型，Damage commit 尾臂）：每回合第一次受伤后置 `zlfanshi_terra` 旗，同回合再次受伤 → 随机一名其他角色受 1 伤（极点同位；上游 addTempSkill boss_zlfanshi_terra 交替同口径；随机→首个近似）。旗清零=回合结束全清（封印解除同位）。新 `PlayerState.zlfanshi_terra`：canonical hash 推入 + 存档玩家段 enc/dec，`ARCHIVE_FORMAT_VERSION` 61→**62**。

条目 `("转轮王", Qun, &[魔道, 轮回, 往生, 反噬], 6, Male)`（BOSS_GENERALS[47]）；`STANDARD_RULESET_VERSION` 268→**269**。FROZEN 尾追加「轮回」「往生」「反噬」（157–159 名）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向回归 | `boss_zhuanlunwang_lunhui_swap_at_judge`（hp 压 2→与 0 号换血+魔道摸 2+往返）、`boss_zhuanlunwang_wangsheng_virtual_savage`（恒定虚拟南蛮，空手全 Pass→0/2/3 各 -1+守恒）、`boss_zhuanlunwang_fanshi_damage_toggle`（三连伤 1：首伤置旗/二伤反噬 0 号-1/三伤不再反噬）——3/3 绿 |
| 版本探针 | **字段批口径**：v268 提交时（无字段+268）全绿；268+字段与 269+字段两态全差且 actual 互异（版本位移可见）；按字段批全量重录 |
| 14 golden | 程序化行级重录 14/14，重录依据注释链追加 v269 字段批条目 |
| 全量 | **630/0/0**（627→630，新增 3 测） |
| Gate 1b | 独立 target 目录 `RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` **exit 0** |

## 已知边界与登记

- 轮回/反噬随机→座位序首个确定性近似（既登记差异族）；轮回体力直写无事件面；往生恒定南蛮、虚拟牌无 useCard 事件明细（use_virtual_card 载体口径）；反噬 terra 旗清零时点=当前回合结束（上游 addTempSkill 当前回合结束同口径）。
- **十殿阎罗 7/13**：楚江王/五官王/宋帝王/卞城王/泰山王/转轮王✓（+秦广王待查）；勘验完备片已全部落地。余：阎罗王/都市王/平等王/孟婆/地藏王（初判完，需逐片勘验落包）+庞统/诸葛亮专项+族级机制 3 项（需 D 级决策）。
