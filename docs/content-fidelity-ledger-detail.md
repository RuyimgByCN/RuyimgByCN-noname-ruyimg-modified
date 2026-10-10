# 内容保真度明细清单（机器生成，勿手改）

> 生成自 `python3 scripts/gen-fidelity-ledger.py`（2026-09-15）。
> 每个不完整 `.skill` 的 `// 近似` 或 `// TODO` 注释原文收录。complete 类只计数不列。

**总计**：5595 个 `.skill`，其中近似 573、TODO 0、完整 5022。

## 包 `bingshi`（41 个，不完整 2）

### 近似（2）

| 技能 | 路径 | 省略说明 |
|---|---|---|
| `old_pottuntian` | content/original/ported/bingshi/old_pottuntian.skill | （蓄力未建模，已注明）。 |
| `potheyun` | content/original/ported/bingshi/potheyun.skill | 上游 charlotte 排除以 locked/non_gainable/lord_only 过滤近似 |

## 包 `clan`（89 个，不完整 9）

### 近似（9）

| 技能 | 路径 | 省略说明 |
|---|---|---|
| `clanbaozu` | content/original/ported/clan/clanbaozu.skill | 限定技每局一次以 optional 近似（olxushen/batch P 同款）；已横置时仍弹确认 |
| `clanfenchai` | content/original/ported/clan/clanfenchai.skill | (1)「改为红桃」以「可用红色牌替换」近似（红桃为红子集，实际可选方片）； |
| `clanfenshi` | content/original/ported/clan/clanfenshi.skill | (1) 「每回合限一次」无对应 trigger 次数原语，以 optional 确认近似（玩家可 |
| `clanhuanghan` | content/original/ported/clan/clanhuanghan.skill | (1) 保族恢复/已发动计数未建模（需跨技能状态）；(2) 弃牌为确定性前 n 张 |
| `clanjiejian` | content/original/ported/clan/clanjiejian.skill | (1) isFirstTarget 以用牌后时机近似；(2) 非装备未过滤；(3) 目标应为事件 |
| `clanjiexuan` | content/original/ported/clan/clanjiexuan.skill | (1) 转换技☯状态门控未建模——上游同一时刻只有一种形态，DSL 双开； |
| `clanmuyin` | content/original/ported/clan/clanmuyin.skill | 上游族限定为字面「陈留吴氏」，DSL 以「与拥有者共享任一族」泛化（对单族武将 |
| `clanshize` | content/original/ported/clan/clanshize.skill | (1) 攻击范围增益仅作用于自己（上游可选宗族角色——modifier 值源只随技能 |
| `clanxieshu` | content/original/ported/clan/clanxieshu.skill | (1) 弃牌为确定性前 n 张（上游手选）；(2) 自伤时双入口可能各触发一次； |

## 包 `collab`（151 个，不完整 25）

### 近似（25）

| 技能 | 路径 | 省略说明 |
|---|---|---|
| `dchaoshi` | content/original/ported/collab/dchaoshi.skill | (1)「点数之和≥100」门槛未建模（expansion 点数求和原语缺失，见缺口登记）； |
| `dchuibian` | content/original/ported/collab/dchuibian.skill | （猜色与获取/弃牌分支未建模，已注明）。 |
| `dclbjiuxian` | content/original/ported/collab/dclbjiuxian.skill | view_as 将普通锦囊当酒使用，但丢失了「仅限多目标锦囊」的精确 |
| `dcrende` | content/original/ported/collab/dcrende.skill | take_one_card 仅拿1张（原为2张），choose_use_virtual_sha 仅限杀（原为任意基本牌）， |
| `dcsantou` | content/original/ported/collab/dcsantou.skill | 无条件防止伤害（以 prevent_current_damage 表达），丢失了体力值 |
| `dcshentui` | content/original/ported/collab/dcshentui.skill | 保留了主动加范围+虚拟杀的核心循环，丢失了「点数与攻击范围比较」 |
| `dczuandai` | content/original/ported/collab/dczuandai.skill | 。 |
| `mbshezi` | content/original/ported/collab/mbshezi.skill | 为出牌阶段直接获取目标一张装备牌。 |
| `mbzhengpeng` | content/original/ported/collab/mbzhengpeng.skill | 为回合结束时摸1张牌。 |
| `natufu` | content/original/ported/collab/natufu.skill | 使用普通锦囊后可选摸 1 或选人造成 1 点伤害（条件分支与火焰属性未建模）。 |
| `natuheng` | content/original/ported/collab/natuheng.skill | 提供手牌 rep 闪与无次数限制出杀；「手牌构成门控」与每轮限一次未精确建模。 |
| `natulie` | content/original/ported/collab/natulie.skill | 固定调整至 2 张手牌；递增 X 与「弃牌后造成伤害」未建模。 |
| `natuxing` | content/original/ported/collab/natuxing.skill | 开局摸 5 张（兴区未建模，以手牌直增代替）；造成伤害后摸 1； |
| `natuyi` | content/original/ported/collab/natuyi.skill | 选 1–2 名有牌目标后各获其一张 he，再可选视为使用无距离【杀】。 |
| `olanle` | content/original/ported/collab/olanle.skill | 为受伤时摸1张牌。 |
| `oldianbu` | content/original/ported/collab/oldianbu.skill | 游戏开始摸 5 张（十三张与扑克组合未建模）。 |
| `oldigong` | content/original/ported/collab/oldigong.skill | 为使用伤害牌时伤害+1。 |
| `oldingxi` | content/original/ported/collab/oldingxi.skill | 为弃牌阶段结束时摸1张牌。 |
| `olfengzhu` | content/original/ported/collab/olfengzhu.skill | 为非锁定、可选触发，摸1张牌（略去体力值计数与标记改名）。 |
| `olliangzi` | content/original/ported/collab/olliangzi.skill | 装备当酒；无次数限制与势力计数未建模。 |
| `olsanou` | content/original/ported/collab/olsanou.skill | 伤害来源→摸牌+加标记已精确表达。 |
| `olsuizhong` | content/original/ported/collab/olsuizhong.skill | 受伤+低血量→回复体力+弃置一名其他角色手牌/装备。 |
| `olweidai` | content/original/ported/collab/olweidai.skill | 成为牌的目标后加 1 上限（势力门控与压上限/变更势力未建模）。 |
| `quanjia` | content/original/ported/collab/quanjia.skill | （登记债）：①机内校验在 optional 确认之后——确认可能空开（filter 未含 |
| `renqianyao` | content/original/ported/collab/renqianyao.skill | 保留核心：按轮数摸牌 + 虚拟【杀】。缺失三项轮数门槛增益， |

## 包 `diy`（35 个，不完整 1）

### 近似（1）

| 技能 | 路径 | 省略说明 |
|---|---|---|
| `nsdiemou` | content/original/ported/diy/nsdiemou.skill | （登记债）：①game.players.length（含死亡？上游口径为存活）以 alive_count() |

## 包 `huanle`（2 个，不完整 1）

### 近似（1）

| 技能 | 路径 | 省略说明 |
|---|---|---|
| `mianju` | content/original/ported/huanle/huanle_impl.skill | 旧 JS 按已受伤自动选回复、否则摸牌；DSL 以 choose_draw_or_recover |

## 包 `huicui`（253 个，不完整 31）

### 近似（31）

| 技能 | 路径 | 省略说明 |
|---|---|---|
| `anyong` | content/original/ported/huicui/anyong.skill | 造成1点伤害 |
| `chijian` | content/original/ported/huicui/chijian.skill | 以 ignore_armor + sha_use_limit 表达青钢穿透与额外出杀； |
| `dcdeshao` | content/original/ported/huicui/dcdeshao.skill | 摸1张牌；旧JS有额外条件/选择逻辑 |
| `dcfanhuo` | content/original/ported/huicui/dcfanhuo.skill | 限定发动后立即失去 3 点体力；兴邦次数提升与回合末延后失体未建模。 |
| `dcfanshi` | content/original/ported/huicui/dcfanshi.skill | 回复2点体力 |
| `dcfudou` | content/original/ported/huicui/dcfudou.skill | 摸1张牌；旧JS有额外条件/选择逻辑 |
| `dcgangjian` | content/original/ported/huicui/dcgangjian.skill | 摸1张牌；旧JS有额外条件/选择逻辑 |
| `dcguangshi` | content/original/ported/huicui/dcguangshi.skill | 摸1张牌；旧JS有额外条件/选择逻辑 |
| `dcguyin` | content/original/ported/huicui/dcguyin.skill | 摸1张牌；旧JS有额外条件/选择逻辑 |
| `dcjinjin` | content/original/ported/huicui/dcjinjin.skill | 摸1张牌；旧JS有额外条件/选择逻辑 |
| `dcjizhong` | content/original/ported/huicui/dcjizhong.skill | 令目标摸2张牌 |
| `dclongyuan` | content/original/ported/huicui/dclongyuan.skill | 回复1点体力 |
| `dcmoshou` | content/original/ported/huicui/dcmoshou.skill | 摸1张牌；旧JS有额外条件/选择逻辑 |
| `dcmouli` | content/original/ported/huicui/dcmouli.skill | 回复1点体力 |
| `dcqingren` | content/original/ported/huicui/dcqingren.skill | 摸1张牌；旧JS有额外条件/选择逻辑 |
| `dcqinqing` | content/original/ported/huicui/dcqinqing.skill | 摸1张牌；旧JS有额外条件/选择逻辑 |
| `dcqinshen` | content/original/ported/huicui/dcqinshen.skill | 摸1张牌；旧JS有额外条件/选择逻辑 |
| `dctujue` | content/original/ported/huicui/dctujue.skill | 摸1张牌；旧JS有额外条件/选择逻辑 |
| `dcxingbang` | content/original/ported/huicui/dcxingbang.skill | 摸一并强制交一张 he + 获一张手牌；循环比牌与虚拟杀未建模。 |
| `dcyijia` | content/original/ported/huicui/dcyijia.skill | 摸1张牌；旧JS有额外条件/选择逻辑 |
| `dcyongdi` | content/original/ported/huicui/dcyongdi.skill | 令目标摸1张牌 |
| `dcyuguan` | content/original/ported/huicui/dcyuguan.skill | 减少体力上限 |
| `dczhuguo` | content/original/ported/huicui/dczhuguo.skill | 摸1张牌；旧JS有额外条件/选择逻辑 |
| `dczhuoli` | content/original/ported/huicui/dczhuoli.skill | 回复1点体力 |
| `dczifu` | content/original/ported/huicui/dczifu.skill | 摸1张牌；旧JS有额外条件/选择逻辑 |
| `liedan` | content/original/ported/huicui/liedan.skill | 摸1张牌；旧JS有额外条件/选择逻辑 |
| `shiwu` | content/original/ported/huicui/shiwu.skill | 仅保留「视为决斗」主收益；伤害后获牌与夺剑/死后授剑未建模。 |
| `yaopei` | content/original/ported/huicui/yaopei.skill | 回复1点体力 |
| `zhanwan` | content/original/ported/huicui/zhanwan.skill | 摸1张牌；旧JS有额外条件/选择逻辑 |
| `zhengding` | content/original/ported/huicui/zhengding.skill | 回复1点体力 |
| `zhuangrong` | content/original/ported/huicui/zhuangrong.skill | 条件「hp==1 \|\| hand==1」的逻辑或未建模（DSL 无逻辑或，登记）；回复1点体力 |

