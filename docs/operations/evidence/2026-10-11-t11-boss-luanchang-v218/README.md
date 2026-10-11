# 2026-10-11 T11 boss 内容第五批续片：梼杌乱常（规则 v218）——梼杌四技位全齐

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `e2adfb61`；梼杌批续片按指令最小切片——仅乱常(+switch)，冥顽[目标锁定集] 登记下批）。上游依据：`mode/boss.js` `boss_luanchang(+switch)`（5771/5594：phaseZhunbeiBegin 视为用【南蛮入侵】+phaseJieshuBegin 视为用【万箭齐发】；switch changeHp hp≤max/2 enableSkill）。

## 建模

- **乱常**三件：①**激活门控**（`commit_damage` 尾第五并列臂：体力首次≤max/2 → `SkillInstance.enabled` 直写+SkillToggled；装配即未启用=apply 特判扩隐贼/延妒/积财/乱常；触发计划层过滤 disabled——门控即触发开关）；②**准备阶段虚拟南蛮**（触发技 PhaseBegin forced + `LuanChangMachine` 一步：`use_virtual_card(owner, SavageAttack, 全其他存活)`——虎煞虚拟南蛮先例同款，载体=牌堆底实体牌、`use_as=SavageAttack`）；③**结束阶段虚拟万箭**（kernel `Phase::End` begin 臂查询式：enabled 持有者 → `use_virtual_card(owner, ArcheryAttack, 全其他存活)`）。
- 新 `MachineSnapshot::BossLuanchang { owner }` 变体：archive tag **63**/canonical_hash **64**（两空间独立）/快照校验/恢复臂 None/a0 样例尾+variant_index 61。
- FROZEN 尾加「乱常」（73 名）。

## 登记差异

- `boss_luanchang_switch` 独立 switch 技不落表（门控并入主技 enabled，隐贼/延妒/积财先例同构）。
- 虚拟 AOE 目标=全部其他存活角色（上游 `canUse(nanman/wanjian, current)` 过滤——原生 AOE 响应面天然承载合法性）。
- **冥顽未落**（下批）：回合内用牌伤害后目标锁定集（storage）+每用牌摸 1+playerEnabled 目标 mod——涉及 PlayCard 目标校验挂点与回合内 storage 面。

## 验证

| 项 | 结果 |
|---|---|
| 回归 `boss_taowu_luanchang_virtual_aoe` | 装配即未激活 → hp 置半线一次伤害激活 → 准备阶段虚拟南蛮+结束阶段虚拟万箭（**日志佐证**：载体 card97=fire_sha 的南蛮 AOE——2 号弃杀响应、0/3 各-1；载体 card49=tao 的万箭 AOE——0/3 各-1；响应打杀/闪分布正确）——方向性断言（End 帧后 resolution 自动续跑多轮，贪欲 v215 同款不可停帧） |
| a0 守卫 | 62 变体穷尽+tag 唯一/往返 |
| 既有回归 | 拟态/饕餮/穷奇/混沌/烛阴/关卡全部不变；native_kernel 220/0 |
| 版本回退探针（218→217 复跑 golden） | 2/2 全绿（位移唯一来源=版本字段） |
| 14 golden 重录 | 全部更新后 2/2 绿（重录依据注释入文件头） |
| 全量 | **565 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 217→**218**（存档不变 v48） |
