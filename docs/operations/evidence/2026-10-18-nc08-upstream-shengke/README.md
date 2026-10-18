# NC08 第⑦层样本 6：`_shengke`（五行生克，L06 wuxing）（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层「上游行为」对照样本 **6/37 来源**
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：由上游 JS 推导的 **6 条规则点**：**4 条对齐**（全局技、锁定、双方有属性门槛、
  五克/五生关系），**4 处差异登记**（S1 弃牌 `he` 自选 → 确定性首张、S2 属性牌默认比例、
  S3 全局技 vs 每座位实例、S4 `useCardToBegin` vs `CardUsed` 帧）
- 来源覆盖：L06 wuxing**首次**进入第⑦层上游对照（与样本 5 的 L19 欢乐卡牌为两个不同来源）

## 1 上游源码（归档快照，逐字引用；见 [upstream-shengke.js.txt](upstream-shengke.js.txt)）

`archive/upstream/noname-liuli/noname/extension/wuxing/extension.js:62-172`（`ai.effect` 为策略面，略去不引）：

```js
			_shengke:{
				trigger:{target:'useCardToBegin'},
				forced:true,
				popup:false,
				filter:function(event,player){
					if(_status.connectMode) return false;
					return event.card.wunature&&player.wunature;
				},
				content:function(){
					switch(trigger.card.wunature){
						case 'metal':
						switch(player.wunature){
							case 'wood':
								if(player.countCards('he')){
								game.log(player,'被'+get.translation(trigger.card.wunature)+'属性的卡牌克制');
								player.chooseToDiscard('你被金属性卡牌克制，需弃置一张牌',true,'he').ai=get.disvalue;player.popup('金克木')}return;
							case 'water':
								game.log(player,'得到'+get.translation(trigger.card.wunature)+'属性卡牌的加成');
								player.draw();player.popup('金生水');
								return;
						}
						return;
						case 'wood':
						switch(player.wunature){
							case 'soil':
								if(player.countCards('he')){
								game.log(player,'被'+get.translation(trigger.card.wunature)+'属性的卡牌克制');
								player.chooseToDiscard('你被木属性卡牌克制，需弃置一张牌',true,'he').ai=get.disvalue;player.popup('木克土')}return;
							case 'fire':
								game.log(player,'得到'+get.translation(trigger.card.wunature)+'属性卡牌的加成');
								player.draw();player.popup('木生火');
								return;
						}
						return;
						case 'water':
						switch(player.wunature){
							case 'fire':
								if(player.countCards('he')){
								game.log(player,'被'+get.translation(trigger.card.wunature)+'属性的卡牌克制');
								player.chooseToDiscard('你被水属性卡牌克制，需弃置一张牌',true,'he').ai=get.disvalue;player.popup('水克火')}return;
							case 'wood':
								game.log(player,'得到'+get.translation(trigger.card.wunature)+'属性卡牌的加成');
								player.draw();player.popup('水生木');
								return;
						}
						return;
						case 'fire':
						switch(player.wunature){
							case 'metal':
								if(player.countCards('he')){
								game.log(player,'被'+get.translation(trigger.card.wunature)+'属性的卡牌克制');
								player.chooseToDiscard('你被火属性卡牌克制，需弃置一张牌',true,'he').ai=get.disvalue;player.popup('火克金')}return;
							case 'soil':
								game.log(player,'得到'+get.translation(trigger.card.wunature)+'属性卡牌的加成');
								player.draw();player.popup('火生土');
								return;
						}
						return;
						case 'soil':
						switch(player.wunature){
							case 'water':
								if(player.countCards('he')){
								game.log(player,'被'+get.translation(trigger.card.wunature)+'属性的卡牌克制');
								player.chooseToDiscard('你被土属性卡牌克制，需弃置一张牌',true,'he').ai=get.disvalue;player.popup('土克水')}return;
							case 'metal':
								game.log(player,'得到'+get.translation(trigger.card.wunature)+'属性卡牌的加成');
								player.draw();player.popup('土生金');
								return;
						}
						return;
					}
				},
```

> 上引为 `extension.js:62-133` **逐字全文**（`trigger`/`filter`/`content` 五个 `case` 全在）；
> `ai.effect`（策略面，`:134-170`）按边界声明**不在规则对照范围**，见
> [upstream-shengke.js.txt](upstream-shengke.js.txt)。

属性初始化（同文件 `:27-61`，逐字见 txt）：每名角色 `list.randomGet()` 一个属性；
每张牌以 `Math.random() > (parseFloat(lib.config.wuxing_num_playpackconfig)||0)` 决定是否
获得属性——**未配置时默认 0**（即默认 0 张属性牌），与同扩展 `help` 文案「牌堆中三分之一的牌
会随机获得一个属性」不一致。

`_` 前缀语义：`noname/game/index.js:8859-8861` `if (skillId[0] === "_") game.addGlobalSkill(skillId)`
——`_shengke` 是**全局技**（全体角色自动生效，不需要授予）。

## 2 由 JS 推导的规则点 → 原生验收（新增 `nc08_upstream_shengke.rs`，5 项）

