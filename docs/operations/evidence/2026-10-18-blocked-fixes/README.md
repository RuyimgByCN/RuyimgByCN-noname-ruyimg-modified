# 待实施的修复清单（受阻于**未跟踪的 host/kernel 拆分**）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08 / NC09 / NC12**（跨工作包）
- 规则版本 v322（工作树）/ 存档格式 v105
- 背景：`crates/engine-native/src/content/host/**`（含 `step/*.rs`）与 `crates/engine-native/src/kernel/**`
  （含 `events.rs`、`equipment.rs`）**均为未跟踪文件**（他人 host.rs→host/ 模块拆分的在途成果）。
  按「只 `git add` 自己改动的文件」，本 agent **不把他人未提交改动并入自己的提交**，因此下列修复
  **已完成定位与修法设计，但未实施**；待拆分落定（或由拆分作者）后按本文逐条应用即可。

## 1 悬空引用 panic（NC12 长程，146 次被捕获故障）

**症状**：`machine.rs:408 card N not in hand of P`（如 `card 93 not in hand of 0`），
发生在 `ai::plan_action` 的**模拟提交**内；真实提交同样会 panic（防护仅捕获模拟/探测面）。

**根因（已钉到函数）**：`kernel/equipment::commit_equip` → `take_hand_card` 找不到牌；
回溯链 `commit_equip ← commit_frame ← resolution::run ← finish_segment ← submit`。

**因果**：`content/host/step/swd_card_effects.rs` 的 `SwdZhongxinghujiaCard` 效果在**同一步**内
① 发现防具实体 `x` → `hand.push(x)` → `emit(EventData::Equipped{player: t, card: x})`（**排队帧**）；
② 随机弃置目标一张手牌（`hand[k]`）→ 可能选中**刚放入的 `x`**；
③ `Equipped` 帧提交时 `x` 已不在手牌 → panic。

**修法（二选一）**：

```rust
// 方案 B（最小改动，推荐）：随机弃置候选排除刚放入的 x
let hand: Vec<CardId> = ctx
    .core
    .player(t)
    .hand
    .iter()
    .copied()
    .filter(|c| *c != x)          // ← 排除刚装配的防具
    .collect();
if !hand.is_empty() {
    let k = ctx.core.rng.gen_range(hand.len() as u32) as usize;
    let c = hand[k];
    ctx.discard_from_hand(t, c);
    ctx.log(LogEvent::CardDiscarded { player: t, cards: vec![c] });
}
```

```rust
// 方案 A：把「随机弃置」整体移到发现/装配之前（需按上游同族顺序核对后再选）
```

**验收**：`cargo test --offline -p engine-native --test nc12_dangling_reference_repro -- --ignored`
（seed 44170）应转为通过；长程 `sim_faults`/`probe_faults` 与两个故障日志应保持空。

## 2 `rangjie`（让节）「移动牌」分支漏摸 1 张（NC08 样本 8）

**上游**（`史诗卡牌/extension.js`）：非取消路径在**分支前**无条件 `player.draw()`。

**修法**：`content/host/step/skills_cards.rs` 的 `HostStage::SelectNature` 分支：

```rust
1 => {
    ctx.draw_cards(self.owner, 1);   // ← 补：上游 player.draw() 在分支前无条件执行
    // …既有「移动牌」逻辑
}
```

**验收**：`cargo test --offline -p engine-native --test nc08_upstream_rangjie -- --ignored`
（`upstream_rangjie_move_branch_must_draw_one`）移除 `#[ignore]` 后应通过。

## 3 五行盘 `usable:1` 失效（NC08 样本 6 附带发现）

**上游**：`usable:1` 由 `player.getStat` + 阶段重置强制（`library/index.js`、`player.js`）。

**修法**：`content/host.rs` 的 `host_skill_def` limits 分支加入
`HostBehavior::ChangeCardNature`（该技仅出牌阶段可用，按回合等价于按阶段）。

**验收**：同阶段二次发动应被拒（探针实测当前可二次发动）。

## 4 说明：不受阻的相关项

- `xicai`（戏才）**行为面夹具**：属**测试侧**改动（`crates/engine-native/tests/**`，可自行提交），
  与拆分无关；当前以 `#[ignore]` 守护；
- 灵雪（`lingxue`）行为面：需改走**真实回复路径**（测试侧），根因见
  [nc08-upstream-xianjian 证据](../2026-10-18-nc08-upstream-xianjian/README.md) §5。

## 5 应用顺序建议

1（panic，影响长程稳定性）→ 3（规则面损失，一行）→ 2（规则面损失，一行）→ 4（测试侧夹具）。
应用后请同步：进度主稿 NC08/NC12 行、规则基准差异登记、以及各样本证据中的 `#[ignore]` 说明。

