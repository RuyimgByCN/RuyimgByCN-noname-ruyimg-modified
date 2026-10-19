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

## 9 青囊（`qingnang`）不能自我治疗（NC08 样本 29 新发现，2026-10-19）

**症状（规则损失）**：上游 `qingnang` 的 `filterTarget` **未排除自己**（只要已受伤即可，含自己）；
原生 `content/standard/skills/qing_nang.rs` 的 `activate` 要求存在 **`i as PlayerId != owner`**
的已受伤角色 → **仅自己受伤时不可发动**（行为断言已复现，见样本证据的 self-target-diagnostic）。

**修法**：`activate` 的受伤者判定去掉 `i != owner`；`TargetAsk` 阶段的候选生成同步放开自己
（若候选生成另有 `!= owner` 过滤）。

**验收**：`upstream_qingnang_self_target_difference_is_registered` 应改为断言**可发动**，
并补「青囊自己后体力 +1」的行为用例。

## 10 急救（`jijiu`）缺回合外限制（NC08 样本 30 新发现，2026-10-19）

**症状（原生更宽松）**：上游 `jijiu` 的 `viewAsFilter` 含 `player !== _status.currentPhase`
（**回合外**）；原生 `crates/engine-native/src/snap/asks.rs` 的 `is_tao_like` 只判花色为红
→ **自己回合内濒死救援时也可用红牌当桃**（上游不允许）。

**修法**：`is_tao_like` 的急救分支加入「响应者不是当前回合角色」判定（需把 `turn.current` 或
等价状态传入该函数——当前签名已含 `core`，可直接读取）。

**验收**：`upstream_jijiu_turn_restriction_difference_is_fixed` 应改为断言**存在**回合判定，
并补行为用例（自己回合内红牌**不可**当桃 / 他人回合内**可**当桃）。

## 11 「牌区 `he`/`hes`/`hs`」差异族（**6 例**）：命令面/候选生成限定手牌（NC08 样本 25/26/31/61 汇总，2026-10-19）

> **族内 6 例**：`zhiheng`(he) / `qixi`(hes) / `guose`(hes) / `wusheng`(hes) / `lijian`(he) /
> **`qingguo`(hs，样本 61 新增——响应候选只遍历 `.hand`)**。

**现象（3 例）**：`zhiheng`（`position: "he"`）、`qixi`/`guose`（`position: "hes"`）等上游技能允许
**装备区（及判定区）**的牌参与转化或作为费用；原生对应路径只覆盖**手牌**。

**精确定位（本轮核对）**：`crates/engine-native/src/engine/play_card.rs:71`

```rust
if !core.player(core.turn.current).hand.contains(&card) {
    return Err(RejectReason::CardNotInHand);
}
```

→ 原生**出牌命令**（`PlayCard`）要求牌在**当前回合角色手牌**中 → **装备区/判定区的牌无法经该命令使用**
→ 因此 `hes` 类技能的「装备/判定区入口」在**命令面**即不可达（**不是**结算逻辑缺失）。

**修法（建议一并做）**：
1. `PlayCard` 的牌区校验放宽为「手牌 ∪ 装备区 ∪ 判定区（按技能声明的 `position` 允许的牌区）」，
   并在结算时按牌区正确移除（装备需撤销装备技、判定区需从 `judge` 移除）；
2. 或新增**独立命令**（如 `PlayCardFromZone`）以避免影响既有手牌路径的校验强度；
3. 各 `viewAs`/费用候选生成处同步放开牌区。

**附带发现（已核对，**不成立，已撤回**）**：曾疑「校验用 `core.turn.current` 而非 `actor`」有风险；
经核对 `play_card.rs:12` **已有 `if actor != core.turn.current { … }` 守卫**（非当前回合角色的出牌
请求先被拒）→ 二者在该路径上**等价**，**不构成问题**（本条登记后即撤回，不写成缺陷）。

**验收**：为 `zhiheng`/`qixi`/`guose` 各补「装备区牌可参与」的行为用例；既有手牌路径用例保持不变。

## 12 枭姬（`xiaoji`）只覆盖「替换」失去装备（NC08 样本 32 新发现，2026-10-19）

**症状（规则损失）**：上游 `xiaoji` 的 `trigger` 为 `loseAfter` + `es` 计数（覆盖**所有**失去装备方式：
替换 / 被弃置 / 被获得 / 被拆…）；原生 `content/standard/skills/xiao_ji.rs` 的 filter **仅**匹配
`EventData::Equipped { replaced: Some(_) }` → 装备被【过河拆桥】拆走时**不触发**（行为断言已复现）。

**修法**：为「装备区失去牌」引入统一事件（或在各 `equips.retain(...)` 路径补发 `Equipped`/
新的 `EquipLost` 事件），`xiao_ji.rs` 的 filter 改为「失去的装备区牌数 > 0」，并按数量 ×2 摸牌。

**验收**：`upstream_xiaoji_discard_loss_difference_is_registered` 应改为断言**触发并摸 2**；
补「一次失去两张装备 → 摸 4」的用例。

## 13 兵粮寸断无距离限制（NC08 样本 33 附带发现，2026-10-19）

**症状（原生更宽松）**：上游【兵粮寸断】要求**距离 1**（延时锦囊的 `targetInRange` 默认距离限制）；
原生 `crates/engine-native/src/engine/play_card.rs:764–769` 的 `CardKind::Bing` 分支
**只查目标数 / 非自己 / 存活**，**无距离检查** → **任何距离**都可使用（对所有人，不只奇才持有者）。

**修法**：在 `CardKind::Bing` 分支补距离校验（`computed_distance > 1 → IllegalTargets`），
并把既有豁免位（奇才/神恩/守翼/探囊等，见 `CardKind::Snatch` 分支的写法）一并纳入
→ 同时**自动收敛**样本 33 登记的「奇才覆盖面」差异（同一豁免位）。

**验收**：新增「未持奇才时距离 >1 的兵粮寸断被拒」与「持奇才时可用」两条行为用例。

## 14 连营（`lianying`）阈值 off-by-one（NC08 样本 37 新发现，2026-10-19）

**症状（原生更宽松）**：上游 `lianying` 要求**失去后手牌为空**（`if (player.hasCards("h")) return false`）；
原生 `crates/engine-native/src/content/standard/skills/lian_ying.rs` 的 filter 为
`if ctx.state.player(owner).hand.len() > 1 { return false }`，而该判定发生在**牌已离手之后**
→ **手牌剩 1 张时仍触发**（实测出现 `trigger_confirm`，上游不应触发）。

**修法**：filter 改为按「失去后手牌为空」判定（`hand.len() > 0 → false`，或改为在事件前取手牌数
并判断「将失去的是最后一张手牌」）。

**验收**：`upstream_lianying_threshold_off_by_one_is_registered` 应改为断言**无确认门**，
并补「剩 1 张手牌时失去 1 张 → 不摸牌」的行为用例。

## 15 突袭（`tuxi`）无条件弃 2 张手牌（NC08 样本 39 定位，2026-10-19）

**症状**：上游 `tuxi` 用 `trigger.changeToZero()` **取消本次摸牌**（不摸也不弃）+ `gainMultiple`
取 1–2 张 → 净手牌应为 **+取牌数**；原生 `crates/engine-native/src/content/standard/skills/tu_xi.rs`
的 `Read` 阶段**无条件 `hand.pop()` 弃置 2 张**（注释称「弃置本阶段固定摸到的牌」）→ 若固定摸牌
**尚未发生**，就弃掉玩家**原有手牌** → 实测净变化 **0**（取 2 − 弃 2）。

**修法**：按「本阶段是否已固定摸牌」决定弃置张数（记录已摸数并只弃这些牌），或直接实现
`changeToZero` 语义（取消摸牌，不弃任何牌）。

**验收**：`upstream_tuxi_takes_cards_and_skips_draw` 的净变化断言应改为 **+取牌数**（选 2 名目标 → +2）。

## 16 **「按点数计次」族**：淑慎（`shushen`）/ 遗计（`yiji`）（2026-10-19；样本 46 后升为族项）

> **族化说明**：样本 46（遗计）发现**同一缺口** → 本项由单技能修复项**升级为族项**，
> 修法与验收需**覆盖两处**（并建议排查其它 `getIndex → event.num` 的技能）。

### 16.1 淑慎（`shushen`）

**症状（规则损失）**：上游 `shushen` 的 `getIndex(event) { return event.num || 1; }` 表示
**一次回复 N 点应触发 N 次**（每次各自选目标、各自摸牌）；原生 `content/standard/skills/shu_shen.rs`
的 filter 只判 `amount > 0`，机器在 `Read` 阶段**单次结算** → 回复 2 点只触发 1 次。

**修法**：按 `Recover.amount` 计次（循环 N 次询问/结算，或改为按次挂起多个询问）。