## 包 `jsrg`（181 个，不完整 45）

### 近似（45）

| 技能 | 路径 | 省略说明 |
|---|---|---|
| `jsrg_new_juezhi` | content/original/ported/jsrg/jsrg_new_juezhi.skill | 不废除对应栏；回合内伤害+1 未建模。 |
| `jsrg_new_juxia` | content/original/ported/jsrg/jsrg_new_juxia.skill | 不比较双方技能数。 |
| `jsrgbaowei` | content/original/ported/jsrg/jsrgbaowei.skill | 默认对事件来源造成伤害（丢失了目标选择分支和人数判断逻辑）。 |
| `jsrgbazheng` | content/original/ported/jsrg/jsrgbazheng.skill | 上游 debateShowOpinion 时点按颜色改写（同色即同票）；DSL 钉死为 |
| `jsrgcangchu` | content/original/ported/jsrg/jsrgcangchu.skill | ）optional → choose_optional_targets_draw(self, 1, 1, 5, any) |
| `jsrgchaozheng` | content/original/ported/jsrg/jsrgchaozheng.skill | P1 参与者亮「首张手牌」为意见（上游 AI 随机）；无手牌者不参与（同上游）； |
| `jsrgchengxu` | content/original/ported/jsrg/jsrgchengxu.skill | 不限同势力，所有角色均不能响应你的杀与普通锦囊。 |
| `jsrgchushi` | content/original/ported/jsrg/jsrgchushi.skill | （未建模）：①非主公门槛（取反不可达，改为可选发动自律）；② |
| `jsrgdailao` | content/original/ported/jsrg/jsrgdailao.skill | 摸两张牌并跳过剩余阶段（丢失了“无可用手牌”的前置条件、展示手牌、直接结束回合的精确语义）。 |
| `jsrgdaimou` | content/original/ported/jsrg/jsrgdaimou.skill | 上游 addTempSkill jsrgdaimou_used 回合末移除）。 |
| `jsrgdaoren` | content/original/ported/jsrg/jsrgdaoren.skill | （登记债）：①伤害按无来源处理（上游 damage("nocard") 同口径 ✓）；② |
| `jsrgdingce` | content/original/ported/jsrg/jsrgdingce.skill | （登记债）：①同色分支的虚拟【洞烛先机】未结算——该牌不在引擎牌池 |
| `jsrgeqian` | content/original/ported/jsrg/jsrgeqian.skill | ——再次发动需等下个结束阶段。 |
| `jsrgfangjie` | content/original/ported/jsrg/jsrgfangjie.skill | （未建模②段）：蓄谋牌存在性取反不可达（DSL 无 !），无蓄谋牌分支 |
| `jsrgfayi` | content/original/ported/jsrg/jsrgfayi.skill | 上游「任意名意见不同角色」以单目标近似；意见异同约束未在目标选择中 |
| `jsrgfeiyang` | content/original/ported/jsrg/jsrgfeiyang.skill | 上游弃判定区「一张」，DSL 弃全部（判定区仅一张时精确）。 |
| `jsrgfumou` | content/original/ported/jsrg/jsrgfumou.skill | （未建模）：①议事意见不同者的颜色禁打（跨目标颜色禁令）；②目标 |
| `jsrgfuni` | content/original/ported/jsrg/jsrgfuni.skill | /未建模（③段 buff 强命）：弃牌堆影联动后的「本回合使用牌无距离 |
| `jsrgfuran` | content/original/ported/jsrg/jsrgfuran.skill | 受伤害后直接回复1点体力（丢失了“来源不在攻击范围内”的判定和延迟到回合结束的时机）。 |
| `jsrgfusha` | content/original/ported/jsrg/jsrgfusha.skill | 视为对一名目标角色造成1点伤害（丢失了限定技标签、伤害值动态计算、觉醒技联动、攻击范围判定）。 |
| `jsrgjiaohao` | content/original/ported/jsrg/jsrgjiaohao.skill | （未建模①段）：他人将装备置入你的装备区需 per-kind 装备转移原语。 |
| `jsrgjiebing` | content/original/ported/jsrg/jsrgjiebing.skill | 不检验蓄谋数与主公体力；以 mark_count 门控。 |
| `jsrgjinfa` | content/original/ported/jsrg/jsrgjinfa.skill | ）+ |
| `jsrgjueyin` | content/original/ported/jsrg/jsrgjueyin.skill | 本回合全场受伤+1 未建模。 |
| `jsrgniluan` | content/original/ported/jsrg/jsrgniluan.skill | 不消耗弃牌；不筛选「未对你造成过伤害」；摸两张牌分支未建模。 |
| `jsrgpingrong` | content/original/ported/jsrg/jsrgpingrong.skill | 死亡等场景下挂起额外回合的引擎默认顺序处理；尾随 check 读聚合 |
| `jsrgqiantun` | content/original/ported/jsrg/jsrgqiantun.skill | （登记债）：①拼点双方依次明选（上游 chooseToCompare 暗选同时亮出， |
| `jsrgqingping` | content/original/ported/jsrg/jsrgqingping.skill | 摸 alive_count() 张牌（丢失了“所有在范围内角色手牌≤你”的前置条件判断）。 |
| `jsrgshacheng` | content/original/ported/jsrg/jsrgshacheng.skill | ②目标域为「任一本回合失去过牌的其他角色」宽于上游的「此杀目标之一」； |
| `jsrgshanzheng` | content/original/ported/jsrg/jsrgshanzheng.skill | （P2 表达力边界，逐点注明）：①上游参与者含自己（自己也是意见方），此处 |
| `jsrgsheju` | content/original/ported/jsrg/jsrgsheju.skill | P1 亮「首张手牌」为意见；「唯一目标」约束未建模（任意杀指定即触发）； |
| `jsrgshelun` | content/original/ported/jsrg/jsrgshelun.skill | （P2 表达力边界，逐点注明）：①议事参与者为「所有其他有手牌角色」， |
| `jsrgshishou` | content/original/ported/jsrg/jsrgshishou.skill | 「本回合不能使用牌」(jsrgshishou_nouse) 的 cardEnabled/cardUsable modifier + jsrgshishou_burn |
| `jsrgtushe` | content/original/ported/jsrg/jsrgtushe.skill | 使用牌后摸 event_target_count 张牌（丢失了“无基本牌手牌”判定、展示手牌确认、chooseBool可选）。 |
| `jsrgwudao` | content/original/ported/jsrg/jsrgwudao.skill | 不检验手牌为 0；以 mark_count 门控。 |
| `jsrgxiongbao` | content/original/ported/jsrg/jsrgxiongbao.skill | 上游「额外展示一张」实为固定两意见（fixedResult 双条目）； |
| `jsrgxuchong` | content/original/ported/jsrg/jsrgxuchong.skill | （未建模⒉分支）：令当前回合角色本回合手牌上限+2 需跨目标临时 |
| `jsrgxushi` | content/original/ported/jsrg/jsrgxushi.skill | 上游为「卡牌-角色顺序配对、任意名」多目标给予，DSL 单目标 |
| `jsrgyangming` | content/original/ported/jsrg/jsrgyangming.skill | （登记债）：①拼点双方依次明选（chooseToCompare 暗选同时亮出）；② |
| `jsrgyaoyan` | content/original/ported/jsrg/jsrgyaoyan.skill | （未建模）：①逐角色 chooseBool 同意→以发起者一次多选代替； |
| `jsrgzhasi` | content/original/ported/jsrg/jsrgzhasi.skill | 不检验伤害≥体力；不换制衡/猘横；不进入「不计入距离和座次」。 |
| `jsrgzhenglve` | content/original/ported/jsrg/jsrgzhenglve.skill | 授标不检查目标已有标记；允许放弃（上游 bool:true 强制选满）。 |
| `jsrgzhenqiao` | content/original/ported/jsrg/jsrgzhenqiao.skill | 攻击范围+1修饰（丢失了“额外指定一个目标”的效果计数增加）。 |
| `jsrgzhiheng` | content/original/ported/jsrg/jsrgzhiheng.skill | 不检验「因牌的效果」与「目标本回合响应过你」。 |
| `jsrgzuozhan` | content/original/ported/jsrg/jsrgzuozhan.skill | （登记债）：①enterGame/phaseBefore 双时点以 game_started 单点近似；② |

## 包 `key`（250 个，不完整 85）

### 近似（85）

