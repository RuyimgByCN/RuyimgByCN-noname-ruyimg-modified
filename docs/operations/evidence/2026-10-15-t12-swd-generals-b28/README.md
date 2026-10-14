# T12 swd 武将面批 28：破月——拓跋玉儿 2/3（黑杀无距/红杀无限/直击）（2026-10-15）

## 承载

**破月 poyue**（tag 288，键数 287→**288**；目录标记技+三内核/引擎钩，
poyue2 group 子技同承载）：

- 黑杀无视距离：engine.rs sha_range_free || 链追加（黑花色+持技；上游
  mod targetInRange black 同口径，焚天 red 变体同款）。
- 红杀不计入限一次：**双位豁免**——PlayCard 限一次检查不查（poyue_red_
  free）+kernel 计数不置位（上游 cardUsable Infinity+stat-- 净零同口
  径）。黑杀照常计数：后续黑杀仍受一杀限制。
- 红杀直击（poyue2 forced directHit）：跳过防具判定+整段跳过闪询问直接
  emit_sha_damage（焚天 directHit+哨卫直中两先例合成；首版仅过滤出闪选
  项仍开窗，二版改为整段跳过）。

**niepan 登记**：拓跋玉儿第 3 技=基础包庞统限定技，native 已登记外部包
技依赖不落（Boss 凤雏 6055 行同口径），cards.json 该技 mapping null+
reason 注。拓跋玉儿 **2/3 部分转正**。

golden 无位移（钩子全部持技门控，标准场景不可达）；无新字段、无版本
bump。

## 测试（swd_generals_b28_poyue_black_range_red_free_direct）

| 面 | 断言 |
|---|---|
| 正例① | 黑杀打距离 2 的 p2 命中（无距离技本应拒） |
| 正例② | 红杀连续两张均受理并命中（不计限一次） |
| 正例③ | 红杀对持闪 p1 直中且无出闪询问 |
| 反例④ | 黑杀照常出闪询问（respond_shan 窗在） |
| 反例⑤ | 撤技后黑杀打距离 2 被拒 |

测试侧勘误三处：② 段 p1 体力基线未设（濒死噪声）；③ 首版仅滤选项仍开
窗（directHit 应整段跳过）；Gate 拦冗余 clone（已修）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 g1_shahai_swd | **39/39** |
| engine-native 全量 | **758/0/0**（golden 不变）——[日志](engine-native-tests.log) |
| Gate 1b | 拦冗余 clone（已修）→ exit 0——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- swd 武将面：**21/79 名、26/241 技转正；整将维持十七**（拓跋玉儿 2/3，
  niepan 外部包技依赖）。
- 剩余部分转正：壶中仙 2/3（壶境挂卡牌面对账批）。
- 挂账不变：swd 卡牌面对账批（92 vs 45）。
- 批 29 候选：swd 卡牌面对账批（对账上游 92 条 vs 包内 45 张，产出补切
  清单；壶境随炼妖壶入包批承接）。