**验收**：新增「一次回复 2 点 → 两次目标选择与两次摸牌」的行为用例；
现有两条单点用例保持不变。

### 16.2 遗计（`yiji`）（样本 46 追加）

**症状（规则损失）**：上游 `yiji` 的 `getIndex(event, player, triggername) { return event.num; }`
表示**受 N 点伤害应触发 N 次**；原生 `content/standard/skills/ji_ji.rs` 的 filter 只判
`amount > 0`，机器**单次结算**（`draw_cards(owner, 2)` 一次）。

**修法**：按 `Damage.amount` 计次（与 16.1 同一改法）。

**验收**：新增「一次受到 2 点伤害 → 触发两次遗计」的行为用例。

### 16.3 族内排查（**本轮已完成**）

对上游 `character/standard/skill.js` 全表扫描 `getIndex` → **共 4 个技能**：

| 技能 | `getIndex` 返回 | 是否本族 | 处置 |
|---|---|---|---|
| `stdshushen` 淑慎 | `event.num \|\| 1` | **是**（回复点数） | 16.1 |
| `yiji` 遗计 | `event.num` | **是**（伤害点数） | 16.2 |
| **`xiaoji` 枭姬** | `evt.es.length`（**失去的装备牌数**） | **是**（失去张数） | **16.4（本轮新增）** |
| `xinfu_jiyuan` 新服济援 | `[event.player]`（**玩家数组**，非计数） | 否 | 不属本族 |

→ **族内共 3 处**（淑慎/遗计/枭姬）；`xinfu_jiyuan` 语义不同，**排除**。

### 16.4 枭姬（`xiaoji`）未按失去张数计次（样本 32 的补充发现）

**症状（规则损失）**：上游 `xiaoji` 的 `getIndex` 返回 `evt.es.length` → **一次失去 N 张装备
应摸 2N 张**；原生 `content/standard/skills/xiao_ji.rs` **一次触发固定摸 2**（样本 32 已登记
「失去方式覆盖面」，**本条补计次缺口**）。

**修法**：按失去的装备牌数计次（与 16.1/16.2 同一改法）。

**验收**：新增「一次失去两张装备 → 摸 4」的行为用例（样本 32 的待办已提及，此处并入本族）。

## 17 护驾（`hujia`）在非身份局亦可发动 —— **经核对为「有意的设计差异」，非缺陷**（2026-10-19）

> **更正（同会话第 7 次更正）**：本条最初按「修复项」登记；随后核对 `state.rs:1118` 的**文档注释**
> 发现该行为是**有意设计**：
> ```rust
> /// 主公资格（zhu 技能：护驾/激将）：身份局要求主公身份；
> /// 无身份体系的模式持技即可。
> pub fn is_lord(&self, p: PlayerId) -> bool { … None => true }
> ```
> → 因此**不列为修复项**，改列为**已声明的设计差异**；下面保留排查与风险核对结论备用。

### 原始登记（保留）

**症状（原生更宽松）**：上游 `hujia` 的 `filter` 要求 `player.hasZhuSkill("hujia")`（**主公技旗标**，
仅身份局成立）；原生 `crates/engine-native/src/kernel/sha.rs` 的 `hujia_active` 用
`ctx.core.is_lord(self.target)`，而 `state.rs` 的 `is_lord` 在**非身份局返回 `true`**
（`match self.identity_roles() { Some(roles) => …, None => true }`）→ **原型/单挑模式下护驾可发动**。

**全库排查结果（同轮完成，`grep -rn is_lord crates/engine-native/src/` → 定义 1 处 + 调用 4 处）**：

| 调用点 | 技能/机制 | 是否主公技语义 | 是否受影响 |
|---|---|---|---|
| `kernel/sha.rs:300` | `hujia` 护驾 | 是（上游 `zhuSkill`） | **是**（本项） |
| `content/standard/skills/jijiang.rs:29` | `jijiang` 激将 | 是（注释明写「**身份局须主公身份**」，但实现用 `is_lord`） | **是** |
| `content/standard/skills/wang_zun.rs:46` | `wang_zun` 王尊 | 是（摸牌加成随主公资格） | **是** |
| `kernel/dying.rs:167` | `jiuyuan` 救援 | 是（孙权主公技，吴势力他人代桃） | **是** |

→ **4 处调用全部为主公技语义**，因此 `is_lord` 的「非身份局返回 true」使**四处均在原型/单挑模式下越权可用** ✗
（**同一根因、四个症状**）。

**修法（建议改在根因处）**：为「主公技资格」引入**独立判定**（如 `has_zhu_skill(p)` =
`identity_roles().is_some() && is_lord(p)`），并把上述 4 处调用改到该判定；
**不要**直接改 `is_lord` 的语义（`state.rs:1124` 的 `None => true` 可能被非主公技路径依赖——改动前需核对）。

**风险核对（同轮完成）**：`is_lord` 在**引擎内**的调用点**恰为上表 4 处**（其余命中在测试夹具字段、
DSL 词汇表字符串、devtools 等处，与本语义无关）→ 因此**改这 4 处即可**，
`state.rs` 的 `None => true` **当前无其它依赖**（仍建议改动时保留该函数语义、另加主公技判定）。

**参照实现（旧线）**：`crates/engine-core` 的 DSL 过滤 `owner_is_lord`
（`trigger/dispatch/filters_owner_alive_ddd_fengzheng_round_pending.rs:1255`）用的是
**`p.identity == Identity::Lord`**（**要求显式主公身份**）→ 旧线**严格**、原生**宽松**
→ 修法可直接采用旧线口径（**跨线一致性**）。

**结论（更正后）**：
- **不是缺陷**：`is_lord` 的 `None => true` 有**明确文档注释**（「无身份体系的模式持技即可」），
  且**原型模式本身不是上游模式** → 「与上游等价」在此模式**无定义** → **设计差异成立**；
- **仍然有效的产出**：①**4 处调用点清单**（护驾/激将/王尊/救援）；②**风险核对**（引擎内无其它依赖）；
  ③**旧线参照**（`owner_is_lord` 用显式主公身份）——若将来决定**对齐旧线**，按上述三点即可执行；
- **不新增修复项**；如产品需要「原型模式下主公技不可发动」，再按①③执行。

## 18 反馈（`fankui`）自伤时仍可拿手牌（NC08 样本 45 新发现，2026-10-19）

**症状（原生更宽松）**：上游 `fankui` 的 `position` 为 `event.source !== player ? "he" : "e"`
→ **自伤时仅装备区**；原生 `content/standard/skills/fan_kui.rs` 的取牌为
`hand.first().or_else(equips.first())` → **自伤时仍可先拿手牌**。

**修法**：取牌时按「来源是否为自己」选择牌区（自伤 → 只看 `equips`）。

**验收**：新增「自伤触发反馈 → 只能拿装备区牌」的行为用例；他人伤害用例保持通过。

## 附：反馈的**另一项**差异（①取牌选择权）登记在「玩家选择权缺失」族

上游 `gainPlayerCard({forced:true})` 由**玩家选牌**；原生取确定性首张 → 与 `lijian`
（自动选目标）同族，**当前 2 例**；随该族一并评估，**不单列修复项**。

## 19 **维护性风险族**：`mod` 规则量修饰 ↔ 显式分支（3 例，**建议性收敛**，非缺陷）（2026-10-19）

**族内 3 例**（当前**语义等价**，但规则量新增消费点时会漏改）：

| 技能 | 上游 `mod` | 原生显式分支 | 漏改场景 |
|---|---|---|---|
| `kongcheng` 空城 | `targetEnabled`（统一拦截） | `play_card.rs`/`play_sha.rs`/`asks.rs` **3 处** | 新增「杀/决斗目标路径」 |
| `qianxun` 谦逊 | `targetEnabled`（统一拦截） | `play_card.rs` **2 处** | 新增「顺手牵羊/乐不思蜀类目标路径」 |
| `paoxiao` 咆哮 | `cardUsable → Infinity` | `play_sha.rs` **限次处 bypass** | 新增【杀】限次消费点 |

**建议（非必须）**：为「规则量修饰」引入**统一查询**（如 `card_usable_limit(p, kind)` /
`target_enabled(p, kind, target)`），把 3 处（及未来同类）改为**查询该函数**；
**不建议**在未统一前逐个新增 bypass（会持续扩大漏改面）。

**验收**：收敛后为三技能各保留现有行为用例（本目录样本 47/48/49 的用例即可复用），
并新增「新增一条目标/限次路径时自动生效」的**结构性用例**（如断言所有目标校验路径均经该查询）。

## 20 无双（`wushuang`）未排除「必中」目标（NC08 样本 50 新发现，2026-10-19）