| 技能 | 路径 | 省略说明 |
|---|---|---|
| `akane_quanqing` | content/original/ported/key/akane_quanqing.skill | 交给改为弃置；不限攻击范围；点数分支与弃牌/伤害未建模。 |
| `akane_yifu2` | content/original/ported/key/akane_yifu2.skill | 交给改为弃置；不限主公技持有者；不返还手牌。 |
| `ao_diegui` | content/original/ported/key/ao_diegui.skill | (1)「七影蝶交给一名角色」以自己取回近似（give-to-other 原语缺失）； |
| `ao_kuihun` | content/original/ported/key/ao_kuihun.skill | 不选对手手牌进扩展区；距离/出杀/手牌上限随花色数变化未建模。 |
| `ao_shixin` | content/original/ported/key/ao_shixin.skill | 不检验扩展区花色数≥3；以 mark_count 门控。 |
| `asara_shelu` | content/original/ported/key/asara_shelu.skill | 不展示对方手牌；不比较花色/点数；回复分支未建模。 |
| `erika_shisong` | content/original/ported/key/erika_shisong.skill | 同位序同类型摸牌子句（跨回合 useCard 历史对照）未建模。 |
| `fuuko_xingdiao` | content/original/ported/key/fuuko_xingdiao.skill | 不展示为「星」；不计入手牌上限/不可弃未建模；其他角色取「星」未建模；每轮开始前不再摸至。 |
| `haruko_haofang` | content/original/ported/key/haruko_haofang.skill | 无中额外摸两张未建模（card-name-scoped 过滤器无 wuzhong）。 |
| `hina_shenshi` | content/original/ported/key/hina_shenshi.skill | 不限神势力；不打 gaintag；不把一张牌置顶；应变子技未建模。 |
| `hiroto_tuolao` | content/original/ported/key/hiroto_tuolao.skill | 不检验回合数与虎驭失去历史；以 mark_count 门控；时机取结束阶段而非 phaseAfter。 |
| `inari_baiwei` | content/original/ported/key/inari_baiwei.skill | 不提供动态牌名按钮，固定四条基本牌通道；毒本就不在通道内。 |
| `inari_baiwei_draw` | content/original/ported/key/inari_baiwei_draw.skill | 上游匹配 indexOf==0（含 inari_baiwei* 变体），本定义精确匹配主 inari_baiwei(3124826192)； |
| `kamome_huanmeng` | content/original/ported/key/kamome_huanmeng.skill | （登记债）：逐张二选（顶/底）近似上游拖拽任意排列——堆内自由序未建模 |
| `kamome_yangfan` | content/original/ported/key/kamome_yangfan.skill | 不按失去张数倍算；开局装备行李箱未建模。 |
| `kano_liezhen` | content/original/ported/key/kano_liezhen.skill | 不检验本回合是否用过牌；不选类型视为使用；仁库未建模。 |
| `kengo_guidui` | content/original/ported/key/kengo_guidui.skill | 不先检验是否已废除，每回合都尝试恢复五栏。 |
| `kengo_weishang` | content/original/ported/key/kengo_weishang.skill | 一次废除全部栏而非自选一栏；加成常驻，不按已废栏种类分别生效。 |
| `kengo_weishang_sha` | content/original/ported/key/kengo_weishang_sha.skill | 不检验武器栏是否已废除。 |
| `key_huanjie` | content/original/ported/key/key_huanjie.skill | 不翻转已是底摸的方向；判定阶段从牌堆底判定未建模；非摸牌阶段的摸牌不改向。 |
| `komari_tiankou` | content/original/ported/key/komari_tiankou.skill | 上游 chooseControl 的发动权）→ 花色守卫（红桃=2/方片=4，card_attribute 属性 2）→ |
| `kotori_skill_jin` | content/original/ported/key/kotori_skill_jin.skill | 不展示本阶段获牌，不去重检验花色。 |
| `kotori_skill_key` | content/original/ported/key/kotori_skill_key.skill | 护甲未建模；扣体力与摸牌同时发生，非回合结束延迟。 |
| `kotori_skill_qun` | content/original/ported/key/kotori_skill_qun.skill | 不检验双条件阈值，不发临时标记，改为持久手牌上限加成。 |
| `kotori_skill_wei` | content/original/ported/key/kotori_skill_wei.skill | 只保留「你弃后摸」分支；对方二选一未建模。 |
| `kud_buhui` | content/original/ported/key/kud_buhui.skill | 不弃装备摸等量；不获得「冲阵」。 |
| `kyoko_jingce` | content/original/ported/key/kyoko_jingce.skill | 不检验使用牌数；额外阶段未建模。 |
| `kyoko_shelie` | content/original/ported/key/kyoko_shelie.skill | 不检验装备区花色种类 > 1（争议计数）门控。 |
| `kyoko_zhiheng` | content/original/ported/key/kyoko_zhiheng.skill | 装备四花色且弃光手牌的额外摸 1 未建模。 |
| `kyou_duanfa` | content/original/ported/key/kyou_duanfa.skill | 非限定技标记，靠卸技实现一次；本回合被指定后摸牌未建模；任意伤害只要体力不够都会弹出。 |
| `liyingxia_sanli` | content/original/ported/key/liyingxia_sanli.skill | 不按第 1/2/3 次分支；第 3 次交牌未建模。 |
| `liyingxia_wumai` | content/original/ported/key/liyingxia_wumai.skill | 不限蜀势力；不授予技能；摸牌数不按受伤角色计数。 |
| `liyingxia_zhenjun` | content/original/ported/key/liyingxia_zhenjun.skill | 不按使用计数扩目标数；下回合伤害+1 未建模。 |
| `lucia_zhenren` | content/original/ported/key/lucia_zhenren.skill | 不限其他角色回合；弃等量名角色装备/判定区未建模。 |
| `mia_fengfa` | content/original/ported/key/mia_fengfa.skill | 不统计上回合使用牌数，固定 +1。 |
| `mia_qianmeng` | content/original/ported/key/mia_qianmeng.skill | 不把牌插入牌堆中部；使命成败与衍生技未建模。 |
| `mia_shihui` | content/original/ported/key/mia_shihui.skill | 摸牌阶段改写与上回合弃置计数未建模；先回体再弃牌（弃牌选牌不恢复后续 VM）。 |
| `midori_nonghuan` | content/original/ported/key/midori_nonghuan.skill | 不按体力限次；不限每名角色一次；不交还；不清除历史。 |
| `midori_tishen` | content/original/ported/key/midori_tishen.skill | 不变身；靠卸技实现一次。 |
| `miki_hydrogladiator_skill` | content/original/ported/key/miki_hydrogladiator_skill.skill | 弃牌为确定性前两张；距离 1 群体弃牌未建模。 |
| `miki_zhiluo` | content/original/ported/key/miki_zhiluo.skill | 不检验装备空/攻击范围；视为【杀】分支未建模。 |
| `minagi_peiquan` | content/original/ported/key/minagi_peiquan.skill | 不消耗米券；不记录已选选项；弃牌与亮牌使用未建模。 |
| `mio_tishen` | content/original/ported/key/mio_tishen.skill | 不变身；靠卸技实现一次。 |
| `misuzu_nongyin` | content/original/ported/key/misuzu_nongyin.skill | 不排除锦囊；转化牌进判定区当乐未建模。 |
| `misuzu_zhongyuan` | content/original/ported/key/misuzu_zhongyuan.skill | (1) 花色点数受限于所持牌（上游任意 chooseButton）；(2) 限定技每局一次未建模 |
| `nagisa_fuxin` | content/original/ported/key/nagisa_fuxin.skill | 不限自己回合；不覆盖弃牌/获牌；复原武将牌未建模。 |
| `nao_shouqing2` | content/original/ported/key/nao_shouqing2.skill | 不限【桃】为素材、不限「拥有守琴的受伤角色」；标记与手牌上限+1 未建模。 |
| `nao_shouqing3` | content/original/ported/key/nao_shouqing3.skill | 「手牌上限+标记数」modifier + 标记累计需 mark modifier 原语（countMark 作为 hand_limit |
| `nao_wanxin` | content/original/ported/key/nao_wanxin.skill | 不筛选「本回合受过伤害」；不翻回/解锁/解除横置。 |
| `nk_shekong` | content/original/ported/key/nk_shekong.skill | 只保留「你摸等量、其摸 1」分支；对方弃等量未建模。 |
| `riki_mengzhong` | content/original/ported/key/riki_mengzhong.skill | 不检验「问计获得过牌≥3」；以 mark_count 门控。 |
| `rumi_shuwu2` | content/original/ported/key/rumi_shuwu2.skill | 不评估体力/出杀/锦囊伤害条件；失去体力分支未建模。 |
| `ryoichi_baoyi` | content/original/ported/key/ryoichi_baoyi.skill | 按失去一张计；循环弃牌与性别筛选未建模。 |
| `sasami_baoqiu` | content/original/ported/key/sasami_baoqiu.skill | 。 |
| `sasami_miaobian` | content/original/ported/key/sasami_miaobian.skill | 上游 changeHp 单事件以 hp_lost+hp_recovered 两触发近似；只增不减（与上游一致）； |
| `satomi_daohai` | content/original/ported/key/satomi_daohai.skill | 结束阶段视为五谷未建模；不限「本回合获得的牌」。 |
| `seira_yinyuan` | content/original/ported/key/seira_yinyuan.skill | 不限姻缘绑定；不回复体力；手牌可见未建模。 |
| `shiorimiyuki_banyin` | content/original/ported/key/shiorimiyuki_banyin.skill | 不筛选「已受伤」目标；任意其他角色均可。 |
| `shiorimiyuki_tingxian` | content/original/ported/key/shiorimiyuki_tingxian.skill | 标记牌与「未使用扣血」负面分支未建模，只保留收益面；临时技补授未建模。 |
| `shiroha_guying` | content/original/ported/key/shiroha_guying.skill | 不判定花色；获得「预兆」后改为可选未建模。 |
| `shiroha_jiezhao` | content/original/ported/key/shiroha_jiezhao.skill | 替换选项未限定「预兆」区（含手/装备），且需改写资格回合内全程开放（上游仅 |
| `shiroha_yuzhao` | content/original/ported/key/shiroha_yuzhao.skill | （登记债）：①enterGame/phaseBefore 开局窗口以 game_started 近似 |
| `shiroha_yuzhao_umi` | content/original/ported/key/shiroha_yuzhao_umi.skill | （登记债）：①距离 ≤1 门控以 alive_group_count? 不可表达——省略（上游 |
| `shizuku_sanhua` | content/original/ported/key/shizuku_sanhua.skill | 时机取濒死而非死亡（死亡后无法选目标）；不从牌堆检索四种不同名基本牌。 |
| `shizuku_sizhi2` | content/original/ported/key/shizuku_sizhi2.skill | 不限带「丝织」标记的牌，改为全体黑/红牌。 |
| `shizuru_nianli` | content/original/ported/key/shizuru_nianli.skill | 每轮一次限制（nianli_round）与展示流程未建模；修正此前误标 |
| `sunohara_jianren` | content/original/ported/key/sunohara_jianren.skill | 不按来源性别改摸牌数，一律摸 1。 |
| `tomoya_wangjin` | content/original/ported/key/tomoya_wangjin.skill | 取「不在攻击范围内」分支的摸 2；范围内令对方摸/回体/出杀未建模。 |
| `tomoyo_changshi` | content/original/ported/key/tomoyo_changshi.skill | 「攻击范围内」条件未建模（任意角色回复均触发，含自己）；获得牌≥2 摸牌 |
| `tomoyo_wuwei` | content/original/ported/key/tomoyo_wuwei.skill | 不记录「已以此法使用过的花色」，任意花色均可转化。 |
| `tsumugi_huilang` | content/original/ported/key/tsumugi_huilang.skill | （登记债）：①置入域 "he"（手+装备）收窄为手牌；②消耗面（移去赋使用牌 |
| `tsumugi_huilang2` | content/original/ported/key/tsumugi_huilang2.skill | （登记债）：①动态上限 num=置入张数收窄为字面量 8（choose_targets_draw |
| `umi_chaofan` | content/original/ported/key/umi_chaofan.skill | 不检验花色不同；无体力分支；火焰伤害未建模。 |
| `umi_lunhui` | content/original/ported/key/umi_lunhui.skill | ①候选确认在条件不满足的回合仍会打开（引擎 optional 语义，confirm 后 |
| `yoshino_jueyi` | content/original/ported/key/yoshino_jueyi.skill | 不选绑定目标、不限本回合第一张；不可响应未建模。 |
| `yui_lieyin` | content/original/ported/key/yui_lieyin.skill | 全程强制红牌当杀，不选手选分支、不限出牌阶段、杀当决斗未建模。 |
| `yui_takaramono` | content/original/ported/key/yui_takaramono.skill | 不检验体力≤1 / 有人死亡 / 因「将」获得≥3 张；以 mark_count 门控。 |
| `yuiko_dilve` | content/original/ported/key/yuiko_dilve.skill | 牌堆底替换拼点牌（onCompare bottomCards）未建模；训诫变体分支未区分。 |
| `yukito_yaxiang` | content/original/ported/key/yukito_yaxiang.skill | 不变身（reinit 无原语）；靠卸技实现一次。 |
| `yusa_yanyi` | content/original/ported/key/yusa_yanyi.skill | 单目标；不限距离/攻击范围；满血你摸牌与交牌未建模。 |
| `yuzuru_deyi` | content/original/ported/key/yuzuru_deyi.skill | 以 mark_count 门控，非 awakenSkill；不检验觉醒条件外的其他前置。 |
| `yuzuru_kunfen` | content/original/ported/key/yuzuru_kunfen.skill | 不检验往生标记；交给其他角色 2 张未建模。 |
| `yuzuru_quji` | content/original/ported/key/yuzuru_quji.skill | 弃牌数不强制等于已损体力；弃黑不失去体力。 |
| `yuzuru_wangsheng` | content/original/ported/key/yuzuru_wangsheng.skill | 以 mark_count 门控，非 awakenSkill；不阻止死亡结算本身；交给全部牌未建模。 |
| `yuzuru_wuxin` | content/original/ported/key/yuzuru_wuxin.skill | 弃牌为确定性前 1 张（上游手选）。 |

