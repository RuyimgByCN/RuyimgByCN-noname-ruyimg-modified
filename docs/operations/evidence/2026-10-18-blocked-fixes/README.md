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