**症状**：上游 `wushuang1` 的 filter 为
`event.card.name === "sha" && !event.getParent()?.directHit.includes(event.target)`
→ 目标被标记为**必中**（`directHit`）时，无双**不**要求两张【闪】；
原生 `crates/engine-native/src/kernel/sha.rs` 的 `source_has_wushuang` **只判来源是否持技**
→ **必中目标仍需两张【闪】**（与上游不符）。

**修法**：`sha` 结算机器在判定无双前检查本次【杀】对**该目标**是否 `directHit`（必中），
必中则跳过无双加倍。

**验收**：新增「必中（如带必中效果）的【杀】对无双持有者使用 → 目标只需一张【闪】」的行为用例；
现有「普通【杀】需两张【闪】」用例保持通过。

**注（已核对，2026-10-19 同轮）**：上游 **`wushuang2`**（决斗方向）的 filter **仅**
`event.card.name === "juedou"` —— **不含** `directHit` 排除 → **决斗方向本无必中语义**，
原生 `duel.rs` 的实现与该口径**一致**、**无需改动** → **本项修复范围仅限【杀】方向**。

## 21 飞影（`feiying`）未实现（NC08 样本 51 新发现，2026-10-19）

**症状（规则损失：标准面技能未实现）**：上游 `feiying` 为
`mod: { globalTo(from, to, distance) { return distance + 1; } }`（**他人计算与你的距离 +1**）；
原生 `crates/engine-native/src/` **全库无飞影实现**（`names.rs` 实现索引中亦无「飞影」）。

**修法（现成模式）**：复用 `state.rs` 的 **`plus` 累加面**（义从/谛听/云身等同类实现同款），
在 `globalTo` 方向加 1。

**验收**：新增「他人对持飞影者的距离 +1」的行为用例（可用 `computed_distance` 直接断言）。

**附：未实现技能的**排查方式**（重要）**：两次**自动**排查（上游键名匹配、中文名匹配）**均失败**
（详见 [source-coverage §1h/§1i](../2026-10-18-nc08-source-coverage/README.md)）→ 未实现清单
只能**人工逐条**建立（以 `names.rs` 索引 + 逐个读上游为准），**不采信**任何名称匹配脚本的输出。

## 22 标准面**未实现技能清单**（6 项，2026-10-19 第 3 次排查成功）

**来源**：[source-coverage §1j](../2026-10-18-nc08-source-coverage/README.md)（映射来源修正为
`character/standard/translate.js` 后成功）。上游标准面 **70 个键 → 6 个未实现**：

| 技能 | 上游键 | 上游实现要点 | 原生 |
|---|---|---|---|
| 轻狡 | `stdqingjiao` | `zhuSkill`（主公技，`phaseJieshuBegin`） | 无 |
| 忠义 | `zhongyi` | `phaseUse` + `limited`（限定技） | 无 |
| 飞影 | `feiying` | `mod.globalTo → distance + 1` | 无（第 21 项） |
| 战神 | `zhanshen` | `phaseZhunbeiBegin` + `juexingji`（觉醒技） | 无 |
| 神戟 | `shenji` | `mod.selectTarget/cardUsable`（【杀】多目标） | 无 |
| 将驰 | `new_jiangchi` | `phaseDrawEnd` | 无 |

**修法**：逐项按上游语义实现（飞影可直接复用 `plus` 累加面；神戟可参照多目标校验；
战神/忠义为觉醒/限定技，可参照现有 `juexingji`/`limited` 实现模式）。

**验收**：每项各补行为用例（**轻狡/忠义/战神/将驰**需构造对应阶段场景；**飞影**可用
`computed_distance` 直接断言；**神戟**可断言【杀】目标数上限）。

**注**：本条与第 21 项（飞影）**部分重叠**——飞影单列在前，本条给出**完整 6 项清单**；
实现时以本条为总表。

## 23 **必中标记通道**未统一：写入方（铁骑）+ 读取方（无双）（样本 50/52 互相印证，2026-10-19）

**症状（同一机制面、两端缺失）**：

| 端 | 上游 | 原生 |
|---|---|---|
| **写入方** `tieji` 铁骑 | 判定成功 → `trigger.getParent().directHit.add(target)`（**通用必中标记**） | 存 **杀机器字段** `tieji_red`（仅该次结算可读） |
| **读取方** `wushuang` 无双 | filter 检查 `!directHit.includes(target)`（必中时不加倍） | `source_has_wushuang` **只判是否持技** |

→ 结果：**铁骑判定成功后，无双仍要求两张【闪】**（上游此时只需一张）——**两处需一并修**。

**修法**：
1. **写入方**：`tieji` 判定成功时把目标写入**通用 `directHit`**（放在杀结算机器的共享字段，
   而非 `tieji_red` 这类单技能字段）；
2. **读取方**：`wushuang` 判定前检查该目标是否在 `directHit` 中，**在则跳过加倍**。

**验收**：新增「**铁骑判定成功 + 无双持有者** → 目标只需一张【闪】」的**交互用例**；
并保留样本 50（普通【杀】需两张【闪】）与样本 52（红判定后无出闪询问）的用例。

**附**：样本 52 的差异①（**守约变体**）与既有 `wusheng` 守约登记**同族**（当前 2 例），
随该族一并评估，**不单列修复项**。

## 24 观星（`guanxing`）的 `yizhi` 组合与 `guanxing_fail` 未建模（样本 53 新发现，2026-10-19）

**症状①（`yizhi` 组合）**：上游 `guanxing` 的 X 取值为
`player.hasSkill("yizhi") && player.hasSkill("guanxing") ? 5 : Math.min(5, game.countPlayer())`
→ 持「**遗志**」+「观星」时 **X 固定为 5**；原生 `content/standard/skills/guan_xing.rs` 注释只写
「X = 存活角色数且至多 5」，**未见 `yizhi` 判定**。

**症状②（`guanxing_fail`）——已核对消费方，**属 AI 面而非规则面**（2026-10-19 同轮更正）**：
上游在 `!result.bool || !result.moved[0].length`（**未移动任何牌**）时
`player.addTempSkill("guanxing_fail")`；原生**未见该标记**（该键在上游标准面存在但无中文名，
故未进入第 22 项的 6 项清单）。

**消费方核对**（`grep -rn guanxing_fail archive/upstream/noname/apps/core/`）：

| 消费点 | 用途 |
|---|---|
| `card/standard.js:4367` / `:4743` | **AI 决策**：`source.hasSkillTag("guanxing") && (source !== player \|\| !source.hasSkill("guanxing_fail"))` —— 据上游 typings 注释：「若有此标签且没有 `guanxing_fail`，**人机（队友）将不会对你判定区的延时锦囊使用【无懈可击】**」 |
| `character/offline/.../offline_piracyS.js:1340` | 另一来源（offline），非标准面 |

→ **`guanxing_fail` 只被 AI 决策消费**，**不改变规则判定** → 本症状应记为 **AI 面差异**，
**不计入规则损失**（优先级低于规则面条目）。

**修法**：① X 计算加入 `yizhi` 组合判定；② 未移动任何牌时加 `guanxing_fail` 临时技能
（需先登记该技能键与用途——**其消费方未核对**，登记待办）。

**验收**：①「持遗志 + 观星 → X=5」用例（可用存活人数不同的对局对比）；②「未移动任何牌 → 挂
`guanxing_fail`」用例。

**注**：差异①（时机锚点，判定阶段占位）与 `luoshen` **同族**，随该族评估、**不单列**。

## 25 仁德（`rende`）文件头注释滞后（样本 57 发现，2026-10-19）

**症状（文档不一致，非行为缺陷）**：`crates/engine-native/src/content/standard/skills/rende.rs`
的**文件头注释**写「出牌阶段**限一次**的主动技」，但**注册面**
（`standard_skills.rs:193`，`limits: SkillLimits::default()` + 注释「出牌阶段**可多次发动**（上游
`enable: phaseUse` 无次数限制）；**累计给牌数入回合状态（跨过 2 张回复 1 点）**」）
与**实测**（三次发动均接受；1+1+1 → hp 2→2→3→3）**均表明可多次 + 累计** →
**行为与上游一致**，仅**注释滞后**。

**修法**：把 `rende.rs` 头注释改为「出牌阶段**可多次发动**；**本回合累计**给出跨过 2 张时回复 1 点」。

**验收**：注释与注册面一致（本目录样本 57 的 `upstream_rende_doc_inconsistency_is_fixed` 用例
届时改为断言**不含**「限一次」）。

**注**：`rende.rs` **未跟踪**（属并行拆分）→ 随拆分落定后改，**不阻塞**其它工作。

## 26 耀武（`yaowu`）注释所称「差异」**不成立**（样本 60 核对，2026-10-19）

**症状（注释滞后，与第 25 项仁德同类）**：`crates/engine-native/src/content/standard/skills/yao_wu.rs`
的头注释写「**登记差异**：上游 `chooseDrawRecover` 由 AI/玩家确认，原生为**强制二选**」。

