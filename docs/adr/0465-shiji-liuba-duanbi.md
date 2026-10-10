# ADR 0465：SHIJI 刘巴闭环（断笔，ABI 445）

本批发布刘巴（`liuba`，4129758211，男/蜀/3 血）完成闭环：「断笔」（`duanbi`，
ABI 445）与既有「通夺」（ABI 442）作为完整武将进入选将池。
旧 JavaScript 仅作语义来源。

## 决策

- **ABI 445 `resolve_duanbi(player, targets, stage, skill)`**：限定技
  （DSL `active play_phase once_per_game targets 1 255 other`）——
  stage 0 校验全场手牌总数 > 人数×2（不满足即空转），随后逐目标强制弃
  min(3, floor(手牌/2)) 张（`SelectCards` 强制选择，被弃牌记录到 blob
  "DB1"）；目标队列携带在 choice payload（`[MAGIC, stage, player, skill,
  targets...]`，续体 handler 逐个推进并防止首个目标重复入队）；全部弃完后开
  "选人获牌" choice（可取消）——从 blob 记录筛出仍处弃牌堆的牌，`RuleRng`
  权威随机取至多 3 张移入选中角色手牌。

## 验证

- `original_shiji_zhengsu.rs` 新增第 13 个测试：全场手牌满足条件 → 首个目标
  强制弃 3 张 → 选人获牌 → 至少获得 1 张仍处弃牌堆的被弃牌。
- 打包：91 技能 / 29 武将，`rule_vm_abi_version` 502（max 语义）。
- conformance 3/3 通过；我的 5 套件 28 测试全绿。
