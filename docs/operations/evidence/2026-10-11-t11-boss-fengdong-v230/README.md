# 2026-10-11 T11 应昭批首片：封冻（规则 v229+断链修正至 v230/存档 v53）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `714c4f39`；应昭批按指令最小切片——仅封冻单技，巡游/死缚(+switch) 登记下批）。上游依据：`mode/boss.js` `boss_fengdong`（5079：phaseBegin → 全员 `addTempSkill("fengyin")`）与 fengyin 定义（newjiang/skill.js 2794：mod cardEnabled2 手牌禁用，charlotte）。

## 建模

- **封印存储**：`PlayerState.fengyin_active: bool` 新字段——**存档格式 52→53**（PlayerState 尾 bool；canonical_hash 入参）。
- **封冻**两面（权限型 Noop+查询）：
  1. **回合开始授予**（Judge begin 段内联，神智锚）：持封冻者回合开始 → 全部其他存活角色 `fengyin_active=true`（自身不封）；
  2. **回合结束解除**（`Phase::End` begin 臂：全员清零）。
- **PlayCard 校验**（engine 校验段，冥顽同位）：被封印者使用**手牌区**牌 → `IllegalTargets` 拒（装备区牌不受限——上游 mod 仅 position=h）。
- FROZEN 尾加「封冻」（83 名）。

## 版本断链批合并录

本批同时补录 v228（红莲）/v229（炎狱）的断链跳号：**运行时版本 227→229**（v228 红莲批 bump 后 probe/restore 链停留 228——红莲批 golden 为 228 版+当时代码哈希自洽）。本批为**字段新增批**：PlayerState 字段入 canonical_hash 使全部场景哈希位移（含版本字段）——「位移唯一来源=版本字段」探针形态不适用，以 229 版+当前代码 14 全位移佐证 Boss 分支不可达。

## 验证

| 项 | 结果 |
|---|---|
| 回归 `boss_yingzhao_fengdong_hand_ban` | 应昭回合开始 0/2/3 号获封印（自身不封）→ 回合结束解除 → 解封后 2 号可用手牌 → v53 存档往返封印状态一致 |
| 既有回归 | 招祸/红莲/炎狱/巨源/相柳/梼杌/书精/四凶/关卡全部不变；native_kernel 232/0 |
| 14 golden 重录 | 字段批全位移佐证后 2/2 绿（重录依据注释入文件头） |
| 全量 | **577 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 227→**229**（合并录 v228 红莲）；存档 52→**53** |