**核对结论（逐字读上游实现）**：`noname/library/element/player.js:6968` 的 `chooseDrawRecover`：
- `next.num1` / `next.num2` 默认均为 **1** → 「**摸 1 或回复 1**」（与原生一致）；
- **`next.forced = arg`**（布尔参数）→ 耀武传入 `{ forced: true }` → **`forced = true`**
  → **玩家必须二选、不可取消** —— 与原生「**强制二选**」**同义**。

→ 因此该「登记差异」**不成立** ✗，应**删除**该表述（**不是**原生与上游不一致）。

**修法**：把 `yao_wu.rs` 头注释中的「登记差异：…原生为强制二选」删去，改为
「上游 `chooseDrawRecover({forced:true})` 与原生『强制二选』同义」。

**验收**：本目录样本 60 的 `upstream_yaowu_note_may_be_stale` 用例改为断言注释**不含**
「登记差异」字样（届时随拆分落定一并改）。

## 27 急援（`xinfu_jiyuan`）仅实现濒死臂（样本 63 新发现，2026-10-19）

**症状（规则损失）**：上游 `xinfu_jiyuan`（译名「急援」）的 `trigger` 为
`{ global: ["dying", "gainAfter", "loseAsyncAfter"] }` —— **三个事件臂**（① 任意角色进入濒死；
② **自己给出的牌**被获得；③ 异步失去，如被弃置/被拿走），三臂**共用**同一 `content`
（`await event.targets[0].draw()` → 目标摸一张）；
原生 `content/standard/skills/ji_yuan.rs` 的 `filter` **仅匹配 `EventData::Dying`** → **缺两臂**。

**修法**：按上游补齐 ②③ 两臂（给予臂需判「给予者是自己」；失去臂取异步失去的目标集合），
共用现有「目标摸一张」结算。

**验收**：为 ②③ 各补一条行为用例（②「伊籍给出的牌被他人获得 → 该人摸一张」；
③「伊籍的牌被异步失去 → 对应角色摸一张」）；①濒死臂用例另行补（需构造濒死场景）。

**同族**：与 `xiaoji`（第 12 项）、`lianying`（覆盖面）同属「**事件臂覆盖面**」族（**3 例**）——
修法可一并评估：**统一以「全局失去/获得事件」为入口**，而非逐臂补丁。

### 27b 急援的濒死臂多一层 `dying` 守卫（样本 63 行为面新发现，2026-10-19）

**症状（规则损失）**：`crates/engine-native/src/content/standard/skills/ji_yuan.rs` 的 `step` 为
`if ctx.core.player(player).dying { ctx.draw_cards(player, 1) }` —— **额外要求 `dying == true`**；
上游 `content` 是 `await event.targets[0].draw()`（**无条件**）。实测（真实伤害打至 hp=0）：
`询问=["trigger_confirm","discard_to_limit"]`、`2 号 hp=0 dying=false alive=false`、**手牌未增加**
→ 濒死未被救回时**少摸一张**。

**修法**：去掉 `dying` 守卫（或改为「事件帧中即为濒死者」的判据），与上游无条件 `draw()` 对齐。

**验收**：`upstream_jiyuan_dying_arm_draws_one` 改为断言**手牌 +1**（当前为「登记差异」断言）。

## 28 激将（`jijiang`）**在原生侧不可达**（样本 58 行为面根因，2026-10-19）

**症状（规则损失）**：`crates/engine-native/src/content/standard/skills/jijiang.rs` 实现了 `activate`
与 `HelperAsk` 机器，但**全库检索 `JI_JIANG` 仅命中其自身定义**（`standard_skills.rs` 的注册、
`names.rs` 的名称、`generals.rs` 的武将表）——**没有任何调用点**（对照：`HU_JIA` 在
`kernel/sha.rs`、`JIU_YUAN` 在 `kernel/dying.rs`、`JIE_YIN` 经 `ActivateSkill` 均可达）
→ **激将的代打链路在原生侧不可达**：既不在【杀】响应/使用处被查询，也不在**决斗**处被查询
（样本 58 实测：决斗场景 `询问=[]`、代打未发生）。

**修法**：
1. 在【杀】的**响应/使用**入口（`kernel/sha/step.rs` 的 `respond_sha` 候选面）与**决斗**的杀响应面
   （`kernel/duel.rs`）加入「持激将且具主公资格 → 询问蜀势力代打」；
2. 复用 `jijiang.rs` 现有 `HelperAsk` 机器（无需重写）。

**验收**：样本 58 的 `upstream_jijiang_shu_ally_plays_sha_for_lord`（当前 `#[ignore]`）应改为通过；
另补「【杀】响应面代打」用例。

**同族提示**：与 `HU_JIA`（护驾，`kernel/sha.rs` 可达）**结构同源**，可参照其接入方式。

## 29 【已撤回】运筹武将技「不可达」——实为**夹具数据源错误**（2026-10-19 同轮更正）

**原登记（错误）**：据「`shahai-yunchou` 包只有 23 张卡、无 generals 段」判定运筹武将技
（雄姿等）在产品路径上**不可达**。

**更正**：该包**另有 `character.json`**（10 名武将，技能形如
`{"name": "ycnxiongzi", "title": "雄姿", "mapping": "host:xiongzi"}`）——武将面在
**`character.json`**，`cards.json` **只有卡牌段**。夹具须**双产物桥接**
（`bridge_pack` + `bridge_generals`）后才可 `debug_grant("ycnxiongzi")`。
**实测**：修正夹具后 `雄姿` 行为面**通过**（装备 0/2/3 件 → 摸牌阶段净增 **3/4/4** 张，
向下取整边界成立）→ **技能可达，原判定撤回**。

**教训（已并入方法学）**：与第 12/14 条同源——**断言「缺失/不可达」前必须先查全数据来源**
（本例漏查同包 `character.json`）。

## 30 隙焰（`jieyan`）**目标集合范围**：原生排除使用者（样本 76 新发现，2026-10-19）

**症状（待裁定的规则差异）**：上游 `jieyan` 为 `filterTarget: true` + `selectTarget: -1`
（`multitarget`/`multiline`）→ 按 noname 语义**全部角色**（**含使用者**）各受 1 点火焰伤害；
原生 `content/host/step/yunchou.rs:191` 的 `seats` 过滤 **`i != self.owner`** →
**排除使用者**（实测 hp `[4,4,4,4] → [4,3,3,3]`：0 号不受伤害）。

**裁定（已完成，2026-10-19 同轮）**：**上游含使用者** —— 依据
`noname/library/index.js:11681` 的 `lib.filter.targetEnabled(card, player, target)`：
其对 **布尔** `filterTarget` 直接 `return filterTarget;`（**无「目标不得为使用者」的隐式检查**），
而 `selectTarget: -1` 表示「**全部合法目标**」→ 使用者**是**合法目标 ⇒ 隙焰**应波及使用者**。
（对照：需要排除使用者时上游会写 `filterTarget: (card, player, target) => target !== player`，
如 `jiguantong`/`liuli` 等。）

**修法（已定）**：去掉 `content/host/step/yunchou.rs:204` 的 `i != self.owner` 过滤
（改为「全部存活角色」；如需 `toself` 语义则另行处理）。

**分类**：**规则损失**（上游应对使用者造成 1 点火焰伤害，原生未造成）。

**验收**：样本 76 的 `upstream_jieyan_discards_red_and_damages_others` 中
「0 号 hp 不变」的断言应改为 **hp −1**（并同步更新证据与台账）。

**同族**：与「目标集合范围」族（本项为该族**唯一**成员）后续可扩展核对其它 `selectTarget: -1` 技能。

## 31 厉焰·链杀（`luweiyan2`）：**夹具缺口已修**（子技须单独授予），**虚拟杀仍未产生**（样本 77，2026-10-19）

**第一步（已解决）：夹具缺口** —— 厉焰与链杀在包内是**两名技能**（`character.json`：
`ycnluweiyan`（厉焰）+ `ycnluweiyan2`（厉焰·链杀）；上游以 `group: "luweiyan2"` 关联，包数据并列列出）。
只授予 `ycnluweiyan` 时**完全不触发**（`询问=[]`）；**同时授予两者后出现 `trigger_confirm`**
⇒ 链杀**确实被派发**、过滤器匹配（`u_matches_lw2`）⇒ **原「不可达」判断撤回**。

**第二步（未解决）：虚拟杀未产生** —— 确认触发后：
- `respond_shan` **未出现**、1 号手牌仍含该【闪】；
- 同时**水攻本身已正常结算**（1 号装备 `1 → 0`）⇒ 触发时机/事件派发/`use_as`/目标表**均已排除**；
- ⇒ 剩余候选：**嵌套结算**（`yunchou.rs:280` 的 `use_virtual_card` 在【水攻】自身结算**内部**
  发起，可能被丢弃或延后）。

