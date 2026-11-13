# L10 党锢：实际失败之后的逐行替代核对

2026-11-14（本地时区），HEAD `605201f3f49e10cea61c31d52a61bbb66c85599d` + 主 agent 的酒诗两窗恢复修复，规则333 / NGSA107 / schema5 / ABI759 / WS8。本文只核对原始 `extension/史诗卡牌/extension.js` 与原生实现，没有执行源技能的替代函数；不签等价。

真实入口与失败见 `browser-round4/upstream-k_A.json`、`upstream-k_B.json`、`upstream-k_cancel.json`。三个 fresh 页均经真正 `phase()` → `phaseBefore` → `arrangeTrigger` → `trigger(k_danggu)` → `k_danggu step0`。首个失败位置是 `dui.element.create`，不是“主动暂留”。`precontent()` 已创建真正 EpicFX UI 类；`content()` 在十周年 UI 未启用时返回。没有加载十周年 UI、官方 Spine 或上游打包 Spine 运行库。直接调用真正 `EpicFX.initSCSAnim()` 时，在定义 `getEPICscs` 之前的 `load()` 又因 `dcdAnim` 未定义失败，完整 CDP exceptionDetails 已保留。

以下“未运行”指本轮实际没有越过原始 UI 依赖，不把原生 A/B 当成上游 A/B。

| 原始上游位置与步骤 | 原生位置及本轮核对 | 边界/差异 |
|---|---|---|
| extension.js:2278 trigger `enterGame` 或全局 `phaseBefore`，2282 filter 在 phase 时只收 phaseNumber=0，2328 forced | filtering/common.rs:1047 仅owner `Phase begin`，inited=0或Prepare returning | 表达不相同；本轮只观察初始source真实触发，以及phaseNumber=1/无技能的两个不触发反例。未验证enterGame/他人首回合差异。 |
| extension.js:2407–2433 step0建立个人scsPage、真正scsDialog，调用dui与getEPICscs，然后初始化十人storage | skills_recovery.rs:338初始化inited与0x3ff roster，使用已有flags | source实际到UI创建即失败，未执行十人storage初始化；原生十人初始化已执行并存档恢复。 |
| extension.js:2453从storage随机移除首常侍 | skills_recovery.rs:452–468从mask收集、RuleRng挑first并移出 | 原生seed=202611140010的first=8；source未到此步骤，未对齐Math.random/RuleRng，不据随机结果不同宣称规则差异。 |
| extension.js:2479–2486若余表有高望，随机取其他三名、添高望、洗序，否则取四名 | skills_recovery.rs:472–484先抽四，再必要时把第四替成高望 | 两算法步骤/候选顺序不同；未运行source随机选择，未比较概率或RNG游标。 |
| extension.js:2370 conflictMap；1898 checkConflict只查`map[source]`；2520本地UI锁冲突者 | skills_recovery.rs:485–490过滤；562 `min/max`对称冲突 | 静态具体差异：首8郭胜只在源拒绝7段珪，源允许9高望；原生还拒绝9。原生本seed确为首8，source未越UI，不称runtime复现该差异。 |
| extension.js:2564本地全部锁定则全部解锁；2572–2594远端chooseButton(true)，无合法者随机开放一个 | skills_recovery.rs:490无合法则全部候选，500强制PickTag、不可取消 | 原生A/B/空答复执行并NGSA恢复；source本地与远端选择分支均未执行。原生取消被拒绝不证明source实际取消行为。 |
| extension.js:2544–2567单击选中、再击确认，不能点锁定候选；没有取消控件 | 原生`k_danggu_partner` Tag选择，空答复InvalidChoice保持pending | 原生A=Tag0/active_mask257，B=Tag1/active_mask258；只有原生结果，未有source伙伴UI与选择截图。 |
| extension.js:2598–2634取first/chosen、移除chosen、保存current、按changshi表addAdditionalSkill两项 | skills_recovery.rs:515–535去chosen、保存active_mask并调用grant helper；605仅catalog有该技能def时实际insert_skill，另设l10_scs flags | source未运行到授技；原生本批只验证选择状态与恢复，不验证十名伙伴的完整能力或包依赖安装组合。不能凭flags或未见SkillInstance就下“无消费者”结论。 |

恢复本身没有采用新字段、新状态、新版本或放宽校验。原生三场共6次 pending 编解码恢复成功；A/B续行状态不同，取消仍pending。两个反例无伙伴询问；source正常回合随后摸2张而原生只推进一个阶段，后续手牌不比较。