| # | 上游依据 | 判据（断言） | 结果 |
|---|---|---|---|
| 1 | `_` 前缀 + `game/index.js:8859-8861` | 四名角色开局**均持有** `wuxing_shengke`；角色属性全有、牌堆属性牌非空 | **对齐**（S3 接口差异另记） |
| 2 | `forced: true` | 结算**无** `trigger_confirm` 门 | **对齐** |
| 3 | `event.card.wunature && player.wunature` | 无属性牌 / 无属性角色 → 均不触发 | **对齐** |
| 4 | `case 'metal'`/`case 'wood'`/… 的「克」支 | 金克木 → 角色**强制弃 1 张**；弃牌堆 = 杀 + 弃牌 | **对齐**（范围差异 S1） |
| 5 | 各 case 的「生」支 | 金生水 → 角色**摸 1 张** | **对齐** |
| 6 | 无匹配 case → `return` | 无生克关系（土 vs 金）→ 不弃不摸 | **对齐** |

**原生实现引用**（工作树当前布局）：
- `crates/engine-native/src/content/host/filtering/common.rs:539-543`：`EventData::CardUsed`
  且 `targets` 含 owner、牌在 `card_natures` 表内、owner 有 `wunature`；
- `crates/engine-native/src/content/host/metadata/common.rs:619-625`：`EventKind::CardUsed` /
  `optional:false`（锁定）；
- `crates/engine-native/src/content/host/step/skills_status.rs:680-706`：`KE`/`SHENG` 查表，
  被克→`hand.first()` 弃置、被生→`draw_cards(1)`；
- `crates/engine-native/src/content/host/constants.rs:4-8`：
  `KE=[1,4,3,0,2]`（金克木/木克土/水克火/火克金/土克水）、
  `SHENG=[2,3,1,4,0]`（金生水/木生火/水生木/火生土/土生金），下标 0..4 = 金/木/水/火/土；
- `crates/engine-native/src/engine/construction.rs:437-470`：初始化钩子
  `InitHook::WuxingNatures`——角色/牌属性随机分配（引擎 RNG，1/3）并给**全部座位**插入
  生克技能实例；
- `crates/engine-native/src/engine/play_card.rs`：`CardUsed` 帧在效果机器**之前**入栈
  （例：`:624`、`:651`、`:1764`），故生克在牌生效前结算，与 `useCardToBegin` 同序。

## 3 差异登记（差异 + 影响分析 + 待办）

| 编号 | 差异 | 影响分析 | 待办 |
|---|---|---|---|
| **S1** | 弃牌从「角色自选 `he`」简化为「确定性弃首张手牌」；`countCards('he')` 门槛只体现为「手牌非空」 | 被克时：仅持装备的角色**不会**被强制弃牌（上游会弃装备）；玩家无法保护关键手牌。行为已由 `upstream_shengke_he_range_difference` 固定（装备仍在） | 宿主臂改 `WaitChoice`（观察者=被克角色，选项=`hand+equips`）——与样本 5 差异 K2/K4 同一改造点，宜合并修 |
| **S2** | 属性牌默认比例：上游 `wuxing_num_playpackconfig` 未配置 → **0 张**；原生按包数据 `nature_fraction` `1/3` 直接落地 | 默认对局中「牌有属性」的比例不同 → 生克触发频率不同（角色属性两者都有）。原生选择与扩展 `help` 文案（1/3）一致，但**与归档 JS 的运行默认不一致**；以源码/包数据事实固定（`wuxing.json` `"denominator": 3`） | 决定口径：按归档 JS 默认 0（则需配置面），或明确「按 help 意图 1/3」并写入规则基准差异表 |
| **S3** | 上游 `_` 前缀全局技（`lib.skill.global`，无拥有者）；原生给**每个座位**插一条 `SkillInstance` | 语义等价（全体生效），但接口不同：技能实例表、存档体积、死亡/复活与视图面都按「座位拥有」建模；`debug_has_skill` 对死者也为真 | 若统一为真正全局技，需引擎层「无主技能」概念；暂登记，避免为对齐改宿主模型 |
| **S4** | 上游 `trigger:{target:'useCardToBegin'}`（目标触发、牌生效前）；原生 `EventKind::CardUsed` 帧 | **结算顺序对齐**（`CardUsed` 帧先于效果机器入栈，`engine/play_card.rs`），但事件面/触发名不同：按上游事件名（`useCardToBegin`）观察的消费者（录像/日志投影）需映射 | 事件名映射表登记；本样本已用行为断言固定「牌生效前弃/摸」 |

**同族差异**：S1 与样本 5（苦无 K2/K4）是同一类「`he` 自选 → 确定性首张手牌」简化，
建议在规则基准里合并为一条系统性差异（影响所有 `chooseToDiscard(...,'he')` 类技能）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 上游对照定向 | **5 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | **1182 passed / 0 failed / 2 ignored**（1172 → +10，本批两样本各 +5） | [engine-native-tests.log](engine-native-tests.log) |
| 行数门禁 | 8279 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 5 未完成 / 待办（NC08 第⑦层）

- [ ] 差异 S1/S2 的口径判定（`he` 自选弃牌、默认属性比例）
- [ ] 差异 S3（全局技建模）与 S4（`useCardToBegin` 事件名映射）
- [ ] 属性随机流：原生走引擎 RNG，与上游 `Math.random` **不逐局等价**（规则面等价、随机流不同；g1_wuxing 已登记，本样本沿用）
- [ ] 表现/策略面（`popup`、`ai.effect`、`game.log` 文案、`_status.connectMode` 联机守卫）**不在规则对照范围**（边界声明）