**修法（待定）**：把虚拟杀的发起**移出水攻结算**（如挂到结算后事件），或改为入队延后派发。

**验收**：样本 77 的 `upstream_luweiyan_chain_kill_fires_on_first_target`（当前 `#[ignore]`）应能通过
（出现【闪】响应询问、1 号手牌不含该【闪】）。

**同时登记**：上游「玩家选目标 + 可取消」↔ 原生「自动首目标」（即使修好嵌套，该差异仍在）。

**已排除假设（累计 4 项）**：①机器 `targets` 为空（`factory.rs:511` 已注入）；②`use_as` 未声明
（`Dismantle` 臂 `card_use_as` 已声明）；③`CardUsed` 未派发（狡黠样本 71 已证派发；水攻已结算）；
④子技未授予（**本轮修正**：授予后确认询问出现）。

## 32 运筹（`yunchou`）**四项**技能未实现/效果为空（样本 79 + 83，2026-10-19）

**症状（未实现）**：运筹包 **10 将 / 21 技**中，以下 **3 项在原生侧无实现**
（**逐标识**检索均 0 命中；`behavior.rs`/`keys.rs` 亦无对应变体/键。注意：**标准面同名技能
`JianXiong`（奸雄）存在，但与运筹变体 `diyjianxiong` 无关**——初版「全库 0 命中」的写法不准确，已更正）：

| 上游键 | 译名 | 上游要点 |
|---|---|---|
| `diyjianxiong` | 奸雄·变体 | `mode: ["identity","guozhan"]` + `trigger: { global: "dieBefore" }`；当前回合角色为持技者且身份局死者**非主公** → 改身份并亮明 |
| `xinfu_qiaosi` | 巧思 | 出牌阶段限一次：**投掷六面骰** X 点 → 亮出牌堆顶 X 张并获得 → 二选（交给其他角色 X 张 / 弃 X 张） |
| `xinfu_jingxie` | 精械 | 马钧技能（定义在 `character/mobile/skill.js`） |

**修法**：
1. `diyjianxiong`：宿主行为 + 身份局模式钩子（原生已有 `new_identity` 与身份字段）；
2. `xinfu_qiaosi`：**先核对骰子机制**是否已存在（若无则需新增随机骰面 API + 牌堆顶亮牌 + 二选）；
3. `xinfu_jingxie`：从 `character/mobile/skill.js` 取语义后实现。

**验收**：样本 79 的 `native_lacks_the_three_skills` 断言**逐项翻转**（实现后改为「存在」），
并为三项各补**行为面**用例。

### 第 4 项：`guihan`（归汉）——**已注册但效果为空**（样本 83）

**症状**：键与行为**已注册**（`keys.rs:579` `host:guihan`；运筹 `ycnguihan` 的 `mapping` 即此），
但**效果分支为空**：`step/wudian_status.rs:82` → `Flow::Done`（过滤器恒 `false`——作为**主动技**
方向正确）。上游为**限定技**：自己濒死时可令一名**男性其他角色**与自己**各回复 1 点并各摸 2 张**。

**修法**：在激活路径实现「自己回复 1 + 摸 2；目标回复 1 + 摸 2」，并处理 `limited`（限定技）标记。

**验收**：样本 83 的 `native_guihan_effect_is_empty` 断言**翻转**（效果分支不再为空），
并补行为面用例（自己濒死 + 男性目标）。

**注**：与前三项（**完全未注册**）**成因不同**——本项属「**已注册、效果为空**」，
建议在逐包状态中**分列**。

**同族**：与标准面第 22 项（6 项未实现技能）同类；**建议**把「宿主已实现/未实现/已注册但为空」纳入
来源台账的逐包状态（G0–G5）。

## 33 纵火（`zonghuo`）触发确认后**未弃牌**（样本 80 行为面，2026-10-19）

**症状**：样本 80 实测——0 号（持纵火）对 1 号造成**非火焰**伤害时：
- `询问=["trigger_confirm"]`（纵火触发被询问，**已确认**）；
- **伤害确实落地**（1 号 hp `4 → 3`）；
- 但 **0 号手牌 `2 → 2`（未弃牌）** ✗ —— 上游 `chooseToDiscard` 应弃 1 张（并因此把伤害性质改为火焰）。

**已排除**：①触发未派发（`trigger_confirm` 已出现）；②伤害未落地（hp −1 已证）；
③手牌为空（2 张）；④技能未授予（已 `debug_grant("ycnzonghuo")`）。

**接线核对（2026-10-19 同轮，**接线完整**）**：

| 环节 | 状态 |
|---|---|
| 触发元数据 | `metadata/common.rs:49`：`EventKind::DamageBefore` + `Timing::Normal` + `optional: true` |
| 过滤器 | `filtering/common.rs:495`：来源为本人 + 未防止 + `amount > 0` + **非火焰** |
| 机器工厂 | `factory.rs:212`（目标 = 受伤者）；`stage` **默认 `HostStage::Act`** |
| 步进路由 | `step.rs:694` → `step_yunchou`；效果分支 `step/yunchou.rs:357`（弃手牌首张 + 改伤害性质） |

⇒ **接线完整**，但确认后仍**未弃牌** → 失败点位于**机器/触发运行时内部**（非接线）——
相关文件属**并行拆分的未跟踪文件**（`content/host/**`）→ **受拆分阻塞**（与 28 项修复同类）。

**剩余候选（拆分落定后定位）**：①`optional: true` 确认后**是否真的构造并步进机器**；
②`discard_from_hand` 在该帧内的落点；③`current_frame_id()` 是否指向**伤害帧**（影响性质改写，
但不影响弃牌）。

**验收**：样本 80 的 `upstream_zonghuo_discards_one_without_asking`（当前 `#[ignore]`）应能通过
（0 号手牌 −1；并补「伤害性质变为火焰」的观测用例）。

**同时登记**：上游 `chooseToDiscard`（玩家自选、可取消）↔ 原生「自动弃首张」（与取牌选择权族同族）。

## 34 【已降级】「确认后效果未执行」——烧营为**夹具问题**（已撤回），**纵火仍待查**（2026-10-19）

**更正（同日）**：原把**纵火（样本 80）与烧营（样本 81）**并列为「跨技能共性缺陷」。经复核：

| 样本 | 原判 | 复核结论 |
|---|---|---|
| 81 烧营 | 效果未执行 | **夹具问题** ✗：`debug_set_deck_top(name)` 按**牌种**取牌，同种牌可能多张**不同花色** → 我按花色挑的「红牌」其**牌种的首张**可能是**黑**的 → 亮出的是黑牌 → 自然不烧。**改为「置顶后回读 `deck.last()` 核对花色」** 后：**红牌 → 2 号确实受 1 点火伤**（实测 `[4,4,4,4] → [4,3,3,4]`）⇒ **烧营行为面通过，本条撤回** |
| 80 纵火 | 效果未执行 | **仍待查**：确认询问的**观察者确为 0 号**（`trigger_confirm@obs0`）、伤害已落地（1 号 hp 4→3）、手牌 2 张 → 确认后**仍未弃牌**。接线（元数据/过滤器/工厂/步进路由）已核对完整 → 失败点在**运行时内部**（未跟踪文件）→ **受拆分阻塞** |

**新增夹具纪律（写入规则基准）**：`debug_set_deck_top(name)` **按牌种**取牌 → **同种多花色**时
必须**置顶后回读** `deck.last()` 核对花色（或改用按 id 置顶）；否则会出现「以为放了红牌，实际是黑牌」
的**静默失配**，并可能被误判为「效果未执行」。

**纵火证据强化（同日第二轮）**：改用**标记牌判据**（只放「标记【闪】+ 用于出杀的【杀】」）并走
**真实出杀**路径：手牌 `2 → 1`，而离手的**只是被使用的【杀】**、**标记【闪】仍在手**
（`标记牌是否已弃=false`）⇒ **纵火确实未弃牌**（排除了「出杀消耗一张」的干扰）。
**两种伤害路径**（`debug_deal_damage` 与真实出杀）**均复现**。

**纵火后续（拆分落定后）**：核对 `optional: true` 确认后是否真的构造并步进机器、
`discard_from_hand` 在该帧内的落点（**注意**：出杀路径下 `hand.first()` 可能取到**已被使用的牌**）、
`current_frame_id()` 是否指向伤害帧。
**验收**：样本 80 的 `#[ignore]` 用例应能通过（标记牌被弃）。

## 35 乾坤镖（`qiankunbiao`）目标门槛**未校验 `he` 有牌**（样本 84，2026-10-19）

**症状（原生更宽松）**：上游 `filterTarget(card, player, target)` 为
`target !== player && target.countCards("he") > 0`；原生 `engine/play_card.rs:1684` 的校验只要求
「**恰 1 名**其他存活目标」⇒ 目标 `he` **无牌**时**仍被受理**（实测 `Accepted`）。

