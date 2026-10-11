# 2026-10-11 T11 朱厌批首片：兵先（规则 v224）+ 版本断链审计

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `d61a935b`；朱厌批按指令最小切片——仅兵先单技）。上游依据：`mode/boss.js` `boss_bingxian`（5812 区：global phaseJieshuBegin——`countUsed("sha",true)==0` → 视为对之用杀）与 `boss_zhuyan`（890：hp25/maxHp30/五技）。

## 建模

- `BOSS_GENERALS` 增**朱厌**（群/hp25/`[妖兽, 兵先]`）——**登记**：巨源（体力上限+5+selectTarget mod）/虚失(+switch)（出牌阶段结束自翻面+翻正时全场随机 1-2 伤——随机近似）未落随下批；maxHp30 差异登记同穷奇。
- **兵先**：`Phase::End` begin 臂查询式——他人回合结束 ∧ 其本回合未使用【杀】（**出牌阶段口径**=`turn.sha_used_play_phase`，登记差异：上游 countUsed 含响应杀）→ `use_virtual_card(holder, Sha, [player])`（乱常万箭同款）。
- FROZEN 尾加「兵先」（80 名）。

## **版本断链审计（本批重要发现）**

- **现象**：v224 bump 后 golden **未位移**（前例每批必移）；223 探针亦 FAILED。
- **根因**：v220–v223 四批的 bump/restore python 脚本链存在**断链**（restore 锚失效未写盘致运行时 `STANDARD_RULESET_VERSION` **停留 222**）——四批「重录」实为 **222 版+各批代码**的哈希（每次全量绿=自洽），但**名义版本号与运行时不符**（HEAD 提交 8ae71c2c 的 standard.rs=222 可证）。
- **修正**：本批运行时版本修正至 **224**；golden 重录至 224 值；探针口径变化如实登记（223 版+本批代码 14 全位移佐证 Boss 不可达）；各批**行为面回归不受影响**（各批测试以其代码状态运行并录制，570/0 绿过）。
- **遗留**：基准「逐版本语义表」v220–v223 条目附断链注记（本批已在基准 rationale 登记）；**流程教训**：bump/restore 必须以 grep 断言命中数=1 收尾，禁止静默 replace。

## 验证

| 项 | 结果 |
|---|---|
| 回归 `boss_zhuyan_bingxian_virtual_sha_on_no_sha_turn` | 未出杀回合结束被虚拟杀（4→3）；出杀回合 End 不触发 |
| 既有回归 | 兽九/毒妒/妖兽/梼杌/书精/四凶/关卡全部不变；native_kernel 226/0 |
| 版本回退探针 | 224→223：14 全位移（223 版+本批代码≠v223 批哈希——目录表序含各批 def 追加，探针口径变化已登记） |
| 14 golden 重录 | 全部更新后 2/2 绿 |
| 全量 | **571 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 **222（实际）→224**（断链修正+兵先）；存档不变 v50 |
