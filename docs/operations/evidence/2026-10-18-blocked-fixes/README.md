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

## 11 「牌区 `hes`/`he`」差异族：**命令面限定手牌**（NC08 样本 25/26/31 汇总，2026-10-19）

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

## 16 淑慎（`shushen`）未按回复点数计次（NC08 样本 43 新发现，2026-10-19）

**症状（规则损失）**：上游 `shushen` 的 `getIndex(event) { return event.num || 1; }` 表示
**一次回复 N 点应触发 N 次**（每次各自选目标、各自摸牌）；原生 `content/standard/skills/shu_shen.rs`
的 filter 只判 `amount > 0`，机器在 `Read` 阶段**单次结算** → 回复 2 点只触发 1 次。

**修法**：按 `Recover.amount` 计次（循环 N 次询问/结算，或改为按次挂起多个询问）。

**验收**：新增「一次回复 2 点 → 两次目标选择与两次摸牌」的行为用例；
现有两条单点用例保持不变。

## 17 护驾（`hujia`）在非身份局亦可发动（NC08 样本 44 新发现，2026-10-19）

**症状（原生更宽松）**：上游 `hujia` 的 `filter` 要求 `player.hasZhuSkill("hujia")`（**主公技旗标**，
仅身份局成立）；原生 `crates/engine-native/src/kernel/sha.rs` 的 `hujia_active` 用
`ctx.core.is_lord(self.target)`，而 `state.rs` 的 `is_lord` 在**非身份局返回 `true`**
（`match self.identity_roles() { Some(roles) => …, None => true }`）→ **原型/单挑模式下护驾可发动**。

**修法**：`hujia_active` 改为「**身份局且为主公**」判定（或为 `is_lord` 增加「须身份局」的调用口径），
并核对其它使用 `is_lord` 的技能是否同样受影响（**建议全库排查**）。

**验收**：新增「原型模式下护驾不可发动」的行为用例；身份局主公用例保持通过。