**修法**：在 `play_card.rs:1684` 的 `CardKind::Qiankunbiao | CardKind::Shenhuofeiya` 分支补
「目标 `he` 有牌」校验（**注意**：`shenhuofeiya` 的上游 `filterTarget` **只要求 `target !== player`**，
**不要求** `he` 有牌 ⇒ 两者需**分别**校验，不可合并）。

**验收**：样本 84 的 `upstream_qiankunbiao_target_needs_he_cards`（当前 `#[ignore]`）应能通过（`Rejected`）。

**同类提示（已有反向证据）**：`shenhuofeiya`（神火飞鸦）与乾坤镖**共用**该校验分支 →
修完乾坤镖后须**复核神火飞鸦**不应被误加 `he` 门槛。
**证据**：样本 85（`nc08_upstream_gujian_shenhuofeiya.rs`）已断言「神火飞鸦目标 `he` 无牌时
**必须仍被受理**（`Accepted`）」→ **若把两卡校验合并，该断言会失败**。

## 36 龙须钩（`longxugou`）获得装备后**未自己装备**（样本 86，2026-10-19）

**症状（规则损失）**：上游 `longxugou` 的 `content` 为三段——① `choosePlayerCard("e", target, true)`
（**使用者自选**目标装备区一张）→ ② 目标 `lose(..., ui.special)` → ③
**`if (get.position(event.card) === "s") player.equip(event.card)`（自己装备之）**。
原生实测：询问 `gj_longxu_pick`（**自选，与上游一致** ✅）、目标装备区 `1 → 0` ✅，
但**使用者装备区 `0 → 0`**、该牌进入**使用者手牌**（`手牌含该装备=true`）✗
⇒ **只「获得」未「装备」**。

**修法**：在龙须钩的效果分支把「入手」改为「**直接装备**」（对应上游 step 2 的 `player.equip`），
或保留入手后立即调用装备路径（含装备技生效/替换旧装备的处理）。

**验收**：样本 86 的 `upstream_longxugou_steals_and_equips`（当前 `#[ignore]`）应能通过
（使用者装备区 +1、目标装备区 −1）。

**注**：目标门槛（须有可获得装备）**已正确实现**（实测无装备目标 `Rejected(IllegalTargets)`）。

## 37 鲤鱼汤（`liyutang`）**效果为空**（样本 89，2026-10-19）

**症状（精确化，2026-10-19 同轮更正）**：上游 `liyutang` 的 `content` 为——`target.$gain2(cards)`
（**目标获得此牌**）+ `target.storage.liyutang = card` + `target.storage.liyutang_markcount = 2`
+ `target.addSkill("liyutang")`。原生实测：发动 `Accepted`，但**目标手牌 0 → 0**、
**使用者手牌为空**、**该牌进入弃牌堆**（`弃堆含此牌=true`）⇒ **用牌效果（获得牌/计数/授技）缺失**。

**精度更正（避免以偏概全）**：初版写作「效果为空」**不准确** ✗ ——
`step/gujian.rs:318` 的 `HostBehavior::GujianFoodLiyutang` **已实现子技效果**
（「阶段开始：体力值为全场最少或之一 → **获得 1 点护甲**」+ `food_tick` 计数结算）⇒
缺的是**用牌（`content`）那一段**，**不是**整个技能为空。

**静态普查结论（同日）**：对 `behavior.rs` 中 **23 个 `Gujian*` 行为**逐一检查
`content/host/step/**.rs` 是否引用 → **23/23 均有 step 代码** ⇒ **「用牌效果缺失」无法靠静态引用检出** ✗
→ 只能靠**逐卡行为面抽样**（如本样本）发现 ⇒ **建议**对古剑/同类卡包做**逐卡行为面抽样**。

**已核对**：门槛**已正确实现**（目标已持子技时 `Rejected(IllegalTargets)`）；
键存在（`keys.rs:882` `host:gujian_food_liyutang`）；子技目录名 `gj_food_liyutang`（`constants.rs:58`）。

**修法**：在食物牌效果分支实现「目标获得此牌 + 记 `storage` + `markcount=2` + 授予计时子技」，
并实现**计数的逐轮结算**（子技效果）。

**验收**：样本 89 的 `upstream_liyutang_target_gains_the_card`（当前 `#[ignore]`）应能通过
（目标手牌 +1 且含此牌）；另补计数结算用例。

**同族/共性（2026-10-19 同日扩样 + 精度更正）**：样本 93（`mapodoufu` 麻婆豆腐）**完全同一现象**
（目标手牌 `0 → 0`、牌入弃堆）；**进一步实测**：`debug_has_skill(1, "gj_food_mapodoufu") = true`
⇒ **子技授予成功**（`storage`/`markcount`/`addSkill` 这一面**正常**）。

**精度更正（同日，第二次）**：原写「用牌段**整体**缺失」**不准确** ✗ —— 实测与既有样本（春饼：
`rounds == 5` 断言通过）表明 **授技/计数/子技 tick 均正常** ✓，**唯一缺失的是「本体归属」**：

| 面 | 状态 |
|---|---|
| `target.$gain2(cards)`（**本体归属**：牌应进目标手牌） | ✗ **缺失**（牌直接入弃堆） |
| `storage` / `markcount`（原生以 `rounds` 建模） | ✅ 正常（春饼 `rounds == 5` 断言通过；麻婆豆腐子技授予 `true`） |
| `addSkill`（计时子技） | ✅ 正常（子技可查询到且启用） |
| 子技 tick 效果 | ✅ 已实现（鲤鱼汤：阶段开始体力最少 → +1 护甲 + `food_tick`） |

⇒ **共性结论（**普查全量 12/12**）**：**食物牌的「本体归属」缺失**（牌入弃堆而非目标手牌），其余面正常。

**普查证据（样本 94，`nc08_upstream_gujian_food_survey.rs`）**：对上游 `type: "food"` 的**全部 12 张卡**
（鲈鱼羹/春饼/古洞羹/鲤鱼汤/蜜汁莲藕/虾饺/昙花洞/麻婆豆腐/青团/油羹/茉莉茶/元宝肉）执行同一检查点
「目标是否获得本体」→ **获得 = 0，未获得 = 12** ⇒ **无个案例外**。

**修法（据此收敛）**：只需在 `GJ_FOOD_*` 机器的**用牌入口**把本体交给目标（而非入弃堆），
**无需**改动授技/计数/tick 面。

**反证（样本 95，`ziyangdan` 紫阳丹）**：该卡使用**同一族** `GJ_FOOD_*` 机器，但效果**正常**
（护甲 `0 → 3`、子技授予 `true`）——其上游 `content` **没有 `$gain2`** 段 ⇒ **与「仅 `$gain2` 缺失」
的精确化结论一致**，**不支持**「食物机卡一律失效」的粗放推论。

**建议**：①修复 `GJ_FOOD_*` 机器的**用牌入口**；②对古剑包**全部食物牌**
（春饼/青团/油羹/茉莉茶/虾饺/蜜汁莲藕/鲈鱼汤/古洞羹…）按「目标是否获得此牌」**逐卡抽样**。

## 38 石灰粉（`shihuifen`）的**限制未生效**（样本 90，2026-10-19）

**症状**：上游 `shihuifen`（`global: "g_shihuifen"`，`filterTarget: true`）的 `content` 为——
`_status.currentPhase.chooseToRespond({ name: "shan" })`（**当前回合角色须出【闪】**）→
**未出**则 `_status.currentPhase.addTempSkill("shihuifen", "phaseUseAfter")`
（**该角色本回合无法对其他角色使用卡牌**）。原生实测（由**当前回合角色自己**使用石灰粉，避开
`NotYourTurn`）：

| 观测点 | 实测 |
|---|---|
| 发动 | `Accepted` ✅ |
| **闪的询问** | **`询问=[]`**（无任何询问）✗ —— 上游应有 `chooseToRespond` |
| 限制 | 该角色随后对**其他角色**出【杀】**仍被受理**（`Accepted`）✗ |

**已核对**：`content/host/step/gujian.rs:439` 存在 `GujianShihuifenCard` 阶段机；
`GujianFoodShihuifenBan => Flow::Done`（限制子技**自身为空**——其执行应在出牌校验路径）。

**修法（待定位）**：①核对阶段机的入口/stage（为何无询问）；②在**出牌校验路径**实现
「持 `shihuifen` 限制技时不得对其他角色使用牌」，并在 `phaseUseAfter` 解除。

**验收**：样本 90 的 `upstream_shihuifen_blocks_cards_on_others`（当前 `#[ignore]`）应能通过
（出现闪询问、且未出闪后对其他角色出牌被 `Rejected`）。