## 包 `mobile`（420 个，不完整 26）

### 近似（26）

| 技能 | 路径 | 省略说明 |
|---|---|---|
| `duoduan` | content/original/ported/mobile/duoduan.skill | （登记债）：①「无效」以 CancelIncomingCardUse 近似（上游 excluded |
| `friendbiwei` | content/original/ported/mobile/friendbiwei.skill | （登记债）：①「最大点数」约束在机器入口校验（选择器无该过滤面， |
| `gaoyuan` | content/original/ported/mobile/gaoyuan.skill | （登记债）：①targetEnabled 以「使用者射程内 + 未被取消」近似（未含 |
| `jibing` | content/original/ported/mobile/jibing.skill | （登记债）：①「改为」摸牌替代语义未建模（摸牌照常执行）；②「兵」当 |
| `lianxi` | content/original/ported/mobile/lianxi.skill | 第二句「杀被弃置后视为使用无距无次杀」无对应触发过滤器/原语，未建模。 |
| `mbbeiming` | content/original/ported/mobile/mbbeiming.skill | （登记债）：①gain2 展示表现面未建模；②牌堆顶向下的搜索顺序为引擎 |
| `mbqiangyong` | content/original/ported/mobile/mbqiangyong.skill | （登记债）：①X 以 SHA_PLAYED 回合旗近似（owner 出牌回合内等价； |
| `mbsuwang` | content/original/ported/mobile/mbsuwang.skill | （登记债）：①摸牌阶段改为获得宿望牌+令其他角色摸二面未建模；②useCard |
| `mbxiahui` | content/original/ported/mobile/mbxiahui.skill | 第②句「连诛/黑色牌获得封禁」无对应原语，未建模。 |
| `mbyouye` | content/original/ported/mobile/mbyouye.skill | （登记债）：①分配面（受伤后分配所有「蓄」）未建模；②上游精确门控以 |
| `mbzhenfu` | content/original/ported/mobile/mbzhenfu.skill | （登记债）：①「因弃置失去」以「本回合失去过牌」超集近似 |
| `moucuan` | content/original/ported/mobile/moucuan.skill | （登记债）：①觉醒动画未建模；②“兵”堆供给依赖 jibing 置入面（S1-B |
| `poxiang` | content/original/ported/mobile/poxiang.skill | 所交之牌由原语流程选定（上游自选）；「摸三张不计入手牌上限」未建模； |
| `rebiaozhao` | content/original/ported/mobile/rebiaozhao.skill | （登记债）：①同点数入弃失体力面未建模；②准备阶段移去+摸至全场最高手牌 |
| `rebiaozhao3` | content/original/ported/mobile/rebiaozhao3.skill | （登记债）：①上游 forced → optional 确认先行（benyu 惯例）；②上游目标 |
| `requanji` | content/original/ported/mobile/requanji.skill | （登记债）：①getIndex 逐点拆分未建模（多点伤害一次结算，olquanji 同款）； |
| `rezhennan` | content/original/ported/mobile/rezhennan.skill | （登记债）：①锦囊类型以 CardKind::is_trick 近似（上游 type2 含延迟 |
| `rw_baiyin_skill` | content/original/ported/mobile/rw_baiyin_skill.skill | （登记债）：①arm2「失去装备区的照月狮子盔时回复1摸2」未建模—— |
| `rw_lanyinjia` | content/original/ported/mobile/rw_lanyinjia.skill | lanyinjia2「受杀伤弃置此甲」与「不会无效」锁由装备卡侧承担，技能块不迁移。 |
| `rw_minguangkai_cancel` | content/original/ported/mobile/rw_minguangkai_cancel.skill | 【火烧连营】（huoshaolianying）目标取消缺专用选择器（fire-trick |
| `shuliang` | content/original/ported/mobile/shuliang.skill | （登记债）：①上游 direct+chooseCardButton(bool) 单点决断拆分为触发确认 |
| `tunchu` | content/original/ported/mobile/tunchu.skill | （登记债）：①「有粮不能使用【杀】」mod 面未建模（无 cardEnabled 原语）； |
| `xinbingyi` | content/original/ported/mobile/xinbingyi.skill | （登记债）：①「类型均相同」维度未建模（类型计数无表达式）；②动态上限 |
| `yixiang` | content/original/ported/mobile/yixiang.skill | （登记债）：①"你没有的基本牌"以手牌中不存在的 kind 去重近似 |
| `yuxiang` | content/original/ported/mobile/yuxiang.skill | 上游为布尔 ±1，此处 ±护甲值（护甲≥2 时偏强）；火焰伤害+1 子句因无 |
| `zhengjing2` | content/original/ported/mobile/zhengjing2.skill | （登记债）：①「跳过摸牌阶段」无 draw-skip 原语（skip_next_discard_phase |

## 包 `newjiang`（100 个，不完整 7）

### 近似（7）

| 技能 | 路径 | 省略说明 |
|---|---|---|
| `dcqiaolve` | content/original/ported/newjiang/dcqiaolve.skill | （登记债）：①首现同名牌过滤未建模（全局历史判定无原语）； |
| `dcweijing` | content/original/ported/newjiang/dcweijing.skill | （交牌分支未建模，已注明）。 |
| `duwang` | content/original/ported/newjiang/duwang.skill | （登记债）：①距离修正面保守省略（上游有刺即 +1，DSL 无 min(1,count) |
| `fjzhitu` | content/original/ported/newjiang/fjzhitu.skill | 省略；用牌后摸一张 |
| `mbguli` | content/original/ported/newjiang/mbguli.skill | （已注明）。 |
| `mpsijiu` | content/original/ported/newjiang/mpsijiu.skill | 为本回合他人获得牌后结束阶段摸一张（观看手牌省略，已注明）。 |
| `xvzhi` | content/original/ported/newjiang/xvzhi.skill | （交换未建模，已注明）。 |

## 包 `offline`（819 个，不完整 122）

### 近似（122）

