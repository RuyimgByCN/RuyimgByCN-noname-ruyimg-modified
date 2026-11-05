# R05-J/H 合同依据摘录（2026-11-04，源：docs/operations/evidence/2026-10-31-r05-runtime-contract/proposal-v3.raw.txt §7-§9＋v4/v5 勘误）

## H 投影批规范（v3 §8 原文要点）
- SeatView 增有序 `r05_effects:[{skill:QualifiedSkillRef, tiqi_limit, baoshu_combs, dcbihuo_next, dcbihuo_current}]`
  （Ref 仅 namespace/package_id/logical_skill_def_id；四值 None 或有符号；Combs 可负/存在 0）。
- 每项只在至少一槽存在时输出，按公开 I 稳定排序；Some0 或负 Combs 都不是空项。
- 本人/其他席位/观战/AI 见相同公开值；不输出 raw mark/helper IDs、VmAux、D.pending/历史。
- 死亡 Next 照显、disabled 保值。
- 涕泣 Own 选择 context=`TiqiAdjustment{skill:I,target,delta:u32}`，从已验证本次内容局部派生；
  其他人仅 Other 询问存在；不从 history 重算 d。
- 落点链：E/view.rs→engine/views.rs→wasm/session.rs view_to_json/open_choice_json→
  Worker nativeBackend.ts→engine-api SeatView/ChoiceRequest、server native_view.rs、web types。
- JSON null 与 signed 整数；target 沿 0基/API1基归一。
- WS8＝network_protocol 7→8（engine-api version.rs）。

## J 十二类验收场景（v3 §9 原文，全部 pending）
1. 首回合/额外回合 Begin/Prepare 与翻面跳回合；旧 phase_begin 包过渡≠旧 rule 恢复；新 event/ABI 旧包明确拒绝。
2. D 实际 0/1/2/3、skip/cancel/Omitted/死亡；fixed/普通 Draw/额外 D 边界与 signed pending；禁 handdelta/回合 drawn 替代。
3. 真实 caller 覆盖：xianjian_card_use 110、mtg 617、yunchou 293、phase/draw 165、tricks 18、wang_xi 57（C 批验收面）。
4. Filter/RecordHistory/Transfer/After/Aborted 逐边界恢复；c 入手清 held 后 After 挂起；eligible 历史保留；一 owner、一次 transfer/计数。
5. Tiqi 多 owner live 读/抵消 0、先 draw 再取消、owner 死 forceDie 选符号、target 离场不选、Limit 到期及负上限 clamp。
6. Baoshu 成本后 logSkillBegin 改 maxHp；m=0→+1、m<0 照加、正负抵消 Some0；≤0 不消费；座次/嵌套重置/取消不 doubletoggle。
7. Source 先 End、三点各一次、缺 source/自伤/redirect/链伤、可取消选自己或他人、成功凭据与窗资格分开。
8. source 在 damage/Source 子事件死亡后清除；victim 救回/死亡、num0/notrigger/转 LoseHp、armor 全吸收按独立决策。
9. Next enabled=true/false 各死→保存恢复→复活，enabled 原值不变；只有 alive 且 enabled 的真实 Begin 原子转 Current；当前回合新伤害仍写 Next。
10. disable/remove/expiry/death/replacement 四角色逐格；两个非 official 包 compound 不串槽；同 I 两 owner 合并 target 槽；非法 generic mark/grant 拒绝。
11. 所有等待点字节恢复/合法重答/分片 abort；完整对象、events、choice/context、RNG、ownership、绑定/游标相等；正确 After 不拒、伪 Transfer/错 I/错锁拒档。
12. native/真实 WASM/上游 JS 差分及同公开视图、私有牌不同的反事实；完整 Plan 及有效 apply 相等。
- 登记：新场景 misc；复用 nc_rounds 内 nc04_vm_resume/nc06_generic_content_state/nc10_j2_vm_privacy；autotests=false。
- golden：先解释完整对象/事件/RNG 首分岔与恢复归因，独立 review 后才重录（三步探针）。

## 版本轴（J 收口）
- 规则 332→333、NGSA 106→107（G 批时刻定布局）、ABI 758→759（先 ABI-COORDINATION.md 领取＋ABI-MAX 提 759）、WS 7→8。
- 升号在 J 步合入 main 时统一进行；基线 main 29712b2b（332/106/758/WS7）。