**同类提示**：`jinlianzhu`（金莲珠，样本 87）同为 `global` 子技但**打断窗口可达且生效** ⇒
本项**不是**「global 子技一律不可达」，须按卡定位。

## 39 帝女玄霜（`dinvxuanshuang`）**只回复使用者**（样本 96，2026-10-19）

**症状（目标集合范围）**：上游 `dinvxuanshuang` 为 `selectTarget: -1`（**全部角色**）→
每个角色各 `target.recover()`；原生实测**只回复使用者**（使用者 hp `3 → 4`，其他角色 `3 → 3` 未变）。

**夹具前提 + 源码级确认（已核对）**：该卡按「**无显式目标**」受理（`vec![]`；`vec![1]` → `IllegalTargets`）；
使用者**满血**时 `Rejected(HpFull)`。**源码直接印证**：`engine/play_card.rs:1440` 注释为
「**帝女玄霜：无目标、对自己使用、需已受伤（桃同款校验）**」⇒ 原生**明确**把它建模为
**对自己**的回复牌（桃同款），与上游 `selectTarget: -1`（**全部角色**）**不一致** —— 该差异
**有源码依据、非夹具问题**。

**修法**：按上游 `selectTarget: -1` 把回复与后续「弃 N 摸 N」施加到**全部角色**
（并同步修正 `HpFull` 的校验对象）。

**验收**：样本 96 的 `upstream_dinvxuanshuang_recovers`（当前 `#[ignore]`）应能通过
（使用者与 1 号**都**回复 1）。

**族内第 2 例**：与 `jieyan`（隙焰，样本 76）同属「**目标集合范围**」族——**方向相反**
（隙焰：原生**排除**使用者；本卡：原生**只含**使用者）。

## 40 黑龙鳞片（`heilonglinpian`）的**临时技疑未授予**（样本 97，2026-10-19）

**症状**：上游 `content` 为 `target.changeHujia()`（**+1 护甲**）+ 
`target.addTempSkill("heilonglinpian", { player: "phaseBegin" })`（临时技至**自己回合开始**）。
原生实测：**护甲 +1 生效** ✅（由**其它路径**完成）；但**临时技**经**双重确认未授予**：

1. **名称探测（5 个候选）**：`heilonglinpian` / `gj_heilonglinpian` / `gj:heilonglinpian` /
   `host:heilonglinpian` / `heilonglinpian_skill` → **命中集合为空**（`候选命中=[]`）；
2. **全库 grep（按卡名）**：`grep -rli heilonglinpian crates/engine-native/src/` **仅**命中
   `archive/card_codes.rs`、`content/host/constants.rs`（**键常量**）、`catalog.rs`、
   `engine/play_card.rs`（**目标校验**）、`engine/snapshot.rs`；
3. **源码级自述（决定性）**：该卡的**行为枚举名为 `HostBehavior::JiguanHeilongCard`**
   （`kernel/intrinsic.rs:417` 映射；实现在 `content/host/step/jiguan.rs:352`），其代码为：

   ```rust
   HostBehavior::JiguanHeilongCard => {
       // 获得 1 点护甲（登记差异：防御距离 +1 临时技暂缓）
       ctx.change_hujia(self.owner, 1);
   ```

   ⇒ **原生自己登记了该差异**：临时技的**具体效果是「防御距离 +1」**，状态为**「暂缓」**（未实现）。

⇒ **临时技未实现**（亦无到期移除路径）—— **非「名称不匹配」**（已用多候选名排除，
且已定位到行为实现本身）。

**教训（命名陷阱）**：行为枚举名可能与卡名**不同**（`JiguanHeilongCard` vs `heilonglinpian`）⇒
**按卡名 grep 会漏**；应改按**行为枚举名**或**键常量**检索（与样本 99 玉女元神的
`*_LISTENER_KEY` 同源教训）。

**修法**：实现该临时技（授予 + 效果 + 至**自己回合开始**移除），并确认其与护甲 +1 的**先后关系**。

**验收**：样本 97 应能断言「临时技已授予且在自己回合开始时移除」。

**注**：护甲 +1 **已正确**（本项**只**针对临时技）。

## 41 蜜汁藕（`mizhilianou`）的「**红桃当桃**」转化未实现（样本 112，2026-10-19）

**症状**：上游 `mizhilianou` 的子技 `subSkill.use` 为
`{ enable: "chooseToUse", filterCard: { suit: "heart" }, position: "he", viewAs: { name: "tao" } }`
（**红桃牌当【桃】使用**，`he` 含装备区）；原生实测：
**持有子技后，红桃非桃牌当【桃】使用 → `Rejected(IllegalTargets)`**。

**已排除夹具因素（对照组）**：同一局面下**普通【桃】对自己**→ **`Accepted`**（hp 3 → 4）
⇒ 目标形态正确；且修正后的对照与实验**仅差牌种** ⇒ **差异来自转化缺失**，非夹具。

**修法**：在**出牌路径**为持有 `gj_food_mizhilianou` 者加入「红桃牌 → 桃」的 `viewAs` 转化
（含 `position: "he"` 的装备区入口，见牌区族第 11 项）。

**验收**：样本 112 的 `upstream_mizhilianou_heart_as_tao`（当前 `#[ignore]`）应能通过
（红桃非桃牌 `Accepted` 且 hp +1）。

**同族**：与武圣/倾国/厉焰同属**转化**机制；本项为**转化缺失**（非牌区问题）。

## 42 照明弹（`zhaomingdan`）的**弃牌选择权**归原生（样本 120，2026-10-19）

**症状（选择权）**：上游 `zhaomingdan` 的 `content` 为
`player.discardPlayerCard("hej", target, true)`（**使用者选择**弃目标的哪一张 `hej` 牌，
`next.visible = true`）；原生实测**无任何询问**，**自动弃置目标手牌首张**（装备未被弃）
⇒ 与既有「**确定性首张**」族**同源**（盾牌格挡/瘟疫/乾坤镖等已登记同类）。

**已排除夹具因素**：本样本给目标 **1 张手牌 + 1 件装备**，并**优先尝试选中装备**；
实测**未出现任何询问**且**手牌被弃、装备保留** ⇒ 差异来自**实现**，非夹具。

**修法**：改为向上游一致——由**使用者**在目标的 `hej` 中**选择**一张弃置（可见）。

**验收**：样本 120 的 `upstream_zhaomingdan_discard_and_both_draw` 应能断言
「出现弃牌询问且可选中装备」。

**族**：「确定性首张」（选择权/随机 → 确定性），修法方向见基准 v78（可改用种子 RNG 或恢复玩家选择）。

## 43 激活（`jihuocard`）的**临时技 `jihuocard2` 未实现**（样本 121，2026-10-19）

**症状**：上游 `jihuocard` 的 `content` 为
`if (_status.currentPhase === target) target.addTempSkill("jihuocard2")` + `target.draw()`。
原生实测：**摸 1 生效** ✅（牌堆 `123 → 122`）；但**临时技未授予** ❌ ——
①**候选名探测**（`jihuocard2`/`gj_jihuocard2`/`hearth_jihuocard2`/`gj_hearth_jihuocard2`/`host:jihuocard2`）
**命中集合为空**；②**全库 grep `jihuocard2`**（`crates/engine-native/src/` + 全部 `packages/extensions/*`）
**0 命中** ⇒ **确未实现**（**非**名称不匹配）。

**源码印证**：`kernel/intrinsic.rs:1292` 映射到 `HostBehavior::JihuocardDraw`，
`content/host/step/skills_status.rs:790` 的该分支**只做摸牌**，**无授予临时技**。

**修法**：在**自己回合内**使用本卡时**授予临时技 `jihuocard2`**，并实现其**效果**与**到期移除**。

**验收**：样本 121 应能断言「临时技已授予」（并按原生常量取名，不猜名）。

## 44 神恩术（`shenenshu`）**效果未实现**（样本 122，2026-10-19）

**症状**：上游 `shenenshu` 的 `content` 为「**失去全部手牌** → 获得 **1 + 原手牌数** 张**基本牌**
（`get.typeCard("basic")` 排除毒，`randomGet()` 随机）」。原生实测：发动 `Accepted`，但
**原手牌未失去**、**未获得新牌**（手牌仍为原 2 张、其中基本牌 1 张）⇒ **效果为空**。

**源码自述（决定性）**：`content/standard.rs:52-53` 版本注释原文——
「v40：G1 杀海拾遗·炉石子包切片——灵魂之火/照明弹/急耗卡（CardKind 三种 + 宿主行为；
**`shenenshu` 等复杂牌暂缓**）」⇒ 原生**已登记为「暂缓」**（**非未记录缺口**）。

**修法**：实现「失去全部手牌 + 生成 `1 + n` 张基本牌（排除毒）并直接获得」。