| 技能 | 路径 | 省略说明 |
|---|---|---|
| `cacuixi` | content/original/ported/offline/cacuixi.skill | 占位 |
| `cajingong` | content/original/ported/offline/cajingong.skill | 视为使用【杀】的能力未建模（含子技能/失去体力分支），以主动摸牌 |
| `cajujun` | content/original/ported/offline/cajujun.skill | 晋升） |
| `calianji` | content/original/ported/offline/calianji.skill | 占位 |
| `chunqiubi_skill` | content/original/ported/offline/chunqiubi_skill.skill | 占位 |
| `dragduyi` | content/original/ported/offline/dragduyi.skill | 占位 |
| `dragshiyao` | content/original/ported/offline/dragshiyao.skill | 占位 |
| `dragxueji` | content/original/ported/offline/dragxueji.skill | 晋升）：原「摸 1 占位」与上游无关——晋升为 |
| `hm_cheji` | content/original/ported/offline/hm_cheji.skill | 占位 |
| `hm_guiquan` | content/original/ported/offline/hm_guiquan.skill | 占位 |
| `hm_houying` | content/original/ported/offline/hm_houying.skill | 占位 |
| `hm_jukou` | content/original/ported/offline/hm_jukou.skill | 占位 |
| `hm_qianhu` | content/original/ported/offline/hm_qianhu.skill | 占位 |
| `hm_qianjun` | content/original/ported/offline/hm_qianjun.skill | 重写，active 占位晋升） |
| `hm_qianwei` | content/original/ported/offline/hm_qianwei.skill | 占位 |
| `hm_weichenn` | content/original/ported/offline/hm_weichenn.skill | 占位 |
| `hm_yufeng` | content/original/ported/offline/hm_yufeng.skill | 占位 |
| `hm_zhannei` | content/original/ported/offline/hm_zhannei.skill | 占位 |
| `hsjixu` | content/original/ported/offline/hsjixu.skill | 占位 |
| `hsjunwei` | content/original/ported/offline/hsjunwei.skill | 占位 |
| `hssifen` | content/original/ported/offline/hssifen.skill | 占位 |
| `hsyouchong` | content/original/ported/offline/hsyouchong.skill | 占位 |
| `hszaibi` | content/original/ported/offline/hszaibi.skill | 占位 |
| `huamao_gongxin` | content/original/ported/offline/huamao_gongxin.skill | 占位 |
| `huamao_guose` | content/original/ported/offline/huamao_guose.skill | 占位 |
| `huamao_jiyu` | content/original/ported/offline/huamao_jiyu.skill | 占位 |
| `huamao_lianhuan` | content/original/ported/offline/huamao_lianhuan.skill | 占位 |
| `huamao_liushi` | content/original/ported/offline/huamao_liushi.skill | 占位 |
| `huoxin` | content/original/ported/offline/huoxin.skill | 占位 |
| `jdqicai` | content/original/ported/offline/jdqicai.skill | 占位 |
| `jdsbbeifa` | content/original/ported/offline/jdsbbeifa.skill | 占位 |
| `jdsbjiuyuan` | content/original/ported/offline/jdsbjiuyuan.skill | （登记债）：①吴势力限定未建模——active targets 声明无势力过滤（parse.rs |
| `jdsbtiaoxin` | content/original/ported/offline/jdsbtiaoxin.skill | 占位 |
| `jdsbzhiheng` | content/original/ported/offline/jdsbzhiheng.skill | 弃光手牌额外摸 1 未建模（上游若有则省略）。 |
| `jsprende` | content/original/ported/offline/jsprende.skill | 占位 |
| `jun_hefa` | content/original/ported/offline/jun_hefa.skill | 占位 |
| `jun_henglv` | content/original/ported/offline/jun_henglv.skill | 占位 |
| `jun_renwang` | content/original/ported/offline/jun_renwang.skill | （登记债）：①①式 effect 段（杀次数+N，owner_skill_mark 动态值源需 |
| `jxzhanyan` | content/original/ported/offline/jxzhanyan.skill | 占位 |
| `jxzhaoluan` | content/original/ported/offline/jxzhaoluan.skill | 占位 |
| `jybeirong` | content/original/ported/offline/jybeirong.skill | 占位 |
| `jycailve` | content/original/ported/offline/jycailve.skill | 占位 |
| `jyhuoce` | content/original/ported/offline/jyhuoce.skill | 占位 |
| `nskuishe` | content/original/ported/offline/nskuishe.skill | 占位 |
| `peduzhang` | content/original/ported/offline/peduzhang.skill | 成为牌的目标后摸一张（黑色/唯一/凛与上限未建模）。 |
| `peersheng` | content/original/ported/offline/peersheng.skill | 其他角色获得牌/摸牌阶段你摸 1（一号位与视为用牌未建模）。 |
| `pelinjie` | content/original/ported/offline/pelinjie.skill | 每轮首次准备阶段选人造成 1 点伤害（凛标记与连锁弃牌伤未建模）。 |
| `petijun` | content/original/ported/offline/petijun.skill | 占位 |
| `peyeyan` | content/original/ported/offline/peyeyan.skill | 占位 |
| `pezunwei` | content/original/ported/offline/pezunwei.skill | 占位 |
| `psbicai` | content/original/ported/offline/psbicai.skill | 占位 |
| `psdaohe` | content/original/ported/offline/psdaohe.skill | （登记债）：①上游任意张 [1,∞] allowChooseAll 由其自选——原语为固定 1 张 |
| `psfengxie` | content/original/ported/offline/psfengxie.skill | 占位 |
| `psguixin` | content/original/ported/offline/psguixin.skill | 受伤摸 2；每轮首次准备阶段摸 2（全场各拿一张未建模）。 |
| `pshuxiao` | content/original/ported/offline/pshuxiao.skill | 占位 |
| `psjiefeng` | content/original/ported/offline/psjiefeng.skill | 占位 |
| `pskexue` | content/original/ported/offline/pskexue.skill | 占位 |
| `psliaozou` | content/original/ported/offline/psliaozou.skill | 占位 |
| `psmoyan` | content/original/ported/offline/psmoyan.skill | （登记债）：owner 同时为来源与目标的自伤事件双臂都会命中（转化一次 vs |
| `pssheji` | content/original/ported/offline/pssheji.skill | 占位 |
| `psshenzun` | content/original/ported/offline/psshenzun.skill | 造成伤害时可弃一张 he 并对当前目标再造成 1 点（势力与次数未建模）。 |
| `pstaoluan` | content/original/ported/offline/pstaoluan.skill | 占位 |
| `psxiaolu` | content/original/ported/offline/psxiaolu.skill | 占位 |
| `psyuxue` | content/original/ported/offline/psyuxue.skill | 占位 |
| `pszhanyan` | content/original/ported/offline/pszhanyan.skill | 占位 |
| `pszhiji` | content/original/ported/offline/pszhiji.skill | 占位 |
| `scls_juezhi` | content/original/ported/offline/scls_juezhi.skill | 占位 |
| `scls_miaojian` | content/original/ported/offline/scls_miaojian.skill | 占位 |
| `scls_pingcai` | content/original/ported/offline/scls_pingcai.skill | 占位 |
| `sm_aozhan` | content/original/ported/offline/sm_aozhan.skill | 占位 |
| `sm_jingji` | content/original/ported/offline/sm_jingji.skill | 占位 |
| `sm_lema` | content/original/ported/offline/sm_lema.skill | 占位 |
| `sm_tianwu` | content/original/ported/offline/sm_tianwu.skill | 占位 |
| `smshengshou` | content/original/ported/offline/smshengshou.skill | 占位 |
| `smyanshi` | content/original/ported/offline/smyanshi.skill | 占位 |
| `tychende` | content/original/ported/offline/tychende.skill | 占位 |
| `tydingpan` | content/original/ported/offline/tydingpan.skill | 占位 |
| `tyshencai` | content/original/ported/offline/tyshencai.skill | 占位 |
| `tytanlong` | content/original/ported/offline/tytanlong.skill | 占位 |
| `tytaoyuan` | content/original/ported/offline/tytaoyuan.skill | 占位 |
| `tyxianqi` | content/original/ported/offline/tyxianqi.skill | 占位 |
| `tyyoujun` | content/original/ported/offline/tyyoujun.skill | 占位 |
| `wn_jianshu` | content/original/ported/offline/wn_jianshu.skill | 占位 |
| `xkbimeng` | content/original/ported/offline/xkbimeng.skill | 占位 |
| `xkliaoluan` | content/original/ported/offline/xkliaoluan.skill | 占位 |
| `xy_fenqi` | content/original/ported/offline/xy_fenqi.skill | 占位 |
| `xy_jianzu` | content/original/ported/offline/xy_jianzu.skill | 占位 |
| `yanxiao` | content/original/ported/offline/yanxiao.skill | 占位 |
| `yinling` | content/original/ported/offline/yinling.skill | 占位 |
| `yjjianshu` | content/original/ported/offline/yjjianshu.skill | 占位 |
| `yjjielve` | content/original/ported/offline/yjjielve.skill | 占位 |
| `yjlifeng` | content/original/ported/offline/yjlifeng.skill | 占位 |
| `yjrenwang` | content/original/ported/offline/yjrenwang.skill | 占位 |
| `yjsancai` | content/original/ported/offline/yjsancai.skill | 占位 |
| `yjweiquan` | content/original/ported/offline/yjweiquan.skill | 占位 |
| `yjzezhu` | content/original/ported/offline/yjzezhu.skill | 占位 |
| `ylyg_hunzi` | content/original/ported/offline/ylyg_hunzi.skill | （登记债）：触发面收窄为体力流失（上游 changeHp 含回复至 1 的路径）。 |
| `ylyg_jiang` | content/original/ported/offline/ylyg_jiang.skill | （登记债）：①respond/useCard 应答窗口未移植（card_responded 臂的「交给/令其 |
| `ylygguhuo` | content/original/ported/offline/ylygguhuo.skill | 占位 |
| `ylygniepan` | content/original/ported/offline/ylygniepan.skill | 占位 |
| `ymdanjue` | content/original/ported/offline/ymdanjue.skill | 复用距离 1 杀目标过滤器；目标摸一并强制交一张 he（可选交牌与不计次数未建模）。 |
| `ymfriendyance` | content/original/ported/offline/ymfriendyance.skill | （登记债）：①消耗面（useCard 时移去首张 + 花色/类型判定摸牌）未建模； |
| `ymhengren` | content/original/ported/offline/ymhengren.skill | 其他角色出牌阶段开始（phase_play）与结束均可选择视为使用【杀】； |
| `ymqianling` | content/original/ported/offline/ymqianling.skill | 对一名其他角色摸一张（展示/比色/交换未建模）。 |
| `ymqueying` | content/original/ported/offline/ymqueying.skill | 提供视为杀/闪；【影】与手牌<上限门控未建模。 |
| `ymsaxue` | content/original/ported/offline/ymsaxue.skill | 横置一名其他角色 + 对其造成 1 点火焰伤害；双目标分配 2 点火伤未建模。 |
| `ymxianfeng` | content/original/ported/offline/ymxianfeng.skill | 受伤后弃来源一张 he；「首次花色进弃牌堆则获得」未建模。 |
| `yymujun` | content/original/ported/offline/yymujun.skill | 占位 |
| `yyxiongye` | content/original/ported/offline/yyxiongye.skill | 占位 |
| `yyxuanshi` | content/original/ported/offline/yyxuanshi.skill | 占位 |
| `zc26_dimeng` | content/original/ported/offline/zc26_dimeng.skill | 占位 |
| `zc26_huaxiu` | content/original/ported/offline/zc26_huaxiu.skill | 占位 |
| `zc26_longchen` | content/original/ported/offline/zc26_longchen.skill | 占位 |
| `zc26_weimu` | content/original/ported/offline/zc26_weimu.skill | 占位 |
| `zj_fuli` | content/original/ported/offline/zj_fuli.skill | ①濒死 enable 门以 dying 触发器自律表达；②上游 limited/awaken 以 storage |
| `zj_huairou` | content/original/ported/offline/zj_huairou.skill | 占位 |
| `zj_liefa` | content/original/ported/offline/zj_liefa.skill | 占位 |
| `zj_qiangyi` | content/original/ported/offline/zj_qiangyi.skill | 占位 |
| `zj_tuicheng` | content/original/ported/offline/zj_tuicheng.skill | 占位 |
| `zj_tuizhi` | content/original/ported/offline/zj_tuizhi.skill | 占位 |
| `zombieshibao` | content/original/ported/offline/zombieshibao.skill | 占位 |
| `zylianji` | content/original/ported/offline/zylianji.skill | ①三段递进合并为单次触发（摸1+跳剩余+仅Play额外回合，原「受伤回复」 |

## 包 `old`（26 个，不完整 3）

### 近似（3）

| 技能 | 路径 | 省略说明 |
|---|---|---|
| `chouyuan` | content/original/ported/old/chouyuan.skill | 对目标弃一张 he；X 与弃牌堆获杀未建模。 |
| `ciju` | content/original/ported/old/ciju.skill | 给自己挂载本回合 hand_limit=0 + 用牌无限（目标应为当前回合角色；牌名切换解除未建模）。 |
| `jiechou` | content/original/ported/old/jiechou.skill | 每回合限一次可选择造成 1 点伤害（失去最后伤害牌触发未建模）。 |

## 包 `onlyOL`（182 个，不完整 8）

### 近似（8）

| 技能 | 路径 | 省略说明 |
|---|---|---|
| `dclihuo` | content/original/ported/onlyOL/dclihuo.skill | ）+ |
| `olchaozheng` | content/original/ported/onlyOL/olchaozheng.skill | （未建模尾段）：①按意见组人数摸至多 2 张（callback 追加）； |
| `olenyuan2` | content/original/ported/onlyOL/olenyuan2.skill | 不限定红色/仅手牌；「或失去 1 点体力」分支未建模（无牌时静默跳过）。 |
| `olfuzai` | content/original/ported/onlyOL/olfuzai.skill | 无装备时的虚拟坐骑子句未建模；修正此前误标（noop 注记称 audio-only， |
| `olpaiyi` | content/original/ported/onlyOL/olpaiyi.skill | （登记债）：①「交给」收窄为「弃置」；②摸牌数 min(7, 剩余堆数)（批 231 清偿上限 7 债）、 |
| `olquanji` | content/original/ported/onlyOL/olquanji.skill | （登记债）：①上游 target=当前回合角色；原语目标为任意其他有手牌角色 |
| `olrenxin` | content/original/ported/onlyOL/olrenxin.skill | 重写，批 159 复核证伪池晋升） |
| `tianxing` | content/original/ported/onlyOL/tianxing.skill | （登记债）：①「获得蛊囤所有牌」（pile→手牌整取）未建模——S1-B 原语批次； |

## 包 `refresh`（297 个，不完整 54）

### 近似（54）