## 5 集智（`jizhi`）覆盖范围差异（NC08 样本 19 新发现，2026-10-19）

**症状（规则损失）**：上游标准面 `jizhi` 的 filter 为 `get.type(event.card) === "trick"`（**任意锦囊**），
而原生 `crates/engine-native/src/content/standard/skills/ji_zhi.rs` 的 filter **只列 5 种**：
`SavageAttack | ArcheryAttack | Duel | Chain | Nullify` → 使用【无中生有】【桃园结义】等**不触发**集智
（实测：【无中生有】手牌净 +1，而按上游应为 +2）。

**修法**：把 filter 改为「使用普通锦囊（`CardKind` 为锦囊类）即触发」——需按目录的锦囊类集合实现
（`catalog` 中锦囊种类的判定，或 `u.use_as` 归一后判类型），并保留 `forced`（锁定技）语义。

**验收**：`cargo test --offline -p engine-native --test nc08_upstream_jizhi` 中
`upstream_jizhi_scope_difference_is_registered` 的净手牌断言应改为 **+2**，并把该用例改名去掉
「difference」字样。

**另一处（差异②，待口径判定）**：上游 `sbjizhi` gaintag + `sbjizhi_mark` 子技（摸到的牌**本回合不可被弃置**）
原生未实现。

## 6 刚烈（`ganglie`）缺「弃两张手牌」分支（NC08 样本 21 新发现，2026-10-19）

**症状（规则损失）**：标准面 `ganglie` 在判定**非红桃**后，先让**伤害来源二选一**：
`source.chooseToDiscard({selectCard: 2})`（**弃置两张手牌**），未弃（或手牌不足 2）才 `source.damage()`；
而原生 `crates/engine-native/src/content/standard/skills/gang_lie.rs` 判定非红桃后**直接**对来源造成 1 点伤害，
**没有**弃牌询问分支。

**修法**：判定成功后插入「来源手牌 ≥ 2 → `WaitChoice{SelectCards, min:2, max:2}` 弃两张；否则/未选 →
对来源 1 点伤害」的机器步骤（需按 `HostMachine`/`SkillMachine` 既有模式实现，含可取消与不足两张的短路）。

**验收**：`cargo test --offline -p engine-native --test nc08_upstream_ganglie` 中
`upstream_ganglie_discard_branch_difference_is_fixed` 应改为断言**存在**弃牌询问，并新增
「来源弃两张后不受伤害」的行为用例。

## 7 武圣「打出」路径未覆盖（NC08 样本 23 验证，2026-10-19）

**症状（规则损失）**：标准面 `wusheng` 的 `enable: ["chooseToUse","chooseToRespond"]` 含**打出**；
而原生 `kernel/duel.rs:70` 在决斗要求打出【杀】时，选项**只由实体【杀】构成**
（`filter(|c| catalog.card_kind(*c) == CardKind::Sha)`）→ **不含**武圣可转化的红色牌。
行为断言已复现（`upstream_wusheng_respond_path_not_covered`）。

**精确定位（同轮核对）**：**「闪」方向**的响应转化**已实现**（`kernel/sha/step.rs:1207–1243`：
龙胆【杀】、倾国黑牌；无双双闪臂 `:369–392` 同），**仅「杀」方向缺失** → 修法可**照搬**该模式。

**修法**：在**杀响应**的选项构造处（决斗/南蛮/万箭/借刀等要求打出【杀】的路径）加入**转化候选**：
持有 `WU_SHENG` 且牌为红色（或持有 `LONG_DAN` 且为【闪】）时，把该牌作为「视为【杀】」的候选加入，
并在应答消费时按 `viewAs` 语义结算（现有 `play_card.rs:154` 的 kind 改写逻辑可复用为函数）。

**验收**：`upstream_wusheng_respond_path_not_covered` 应改为断言**包含**红牌候选；
并补「用红牌响应决斗后该牌进入弃牌堆、决斗继续」的行为用例。

## 8 制衡（`zhiheng`）候选缺装备区牌（NC08 样本 25 新发现，2026-10-19）

**症状（规则损失）**：上游 `zhiheng` 的 `position: "he"` 含**装备区**；原生
`content/standard/skills/zhi_heng.rs` 的 `Select` 阶段候选**只由 `player.hand` 构成**，
`activate` 也要求**手牌非空** → 装备牌不可用于制衡（行为断言已复现）。

**修法**：候选改为 `hand ∪ equips`（并放宽 `activate` 的空手牌前置为「手牌或装备非空」）；
`Draw` 阶段的弃置需按牌区分别走 `discard_from_hand` / 卸装撤销装备技（既有 `mtg.rs` 的
海岛弃装备分支可参考）。

**验收**：`upstream_zhiheng_equip_zone_difference_is_registered` 应改为断言**包含**装备牌，
并补「弃装备后该装备技被撤销」的行为用例。