**验收**：样本 122 的 `upstream_shenenshu_replaces_hand_with_basics`（当前 `#[ignore]`）应能通过
（原手牌全失去、新得 `1 + n` 张且全为基本牌）。

## 45 视界术（`shijieshu`）**目标集合 + 末尾弃 1**（样本 124，2026-10-19）

**症状一（目标集合）**：上游 `filterTarget: !target.isMin()` ⇒ **指定其他角色**；
原生 `engine/play_card.rs:1481-1485` 注释「**世界树：自-target**」且**要求 `targets` 为空**
（传显式目标 → `Rejected(IllegalTargets)`）⇒ **效果只作用于使用者**。

**症状二（末尾弃 1 缺失）**：上游 `step 3` 为 `if (target.countCards("he")) target.chooseToDiscard("he", true)`
（**强制弃 1**）；原生实测 `弃牌=false`，`he` 总数 `1 → 2`（= **+2 装备 −1 打出本体**）
⇒ **末尾弃牌未发生**。

**修法**：①按上游把目标改为**其他非体力最少角色**；②补**末尾弃 1**（`he`，强制）。

**验收**：样本 124 应能断言「指定其他角色成功」且「`he` 净 **+1**（+2 −1）」。

**族**：「目标集合范围」**第 3 例**（前两例 `jieyan`/`dinvxuanshuang`）。

## 46 闪电箭（`shandianjian`）**交付但无实现，被通用「杀」路径接管**（样本 125，2026-10-19）

**症状**：上游 `shandianjian` 的 `content` 为「**目标展示一张手牌**（`chooseToShow` + `showCards`）→
**使用者须弃一张同花色手牌**（`filterDiscard: { suit: get.suit(cards2[0]) }`）→（否则后续分支）」。
原生实测：发动后**要求目标出【闪】**（`respond_shan`）并造成 **1 点伤害**（hp `4 → 3`）
⇒ **被当作通用「杀」类处理**，与上游语义**完全不同**。

**源码核对（交付但无实现、且无登记）**：
①`grep -rli shandianjian crates/engine-native/src/` → **0 命中**；
②包内 `cards.json` **有**该卡（已交付）；
③`content/standard.rs` 的 v40 注释**未提及**本卡（只列「灵魂之火/照明弹/急耗卡」）；
④`docs/**` 检索「闪电箭/shandianjian」→ **0 命中**。

**修法**：实现「展示手牌 → 使用者弃同花色 → 未弃时的雷伤分支」，并把它从通用「杀」路径**分离**
（核对 `CardKind` 归类）。

**验收**：样本 125 应能断言「出现**展示手牌**与**弃同花色**询问」，且**不出现 `respond_shan`**。

## 47 火山（`huoshan`）**交付但无实现、且无登记**（三维核对，2026-10-19）

**症状**：对**全 37 来源已交付卡**做「**实现维**（`crates/engine-native/src`）+ **登记维**（`docs/**`）」
双维检索（归一化：剥离来源前缀**与尾部数字**）后，**唯一**同时**两维皆无**的卡是
**`huoshan`（火山，yunchou 包）** ⇒ **包内交付、无原生实现、无任何登记**。

**同批结果**（见证据目录）：
- `shandianjian`（闪电箭）：**有登记但无实现** —— 与样本 125 的行为面发现**独立互证**（第 46 项）；
- `kuwu`（苦无）：**有登记但无实现** ⇒ 属**已知事项**。

**行为面基线（样本 126，同日；含同日更正）**：`targets: vec![]` 与 `targets: vec![0]`（显式自己）
**均被拒**，但 **`targets: vec![1]`（其他角色）→ `Accepted`** ⇒ **并非「完全不可用」**（初版判断过宽，已更正）：

| 形态 | 结果 |
|---|---|
| `vec![]` | `Rejected(IllegalTargets)` |
| `vec![0]`（自己） | `Rejected(IllegalTargets)` |
| **`vec![1]`（其他角色）** | **`Accepted`** |

⇒ 真实症状有二：①**目标形态与上游相反**（上游 `toself` **只对自己**，原生**只受理其他角色**）——
属「**目标集合范围**」族**第 4 例**；②受理后**判定区 `0 → 0`** ⇒ **效果未实现**。

**上游语义（已读源码，逐字）**：`type: "delay"`（**延时锦囊**）、`cardcolor: red`、`cardnature: fire`、
`toself`、`selectTarget: [-1,-1]`；`judge`: **红桃 2–9 → 通过**（−6）；`judge2`: **反转**；
`effect`: **自己受 2 点火伤（`nosource`）** + **距离 ≤1 的其他角色**（按座位序）后续结算。

**修法**：按上述语义实现**延时锦囊**（进判定区 + 判定 + 反转 + 效果），并补登记。

**验收**：新增样本应能通过，且实现维/登记维均可检索到该卡。

**方法备注（重要）**：本项由「**三维核对**」（交付 / 实现 / 登记）发现 —— 该核对**能发现
按名字 grep 实现与按文档查登记都漏掉的卡**；**归一化必须包含尾部数字**，否则 swd 会误报 21 项。

## 48 **空选应答导致引擎 panic**（健壮性，2026-10-19 受理普查发现）

**症状（健壮性/校验缺失）**：当一次 `min ≥ 1` 的询问被以**空选择**应答时，宿主机器在恢复后调用
`Machine::answer_first()`（`crates/engine-native/src/machine.rs:314`）会命中
`expect("non-empty selection")` ⇒ **进程 panic**，而**不是**把该应答作为**非法输入拒绝**。

**复现路径（已核实）**：受理面普查（台账第 127 行）首轮**未应答**中途询问（测试助手 `drain()`
以空选应答）⇒ 在 `toulianghuanzhu`（偷梁换柱）上触发 panic；
**专项复测**（`nc08_upstream_yunchou_toulianghuanzhu.rs`）**正确应答后不再 panic**
（询问 `wd_touliang_own` + `wd_touliang_target`）⇒ **确认根因是「空选应答」而非该卡本身**。

**影响面**：测试与外部客户端**都可能发出不合法的空选**；panic 会**终止进程**（服务器侧尤其严重），
应改为**返回拒绝**（如 `Rejected(InvalidChoice)`）并保持状态一致。

**修法**：在应答校验处**校验 `min`/`max` 与选项合法性**，不合法则**拒绝并保持询问**（或按取消语义处理），
**不进入 `expect`**；`answer_first()` 改为返回 `Option`/`Result`。

**验收**：新增用例——对 `min ≥ 1` 的询问提交**空选**时应得到**拒绝**（或可取消语义），**且不 panic**。

## 49 地载阵（`dizaizhen`）**模式门槛缺失**（样本 127，2026-10-19）

**症状（门槛缺失）**：上游 `dizaizhen` 带 **`mode: ["guozhan"]`**（**仅国战模式可用**）；
原生实测在**身份局（4 人）**下 **`targets: vec![]` → `Accepted`** 且**效果生效**（目标摸 1，手牌 `1 → 2`）
⇒ **模式门槛未校验**。

**修法**：为 `CardKind::Dizaizhen`（及**其余带 `mode: ["guozhan"]` 的卡**）加入**模式门槛**校验，
身份局下应 `Rejected`（或以「不可用」方式从可选中排除）。

**验收**：样本 127 应能断言「**身份局下被拒**」；并新增国战模式下「摸 1 生效」的用例。

**族**：「**门槛缺失**」**第 2 例**（第 1 例：`qiankunbiao` 的 `he` 门槛，第 35 项）。

### 第 49 项补充：阵法卡模式门槛**普遍性核对**（2026-10-19）

上游 `zhenfa/card.js` 的 **10 张阵法卡全部**带 `mode: ["guozhan"]`；已交付 **5 张**经身份局普查：

| 卡 | 结果 | 定性 |
|---|---|---|
| **`dizaizhen`（地载阵）** | `[]` → **受理** | **门槛缺失（已确认）** |
| `tianfuzhen` / `qixingzhen` / `niaoxiangzhen` / `changshezhen` | 三形态均**被拒** | **原因未定**（门槛可能已生效，或其它条件未满足） |

⇒ 修法范围**至少**含 `dizaizhen`。

**逐卡定性（读上游 `enable`/`filterTarget` 后）**：`tianfuzhen`（需**存在大势力**）、`qixingzhen`（需**围攻/被围攻**）、
`changshezhen`（需 **`inline` 队列**关系）三张的拒绝**合理**（条件依赖国战概念，身份局天然不满足）；
**`niaoxiangzhen` 待专项核对**——其上游 `enable` **恒真**、`filterTarget` 仅要求**双方身份不同**，
身份局**满足该条件**却实测三形态均被拒 ⇒ **须专项用例确认**（可能另缺条件或另有门槛）。

（另：未交付的 5 张阵法卡正是转换缺口清单的 5 项 —— **两份独立核对一致**。）