| 技能 | 路径 | 省略说明 |
|---|---|---|
| `chulao` | content/original/ported/refresh/chulao.skill | （登记债）：①目标选择器未提供「势力各异」过滤面（机器入口校验， |
| `dcanxu` | content/original/ported/refresh/dcanxu.skill | （登记债）：①「手牌数不同」约束在机器入口校验（激活即拒），目标 |
| `decadechunlao2` | content/original/ported/refresh/decadechunlao2.skill | 上游受伤侧亦可触发（本实现仅来源侧）；amount 守卫在 body（不满足仍弹确认）。 |
| `decadelihuo` | content/original/ported/refresh/decadelihuo.skill | modifier 为全程生效（上游为每次使用可选），所有【杀】视为火。 |
| `decadelihuo3` | content/original/ported/refresh/decadelihuo3.skill | 「指定多名目标」门槛未建模（单目标火杀伤害亦触发）；伤害属性由过滤器保证。 |
| `decadezishou_zhiheng` | content/original/ported/refresh/decadezishou_zhiheng.skill | 不检验「本回合未指定他人」；不要求弃牌花色互异（重铸任意已选手牌）。 |
| `gzquanji` | content/original/ported/refresh/gzquanji.skill | （登记债）：①上游每分支各限一次（triggername 去重）收窄为 skill 级共享 |
| `olduanliang` | content/original/ported/refresh/olduanliang.skill | （登记债）：「本回合未造成伤害时无距离限制」的条件距离 mod 未建模 |
| `olfangquan` | content/original/ported/refresh/olfangquan.skill | 旧JS事件 phaseUseBefore → DSL事件 phaseUseBefore |
| `olfangquan2` | content/original/ported/refresh/olfangquan2.skill | 旧JS事件 phaseDiscardBegin → DSL事件 phase_discard |
| `oljiuchi` | content/original/ported/refresh/oljiuchi.skill | 旧JS事件 damageEnd → DSL事件 damage |
| `ollihuo2` | content/original/ported/refresh/ollihuo2.skill | 旧JS事件 useCard2 → DSL事件 useCard2 |
| `ollihuo3` | content/original/ported/refresh/ollihuo3.skill | 旧JS事件 useCardEnd → DSL事件 useCardEnd |
| `ollihuo4` | content/original/ported/refresh/ollihuo4.skill | 置入手牌一张而非结算后的那张杀实体（原语无从结算区直置，与旧 ollihuo 注记 |
| `olluanji_remove` | content/original/ported/refresh/olluanji_remove.skill | 旧JS事件 useCard2 → DSL事件 useCard2 |
| `olruoyu` | content/original/ported/refresh/olruoyu.skill | 旧JS事件 phaseZhunbeiBegin → DSL事件 phase_prepare |
| `olzhiji` | content/original/ported/refresh/olzhiji.skill | （登记债）：①效果顺序对调——先减上限后摸/回（上游先摸/回再减上限； |
| `qianxin` | content/original/ported/refresh/qianxin.skill | 旧JS事件 damageSource → DSL事件 damage |
| `qingjian` | content/original/ported/refresh/qingjian.skill | 旧JS事件 gainAfter → DSL事件 gainAfter |
| `rechanyuan` | content/original/ported/refresh/rechanyuan.skill | 旧JS事件 changeHp → DSL事件 changeHp |
| `refangquan` | content/original/ported/refresh/refangquan.skill | 旧JS事件 phaseUseBefore → DSL事件 phaseUseBefore |
| `reguanxing` | content/original/ported/refresh/reguanxing.skill | （登记债）：①逐张二选（顶/底）近似上游拖拽任意排列——堆内自由序未建模 |
| `rehongyan` | content/original/ported/refresh/rehongyan.skill | 旧JS事件 loseEnd → DSL事件 loseEnd |
| `rejigong2` | content/original/ported/refresh/rejigong2.skill | 旧JS事件 phaseUseEnd → DSL事件 phase_play_end |
| `rejinjiu2` | content/original/ported/refresh/rejinjiu2.skill | 旧JS事件 damageBegin3 → DSL事件 damageBegin3 |
| `rejiqiao` | content/original/ported/refresh/rejiqiao.skill | 旧JS事件 phaseUseBegin → DSL事件 phaseUseBegin |
| `rejiuyuan` | content/original/ported/refresh/rejiuyuan.skill | 旧JS事件 recoverBefore → DSL事件 recoverBefore |
| `rejuece` | content/original/ported/refresh/rejuece.skill | 旧JS事件 phaseJieshuBegin → DSL事件 phase_end |
| `rejunbing` | content/original/ported/refresh/rejunbing.skill | 旧JS事件 phaseJieshuBegin → DSL事件 phase_end |
| `reluoyi` | content/original/ported/refresh/reluoyi.skill | 旧JS事件 phaseDrawBegin1 → DSL事件 phaseDrawBegin1 |
| `repingkou` | content/original/ported/refresh/repingkou.skill | 旧JS事件 phaseEnd → DSL事件 phaseEnd |
| `reqiaoshi` | content/original/ported/refresh/reqiaoshi.skill | 旧JS事件 phaseJieshuBegin → DSL事件 phase_end |
| `reqieting` | content/original/ported/refresh/reqieting.skill | 旧JS事件 phaseEnd → DSL事件 phaseEnd |
| `reshizhi` | content/original/ported/refresh/reshizhi.skill | 旧JS事件 damageEnd → DSL事件 damage |
| `reshuangxiong1` | content/original/ported/refresh/reshuangxiong1.skill | 旧JS事件 phaseDrawBegin1 → DSL事件 phaseDrawBegin1 |
| `reshuangxiong2` | content/original/ported/refresh/reshuangxiong2.skill | 旧JS事件 damageEnd → DSL事件 damage |
| `residi3` | content/original/ported/refresh/residi3.skill | 旧JS事件 phaseUseEnd → DSL事件 phase_play_end |
| `retishen` | content/original/ported/refresh/retishen.skill | 旧JS事件 phaseZhunbeiBegin → DSL事件 phase_prepare |
| `retishen2` | content/original/ported/refresh/retishen2.skill | 旧JS事件 phaseJieshuBegin → DSL事件 phase_end |
| `retuxi` | content/original/ported/refresh/retuxi.skill | 旧JS事件 phaseDrawBegin2 → DSL事件 phaseDrawBegin2 |
| `reyanyu2` | content/original/ported/refresh/reyanyu2.skill | 旧JS事件 phaseUseEnd → DSL事件 phase_play_end |
| `reyiji2` | content/original/ported/refresh/reyiji2.skill | 旧JS事件 phaseDrawBegin → DSL事件 phase_draw |
| `reyingshi` | content/original/ported/refresh/reyingshi.skill | 旧JS事件 phaseUseBegin → DSL事件 phaseUseBegin |
| `rezhanjue_effect` | content/original/ported/refresh/rezhanjue_effect.skill | 旧JS事件 useCardAfter → DSL事件 card_used |
| `shebian` | content/original/ported/refresh/shebian.skill | （登记债）：上游 moveCard 为装备牌口径的任意移动；原语可选集含判定区 |
| `wulie` | content/original/ported/refresh/wulie.skill | 旧JS事件 phaseJieshuBegin → DSL事件 phase_end |
| `wulie2` | content/original/ported/refresh/wulie2.skill | 旧JS事件 damageBegin3 → DSL事件 damageBegin3 |
| `xinbuyi` | content/original/ported/refresh/xinbuyi.skill | 旧JS事件 dying → DSL事件 dying |
| `xinjiefan2` | content/original/ported/refresh/xinjiefan2.skill | 旧JS事件 phaseEnd → DSL事件 phaseEnd |
| `xinleiji_misa` | content/original/ported/refresh/xinleiji_misa.skill | 旧JS事件 judgeEnd → DSL事件 judgeEnd |
| `xinzhiyan` | content/original/ported/refresh/xinzhiyan.skill | 旧JS事件 phaseJieshuBegin → DSL事件 phase_end |
| `xinzili` | content/original/ported/refresh/xinzili.skill | （登记债）：①觉醒动画未建模；②“权”堆供给依赖 xinquanji 置入面（S1-B） |
| `xsqianxin` | content/original/ported/refresh/xsqianxin.skill | 旧JS事件 damageSource → DSL事件 damage |
| `zhuhai` | content/original/ported/refresh/zhuhai.skill | 旧JS事件 phaseJieshuBegin → DSL事件 phase_end |

## 包 `sb`（4 个，不完整 1）

### 近似（1）

| 技能 | 路径 | 省略说明 |
|---|---|---|
| `sbdengxian` | content/original/ported/sb/sb_meta.skill | 。 |

## 包 `shahai`（2 个，不完整 2）

### 近似（2）

| 技能 | 路径 | 省略说明 |
|---|---|---|
| `zuiji` | content/original/ported/shahai/shahai_impl.skill | 已逐条注明；其余技能 |
| `qingshu` | content/original/ported/shahai/shahai_meta.skill | （无运行时行为）。本文件由上游 skill.js/translate.js 自动生成， |

## 包 `shenhua`（160 个，不完整 1）

### 近似（1）

| 技能 | 路径 | 省略说明 |
|---|---|---|
| `fangquan2` | content/original/ported/shenhua/fangquan2.skill | ①弃一张手牌走引擎既有选择屏障（与上游 chooseToDiscard 等价）； |

## 包 `shiji`（164 个，不完整 11）

### 近似（11）

| 技能 | 路径 | 省略说明 |
|---|---|---|
| `chuhai2` | content/original/ported/shiji/chuhai2.skill | 以造成伤害后摸一张代替（拼点关联角色存储、空装备栏条件、 |
| `fubi` | content/original/ported/shiji/fubi.skill | （fubi2 加成为无条件，已注明）。 |
| `jianzhan` | content/original/ported/shiji/jianzhan.skill | （已注明）。 |
| `lingce` | content/original/ported/shiji/lingce.skill | （牌名限定未建模，已注明）。 |
| `luanchou` | content/original/ported/shiji/luanchou.skill | （标记与授技未建模，已注明）。 |
| `mbyongjue` | content/original/ported/shiji/mbyongjue.skill | （要求自己非目标），“自己是被杀目标”的边缘场景不触发。 |
| `mjchenshi_target` | content/original/ported/shiji/mjchenshi_target.skill | （牌名限定未建模，已注明）。 |
| `refubi` | content/original/ported/shiji/refubi.skill | 增益偏离大——原语把开局效果表达为交予一张手牌/装备牌（无标记载体）， |
| `scfuhai` | content/original/ported/shiji/scfuhai.skill | 已随原语就位晋升）+ 摸 1。 |
| `tspowei` | content/original/ported/shiji/tspowei.skill | 职责技状态机、标记散布/移除、二选一分支与神助授予均未建模；以造成 |
| `xinmouli` | content/original/ported/shiji/xinmouli.skill | （备牌指定未建模，已注明）。 |

## 包 `sixiang`（174 个，不完整 14）

### 近似（14）

| 技能 | 路径 | 省略说明 |
|---|---|---|
| `stdaocai` | content/original/ported/sixiang/stdaocai.skill | 用 draw_cards(self, 1) 代替「观看两张选一张」——收益接近（均得1牌），但少了选择权与牌堆顶控制。 |
| `stdchenghao` | content/original/ported/sixiang/stdchenghao.skill | 用 draw_cards(self, 1) 代替「观看牌堆顶并交给任意角色」——收益接近（均从牌堆获得1牌）， |
| `stdchengji` | content/original/ported/sixiang/stdchengji.skill | view_as sha from 2 张手牌/装备区牌——丢失了「两张颜色不同」的 |
| `stdchuyi` | content/original/ported/sixiang/stdchuyi.skill | 用 modify_current_damage(self, 1) 在 damage_before 非目标时增加伤害。 |
| `stdduwu` | content/original/ported/sixiang/stdduwu.skill | active 弃手牌→造成1点伤害。丢失：①攻击范围内的目标约束（active targets 不可按距离过滤） |
| `stdguijie` | content/original/ported/sixiang/stdguijie.skill | view_as shan from 2 张手牌/装备区牌。丢失 |
| `stdnaman` | content/original/ported/sixiang/stdnaman.skill | view_as 将手牌+装备区基本牌转南蛮；旧 JS 的 complexSelect（选牌数=目标数）未保留， |
| `stdpolu` | content/original/ported/sixiang/stdpolu.skill | choose_other_discard_target_card 用 hand_equip（含手牌）而非仅 equip 区， |
| `stdruixi` | content/original/ported/sixiang/stdruixi.skill | phase_any_end + choose_use_virtual_sha(ignore_distance) 保留无视距离出杀； |
| `stdyibing` | content/original/ported/sixiang/stdyibing.skill | 用 take_one_event_target_hand 取濒死角色一张手牌，无法区分手牌区/装备区自选。 |
| `stdyoudi` | content/original/ported/sixiang/stdyoudi.skill | view_as 红色牌当顺手牵羊。旧 JS 限定「结束阶段」的触发窗口 DSL 中无法与 view_as response 窗口结合表达。 |
| `stdzhanjue` | content/original/ported/sixiang/stdzhanjue.skill | view_as 决斗（所有手牌）。「使用后摸一张牌」的触发（useCardAfter + skill == stdzhanjue）需在 trigger 中匹配「此技能创建的决斗使用后」事件，但 DSL 无法按 skill 名过滤触发。 |
| `stdzhengnan` | content/original/ported/sixiang/stdzhengnan.skill | view_as 红色牌当杀。旧 JS 的「准备阶段触发」「杀造成死亡后摸牌」的窗口与条件 DSL 无法精确表达。 |
| `stdzongxuan` | content/original/ported/sixiang/stdzongxuan.skill | 用 choose_other_discard_target_card 弃置目标的装备/判定区牌。 |

## 包 `sp`（694 个，不完整 39）

### 近似（39）

| 技能 | 路径 | 省略说明 |
|---|---|---|
| `benyu` | content/original/ported/sp/benyu.skill | ①上游可选 cancel/单条件直走，DSL 为 optional 确认 + 强制二选一（条件不满足 |
| `chixueqingfeng` | content/original/ported/sp/chixueqingfeng.skill | markAuto（记录特定杀）需 expansion 原语，本定义实现 addTempSkill 核心行为。 |
| `danlao` | content/original/ported/sp/danlao.skill | cancel_incoming 与杜达元帅（ADR 0608）共用原语——额外获得被免除的 |
| `fanxiang` | content/original/ported/sp/fanxiang.skill | (1) 不检验良助 storage / 已受伤角色；(2) 觉醒以 mark_count 门控， |
| `juesheng` | content/original/ported/sp/juesheng.skill | (1) 消耗一张手牌，而非零成本虚拟决斗；(2) 每回合一次近似限定技； |
| `kuanshi` | content/original/ported/sp/kuanshi.skill | （登记债）：①授予者跳过摸牌阶段未建模——引擎无 per-player 未来回合 |
| `leiluan` | content/original/ported/sp/leiluan.skill | 不按轮次剔除已用基本牌名；张数固定 1 而非横置数；无酒；无后续效果。 |
| `mingzhe` | content/original/ported/sp/mingzhe.skill | 同 olmingzhe（ADR 0637）：放宽「红色牌筛选」为「任意牌」（副作用良性）。 |
| `olcuipo` | content/original/ported/sp/olcuipo.skill | 「baseDamage++」（伤害修正）需新 modifier 原语，本定义实现 filter + 非伤害牌的摸牌分支； |
| `olfeibai` | content/original/ported/sp/olfeibai.skill | 不限非黑色；回复+1 与阴阳切换未建模。 |
| `olfeibian` | content/original/ported/sp/olfeibian.skill | 不覆盖「成为目标」窗口；弃牌由自己选择而非随机；出牌时限未建模。 |
| `olfengyan` | content/original/ported/sp/olfengyan.skill | 用牌/打出窗口与二选一交弃未建模。 |
| `olfudao` | content/original/ported/sp/olfudao.skill | 交牌、弃牌记账与回合结束对拍摸牌未建模。 |
| `olhuiyun` | content/original/ported/sp/olhuiyun.skill | 结算后三项选择与每轮限次未建模。 |
| `oljiyun` | content/original/ported/sp/oljiyun.skill | 粮区/使用后收回未建模；摸牌目标上限固定 3。 |
| `olmingzhe` | content/original/ported/sp/olmingzhe.skill | isPhaseUsing = in_play_phase，对当前回合事件等价）+ |
| `olpingyuan` | content/original/ported/sp/olpingyuan.skill | 霹雳车装备/移动未建模；视为杀不检验已装备霹雳车。 |
| `olpingzhong` | content/original/ported/sp/olpingzhong.skill | 空手数+1 改为固定摸 1；额外指定目标未建模。 |
| `olqisi` | content/original/ported/sp/olqisi.skill | 不保证装备/副类别；少摸换装备未建模。 |
| `olsiqi` | content/original/ported/sp/olsiqi.skill | 不检验牌堆底颜色；亮出并使用红牌改为摸 1。 |
| `oltaohuai` | content/original/ported/sp/oltaohuai.skill | 使用牌后摸一张（点数极值判定、转换技阴阳与可选弃牌未建模）。 |
| `olxianying` | content/original/ported/sp/olxianying.skill | 弃牌张数去重、同名回合末视为使用未建模。 |
| `olxinggu` | content/original/ported/sp/olxinggu.skill | 坐骑区与结束阶段交易未建模。 |
| `olzhubi` | content/original/ported/sp/olzhubi.skill | 重铸改为摸 1；次数不按体力上限。 |
| `olzonglve` | content/original/ported/sp/olzonglve.skill | 获牌不限「此杀/非实体杀」、也不分区域各一张。 |
| `pxlangya` | content/original/ported/sp/pxlangya.skill | （登记债）：逐张二选（顶/底）近似上游拖拽任意排列——堆内自由序未建模 |
| `pxqixian` | content/original/ported/sp/pxqixian.skill | （登记债）：①sha/guohe 使用合法性按「其他存活角色」宽口径（上游走 |
| `qinmian` | content/original/ported/sp/qinmian.skill | （登记债）：①frequent 锁定为强制自动（Prepare/damage 直摸）； |
| `refuhan` | content/original/ported/sp/refuhan.skill | 限定一次）；fanghun 标记由方魂技能维护（def 3479603365）。 |
| `resilv` | content/original/ported/sp/resilv.skill | gaintag 标记与 _restore 弃置回血子技能需 expansion 支持，本定义实现主技能的摸牌 |
| `songci` | content/original/ported/sp/songci.skill | 始终令目标摸 2，不检验 hand>hp 弃牌分支；每名角色一次与弃牌阶段摸牌未建模。 |
| `spolxudai` | content/original/ported/sp/spolxudai.skill | ①上游 useCard 且 respondTo 的角落窗口未单列（card_responded 已覆盖响应面）； |
| `tuifeng2` | content/original/ported/sp/tuifeng2.skill | （登记债）：①addTempSkill（回合级临时）以永久 add_skill 近似；②动画 |
| `wangzun` | content/original/ported/sp/wangzun.skill | markSkill/storage.wangzun（一次性标记）+ 触发者手牌上限-1 需 mark modifier 原语， |
| `xinfu_dianhua` | content/original/ported/sp/xinfu_dianhua.skill | （登记债）：逐张二选顶/底近似拖拽自由排列（批 176 同款）；无标记时 N=0 |
| `xinfu_falu` | content/original/ported/sp/xinfu_falu.skill | （登记债）：enterGame 补齐分支并入 game_started；loseAsyncAfter 多区域 |
| `xinfu_yinshi` | content/original/ported/sp/xinfu_yinshi.skill | 不检验空防具栏、荐杰标记、属性/锦囊类型，防止受到的一切伤害。 |
| `xinfu_zhennan` | content/original/ported/sp/xinfu_zhennan.skill | 随机 1-3 点伤害定为固定 1 点（引擎无随机数原语）。 |
| `yuanchou` | content/original/ported/sp/yuanchou.skill | 不限黑色【杀】、也不区分「你使用 / 你成为目标」两向， |

## 包 `sp2`（271 个，不完整 24）

### 近似（24）

| 技能 | 路径 | 省略说明 |
|---|---|---|
| `baijia` | content/original/ported/sp2/baijia.skill | 仅保留 recover_hp + add_mark 收益，觉醒与技能替换未建模。 |
| `biaozhao` | content/original/ported/sp2/biaozhao.skill | （登记债）：①biaozhao2/3 消耗面（同点数入弃失体力/准备阶段移去+摸至全场 |
| `biaozhao2` | content/original/ported/sp2/biaozhao2.skill | （登记债）：①上游区分「一般弃置→弃牌者获得」与「其他路径→表入弃牌 |
| `biaozhao3` | content/original/ported/sp2/biaozhao3.skill | （登记债）：①仅弃置/结算首张「表」（上游 expansion 单张门控同口径 ✓）； |
| `chaofeng` | content/original/ported/sp2/chaofeng.skill | 按弃牌摸牌基础收益建模，花色/类型条件加成未建模（需 expansion 或运行时花色检查）。 |
| `dcbihuo` | content/original/ported/sp2/dcbihuo.skill | 本回合内对玩家增加当前回合摸牌数（但语义错位 — 旧 JS 是下回合）。 |
| `dcbingji` | content/original/ported/sp2/dcbingji.skill | （登记债）：①展示所有手牌表现面未建模；②sha 目标合法性按「其他存活角色」 |
| `dccanyun` | content/original/ported/sp2/dccanyun.skill | （登记债）：①「本回合未选过的角色」跨激活去重未建模（单次激活内 |
| `dcfuning` | content/original/ported/sp2/dcfuning.skill | 保留 draw_cards(2) 基础收益，递增弃牌与 history 追踪无法表达。 |
| `dchumei` | content/original/ported/sp2/dchumei.skill | （登记债）：①「每项限一次」收窄为每次发动限选一项（跨发动去重需 |
| `dcjinjian` | content/original/ported/sp2/dcjinjian.skill | 保留 add_mark 标记累积和受伤/造成伤害触发。 |
| `dcmoukui` | content/original/ported/sp2/dcmoukui.skill | （登记债）：①未命中追罚面（both+未命中→目标弃 owner 一张）留 |
| `dcniji` | content/original/ported/sp2/dcniji.skill | （登记债）：①「可先使用其中一张」未建模（结束阶段全部直接弃置）。 |
| `dcxiuwen` | content/original/ported/sp2/dcxiuwen.skill | 保留摸1牌核心收益。本局首次使用去重条件(牌名storage追踪)无原语，未建模。 |
| `decadexushen` | content/original/ported/sp2/decadexushen.skill | awakenSkill/addSkills 需限定技机制，本定义实现濒死回复 + addTempSkill 核心行为。 |
| `fuyuan` | content/original/ported/sp2/fuyuan.skill | 上游 respond 含打出（如被借刀回应）与响应两类窗口，DSL card_responded |
| `panshi_damage` | content/original/ported/sp2/panshi_damage.skill | 伤害+1效果，跳过出牌阶段无原语未建模。 |
| `qibie` | content/original/ported/sp2/qibie.skill | 保留弃所有手牌+回血+摸牌核心收益（摸牌量固定为2，旧JS为弃牌数+2）。 |
| `spshicai` | content/original/ported/sp2/spshicai.skill | （登记债）：①子面「可令一名角色使用获得的牌，摸一张牌」（spshicai |
| `sptunjiang` | content/original/ported/sp2/sptunjiang.skill | 保留摸 alive_group_count() 张牌核心收益；「出牌阶段未指定其他角色」条件无对应 filter。 |
| `starzhongyan` | content/original/ported/sp2/starzhongyan.skill | （登记债）：①交换次序为机内固定（亮牌原位换入手牌牌），上游 |
| `xinfu_yisuan` | content/original/ported/sp2/xinfu_yisuan.skill | ①上游 filterInD 取「结算区+弃牌堆」中属于此牌的实体牌，引擎 |
| `yinyi` | content/original/ported/sp2/yinyi.skill | 上游 hasNature("linked") 排除（铁索连环传导伤害不触发）未建模——引擎 |
| `zhangu` | content/original/ported/sp2/zhangu.skill | hand_empty 触发保留摸牌+扣血收益，无法保证"不同类型"与"手牌或装备均为空"的精确判定。 |

## 包 `sxrm`（9 个，不完整 1）

### 近似（1）

| 技能 | 路径 | 省略说明 |
|---|---|---|
| `sxrmjiehuo` | content/original/ported/sxrm/sxrm_meta.skill | （无运行时行为）。详见包 README 的可表达性清单。 |

## 包 `tw`（370 个，不完整 22）

### 近似（22）

| 技能 | 路径 | 省略说明 |
|---|---|---|
| `chijie` | content/original/ported/tw/chijie.skill | 为 |
| `twchungang` | content/original/ported/tw/twchungang.skill | （登记债）：①「使用以此法摸到的牌时需弃置一张」附加面未建模； |
| `twdaigui` | content/original/ported/tw/twdaigui.skill | （登记债）：①选获为机内 SelectCards 单选（上游 chooseButton 拖拽 UI） |
| `twdangjiang` | content/original/ported/tw/twdangjiang.skill | ）。 |
| `twjianyan` | content/original/ported/tw/twjianyan.skill | 有角色濒死时你回复 1 点体力；虎踞分发与入幻未建模。 |
| `twjianying` | content/original/ported/tw/twjianying.skill | 跨回合比较上一张使用牌的花色/点数；动态 view_as 与不计次数未建模。 |
| `twjieqiu` | content/original/ported/tw/twjieqiu.skill | ①恢复节奏=目标每弃一张即时恢复一槽（引擎 CardsMoved-Discard 钩子， |
| `twjunsi` | content/original/ported/tw/twjunsi.skill | 为：每次造成伤害摸1，每次受到伤害弃1（不限次）。 |
| `twlihuo2` | content/original/ported/tw/twlihuo2.skill | 上游 useCard 阶段增目标；DSL 在指定后窗口调整（须处于真实杀结算）。 |
| `twliwu` | content/original/ported/tw/twliwu.skill | 自己造成或受到伤害后摸一张；基本牌判定与轮次伤害加成未建模。 |
| `twmingren` | content/original/ported/tw/twmingren.skill | （登记债）：①置换面（出牌阶段开始/结束时用手牌/装备替换「任」）未建模； |
| `twmouzhi` | content/original/ported/tw/twmouzhi.skill | （登记债）：①存储为机内 content_storage 而非上游 history 末条推导 |
| `twsaoting` | content/original/ported/tw/twsaoting.skill | 同时提供伤害牌转决斗与转酒（转换技阴阳切换未建模）；受伤目标摸牌未建模。 |
| `twsbmingren` | content/original/ported/tw/twsbmingren.skill | （「任」expansion 无对应原语，以回合开始摸一张 |
| `twshiji` | content/original/ported/tw/twshiji.skill | 不保证花色不同；弃牌记花色/展示摸牌/火攻弃牌未建模。 |
| `twtongce` | content/original/ported/tw/twtongce.skill | ①交牌臂（其交给你一张牌）无「选定角色交牌」通用原语，省略； |
| `twwushen` | content/original/ported/tw/twwushen.skill | 不限手牌区（全区域红桃均改写）；梦魇额外目标与「本回合首次使用」门控未单独建模 |
| `twxingwu` | content/original/ported/tw/twxingwu.skill | （登记债）：①消耗面（≥3 张移去三张+弃装备区全部+性别分支伤害）未建模 |
| `twxushen` | content/original/ported/tw/twxushen.skill | 实现：每轮开始可选「摸 2 失 2」（choose 仅二支，1-4 点自选未建模；限定觉醒 |
| `twyiju` | content/original/ported/tw/twyiju.skill | （登记债）：①attackRangeBase=体力值与 cardUsable 两个 mod 未建模 |
| `twyouye` | content/original/ported/tw/twyouye.skill | （登记债）：①分配面（受伤后将所有攸业牌分配给其他角色、当前回合角色至少 |
| `twzhenliang` | content/original/ported/tw/twzhenliang.skill | 转换技阴阳与「任」颜色摸牌未建模；主动不检验攻击范围、不消耗弃牌； |

## 包 `xianding`（511 个，不完整 18）

### 近似（18）

| 技能 | 路径 | 省略说明 |
|---|---|---|
| `beishui` | content/original/ported/xianding/beishui.skill | 用 phase_prepare + owner_alive_not_target_once 限制单次触发，以 change_max_hp + add_skill 模拟觉醒效果。 |
| `choujue` | content/original/ported/xianding/choujue.skill | 用 phase_any_end + owner_alive_not_target_once 限制单次触发。 |
| `dcchengchong` | content/original/ported/xianding/dcchengchong.skill | 上游递归守卫）+ |
| `dcmingjie` | content/original/ported/xianding/dcmingjie.skill | 开局摸 1 并令一名其他角色摸 1；有角色死亡时你失去 1 点体力上限。 |
| `dcpijian` | content/original/ported/xianding/dcpijian.skill | （登记债）：①目标「一名角色」（含自己）收窄为其他角色（choose_other |
| `dcsbjianwei` | content/original/ported/xianding/dcsbjianwei.skill | （登记债）：①起始手牌数+4 面（gameDrawBegin 数值改写）未建模—— |
| `dcsbjinhuo` | content/original/ported/xianding/dcsbjinhuo.skill | 出牌阶段结束时若手牌少于上限则摸至上限；失效非锁定技未建模。 |
| `dcsbxieshu` | content/original/ported/xianding/dcsbxieshu.skill | 本回合首个非装备牌目标处获其一张 he；令目标无效未建模。 |
| `dcsbzongzi` | content/original/ported/xianding/dcsbzongzi.skill | 仅以 lifetime_damage_dealt≥3 判定（不含受伤累计）；固定走「换矜祸+再加上限」分支； |
| `dcxianfu` | content/original/ported/xianding/dcxianfu.skill | 成为其他角色牌的目标时获来源一张 he（跳过阶段、无效与互看未建模）。 |
| `dcxingwu` | content/original/ported/xianding/dcxingwu.skill | 「满三张弃装备+伤害」后半句未建模（需按堆花色集合查询+多目标弃装原语）。 |
| `guili` | content/original/ported/xianding/guili.skill | ②「额外回合」每场仅一次以 storage key2 锁定（上游按绑定唯一性天然 |
| `mansi` | content/original/ported/xianding/mansi.skill | 重写，2026-08-11 旧头注「无精确 nanman-only |
| `mengqing` | content/original/ported/xianding/mengqing.skill | phase_prepare + owner_alive_not_target_once 限单次，change_max_hp + recover_hp + remove_skill + add_skill 模拟觉醒换技能。 |
| `qianlong` | content/original/ported/xianding/qianlong.skill | （登记债）：①frequent 自动口径锁定为 optional 确认；②chooseToMove |
| `tuiyan` | content/original/ported/xianding/tuiyan.skill | （登记债）：上游为纯观看+逆序放回（无选择收益）；guanxing_top 允许逐张 |
| `zhafu` | content/original/ported/xianding/zhafu.skill | （登记债）：①负担面（标记角色弃牌阶段保留一张、其余交给授予者）未建模 |
| `zhushi` | content/original/ported/xianding/zhushi.skill | （登记债）：①「其可令你摸牌」的确认方为回复的魏角色，DSL optional 确认固定 |

## 包 `yijiang`（278 个，不完整 20）

### 近似（20）

| 技能 | 路径 | 省略说明 |
|---|---|---|
| `jieyue` | content/original/ported/yijiang/jieyue.skill | 保留。 |
| `oldzhenlie` | content/original/ported/yijiang/oldzhenlie.skill | （牌堆顶来源与弃原判牌未建模，已注明）。 |
| `qianxi2` | content/original/ported/yijiang/qianxi2.skill | 实际效果由外部skill qianxi在回合结束后removeSkill，花色由storage.qianxi2决定。 |
| `qinwang` | content/original/ported/yijiang/qinwang.skill | 用view_as表达转化杀，后续蜀势力代打逻辑因无轮询机制无法建模。 |
| `qiuyuan` | content/original/ported/yijiang/qiuyuan.skill | 用choose_other_give_one_hand_equip近似交闪，额外目标逻辑无对应原语。 |
| `reanjian` | content/original/ported/yijiang/reanjian.skill | 用damage_before + modify_current_damage近似+1，条件过滤用owner_is_target_per_damage_point。 |
| `reduodao` | content/original/ported/yijiang/reduodao.skill | （武器限定未建模，已注明）。 |
| `reqingxi` | content/original/ported/yijiang/reqingxi.skill | +1伤害和判定不可响应。 |
| `reshenduan` | content/original/ported/yijiang/reshenduan.skill | 用discarded条件触发，选择目标使用兵粮寸断（无法精确限定只能从弃牌堆选取特定牌）。 |
| `rexingxue` | content/original/ported/yijiang/rexingxue.skill | 用phase_end触发，以draw_selected模拟摸牌，手牌>体力的top-deck逻辑无法分段处理。 |
| `reyanzhu` | content/original/ported/yijiang/reyanzhu.skill | 目标给你一张装备，或受到1点伤害（无法建模二选一和技能修改） |
| `reyanzhu2` | content/original/ported/yijiang/reyanzhu2.skill | damage_before时increase damage（无法追踪mark计数）。 |
| `reyonglve` | content/original/ported/yijiang/reyonglve.skill | phase_judge触发，弃判定牌后视条件使用杀或摸牌。 |
| `shibei_old` | content/original/ported/yijiang/shibei_old.skill | 统一用judge处理，无法根据shibei2标记切换判定方向。 |
| `xiansi` | content/original/ported/yijiang/xiansi.skill | ，但原版是从目标处移牌；DSL 无「选择他人牌移入自己技能牌堆」原语。 |
| `xinenyuan2` | content/original/ported/yijiang/xinenyuan2.skill | 不限定红色/仅手牌（上游为红色手牌）；「或失去 1 点体力」分支未建模 |
| `xinfencheng` | content/original/ported/yijiang/xinfencheng.skill | ，但丢掉了递增计数和序列化目标处理。 |
| `xinjunxing` | content/original/ported/yijiang/xinjunxing.skill | （类别匹配与翻面未建模，已注明）。 |
| `xinwuyan` | content/original/ported/yijiang/xinwuyan.skill | 。 |
| `xinzhaofu` | content/original/ported/yijiang/xinzhaofu.skill | 全局距离修改。 |

## 包 `yxs`（2 个，不完整 1）

### 近似（1）

| 技能 | 路径 | 省略说明 |
|---|---|---|
| `yxs_fanji` | content/original/ported/yxs/yxs_impl.skill | 1：旧 JS 限定伤害来源为【杀】/【决斗】且你手牌有【杀】，DSL 以任意伤害触发 |
