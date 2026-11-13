"""Build the independently written per-sample review; assert exact selection coverage."""
import json
import hashlib
import math
import random
import re
from collections import Counter
from pathlib import Path

OUT = Path(__file__).parent
ROOT = OUT.resolve().parents[5]
selected = json.loads((OUT / "selected-samples.json").read_text())
rows = {}


def add(source, name, upstream, native, refs, difference, reproduce):
    category, ident = name.split(":", 1)
    assert (source, name) not in rows
    pool = selected["sources"][source]
    item = next(i for i in pool["random_samples"] + pool["interactive_supplements"]
                if i["category"] == category and i["id"] == ident)
    rows[source, name] = {
        **item,
        "sample_kind": "random" if item in pool["random_samples"] else "interactive_supplement",
        "upstream_checked": upstream,
        "native_checked": native,
        "native_refs": refs.split(";"),
        "finding": difference,
        "verification_gap": "",
        "verdict": "存在差异" if difference else "本次静态检查未见差异，仍待运行对照",
        "status_boundary": "简化近似" if difference else "已有原生落点",
        "reproduction_or_next_check": reproduce,
        "executed_native_test": False,
        "executed_upstream_browser": False,
    }


# U06 欢乐卡牌：6 个牌定义及 5 个装备技能，抽样实际上覆盖全来源。
add("U06", "cards:jiuwei", "equip5 宝物，装备授予 jiuwei；卡牌定义自身没有额外选人或取消步骤。", "牌表使用 Jiuwei 牌种，装备授予同名 host 技能；宝物槽由牌种装配。", "packages/extensions/huanle-upstream-noname/assets/cards.json:53;crates/content-native-loader/src/decision34_pack9.rs:507", "", "与猴子互相替换宝物时检查旧装备离场、技能撤销；酒尾效果另见 skills:jiuwei。");
add("U06", "cards:kuwu", "equip1，攻击距离修正 -1；unique/nomod/nopower，装备附苦无技能。", "Kuwu 命名武器与同名 ShaDamageForceDiscard 绑定；其技能从装备授予。", "packages/extensions/huanle-upstream-noname/assets/cards.json:41;crates/content-native-loader/src/host_skill_bindings.rs:124", "unique/nomod/nopower 的跨技能装备修改限制没有在本次命名牌表看到对应元数据；不能由武器射程与弃牌正例推定这三个约束已保留。", "让允许变更装备能力的技能作用于苦无，比较是否被 unique/nomod/nopower 禁止；单独核射程2与无武器1。");
add("U06", "cards:mianju", "equip2 防具，仅授 mianju；翻面由对应 turnOverBefore 技能取消。", "Mianju 防具实体及同名 host 绑定存在，装备技能在离场时撤销。", "packages/extensions/huanle-upstream-noname/assets/cards.json:47;crates/engine-native-content/src/content/host/step/skills_status.rs:991", "", "装备/移除面具各触发一次真实翻面；装备时保持原朝向，移除后可翻。");
add("U06", "cards:monkey", "equip5，装备授猴子偷桃；卡定义不产生额外实体。", "Monkey 宝物实体，授 HuanleMonkeyCard；与同槽宝物通过装备路径互斥。", "packages/extensions/huanle-upstream-noname/assets/cards.json:35;crates/content-native-loader/src/host_skill_bindings.rs:122", "", "依次装备猴子和酒尾，确认仅当前宝物技能生效；偷桃分支另见 skills:monkey。");
add("U06", "cards:shoulijian", "基本牌，选一名其他角色，outrange.global=2；目标可弃 he 中一张装备，取消或无装备则受1伤。", "ShouLiJian在PlayCard校验恰1名其他存活目标且computed_distance>1；目标选装备/取消受伤进入TargetDiscardEquipOrDamage。", "crates/engine-native/src/engine/play_card.rs:769;crates/engine-native-content/src/content/host/intrinsic.rs:927", "", "8人局分别选计算距离1和3的其他角色；上游距离小于2不合法。合法目标分别弃手中装备、弃装备区牌、取消，检查伤害与实体守恒。");
add("U06", "cards:xuelunyang", "equip5，装备授写轮眼，无卡牌额外选人。", "Xuelunyang 宝物，装配附同名 HuanleXuelunyangCard。", "packages/extensions/huanle-upstream-noname/assets/cards.json:59;crates/content-native-loader/src/host_skill_bindings.rs:125", "", "替换宝物后不再出现写轮眼询问；技能候选与取消差异见 skills:xuelunyang。");
add("U06", "skills:jiuwei", "自己 phaseEnd 可选发动；已受伤回复1，否则摸1；取消无效果。", "filter 锚 TurnEndWindow after=false，恢复/摸牌分支与当前受伤状态一致；经 optional gate 决定发动。", "crates/engine-native-content/src/content/host/filtering/common.rs:602;crates/engine-native-content/src/content/host/step/skills_recovery.rs:295", "", "phaseEnd 用受伤/满血两局及取消分支，确认不是结束阶段开始或别人的回合触发。");
add("U06", "skills:mianju", "自己 turnOverBefore 强制取消翻面，不问玩家，正反面均保持当前面。", "TurnOverBefore 且当前装备含 Mianju 时，写回 cancelled=true。", "crates/engine-native-content/src/content/host/filtering/common.rs:756;crates/engine-native-content/src/content/host/step/skills_status.rs:991", "走 TurnOverBefore 的路径可取消；U08 xumou 直接翻面再发 TurnOvered 会绕过此技能，属于组合差异。", "面具持有者发动 U08 蓄谋，预期摸3仍保持正面；再用正常翻面事件作阳性对照。");
add("U06", "skills:monkey", "global useCardToBegin；自己当前 equip5 名含monkey，他人用桃且实体载体仍在处理区才可发动；取消不弃宝物。发动结束桃目标事件，弃猴子并获得所有仍在处理区的载体。", "CardUsed 前置窗口，要求他人有效桃且 card Some；确认后弃猴子，取单个载体自手/桌面，整帧 cancelled=true。", "crates/engine-native-content/src/content/host/filtering/common.rs:721;crates/engine-native-content/src/content/host/step/skills_status.rs:939", "过滤缺载体仍在处理区条件；允许从手中取且仅一个 card，不能表达上游多载体/某载体已离开处理区的 filterInD 语义。", "用多张载体视为桃，及在更早触发已移走载体两局；前者应全取仍在处理区者，后者应不发动。普通实体桃确认/取消分别核对血量和牌区。");
add("U06", "skills:xuelunyang", "自己 phaseBegin，须有其他角色持非charlotte技能；先可取消选人，再可取消选该角色的非charlotte技能，成功临时获得到回合结束。", "TurnBegin 装备检查；目标只需 skills map 有项，技能枚举不过滤 charlotte/enabled；选人可取消，第二询问 allow_cancel=false。", "crates/engine-native-content/src/content/host/filtering/common.rs:732;crates/engine-native-content/src/content/host/step/skills_targets.rs:212", "候选技能过滤过宽，且第二级选择不能取消。", "让唯一其他角色只持charlotte辅助技，预期无候选；再持两普通技能，选A/B及第二级取消，取消后不得授技。原生已有 g1_huanle/u06.rs:1017，但需补这两反例。");
add("U06", "skills:kuwu", "自己杀造成 damageSource，排除连环与受害者_notrigger，目标有he才强制由目标选弃1。", "非防止且正伤害、非from_chain、实体杀、受害者存活有he；目标 WaitChoice 选择手牌/装备并弃置。", "crates/engine-native-content/src/content/host/filtering/common.rs:592;crates/engine-native-content/src/content/host/step/skills_status.rs:665", "过滤依实体牌种判杀，未体现 use_as 杀的转化与 _notrigger 排除；不能把实体杀正例覆盖当完整条件一致。", "用黑锦囊经禁言转杀造成伤害，苦无应照常让目标弃牌；连环伤害与目标无he各不触发。g1_huanle/u06.rs:883 是现有入口。");

# U07 玩点论杀。
add("U07", "cards:diaobingqianjiang", "普通锦囊，自己及有手牌者全体目标；先自己摸1、对战可选结算方向，再亮ceil(存活人数/2)池牌。每目标可不换，换时自选池牌及手牌；末尾使用者可选任意非空子集按任意序置顶，剩余弃置。", "WudianDiaobingCard 亮池、逐 targets 问手牌，但池牌固定 remove(0)，收尾直接弃池。", "crates/engine-native-content/src/content/host/step/wudian_cards.rs:655;crates/engine-native-content/src/content/host/step/wudian_cards.rs:715", "缺池牌自由选择、对战方向及末尾置顶选择。", "4人各1手牌，亮两种不同池牌，第一目标选择第二张；结束时使用者选1张置顶。对照池顺序、牌堆顶与取消换牌分支。");
add("U07", "generals:wdb_liuyan", "男、群、3体力，技能 juedao/geju；武将定义不含可选结算。", "同名男群3及两技能按原顺序登记。", "crates/content-native-loader/src/decision34_pack9_data.rs:133", "", "安装装配后核性别/势力/3上限和两技能；这里未据武将表推定两技能规则等价。");
add("U07", "generals:wdb_zaozhirenjun", "男、魏、3体力，liangce/jianbi/diyjuntun，复姓姓名元数据枣|祗-任|峻。", "男魏3和三技能一致；ExtensionGeneral 表保留符号名，未携姓名拆分。", "crates/content-native-loader/src/decision34_pack9_data.rs:137", "姓名拆分元数据未在原生武将模型中保留，相关按姓规则/显示不能由基础装配证明。", "核三技能装配及名字显示；若用按姓判定技能，把该双人武将列为专门对照。");
add("U07", "skills:choudu", "phaseUse限1，弃1张he，选任意能用调兵者作为使用者；以该角色为座序，对全体使用调兵遣将。选牌/目标取消不消耗。", "弃牌后 m.owner=t，m.targets=vec![t]，仅该目标进入调兵换牌机。", "crates/engine-native-content/src/content/host/step/wudian_cards.rs:572;crates/engine-native-content/src/content/host/step/wudian_cards.rs:645", "将使用者错误兼作唯一目标；其余角色失去换牌窗口，且沿用调兵池/收尾差异。", "4人各有手牌，筹度选择1号作为用牌者；上游0/1/2/3均有换牌机会，原生只有1号。g1_wudian/u07.rs:551未验证全员。");
add("U07", "skills:fuchou", "target shaBefore，自己有he可交1给来源；取消选择照常响应杀，交牌只取消该目标杀事件并记录来源，没有返还杀载体。", "ShaMachine 选择后把桌面杀返还 source 手牌，再转交所选he并记 fuchou_marked。", "crates/engine-native-kernel/src/kernel/sha/resume.rs:50", "额外返还杀实体，牌消耗与上游不符。", "攻击者仅1实体杀，防御者交1手牌；结束攻击者应只有所得牌，杀在弃牌堆。另测交装备/取消。现有入口 g1_wudian/u07.rs:1086。");
add("U07", "skills:jinyan", "hp<=2才可将黑锦囊当杀使用或打出；同时禁止这些黑锦囊以原名使用、响应、救援，hp>2不施限制。", "实际play_rules只在hp<=2时禁止黑锦囊原名PlayCard；WudianJinyan host无触发且步进Done，未给该绑定装配view_as或四渠道响应禁令。", "crates/engine-native/src/engine/play_rules.rs:57;crates/engine-native-content/src/content/host/step/wudian_status.rs:154;crates/content-native-loader/src/host_skill_bindings.rs:158", "只实现主动使用位的黑锦囊禁用；原绑定没有上游chooseToRespond/chooseToUse黑锦囊转杀及响应/救援禁用，不能从这一个PlayCard门视为四面一致。", "2体力黑无懈不能原名响应，却应能当杀响应决斗；升3体力后可原名无懈且不可转杀。检查持久询问保存恢复。");
add("U07", "skills:moshou", "纯mod.targetEnabled：乐不思蜀、兵粮寸断不能以持有者为目标；闪电不在此禁单。无需询问。", "MoshouImmunity 无事件机器，目标修饰专门屏蔽乐/兵，未以所有延迟锦囊概括。", "crates/engine-native-content/src/content/host.rs:5;crates/engine-native-content/src/content/host/step/skills_status.rs:705", "", "分别用乐、兵、闪电检查合法目标，移除技能后恢复；不把 host filter=false 误判为缺失。");
add("U07", "skills:youdishenru", "source damageEnd，若杀命中自己storage指定者则删除追杀标记；silent，无玩家选择，是诱敌深入循环的终止条件。", "辅助技能绑定WudianDiaobingCard且filter=false；实际ShaMachine响应诱敌时开一次追杀选择，resume接受后发一个新CardUsed便Flow::Done，没有保留追杀到命中为止的外层续行。", "crates/content-native-loader/src/host_skill_bindings.rs:51;crates/engine-native-content/src/content/host/filtering/common.rs:1046;crates/engine-native-kernel/src/kernel/sha/step.rs:874;crates/engine-native-kernel/src/kernel/sha/resume.rs:302", "不是仅因辅助host无机器判漏：实际替代路径只允许一张后续杀并结束，若后续杀再被闪，不会按上游storage追杀直至命中或放弃。", "诱敌后首杀未命中、第二杀命中指定者，再进行第三次杀；应命中即停止且移除临时追杀标记，不能触发换牌。");
add("U07", "skills:youzhan", "global shaBefore，距该杀目标<=1且有he装备可选弃1；取消无事。弃后由目标使用诱敌深入，攻击者可持续出杀，直到命中或放弃受伤。", "CardUsed 只取 targets[0]过滤；自选弃装备后取消整张杀，直接开攻击者杀询问。", "crates/engine-native-content/src/content/host/filtering/common.rs:1020;crates/engine-native-content/src/content/host/step/wudian_cards.rs:439", "多目标杀只看首目标，且整帧cancel会影响其他目标；并非逐shaBefore目标独立处理。", "一杀目标A/B，只有B距离援战者<=1；上游应为B出现援战而A正常结算。单目标再测弃装备/取消及杀循环。现有 g1_wudian/u07.rs:344。");
add("U07", "skills:zhucheng2", "有筑城牌时target shaBefore可发动；攻击者自选恰好筑城数张he，取消令该目标杀无效，杀载体仍消耗。", "只开放攻击者一张手牌/Pass；选1后其余随机从手牌弃，Pass整牌cancel并返杀。", "crates/engine-native-content/src/content/host/step/wudian_cards.rs:168", "丢失he任意多选、强行随机补弃；无效时返杀与全帧取消也改变上游。", "筑城2张，攻击者手牌1+装备1；应可选两者支付。再选取消，杀应弃且多目标其他人仍结算。");

# L06 五行：只有两个可交互技能；第三项是装备牌。
add("L06", "cards:wuxingpan", "equip5/fullskin，装备授 wuxingpan_skill，本身无额外效果/询问。", "WuxingPan 宝物附 wuxingpan_skill；有同名实体加载和装备技能。", "crates/content-native-loader/src/decision34_pack9.rs:256;crates/content-native-loader/src/decision34_pack9.rs:507", "", "装备后出现改五行入口，替换宝物后入口消失；本项不代替技能多分支对照。");
add("L06", "skills:_shengke", "target useCardToBegin，connectMode直接禁用；卡与目标均有五行时按五克/五生关系：克且有he由目标强制选弃1，生摸1，无关系不变。", "CardUsed 包含目标且两者有nature；KE/SHENG表后选择he弃牌或start_draw1；filter未检查联机。", "crates/engine-native-content/src/content/host/filtering/common.rs:606;crates/engine-native-content/src/content/host/step/skills_status.rs:677", "联机禁用条件缺失，且在整CardUsed阶段并非逐目标useCardToBegin；追加目标、被移除目标需对照。", "金→木he选A/B必须不同且不可取消；金→水摸1；金→火不变；再以联网模式同场景确认不触发。l06_wuxing_eight.rs:103。");
add("L06", "skills:wuxingpan_skill", "phaseUse限1，选择手牌且不失去载体，随后在metal/wood/water/fire/soil五项必选其一，永久改该实体属性；第一步可放弃。", "SelectCard→SelectNature→Act写 card_natures，保持手牌实体；第二步 allow_cancel=true。", "crates/engine-native-content/src/content/host/step/skills_card_use.rs:6", "上游属性控制必选，原生新增第二步取消；取消后次数是否返还需真实事件验证。", "同一手牌分别改metal与water，移到他人手再使用保持属性；第一步取消可重开，第二步上游应无取消。保存属性选择挂起后恢复。l06_wuxing_eight.rs:439。");

# L22 段位制裁：5个自动silent/direct钩子，没有可选技能。
add("L22", "configs:eloDifficulty", "枚举eeasy/veasy/easy/normal/hard/vhard/ehard/paiwei，默认paiwei；段位值单机不生效，影响摸牌/判定/拆顺排序。", "parse配置并经resolve_difficulty注入elo_eh/difficulty/online标志，难度供内核elo函数使用。", "crates/content-native-loader/src/elo.rs:550;crates/engine-native-kernel/src/kernel/elo.rs:447", "", "枚举逐值比较友敌diff/eh；paiwei在线与离线分开，normal不改牌堆。不能只测配置序列化。");
add("L22", "configs:eloPlayers", "默认true；配置声明声称限制将池，但本扩展正文没有读取eloPlayers，无可选技能或规则执行步骤。", "restrict_pool原样返回名单；引擎配置写elo_players_enable但不借此删将。", "crates/content-native-loader/src/elo.rs:302;crates/content-native-loader/src/elo.rs:565", "", "开关两值比较相同候选池；记录为配置已登记，不能宣传将池限制已实现。");
add("L22", "configs:timeKiller", "默认none，light/middle/heavy选择AI烧条时长；只影响非玩家控制者的特定前置询问及可选祖安语音。", "枚举转0..3；elo_on_choice_before记录delay_ms与语音线索，未实际阻塞内核。", "crates/content-native-loader/src/elo.rs:558;crates/engine-native-kernel/src/kernel/elo.rs:675", "保存延时标志不证明产品实际延时和语音已播放。", "产品询问路径测none与heavy的实际可交互时刻，玩家本人不延时；此审查不执行墙钟测试。");
add("L22", "skills:_eloDraw", "drawBegin/judgeBegin自动，eh非0、牌堆>1、game.me存在且diff>1；取ceil(num*diff)候选，摸牌按get.value、判定按本次judge函数排序，已知槽保留。无取消。", "elo_on_draw_begin保留候选窗口与已知槽；judge score仅专门处理乐/兵/闪电，其余用card_value。", "crates/engine-native-kernel/src/kernel/elo.rs:336;crates/engine-native-kernel/src/kernel/elo.rs:447", "任意技能判定函数被退化成静态牌值，不能保留上游自定义judge偏好。", "增加一次以红色为好/另一局黑色为好的普通技能判定，同牌堆与难度应产生相反排序；不能只测乐兵闪。");
add("L22", "skills:_eloHandcard", "gainPlayerCardBegin/discardPlayerCardBegin；非me非受控、eh非0、diff>=3；diff3仅1/3随机，通过友敌/eh组合时设visible=true，不问选择。", "elo_on_extract_player_card_begin按source/me/controlled、diff门与RNG设置visible，参与手牌抽取前置。", "crates/engine-native-kernel/src/kernel/elo.rs:540", "", "diff2不公开，diff4指定友敌组合公开；diff3固定引擎种子重复并检查rng推进，手牌位置选择本身仍由提牌流程执行。");
add("L22", "skills:_eloInitCards", "enterGame或初局phaseBefore，game.me存在且phaseNumber为0时，把当前手/装备标为已知；不改变牌归属，无取消。", "elo_try_init_cards用phase_number0及once门，将he写已知槽。", "crates/engine-native-kernel/src/kernel/elo.rs:131", "", "开局手牌与装备已知，后续摸牌不应被这次初始化全标；中途enterGame另测门控。");
add("L22", "skills:_eloMechanism", "enterGame/初局phaseBefore初始化一次game级牌堆插入/取牌重写；要求game.me。此技能自动，没有可取消问询。", "elo_try_init_mechanism首次建标；牌堆相关elo钩子在内核常规入口调用。", "crates/engine-native-kernel/src/kernel/elo.rs:109", "", "两次开局事件不重复初始化；插入已知牌后再摸牌检查已知槽固定，不借helper测试代替真实对局验证。");
add("L22", "skills:_timeKiller", "仅chooseToUseBefore/chooseToRespondBefore/chooseToDiscardBefore，非自己/受控且在线timeKiller非none；随机1.5*等级秒范围，长等待可祖安发言。", "elo_is_choice_before_prompt还匹配choose_target/control/bool/player_card和所有vm_前缀，RNG生成delay标志。", "crates/engine-native-kernel/src/kernel/elo.rs:647;crates/engine-native-kernel/src/kernel/elo.rs:675", "扩大到上游没有的询问会多消耗随机流并引入延时；实际UI烧条未核验。", "同种子只触发一次chooseTarget技能，上游不应烧条或取随机，原生若匹配则rng/延时旗改变；再用respond作阳性对照。");

# L29 龙血玄黄：派生实际实现来自本体mobile.js，不读旧复核说明。
add("L29", "configs:shengjian", "默认true，仅圣剑专属显示开关；不改变道心阈值和规则选择。", "LongxueConfig读settings.shengjian；前端setLongxueHolySword返回开关与effectCue。", "crates/content-native-loader/src/longxue.rs:14;apps/web-client/src/engine/extensionSurfaceRuntime.ts:1512", "仅开关/cue，尚无本次可见剑特效对照。", "同道心状态开关两值检查规则状态不变，并截取真实牌桌剑区；静态配置解析不算画面对齐。");
add("L29", "skills:mbcmfangzhu", "mobile.js:17410，phaseUse限1，选非自己且非前次目标；技能失效分支屏蔽非forever/charlotte，无法用牌分支仅准锦囊；各到目标phaseEnd，前次目标按后续phaseUseAfter解除。", "双询问选目标与Tag，ban只限制手中基本/装备；白板用中文锁定技白名单保留，其余enabled=false。", "crates/engine-native-content/src/content/host/step/skills_targets.rs:536;crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_flags.rs:629", "用锁定技中文白名单代替forever/charlotte，禁牌还多了in_hand限制；目标记录解除时机也需对照。", "对普通锁定技能持有者选择白板应失效，forever未知名称技能应保留；再对装备区转化基本牌检查是否被禁。l29_longxue_eight.rs:686。");
add("L29", "skills:mbcmjiushi", "mobile.js:17251，正面可翻面视为用酒（含自救）；翻面可获得牌堆锦囊；背面受伤后可翻正，需保留其pre-damage分支记录和取消。", "MbcmJiushi正面主动、Damage/TurnOvered分支及锦囊搜索存在，特定boosted杀/from_chain标志排除；主动位由机器处理。", "crates/engine-native-content/src/content/host/step/skills_targets.rs:439;crates/engine-native-content/src/content/host/filtering/common.rs:429", "当前规则入口覆盖不代表chooseToUse救援上下文与翻面前置组合一致；翻面路径的Before是否经过需运行确认。", "正面出酒/濒死自救、背面受非酒诗强化伤害选择翻正或取消，翻面时选取锦囊及面具组合；保存询问。l29_longxue_eight.rs:577。");
add("L29", "skills:mbcmqingzheng", "mobile.js:17117，phaseUseBegin有手牌可发动；选可全弃的一种花色弃其全部，再选有手牌的他人，选其一种花色弃全部；对方弃数较少则造成1伤，途中取消按已执行步骤保留。", "MbcmQingzheng分阶段询问花色/目标/目标花色，按各花色枚举、弃牌并比较数量。", "crates/engine-native-content/src/content/host/filtering/common.rs:425;crates/engine-native-content/src/content/host/step/skills_damage.rs:453", "本次未见所有cardDiscardable检查的对应；不可弃牌花色的候选排除仍待真实运行核对。", "自己某花色含不可弃牌应不出现；双方弃2/1则伤1、弃1/2不伤，首步取消无事。l29_longxue_eight.rs:417。");
add("L29", "skills:mbjuejin", "mobile.js:17510，phaseUse限定：全存活者体力调到1并获得abs(oldHp-1)护甲，自己额外2；只将全员手牌中的闪/桃/酒移出，再加向死存魏。", "限定旗后设全员hp1；hp>1给差值护甲，hp<1未给差值；同时移出牌堆、弃牌堆、手/装/判定等区闪桃酒。", "crates/engine-native-content/src/content/host/step/skills_targets.rs:669", "清理范围扩大；负/零体力角色护甲计算遗漏。", "牌堆、弃牌堆、他人手牌各放一桃；上游仅手牌桃移出，其他保留。另令角色hp0但存活，绝进后应hp1+1护甲，自己再+2。");
add("L29", "skills:mbqianlong", "扩展重写：初局phaseBefore/enterGame道心+20，若自己可对其他魏将发动卫统则+60；gainEnd每次+5、受伤每点+10、造成伤害每点+15，上限99；25/50/75/99按阈值刷新派生。", "首个Phase且phase_number1初始化；boost查其他魏角色持卫统，增益事件映射CardsDrawn/CardsMoved/Damage，刷新阈值。", "crates/engine-native-content/src/content/host/filtering/common.rs:53;crates/engine-native-content/src/content/host/step/skills_status.rs:195", "开局初始化后移，且原生检查其他魏角色持卫统，上游是自己hasZhuSkill(mbweitong,current)；已确认这两条件差异，获得牌多事件映射是否重复只列待核验。", "自己同时持潜龙/卫统且另有无卫统魏角色，开局应60；仅别人持卫统而自己不持则20。移动同1张牌只+5并核20→25解锁。");
add("L29", "surfaces:main1.css", "技能区CSS定义技能按钮、可用/不可用、提示与专属布局；没有对局选择或卡牌结算。", "前端通用扩展surface状态能记录剑和阈值，未看到此CSS所定义布局逐规则移植。", "apps/web-client/src/engine/extensionSurfaceRuntime.ts:901", "仅状态落点，原技能区画面尚未独立对照。", "同分辨率在道心20/50/99截技能区，比较按钮位置、禁用样式与提示；不能用道心数值测试替代CSS。");
add("L29", "surfaces:main1", "main1.js重写技能控件、技能可用性刷新、点击与提示显示，并加载专属技能区；其点击最终调用既有游戏选择流程。", "当前LongxueSurfaceState保存进度/解锁，缺main1整套按钮构建与点击行为的直接移植证据。", "apps/web-client/src/engine/extensionSurfaceRuntime.ts:143;apps/web-client/src/engine/extensionSurfaceRuntime.ts:1509", "有状态不等于真实控件交互一致。", "由牌桌点击清正/酒诗/放逐并返回/取消，比较禁用可用状态；本次未启动浏览器。");
add("L29", "ui_hooks:_CMeffects", "gameStart/dieAfter自动，只有本机曹髦且仍有指定技能才创建剑/道心区；技能全失移除，已建则不重复，包含专属资源动画。", "前端只切shengjian与effectCue，未按角色、技能丢失、死亡维护原DOM生命周期。", "apps/web-client/src/engine/extensionSurfaceRuntime.ts:1512", "缺角色门控/移除生命周期与画面证明，不应与规则技能等价混计。", "曹髦与其他将开局各一局，丢全部派生技能/死亡后检查专属区移除且不重复创建。");

# L10 史诗卡牌。
add("L10", "configs:cardEmoji", "默认false且unshow=true的新菜篮子开关，只属显示配置。", "EpicConfig保留false；前端提供throwEmotionCue，未保留隐藏菜单属性。", "crates/content-native-loader/src/epic_cards.rs:93;packages/extensions/epic-cards-upstream-liuli/assets/config.json:8", "菜单隐藏属性与真实表情画面未移植；配置值不证明渲染。", "检查默认关闭且菜单是否隐藏；开关时同一扔表情事件画面对照，规则状态不应变化。");
add("L10", "configs:cardFold", "默认true，手牌过多时完全折叠并禁滑动；onclick同时保存EpicFX与史诗卡牌两个键。", "包资产及parser默认false，只有bool记录，没有上游双键联动或手牌折叠行为。", "crates/content-native-loader/src/epic_cards.rs:94;packages/extensions/epic-cards-upstream-liuli/assets/config.json:9", "默认值相反，手牌管理效果与兼容键缺失。", "新安装不带旧设置，20张手牌比较默认布局/滑动；开关后重启并核持久化。");
add("L10", "configs:cardFold_proportion", "默认10，3到10八项；关闭完全折叠时控制两张手牌遮挡比例30%..100%。", "EpicConfig无该数值字段，包配置也没有相应键；仅card_fold布尔。", "crates/content-native-loader/src/epic_cards.rs:31;packages/extensions/epic-cards-upstream-liuli/assets/config.json:1", "比例配置缺失。", "完全折叠关闭，分别选3和10，手牌重叠应明显不同并可保存；原生目前无可选入口。");
add("L10", "configs:cardPhantom", "默认true，幻影出牌显示开关，无规则问询。", "资产及parser默认false，仅布尔登记。", "crates/content-native-loader/src/epic_cards.rs:95;packages/extensions/epic-cards-upstream-liuli/assets/config.json:10", "默认值反向，幻影出牌画面仍无本次对照。", "清设置首次用牌比较是否有幻影；关闭仅去幻影，不能改变耗牌/目标。");
add("L10", "configs:shichangshi", "默认true，开关十常侍还原，配合十周年UI替换技能与专属图/控件。", "包默认false；解析布尔与依赖判定存在，规则host k_danggu独立登记。", "crates/content-native-loader/src/epic_cards.rs:48;packages/extensions/epic-cards-upstream-liuli/assets/config.json:12", "默认关闭与上游默认开启不同；技能替换成功不代表专属特效完成。", "仅安装必要依赖、清设置后选择十常侍，检查默认伙伴控件；开关对技能替换与图片分别验。");
add("L10", "configs:skillEffects", "默认true，限定/觉醒特效显示；无独立规则选择。", "配置解析和资产true一致；原生前端通用效果cue承接，未按本源资源效果逐项复刻。", "crates/content-native-loader/src/epic_cards.rs:91;packages/extensions/epic-cards-upstream-liuli/assets/config.json:6", "只登记开关，限定/觉醒特效未在本次真实牌桌比对。", "同限定技能开关各触发一次；动画可见差异和规则限定旗分别记录。");
add("L10", "configs:分割线02", "clear=true、init=true，仅line.png图片分隔，无保存后的规则效果。", "EpicConfig无此装饰项；没有原生对应分隔图。", "crates/content-native-loader/src/epic_cards.rs:31", "纯菜单装饰缺失，不能把“不影响规则”当作视觉等价。", "设置页截图核原分割图及间距；不需要构造对局规则测试。");
add("L10", "skills:rangjie", "damageEnd逐伤害点选择获得牌/可合法移动时移动牌/取消；获得分支先摸1再按所选类型搜索，移动也先摸1。实际源码把控件写'取消'却仅比较'cancel2'，第三项可能继续摸牌/移动。", "每伤害点tag0/1/2；2直接Done，0摸1后从deck正向首张取类型，1选场上ej再选任意存活目的地。", "crates/engine-native-content/src/content/host/step/skills_cards.rs:9", "实际取消标签与原生取消不同；移动候选只要求全场存在ej，未先筛可合法迁移目的地。deck正向find还需对照get.cardPile2的堆顶搜索方向。", "受2伤第一轮选'取消'，记录上游真实是否仍摸牌/开移动，不能手工替换结果为cancel2；另只有不可迁移ej时不应提供移动。l10_epic_cards_eight.rs:609。");
add("L10", "skills:reguhuo", "chooseToUse/Respond，声明当前合法basic/trick及属性杀；group reguhuo_guess负责预用牌质疑、真假和失败；真正执行guess后加本回合禁用。选择取消不应在此前耗机会。", "进入Act即插vm_aux.once，选择载体/牌名后直接发CardUsed，固定20牌种；没有质疑机。", "crates/engine-native-content/src/content/host/step/skills_virtual_use.rs:202;crates/engine-native-content/src/content/host/constants.rs:8", "取消过早消耗机会且省略真假质疑，固定列表不能覆盖安装扩展的新basic/trick。", "首次选牌取消后重新发动；假牌声明杀让对手质疑，必须按refresh.js:10639的guess失败；真牌质疑走相应惩罚。l10_epic_cards_eight.rs:498不覆盖质疑。");
add("L10", "ui_hooks:_emojiRebate1", "emoji.js:700，recoverAfter/dying/gainAfter/turnOverAfter，必须有其他source，按敌友/救援parent/给予者选赞美或羞辱列表并随机播放；非通用手动表情。", "triggerL10EmojiRebate接受任意target/emotion，拼rebate cue；没有事件source/敌友/列表过滤。", "apps/web-client/src/engine/extensionSurfaceRuntime.ts:1352", "仅手动cue函数，实际事件条件与随机表情选择缺失。", "同一回复分别由敌/友/自己发起，检查仅前两者正确对象/列表，取消不适用；比较真实显示而非字符串。");
add("L10", "skills:k_danggu", "初局phaseBefore/enterGame与休整回来初始化十常侍池；随机主常侍，若高望仍在则随机3其他+高望并打乱；按冲突排候选，玩家必选伙伴（全冲突时人类解锁全员，AI随机一个），移出两名授技能。", "随机主后随机抽4，再必要时把第4项替换高望；冲突全空时全开放，伙伴询问强制，授两套技能。", "crates/engine-native-content/src/content/host/step/skills_recovery.rs:338;crates/engine-native-content/src/content/host/step/skills_recovery.rs:452", "高望候选生成与随机消耗/分布不同于先抽3再shuffle；AI全冲突也从全员选择而非上游预随机一个。", "固定主与剩余池含高望，跨多种子比较候选分布和RNG；人类选A/B与休整回来池减两，伙伴询问无取消。l10_epic_cards_eight.rs:717。");

# L28 雷霆万钧。
add("L28", "cards:th_taipingyaoshu", "红桃3防具/equip2，附同名技能和全局AI；离场在外层事件after摸2，再hp>1失1，unequip2时不触发。", "CardDef::armor红桃3+equip技能；CardsLost/CardsMoved机器直接start_draw2并排LoseHp，配合装在身标志。", "crates/content-native-loader/src/decision34_pack9.rs:730;crates/engine-native-content/src/content/host/step/l28_batch_a_asks.rs:830", "上游在外层事件after结算，原生按失牌事件直接推进；unequip2免离场效果的条件本次未在过滤中见到。", "防具被一次多步技能弃置，比较其他步骤完成后才摸2失血；再带unequip2失防具不得触发。");
add("L28", "configs:timeTheme", "默认follow，decade/shousha/follow三项，决定进度条风格；没有规则输入。", "ThunderConfig合法项表和默认follow保留；实际进度条由原生通用UI。", "crates/content-native-loader/src/thunder.rs:95;crates/content-native-loader/src/thunder.rs:164", "配置已登记，但两主题与follow真实效果尚未逐图对照。", "三值在同一询问等待状态截图，follow再切换母主题验证联动。");
add("L28", "generals:th_majun", "男魏3，th_jingxie/th_qiaosi，武将本身无单独选择。", "原生同名男魏3、相同两技能装配。", "crates/content-native-loader/src/decision34_closure.rs:220", "", "装配后验证两技能与3体力；未在本项复核它们完整内容。");
add("L28", "generals:th_mamidi", "男群3，th_chengye/th_buxu。", "同名男群3和两技能一致，补续另作技能样本。", "crates/content-native-loader/src/decision34_closure.rs:218", "", "校武将元数据与技能授予，补续的类别搜牌差异不能由此武将表掩盖。");
add("L28", "skills:th_buxu", "phaseUse按本阶段使用次数递增弃he，expansion少于6且尚有缺六经类别；选择缺类别后按该类filter在牌堆搜索，找不到回减统计；选牌/类别可在承诺前取消。", "成本/类别询问存在，但类别选后直接deck.pop，任意顶牌作为该六经并置mask。", "crates/engine-native-content/src/content/host/step/l28_batch_a_asks.rs:702", "没有按经类搜索，也没保留搜不到退使用统计。", "牌堆顶杀、堆内无懈，选礼应取无懈而非杀；堆内无目标类则技能次数不增长。l28_thunder_eight_match.rs:615。");
add("L28", "skills:th_jieshu", "useCard/respond强制；牌有数值且满足割圆filterNumber（在环不在弧、与已有弧相邻）摸1；环外点数牌不计手限且弃牌阶段不可弃。", "CardUsed/CardResponded用同filter_number并start_draw1；同时写th_jieshu_rank_mask。本次未取得环外不计手限/阶段不可弃的查询落点，不能声称内核派生旗已承接。", "crates/engine-native-content/src/content/host/step/l28_batch_b_geyuan.rs:35;crates/engine-native-content/src/content/host/step/l28_batch_b_geyuan.rs:61", "待核验：摸牌分支已读到，环外牌的手限/强制弃牌组合尚需真实对局对照。", "环1/2/3、弧1时用2摸1，用1/3按环邻接规则；环外K不占手限且不可被阶段弃置，响应也测一张合法点数。");
add("L28", "skills:th_tianshu", "phaseUseBegin有he且全场hejsx无要术，可合并选择弃1和目标；取消不弃。首次可创建唯一红桃3要术，否则从牌堆查找，先给目标，再由目标合法装备。", "先弃牌后选目标，目标无答默认自己；从catalog找首同名CardId直接equip_card_onto，找不到仅授技能。", "crates/engine-native-content/src/content/host/step/l28_batch_a_asks.rs:800;crates/engine-native-content/src/content/host/step/l28_batch_a.rs:503", "先扣成本再选目标改变取消边界；目录里存在定义不代表实体仍可取，直接装备跳过获得/合法用牌路径，缺牌时仅授技也非上游。", "要术实体在弃牌堆/被移出而全场无持有者，发动后不得凭目录复制同id；初步取消不耗牌，目标不能装备时核实际选择。");
add("L28", "skills:th_tiansuan2_3", "下签为charlotte强制damageBegin3，对自己承受的伤害+1，无选择。", "抽下签授技能并置extra_damage，内核伤害修饰读该旗。", "crates/engine-native-content/src/content/host/step/l28_batch_b.rs:583;crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_flags.rs:831", "待核验：需单独授该子技能而不预置旗时确认+1；仅抽签父技能生成旗正例不足证明子技能本身可独立生效。", "直接从合法授技路径只获得th_tiansuan2_3后受1伤应2；无下签1，移除下签恢复1，带伤害上限技能测Begin3顺序。");
add("L28", "skills:th_yeyan", "出牌阶段限定；小业炎至多3人各1火；大业炎弃4张不同花色手牌、失3体力，把3火点分给1或2人（2+1），过程中选数/目标/牌，承诺前可取消。", "模式→目标→四花色牌询问，检查4牌mask，按选择顺序第一人2第二人1或单人3，座序发伤害；限定只resolve时置。", "crates/engine-native-content/src/content/host/step/l28_batch_b_asks.rs:327", "待核验：所选目标分配顺序与本源DOM分配交互，以及失3体力引发濒死后伤害顺序仍需真实对照。", "大炎A2/B1与B2/A1结果必须不同，少花色重问不扣牌，模式/目标取消无限定消耗，失血后救援挂起恢复。l28_thunder_eight_batch_b.rs:440。");
add("L28", "skills:th_zhengjing", "phaseUse限1，真实小游戏决定产出与爆炸；成功选目标，chooseToMove把任意非空子集置目标经典，其余自己拿。取消/失败按小游戏分支结束。", "玩家选择count而非小游戏产出，直接取牌堆；首张固定给目标，其余自己拿；count0/force_boom还有另行取经/摸牌路径。", "crates/engine-native-content/src/content/host/step/l28_batch_a_asks.rs:581", "小游戏结果被人工数量替代，分配被固定为1张，不能表达任意非空子集。", "成功产出3张，选择给目标2张、自留1张；另给目标3张与失败分支，原生当前不可表达。l28_thunder_eight_match.rs:578。");

# L27 选将美化：保留inventory类别；部分ui_hooks实际含规则。
add("L27", "configs:dianjibiaoji", "默认true，点击标记牌库与标记样式美化，重启生效，无规则选择。", "SelectBeautifyConfig以默认true解析此bool。", "crates/content-native-loader/src/select_beautify.rs:109", "只保存设置；点击标记牌库真实控件/外观未在本次对照。", "牌桌带多个标记开关两值重启，点标记比较弹出牌库与样式，不以bool解析测试背书画面。");
add("L27", "configs:shuzianniu", "默认true，数字按钮美化，重启生效；数字决定技能输入但此开关本身只切显示。", "SelectBeautifyConfig默认true，host技能以Tag询问替代原数字加减控件。", "crates/content-native-loader/src/select_beautify.rs:112", "UI只保留Tag输入能力，原加减按钮画面未独立比对。", "奇谋/咒诅数字选择的上下限、取消与键盘/点击可达性，与开关对照截图分别记录。");
add("L27", "skills:bolan", "phaseUseBegin可发动，按已安装武将/技能排除禁将、觉醒/主公/使命等，描述含出牌阶段限一次者随机3选1，到本阶段结束；不选择不获得。", "固定BOLAN_SKILL_POOL只有zhiheng/qiangxi/quhu，询问总为0..3，不随机构建池。", "crates/engine-native-content/src/content/host/step/l27_batch_b.rs:63;crates/engine-native-content/src/content/host/step/l27_batch_b.rs:671", "固定三技能替代动态池与随机抽取；插件技能不能入候选。", "安装含至少第四个合格技能的包，跨种子比候选；选A/B得到不同技能并于phaseUseEnd移除。l27_beautify_batch_b.rs:21。");
add("L27", "skills:clandianzhan", "useCardAfter强制，每轮首次花色，且恰好1目标、目标未横置或有同花色可重铸手牌；横置该目标，重铸全部同花色手牌，再摸1。下轮重新计算历史。", "CardUsed filter只查首次mask，不查单目标条件；结算若单目标则横置，始终摸1，无重铸；四花色齐时立刻重置mask。", "crates/engine-native-content/src/content/host/step/l27_batch_a.rs:398;crates/engine-native-content/src/content/host/step/l27_batch_a.rs:888", "过滤/时机不同，缺重铸，重置从下一轮改成四花色凑齐。", "手里2红桃，用第1张红桃单目标后另1张应重铸并额外摸1；多目标不发动；同轮4花色后第5张已用花色不应再发动。");
add("L27", "skills:new_zhixi", "useCard先计已用数+1，锦囊置止息旗；计数>=当前hp或已用锦囊时禁用/响应；移除清计数与标记。新获得时计数应0。", "host.rs用marks计数>=hp禁牌，技能结算每次+1与锦囊mark；new_meibu授技时却把同计数先设hp。", "crates/engine-native-content/src/content/host.rs:210;crates/engine-native-content/src/content/host/step/skills_status.rs:244;crates/engine-native-content/src/content/host/step/skills_virtual_use.rs:175", "与每步组合使未出牌即封锁；这是实际计数初始化错误。", "3体力者被施止息，第一张杀应合法，出3张后才封；若第一张锦囊后立即封。原生hp初值会首张即禁。");
add("L27", "skills:reqimou", "主技能enable被注释但group reqimou_m2是phaseUse入口；其DOM选1..hp后chooseBool可拒绝，确认先loseHp再useSkill(reqimou)。主content:6107觉醒/置限定，draw(reqimou2)并授本回合reqimou2/mark。", "主动host Tag选1..hp且该询问不在CANCELLABLE_ASKS；答后置used/x，LoseHp x并start_draw x，提供距离/次数旗。", "crates/engine-native-content/src/content/host/step/l27_batch_a_asks.rs:92;crates/engine-native-content/src/content/host/step/l27_batch_a_asks.rs:297;crates/engine-native-content/src/content/host/step/l27_batch_a_asks.rs:641", "摸x本身与上游相同；确定差异是数字询问不可取消，缺上游选数字后chooseBool拒绝分支。失血濒死与摸牌子树先后另待真实运行。", "点击真实reqimou_m2按钮选2后拒绝，应不失血、不摸、不耗限定；原生数字询问无对应拒绝。确认2再比血量-2、摸2、距离/杀次数，并在失血濒死时保存恢复。l27_beautify_match.rs:583。");
add("L27", "skills:xiefang", "初始化及dieEnd/gameDrawEnd刷新女性数标记；距离始终减场上存活女性人数，标记为显示，不问玩家。", "computed_distance每次按有效性别数减，标记在Death/自己Draw结束刷新；实际距离不依赖可能过期显示mark。", "crates/engine-native-kernel/src/state/core_ops_modifiers.rs:240;crates/engine-native-content/src/content/host/step/skills_status.rs:276", "显示刷新时点将gameDrawEnd换成自己摸牌阶段结束；距离主效果静态吻合。", "开局装备/性别变更后看距离与标记，女性死亡后同步减1；把显示时点与规则距离分别记。");
add("L27", "ui_hooks:sbqingzheng", "实际上是清正规则：出牌阶段开始，按标记决定弃自己若干花色所有手牌，再选目标/花色，比较弃数造成伤害与选增标；自定义DOM提供取消。", "原生有sbqingzheng多级询问和结算，非纯惰性UI；通用Tag替代自定义四花色牌展示。", "crates/engine-native-content/src/content/host/step/l27_batch_b_tw.rs:580;crates/engine-native-content/src/content/host/step/l27_batch_b_asks.rs:237", "待核验：inventory类别不能推成无规则效果；尚需逐成本标记/后续选项实跑，并另核原自定义对话框画面。", "自己红/黑两组与目标两组牌，选不同花色产生不同弃数/伤害；在首DOM取消不耗牌，后续挂起恢复；截图花色列表。");
add("L27", "ui_hooks:twxiafeng", "实际规则：phaseUseBegin有暴虐值可选0..min(3,marks)并确认；扣点后本回合前x张出牌不可响应、不计次数/无距离且手限+x，不是仅装饰。", "Tag询问1..min3扣标记，置x/rem/spent供修饰，phaseEnd清；没有数值0，但可外层取消。", "crates/engine-native-content/src/content/host/step/l27_batch_b_tw.rs:22;crates/engine-native-content/src/content/host/step/l27_batch_b_asks.rs:264", "待核验：规则有落点，原数字0和取消确认的等价轨迹、前x张各牌种计数尚未在本次运行；不能标成无规则UI。", "暴虐3选2，前两张锦囊/杀均直击且不计，第三张恢复；另选0、取消不扣标记。保留原数字按钮画面对照。");
add("L27", "ui_hooks:twzhouzu", "实际规则：phaseUse限1且无当前施法，选1..3及其他目标；每个全局phaseEnd递减，到期目标自选弃n张默认手牌并受1雷伤，随后移除。", "设target/rem/x后每PhaseEnd倒数；到期取hand_and_equips前x固定弃，不问受害者选牌，再雷伤。", "crates/engine-native-content/src/content/host/step/l27_batch_b_tw.rs:496", "受害者自选弃手牌被改成固定先取he，可能额外弃装备。此条含规则，不能按UI忽略。", "目标2手牌+装备，咒诅2到期应由目标自选两手牌；另手牌不足而有装备，上游默认h不应自动拿装备补。");
add("L27", "skills:new_meibu", "他人phaseUseBegin，活着且其攻击范围含自己、自己有he可弃1；取消无事。授目标止息及清理到phaseUseEnd；弃非杀非锦囊非黑牌时另授到弃牌者距离1。", "选he可取消，条件与条件性range授技存在；止息计数却初始化hp，临时技能标expires_end_of_turn。", "crates/engine-native-content/src/content/host/filtering/common.rs:90;crates/engine-native-content/src/content/host/step/skills_virtual_use.rs:99", "止息首张前即封锁；expires_end_of_turn与phaseUseEnd清理需核真实后续阶段，不能沿用整回合临时技能覆盖。", "目标3体力，自己弃红装备，目标首张杀应能出且到自己距离1；弃黑牌无距离加成，取消无止息；phaseUseEnd之后响应应恢复。l27_new_meibu_batch3.rs:102。");

# L19 末日乱斗。
add("L19", "configs:zuan_brawl", "默认all，可off/standard/random/all/daily，将祖安娱乐模式接入乱斗，需祖安扩展；不是一个规则可选技能。", "Doomsday loader提供身份链与无尽配置函数，原生包配置没有完整乱斗菜单/模式注册的此枚举消费。", "crates/content-native-loader/src/doomsday.rs:1;packages/extensions/doomsday-brawl-upstream-liuli/assets/config.json:1", "乱斗展示/模式接入枚举未实现，仅有部分规则函数。", "依次选择off与daily进入乱斗菜单，检查候选模式和依赖未安装提示；不能用某一无尽技能测试代替模式注册。");
add("L19", "skills:_guandieskill", "主公dieBefore且无jingcheng：来源加25经验、屏蔽非白名单技能、叛军+2，按官兵/叛军阈值结局；取消死亡并恢复记录的tamaxhp/tahp，清hej/横置/翻面，加jingcheng/notarget与亡标识。", "Dying/Death不检查主公/jingcheng，只按guan_phase依次复活8/8授无双咆哮摸4、6/6授技能摸6，再置反军胜旗。", "crates/engine-native-content/src/content/host/step/l19_batch_b.rs:498;crates/engine-native-content/src/content/host/step/l19_batch_b_asks.rs:235", "整套身份/死亡阻止条件与结算被替换成另一组三阶段boss复活。", "非主公持该技濒死不应触发；主公首次dieBefore应回记录体力、清hej并加jingcheng，不能凭空变8体力摸4。");
add("L19", "skills:_huzhu", "phaseUse限1，人类主/忠弃手桃，选距离<=1的其他受伤主/忠，目标使用虚拟桃；卡/目标在发动承诺前可取消。", "选择手桃→近盟友，弃载体后直接Recover，不发目标使用桃事件；目标候选按非zombie且非反/内筛，owner人类门另在active检查。", "crates/engine-native-content/src/content/host/step/l19_batch_a_asks.rs:169;crates/engine-native-content/src/content/host/step/l19_batch_a.rs:162", "直接回复替代target.useCard桃，绕过猴子、用牌限制和用牌历史；人类身份宽泛过滤也需真实角色校对。", "救援目标装备猴子且第三人装备猴子，发动互助应经过桃使用触发；不使用debug效果，检查CardUsed历史与取消。l19_doomsday_match.rs:418。");
add("L19", "skills:_jiangshi2", "player phaseBegin；自己无_tuizhi、主公_tuizhi==2，按人数区间和主公jisuangailv/fzjsNumber计算概率；成功先die，再改忠、fzjs=0、主公fzjsNumber++。", "自己Prepare且current_tuizhi(owner)==2、非zombie/未转化即必定maxhp+5回复3，改反贼并授僵尸技。", "crates/engine-native-content/src/content/host/step/l19_batch_a.rs:298;crates/engine-native-content/src/content/host/step/l19_batch_a.rs:505", "条件检查主公→自己，去掉随机，死亡/身份/获得能力都改变。", "主公计2而普通人计0、设概率0与1各一局；普通人概率0不得变，概率1应走die/忠路径，不能直接加5上限变反。");
add("L19", "skills:_relifeskill", "global roundStart，仅主公处理game.dead：每死人relife减1，到0按其原maxhp复活、摸ceil(level*.5)，重设ceil(level*.4+.1)及护盾；未到0只显示倒数。", "Dying/Death时自己hp<=0且zuan_relife>0即减1，level重设1，弃手、固定4/4复活摸4。", "crates/engine-native-content/src/content/host/step/l19_batch_b.rs:324;crates/engine-native-content/src/content/host/step/l19_batch_b_asks.rs:219", "回合死人倒数被换成自己即时复活，体力、摸牌、等级均不符。", "主公活着、死者level10/relife2/maxhp6，下一轮仅倒数1仍死，再下一轮复活6并摸5；原生濒死已先复活。");
add("L19", "skills:_wujinmoshiRecover", "recoverBefore，配置>0且player==game.me才给本次trigger.num加配置；不能取消，不另起一次回复。", "metadata为PhaseBegin附PhaseEnd/TurnEnded，filter仅自己End开始且受伤；handle另ctx.recover bonus且bonus至少1，不修改任何当前Recover的amount。", "crates/engine-native-content/src/content/host/step/l19_batch_a.rs:224;crates/engine-native-content/src/content/host/step/l19_batch_a.rs:371;crates/engine-native-content/src/content/host/step/l19_batch_a.rs:586", "recoverBefore加量被替换成结束阶段自动额外回复；不会增强其它时点的回复，没发生原回复也会自愈，并去掉game.me/配置>0的原门。", "game.me缺3血、配置+1：出牌阶段用桃应一次Recover(2)，结束阶段没用桃不应无故回复；配置0/非me两反例，检查事件数/生命值。");
add("L19", "skills:chanceIdentity0", "无enable/trigger，供模式链式useSkill：给player.next设置group/identity/_group='yi'再调用next.useSkill(chanceIdentity)。", "该族绑定L19BatchA但无trigger meta、active只接ganran2/_huzhu、handler兜底Flow::Done；旧VM分支也仅写身份slot旗，未改下一座roles/group。", "crates/engine-native-content/src/content/host/step/l19_batch_a.rs:224;crates/engine-native-content/src/content/host/step/l19_batch_a.rs:385;crates/engine-native-content/src/content/host/step/l19_batch_a.rs:596;crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_l19_l20_l27.rs:150", "原本改变下一座身份并续链的规则被当桩，无实际身份/势力迁移。", "模式真实调用0号chanceIdentity0，1号应yi且2号接下一链；比较identity/group而非仅l19旗。");
add("L19", "skills:chanceIdentity10", "无独立触发，给下一人shiyi身份/势力，人数>11才next.useSkill(chanceIdentity11)。", "该族绑定L19BatchA但无trigger meta、active只接ganran2/_huzhu、handler兜底Flow::Done；旧VM分支也仅写身份slot旗，未改下一座roles/group。", "crates/engine-native-content/src/content/host/step/l19_batch_a.rs:224;crates/engine-native-content/src/content/host/step/l19_batch_a.rs:385;crates/engine-native-content/src/content/host/step/l19_batch_a.rs:596;crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_l19_l20_l27.rs:150", "座位对象与人数门控链缺失；loader纯identity_chain返回值未接成对局语义。", "11人与12人局各真实调用10链，比下一座shiyi、再下一座是否改变；不要把纯函数返回字符串当对局结果。");
add("L19", "skills:shishi", "source dieAfter强制，击杀者maxhp+1并回复1，没有选项。", "Death击杀者过滤，maxhp+=1后Recover1；非击杀者不触发。", "crates/engine-native-content/src/content/host/step/skills_recovery.rs:66;crates/engine-native-kernel/src/vocabulary/host/metadata/common.rs:260", "", "真实死亡由自己/别人/无来源三局，只有自己杀人增长；满血增长上限后仍回复1。");
add("L19", "skills:xunmeng", "source damageBegin，实体/视为杀且非_lianhuan传导时+1伤，自己hp>1失1；1血时不失血，无选择。", "Xunmeng修改伤害+1后按hp>1排LoseHp；过滤为杀与非from_chain。", "crates/engine-native-content/src/content/host/step/skills_status.rs:103;crates/engine-native-kernel/src/vocabulary/host/metadata/common.rs:269", "待核验：实体载体不是杀的use_as杀与伤害父链语义需运行对照；基础正常杀分支静态一致。", "2血用杀基础伤害1变2且自己-1；1血不自损，连环不加；再用装备转杀验证effective_kind。");
add("L19", "skills:ganran2", "filter手装备可重铸，content player.recast(cards)；当前上游enable在注释里，没有普通出牌阶段入口，必须按实际调用者确认能否进入。", "原生active与PhasePlayBegin均提供询问，选手装备可取消，弃后摸1。", "crates/engine-native-content/src/content/host/step/l19_batch_a.rs:260;crates/engine-native-content/src/content/host/step/l19_batch_a_asks.rs:139", "原生新增上游未启用的主动/阶段触发；若上游实际无其它调用则不可把其当可发动技能。", "直接进入出牌阶段看按钮/自动询问是否存在；若上游无入口，记录无法真实触发原因，另由模式合法调用验证recast内容。l19_doomsday_match.rs:373。");
add("L19", "skills:wujinmoshiRE1", "无独立触发，由敌人死亡后的_wujinmoshiRE调用；game.me选择确定/取消是否获得额外技能，确定再useSkill(RE2)。不复活。", "注册Dying，自己hp<=0且有RE1时消耗RE1，满血复活、摸3；无奖励技能询问。", "crates/engine-native-content/src/content/host/step/l19_batch_a.rs:213;crates/engine-native-content/src/content/host/step/l19_batch_a_asks.rs:326", "技能含义被替换为一次濒死复活，奖励与取消完全缺失。", "杀死一名敌人，应由me出现奖励技能确认；取消不授技能。再让me濒死，RE1不应变成复活机会。l19_doomsday_match.rs:519当前测的是另义。");

# L20 标记补充：保留原分类，身份转移不能按UI条目略过。
add("L20", "cards:zhengsu_bianzhen", "这里只定义fullskin展示卡，没有type/enable/content、实体牌堆或玩家选择；供整肃变阵展示。", "MarkSupplementCardSpec登记zhengsu_bianzhen的名字/资源，不以普通规则牌使用。", "crates/content-native-loader/src/mark_supplement.rs:77;crates/content-native-loader/src/mark_supplement.rs:304", "", "在整肃任务展示中核卡名/贴图；不向牌堆凭空加一张可用牌，本项不能计成规则牌结算等价。");
add("L20", "configs:xinshadoudizhu", "默认false，兵临城下斗地主修改/界面美化，提示不建议手杀UI开启；无技能选择。", "配置parser保留false默认并接收布尔；玩法与界面是否按此键切换不能由解析推出。", "crates/content-native-loader/src/mark_supplement.rs:296;packages/extensions/mark-supplement-upstream-liuli/assets/config.json:1", "待核验：解析默认一致，尚未比较兵临城下界面实际消费与开关重启效果。", "同一兵临城下桌面分别false/true截图，检查菜单持久化；常规身份局不应因开关改变规则。");
add("L20", "configs:zhijiechifa", "默认true，重启后修改部分武将时机同步手杀；不是纯装饰配置，需要观察条件内技能触发点。", "MarkSupplementConfig读取true默认；规则host自身已登记触发元数据，parser不是切换这些触发的证明。", "crates/content-native-loader/src/mark_supplement.rs:283;packages/extensions/mark-supplement-upstream-liuli/assets/config.json:1", "待核验：需按该开关的实际引用逐技能比对；本次不把保存bool视为时机切换完成，也不据未运行直接降级。", "开关两值重启，取上游受此条件包围的技能比较phaseBegin/对应阶段窗口；核原生是否随设置改变而非始终固定。");
add("L20", "skills:kuanshi", "自己phaseJieshuBegin可选任一角色，取消不授技；选中者到授予者下一phaseZhunbeiBegin/死亡前，首次damageBegin4且num>1取消整次伤害，授予者跳下次摸牌；1伤不消耗保护。", "End开始选目标可取消，kuanshi/kuanshi2联用protected_by；DamageBefore检查amount>1并prevented，清保护后置授予者skip_next_draw；Prepare/死亡清理。", "crates/engine-native-content/src/content/host/step/l20_batch_b_ext.rs:175;crates/engine-native-content/src/content/host/step/l20_batch_b_ext.rs:700;crates/engine-native-content/src/content/host/step/l20_batch_b_ext.rs:748", "待核验：主体条件/询问/一次防止已对到；damageBegin4相对其他伤害修饰的优先级和重复宽释来源尚需真实组合，不能从单一防2伤正例背书。", "宽释A/B、取消三局；先1伤仍保留，再2伤防止且下一摸牌跳过；3伤被别技减为1时不应耗保护；授予者Prepare前后各打一次。l20_mark_supplement_match.rs:821。");
add("L20", "skills:mibei", "useCardAfter强制；按storage.xingqi保存的不同牌名分类，basic/equip/trick各至少2才成功，从堆每类取1并授xinmouli；准备时星旗为空加失败标记，结束仍空则减1上限并使命失败。", "每次CardUsed/Equipped都增加类型cnt和xingqi_count；虽写xingqi_card:name旗，成功判断仍只看次数，未用旗去重；各类取1、授xinmouli/失败减上限有路径。", "crates/engine-native-content/src/content/host/step/l20_batch_b_mouyi.rs:210;crates/engine-native-content/src/content/host/step/l20_batch_b.rs:482;crates/engine-native-content/src/content/host/step/l20_batch_b.rs:617", "同名重复使用会累计成两个不同名，提前成功；CardUsed载体分类还不能代表转化后牌名/类型。", "真实装配星旗/秘备，各用两张同名杀、两张同名无中、重复同名装备；上游每类只记1不成功，原生cnt到2会成功。另用6不同名正例及整回合不用牌失败。");
add("L20", "surfaces:extension.css", "881行样式含飞扬/跋扈等动画与标记定位，定义展示，不自行发动技能或修改状态；没有选择/取消语义。", "原生标记配置与通用效果状态有落点，没有本次逐动画/定位截图。", "apps/web-client/src/engine/extensionSurfaceRuntime.ts:1;crates/content-native-loader/src/mark_supplement.rs:1", "待核验：未做样式逐项或并排截图，不能凭样式文件存在或状态旗称外观一致。", "相同座位/分辨率触发飞扬、跋扈，分别记录起始/中间/结束帧与标记边界；动画关闭检查规则状态不受影响。");
add("L20", "surfaces:group.css", "17行自定义势力选中框，包括jin/key等颜色；仅展示，无触发/消耗/选择结算。", "原生通用势力展示取faction，尚无该文件框色及边框形状逐图证据。", "crates/content-native-loader/src/mark_supplement.rs:1;apps/web-client/src/engine/extensionSurfaceRuntime.ts:1", "待核验：需真实选将画面对照，不能从势力字段相同推定CSS已复刻。", "在相同主题选中不同势力武将，比较颜色/边框/禁用态，保留正常与高对比度截图。");
add("L20", "ui_hooks:_mianshang_", "player damageZero/damageCancelled，强制播放免伤Spine，无数值修改或玩家选项；普通正伤不播。", "登记Damage/DamageZero/DamageCancelled，命中后只写l20_ui_hook旗；尚无可见Spine时序比对。", "crates/engine-native-content/src/content/host/step/l20_batch_a.rs:116;crates/engine-native-content/src/content/host/step/l20_batch_a.rs:535", "待核验：事件落点与视觉播放分开，本次未证明产品按零伤/取消伤害显示一次免伤动画。", "1伤归零、伤害取消、普通1伤三局，前两应播且只播一次，第三不播；截图不能用flag替代。");
add("L20", "ui_hooks:_shanghaipaishiyongtexiao_", "player useCardBegin，伤害标签卡按本人位置/手杀UI等条件播放牌使用动画；不改变目标/伤害，无询问。", "CardUsed钩子落到通用l20_ui_hook旗；无标签和机位差异的独立可见证明。", "crates/engine-native-content/src/content/host/step/l20_batch_a.rs:127;crates/engine-native-content/src/content/host/step/l20_batch_a.rs:535", "待核验：需比实际伤害牌/非伤害牌及本人/他人画面，不能把所有CardUsed落旗当相同可见效果。", "本人杀、本人桃、他人杀三局并排截帧，核动画出现条件/方向及取消用牌不残留。");
add("L20", "ui_hooks:_xigua_zhuwanzhongji_", "dieBegin且死亡者为主公，忠臣/明忠存在时按条件选择继任；实际交换identity、更新game.zhu并zhuUpdate，不只是标记和音效。", "该名字进入L20BatchA通用UI分支，只记录l20_ui_hook:key/总旗，没有改identity或主公指针的结算。", "crates/engine-native-content/src/content/host/step/l20_batch_a.rs:535;crates/engine-native-content/src/content/host/step/l20_batch_a.rs:43", "漏掉主公继任的身份/主公迁移规则；inventory中的ui_hooks分类不能消除此差异。", "主公死亡且有忠/明忠各一局，比较继任者identity和后续主公技/胜负检测；仅出现动画或旗不算继任成功。");
add("L20", "skills:tspowei", "开局/入场给无围的其他人各1围；global damageEnd移除受伤者全部围；自己phaseBegin把每处围数移给下一他人，无围使命成功授神著；他人有围回合可选弃手牌伤1或符合血量条件拿手牌，取消不动；濒死失败回1弃全部装备。", "首次Turn/Prepare懒初始化各他人围旗1，自己TurnBegin轮转bool围；他人回合有行动/牌询问与取消、damage清围、使命/濒死结算路径。", "crates/engine-native-content/src/content/host/step/l20_batch_b_mouyi.rs:13;crates/engine-native-content/src/content/host/step/l20_batch_b_mouyi.rs:82;crates/engine-native-content/src/content/host/step/l20_batch_b.rs:345", "待核验：标准单来源每处围最多1，bool表达在这些分支未见确定差异；多来源初始化会跳过已有标记，不能虚构双来源自然叠成2。开局/phaseBegin/成功结算优先级、合法可叠围组合仍待真实对照。", "分别选弃手牌伤1与符合血量条件拿手牌、取消，核其余流程；只剩一个围移到下一座，无围到本人phaseBegin成功；濒死失败回1弃全装。若能由合法其他技能叠围再测数量，不直接伪造旗。l20_mark_supplement_match.rs:697。");
add("L20", "skills:wengua", "本源覆盖的wengua对象只有audio:2与group:'wengua5'，没有主动enable/content；真实入口是附带的wengua5给其他角色的问卦流程，不能直接调用wengua内容当作上游主动。", "除wengua5外还为wengua提供本回合主动：选自己he，选置顶摸底或置底摸顶，首/第二询问取消均退once。", "crates/engine-native-content/src/content/host/step/l20_batch_a.rs:550;crates/engine-native-content/src/content/host/step/l20_batch_a.rs:155", "为被覆盖后无主动入口的wengua新增独立主动，不能用这个路径的正例证明该来源规则等价。", "正常安装本源，出牌阶段核wengua本体按钮确实不存在；用实际wengua5入口做供牌者/卦师询问、两端摸牌和取消。l20_mark_supplement_match.rs:425测本体主动，范围需标清。");

# U08 英雄杀。
add("U08", "generals:yxs_aijiyanhou", "女、群、3体力，seyou/sheshi，无特殊生命串、标签或定义内选择。", "同名女群3和两技能元数据一致；本项只核装配。", "crates/content-native-loader/src/decision34_pack9_data.rs:158", "", "安装来源后看女群3与色诱/蛇噬技能；技能具体效果不由武将定义背书。");
add("U08", "generals:yxs_napolun", "男、魏、4体力，tongling/fanpu，无定义内问询或特殊初始区。", "原生男魏4及统领/反扑同名技能列表。", "crates/content-native-loader/src/decision34_pack9_data.rs:162", "", "装配检查4上限与两技能，跨包同名绑定另按技能场景核验。");
add("U08", "generals:yxs_wuzetian", "女、吴、4体力，nvquan/qiandu/yxsweiyi；武将本身无额外选项。", "同名女吴4与三个技能列表保留。", "crates/content-native-loader/src/decision34_pack9_data.rs:154", "", "核选将元数据与三技能授予；女权决斗禁止子技能在独立样本列条件。");
add("U08", "skills:bolehuiyan", "global shaBegin，自己非使用者/该目标、目标有手牌且非受控、无本回合bolehuiyan4；可预测能命中/不能命中/取消。确认后记录预测，命中/被闪按相关辅助分支结算并清；取消不耗回合次数。", "CardUsed只看targets.first且排除含自己的整张牌；可选预测/取消，Damage/ShaDodged按攻击者/目标旗判结果，不记录所预测那张杀的frame或card。", "crates/engine-native-content/src/content/host/step/u08_batch_d.rs:607;crates/engine-native-content/src/content/host/step/u08_batch_d_asks.rs:832;crates/engine-native-content/src/content/host/step/u08_batch_d_asks.rs:766", "逐目标shaBegin变成整牌首目标，漏受控目标过滤；预测后同一来源对目标的其他嵌套伤害也能结算该预测，未约束同一次杀。", "杀有A/B时只B有手牌，应可为B预测；预测外层杀不能命中，先由同来源对同目标触发别的嵌套伤害，不应提前结算。另预测两值、取消及同回合第二次无问询。u08_hero_kill_match_b.rs:311。");
add("U08", "skills:cike", "自己shaBegin可选判定，红色令该目标directHit，黑色由使用者discardPlayerCard(target)弃目标he（当前player.js:7216默认he），无色不处理；取消发动不判定。", "确认后OrdinaryJudge，红置杀不可响应标志；非红分支用目标hand_and_equips构造弃牌候选，由owner选择，与当前上游默认he一致。", "crates/engine-native-content/src/content/host/step/u08_batch_d_ext.rs:448;crates/engine-native-content/src/content/host/step/u08_batch_d_asks.rs:473", "待核验：普通红/黑判定与he区域相符，不把可弃装备误报差异；无色判定上游不处理而原生非红走弃牌，该分支可达性及判定改色组合待运行。", "目标手牌/装备各1，黑判可选两区任一，红判直击，取消不翻判定牌；若存在合法无色判定载体再测不弃，不直接伪造judge旗。");
add("U08", "skills:fenghuo", "chooseToUse，以自己装备区1张牌视为南蛮入侵，取已装备载体并走正常用牌/全局目标；取消不移装备，无主动次数限制。", "询问自己equips一张，发CardUsed use_as SavageAttack；targets由nanman_targets筛出，后续共用CardUsed负责搬载体/撤装备技能。", "crates/engine-native-content/src/content/host/step/u08_batch_c_ext.rs:169;crates/engine-native-content/src/content/host/step/u08_batch_c_asks.rs:281", "待核验：主动选择/取消与南蛮视为已对到；chooseToUse中非自己出牌阶段的合法上下文及装备撤技/实体守恒仍需事件比对。", "穿猴子转南蛮，选A/B装备载体消耗不同且对应技能撤销；取消装备仍在；用牌前后记录目标与所有牌区。");
add("U08", "skills:guifu", "phaseUse限1，选有装备的其他角色，由自己弃其装备区1张，然后双方按座序各摸1；取消选人不发动，不是重铸自己装备。", "目标问询；一装备直接弃，多装备再询问；guifu_draw_both按当前座序对owner/target各draw_direct1，首选取消refund_once。", "crates/engine-native-content/src/content/host/step/u08_batch_d_ext.rs:701;crates/engine-native-content/src/content/host/step/u08_batch_d_ext.rs:42", "待核验：基本分支静态相符；弃装备触发尚未结算时两次draw_direct立即取牌，需与上游await弃牌后再asyncDraw的组合时序比较。", "目标武器/防具各1，选不同装备应保留另一张；取消目标无消耗可重开，自己无装备仍可发动。装备离场先摸牌技时比较牌堆/RNG。u08_hero_kill_match_b.rs:431。");
add("U08", "skills:heqin2", "phaseDrawBegin强制使本阶段trigger.num+1，并组heqin3负责配对清理；不另立一次摸牌事件，无玩家取消。", "阶段过滤后draw_direct1，不修改当前Draw阶段预期数；配对另有清理helper。", "crates/engine-native-content/src/content/host/step/u08_batch_b_ext.rs:296;crates/engine-native-content/src/content/host/step/u08_batch_b_ext.rs:1", "额外直接摸1替代本次摸牌数+1，改变摸牌事件次数/时机，与跳摸、固定摸牌数或摸牌后触发组合不同。", "配对角色正常摸牌应一次3张，记录CardsDrawn次数；配合把本阶段数固定为0/替代摸牌的技能，上游不应先独立拿1。");
add("U08", "skills:nvquan3", "纯targetEnabled：男性使用决斗不可指定持技者，女性决斗合法；读取当前player.sex，无技能选择。", "目标修饰对Duel检查nvquan3/nvquan/旗，但先取主将catalog gender，只有无主将定义才回退effective_gender。", "crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_flags.rs:531", "有主将时绕过当前有效性别，变更性别后仍按原将判断，改变组合合法目标。", "给原男将通过现有合法性别变更技能变女，再用决斗指定女权者应合法；原女变男则应禁。无变性男女对照作基线。");
add("U08", "skills:oldbolehuiyan2", "global judgeEnd强制，预测花色全中摸2、仅颜色中摸1、都错不摸，随后删bolehuiyan存储；不在结算时再选或取消。", "resolve_oldbolehuiyan_judge比较保存suit和判定牌，按2/1/0直接摸后清预测；由OrdinaryJudge及兜底事件驱动。", "crates/engine-native-content/src/content/host/step/u08_batch_d_asks.rs:797;crates/engine-native-content/src/content/host/step/u08_batch_d_asks.rs:947", "待核验：三档数值分支静态相符，OrdinaryJudge后改判/嵌套判定时引用的最终判定对象与清理顺序需实跑。", "预红桃，分别判红桃/方片/黑桃为2/1/0；天妒/改判组合必须用最终牌，仅一次结算，不接受PhaseEnd兜底误配其他判定。");
add("U08", "skills:xumou", "phaseJieshuBegin可发动，await摸3完成后await turnOver，取消不摸不翻；翻面经过turnOverBefore，面具可取消。", "确认后draw_direct3并立即改face_down，再只emit TurnOvered；无TurnOverBefore。", "crates/engine-native-content/src/content/host/step/u08_batch_c_asks.rs:749", "直接改面跳过翻面前取消窗口，面具组合必然仍被翻；摸牌子树前后顺序也未按await串行。", "真实装备U06面具，自己结束阶段确认蓄谋应摸3但不翻面；原生会face_down改变。再取消/无面具两局核基线。");
add("U08", "skills:yixin", "phaseUse限定，he>2且有受伤者，弃2张he选受伤目标；回复min(4,lostHp)，摸4-回复量，承诺前可取消；不允许只有2牌发动。", "active检查he>2，牌/目标询问；确认后弃2、置限定，按目标受伤量回复并补摸到总数4，首步取消不耗限定。", "crates/engine-native-content/src/content/host/step/u08_batch_a.rs:633;crates/engine-native-content/src/content/host/step/u08_batch_a_asks.rs:67", "待核验：条件与数值已对到，回复/摸牌真实事件次序、在目标询问取消后的成本/限定恢复待实跑。", "3张he给缺1血A与缺3血B，分别回1摸3和回3摸1；只有2牌不得发动，首/目标取消不弃不耗限定，挂起存档续问。u08_hero_kill_match.rs:403。");
add("U08", "skills:yxs_fanji2", "自己shaBegin强制，仅get.color(event.card)=='red'时directHit，无选择；黑杀仍可闪。", "ShaMachine按载体catalog.suit红桃/方片判红并查技能，跳闪询问；不使用effective_card_suit。", "crates/engine-native-kernel/src/kernel/sha/step.rs:127", "待核验：普通红/黑杀静态可对；带当前有效花色改写的视为杀需确认上游get.color是否跟随，不据实体suit检查单独断定组合错误。", "红杀/黑杀目标各有闪，再用合法花色转换的杀比较是否还能响应；取消不适用，保存闪窗口验证黑杀未误直击。");
add("U08", "skills:yxsciqiu", "source damageBegin1，杀伤害目标当前满血则+1；若增强后num>=其hp，授目标临时dying子技并移除自己刺球，濒死开始立即死亡；非满血不加。", "Damage机器增加1，致命时记录turn.ciqiu_dying并移除技能；engine濒死入口读取该记录强制死亡；CardUsed还预记满血目标旗。", "crates/engine-native-content/src/content/host/step/u08_batch_b_asks.rs:413;crates/engine-native-kernel/src/kernel/dying.rs:51", "待核验：增强与致命链静态存在，damageBegin1后其他减伤是否仍应预置死亡子技、以及临时记录到期需同场景实跑。", "满血4受杀变2但不移刺球；满血2变2移技能并直接死不救援；非满血不增强，减伤/防伤组合核技能移除与死亡标记。");
add("U08", "skills:yxsdili", "phaseDrawBegin且已受伤，trigger.num加min(2,ceil(lostHp/2))，无选择；满血不加。", "同公式算n后draw_direct n，不修改当前Draw阶段num。", "crates/engine-native-content/src/content/host/step/u08_batch_a_asks.rs:326", "本阶段增加摸牌量被拆为提前独立直接摸牌，事件计数/顺序及替代摸牌组合不同。", "缺1/2/3/4血应本次总摸3/3/4/4且一次CardsDrawn；配合替代摸牌/跳摸确认不会先拿额外n。满血2张不额外触发。");

# L25 祖安武将：29项，配置/展示样本仍在分母和表中。
add("L25", "cards:ssr_jinfan", "该位置实际上是甘宁的phaseUse技能：弃黑色或装备牌，取顶2作铃扩展；铃提供杀无距/无防具等后续，非lib.card实体定义。", "ZUAN_CARDS却登记同名黑桃9、Wuzhong牌种，并随来源加载；另有ssr_jinfan主动host，不能证明此额外实体合法。", "crates/content-native-loader/src/zuan_data.rs:353;crates/content-native-loader/src/zuan_data.rs:359;crates/engine-native-content/src/content/host/step/l25_batch_b_ext.rs:310", "inventory把技能计为牌，原生凭空建立上游不存在的同名实体/无中效果；保留原key/category并报告分类错误。", "加载祖安包列全体牌定义，尝试使用黑桃9的ssr_jinfan：上游仅有技能入口，不能摸到/使用同名无中；另由真技能弃黑牌取铃核实体流转。");
add("L25", "configs:bizhi", "菜单壁纸默认字符串1，选项0..13；onclick将0归1、2归3、4归5并保存extension_祖安设置_bizhi，只影响开局前菜单，非回合动态壁纸。", "ZuanConfig/包资产只有四个新键，没有旧bizhi枚举与归一化；前端activeWallpaper是单独对局状态。", "crates/content-native-loader/src/zuan.rs:13;packages/extensions/zuan-upstream-liuli/assets/config.json:1;apps/web-client/src/engine/surfaces/zuanSurface.ts:57", "旧菜单壁纸键与其枚举/保存转换未映射；activeWallpaper不能代替此独立配置。", "新安装默认1不出壁纸，分别点0/2/4重启应保存1/3/5；进入游戏后此菜单壁纸不应持续。");
add("L25", "configs:hidessr", "默认true，隐藏标准武将增强包且AI/玩家均不能选这些SSR；不仅隐藏菜单文字，还控制可选将池。", "原生祖安配置四键无hidessr，ssr武将数据始终随包登记；本配置没有可选将池门。", "crates/content-native-loader/src/zuan.rs:13;crates/content-native-loader/src/zuan_data.rs:1;packages/extensions/zuan-upstream-liuli/assets/config.json:1", "缺原默认隐藏/将池排除，不能当成纯视觉无规则效果。", "清设置分别true/false建身份局，玩家候选和AI分配中是否出现ssr_jiaxu；仅目录可发现与实际可选要分开。");
add("L25", "configs:newhuihe", "默认new_standard，old/new_standard/new_brighten/new_shine四种回合特效，影响_newZuanPhaseTX资源与动画，不改回合规则。", "包配置无此枚举，ZuanUiRuntimeState也无newhuihe；只有通用阶段cue。", "crates/content-native-loader/src/zuan.rs:13;packages/extensions/zuan-upstream-liuli/assets/config.json:1;apps/web-client/src/engine/surfaces/zuanSurface.ts:48", "四种回合效果的配置分支缺失，不能由一个阶段cue判外观等价。", "四值分别进入同一武将回合，记录滑动/强调/居中与旧版的真实动画差别及持久化；无需规则数值测试。");
add("L25", "configs:perspective", "默认false，近大远小调整场上武将牌缩放，上游明确卡牌/特效尚未兼容；无技能选择。", "ZuanConfig四键无perspective，当前祖安UI状态没有相应座位缩放参数。", "crates/content-native-loader/src/zuan.rs:13;apps/web-client/src/engine/surfaces/zuanSurface.ts:48", "该旧配置及对应座位透视参数未映射，不能宣称与上游布局一致。", "6/8人桌面同一分辨率开关两值，测近/远将牌尺寸比、选中框命中区域和特效锚点。");
add("L25", "configs:showboss", "默认false，开启把剑阁/挑战模式武将引入其它模式，改变候选将池，需要实际依赖；非单纯显示旗。", "包配置无showboss，普通武将装配没有按此旧键扩入剑阁/挑战池的分支。", "crates/content-native-loader/src/zuan.rs:13;packages/extensions/zuan-upstream-liuli/assets/config.json:1", "旧配置与跨模式将池扩展缺映射；原生目录已有某些boss不能代替开关门。", "安装相关依赖后false/true分别开身份局，比对挑战/剑阁名单出现；缺依赖要报告实际不可用，不凭静态牌包计数结案。");
add("L25", "configs:text", "默认false，启用能换武将/位置/手牌并看身份的编辑器；是调试规则能力，非文字装饰。", "有另名enable_editor且仅offline_debug_only允许，拒绝非离线启用；parser没有text→enable_editor兼容映射。", "crates/content-native-loader/src/zuan.rs:16;crates/content-native-loader/src/zuan.rs:115;crates/content-native-loader/src/zuan.rs:200", "旧键不映射且编辑器受新的离线边界限制；这里登记实际产品边界，不能称无规则效果或旧配置原样等价。", "导入text:true旧设置，检查是否出现编辑工具；再用enable_editor:true离线合法入口改手牌/位置和看身份，并单列联网拒绝行为。");
add("L25", "configs:windowHigh", "默认字符串0，可见item只有0，旧1..7已注释；onclick仍保存zuanwindow并按旧值调用moveTo/resizeTo，移动端可能无效。", "原生包配置未保留windowHigh与zuanwindow，窗口大小由宿主管理，祖安状态中无此旧枚举。", "crates/content-native-loader/src/zuan.rs:13;packages/extensions/zuan-upstream-liuli/assets/config.json:1", "旧设置持久化和历史值兼容分支未映射；不能虚构当前菜单仍有1..7选项。", "先核当前菜单仅0；再加载旧3验证桌面1280×720兼容行为与浏览器限制原因，移动端如不支持应单列。");
add("L25", "generals:ssr_jiaxu", "男群3，newweimu/guimou/newluanwu，普通体力数；定义无额外选择。", "原生同名男群3、三技能列表一致。", "crates/content-native-loader/src/zuan_data.rs:78", "", "选将装配核3上限/技能，单独检查跨包同名绑定；不从这个表推定乱武等规则已复核。");
add("L25", "generals:ssr_lusu", "男吴3，dimeng/fuhui；无特殊护甲或体力串。", "同名男吴3及缔盟/福惠技能元数据一致。", "crates/content-native-loader/src/zuan_data.rs:120", "", "清旧角色状态装配，核势力/性别/初HP3和两技能，不把初始属性核对扩成两技能等价。");
add("L25", "generals:ssr_taishici", "男吴，生命串2/4/2，初HP2/上限4/护甲2，kuiwei/newhanzhan；不能按普通4HP解读。", "zuan_data将上限4登记，general_assembly专门给该symbol设置初HP2、护甲2，保留生命串效果。", "crates/content-native-loader/src/zuan_data.rs:90;crates/engine-native-kernel/src/state/general_assembly.rs:12;crates/engine-native-kernel/src/state/general_assembly.rs:35", "", "全新正常装配应2/4且甲2；与仅看目录max_hp4区分，换将/重新装配检查不会重复加甲。");
add("L25", "generals:ssr_yuanshao", "男群4，newluanji/newdingsheng/newshiyi，原定义带zhu标签可入主公候选。", "男群4与三技能表一致，基本元数据表未直接携zhu标签；候选池另有角色选择规则。", "crates/content-native-loader/src/zuan_data.rs:104", "待核验：只核到基本元数据，主公候选是否另由模型白名单保留须核实际选将，未见此表字段不直接判缺失。", "身份模式主公选将中核ssr_yuanshao的候选资格，普通座位也应可用；三技能按定义装配。");
add("L25", "generals:ssr_zhangliang", "男群3，renhong/gaiming/fangtong，无定义内生命/护甲特例。", "同名男群3和三技能列表一致。", "crates/content-native-loader/src/zuan_data.rs:128", "", "正常装配核初体力3、势力与三技能；目标/改判语义另需对应技能样本，不能由本项签署。");
add("L25", "generals:ssr_zuoci", "男群3，shendao/newjiahe，无额外可选定义或特殊生命串。", "同名男群3与神道/假和技能列表一致。", "crates/content-native-loader/src/zuan_data.rs:44", "", "正常装配核性别势力/初HP3/两技能；跨包复用实现的规则范围不包含在元数据结论。");
add("L25", "skills:_bingletexiao1", "自己dieBegin强制清hpunshow/thunderunshow/huoshanunshow等判定覆盖DOM；不改HP或死亡结算，无玩家询问。", "名字被is_l25_ui_hook接入通用分支，只加计数；默认元数据PhaseBegin并附TurnBegin/CardUsed/Damage，不对应dieBegin清覆盖。", "crates/engine-native-content/src/content/host/step/l25_batch_a_ext.rs:11;crates/engine-native-content/src/content/host/step/l25_batch_a_ext.rs:97;crates/engine-native-content/src/content/host/step/l25_batch_a_ext.rs:337", "死亡前清理生命周期变成默认事件计数，未表达指定覆盖层移除；此技能实际为展示，不能用触发旗当画面证明。", "角色带低血/雷/火山覆盖后死亡，dieBegin就应清除，旁人覆盖不受影响；结束后不残留，存活阶段不误清。");
add("L25", "skills:_bingletexiao2", "自己loseEnd且失去至少一张原区j的牌，先清旧判定覆盖，再按仍在j的灾害/低血状态重建；非判定区失牌不处理，没有问询。", "同样默认UI触发/通用计数，没有读失牌原区域，也没有余下判定牌的重建计算。", "crates/engine-native-content/src/content/host/step/l25_batch_a_ext.rs:11;crates/engine-native-content/src/content/host/step/l25_batch_a_ext.rs:97;crates/engine-native-content/src/content/host/step/l25_batch_a_ext.rs:337", "缺原区j条件和按残留判定状态重建，不是一个无条件特效cue可替代。", "判定区同时雷/乐，移除雷后只清雷而乐覆盖保留；丢手牌不重建；随后移除乐全部清除并截图。");
add("L25", "skills:_fromsay", "global useCard，zuantype==old、ransay非0且他人用牌；按性别、敌友、牌名等分支随机语音，旧版专用，无玩家选项。", "通用UI hook按owner相关CardUsed/其他兜底事件计数，不计算old/ransay、敌友/牌名语音池或随机选择。", "crates/engine-native-content/src/content/host/step/l25_batch_a_ext.rs:97;crates/engine-native-content/src/content/host/step/l25_batch_a_ext.rs:337", "事件门/旧版配置和随机语音规则缺失；计数不是原语音选择实现。", "old+raysay开时友/敌女性分别用杀/桃比语音对象与池；本人用牌、new版本和关闭ransay都不应播，记录RNG消费。");
add("L25", "skills:_guanmodeskill", "identity模式主公enterGame/global gameDrawAfter强制授guanfu、记录原maxhp、tamaxhp=Infinity/tahp，公开官府/叛身份；内奸另授guifu_nei，无玩家取消。", "下划线通用UI分类，步进只更新l25_ui_hook/l25_vfx旗，不授guanfu或guifu_nei，也不建立官府模式记录。", "crates/engine-native-content/src/content/host/step/l25_batch_a_ext.rs:11;crates/engine-native-content/src/content/host/step/l25_batch_a_ext.rs:337", "含实质规则的模式初始化被当展示桩；缺授技与模式生命/身份记录。", "官府模式正常开局主公应获guanfu且原HP记录，内奸获guifu_nei；非主公/非identity不触发。检查后续官府死亡流程而非只flag。");
add("L25", "skills:_newZuanPhaseTX", "自己phaseBegin，依newhuihe是否含new_、武将名/资源目录及本人显示条件播放对应回合背景动画；无规则数值和取消。", "名字落默认UI钩子和计数；没有newhuihe枚举、将名资源筛选及三种动画分派。", "crates/engine-native-content/src/content/host/step/l25_batch_a_ext.rs:97;crates/engine-native-content/src/content/host/step/l25_batch_a_ext.rs:337;apps/web-client/src/engine/surfaces/zuanSurface.ts:48", "仅计数，缺该事件的配置/资源分派；可见效果还需截图，不能与规则技能同记等价。", "known将+new_standard/new_brighten/new_shine各进入一次回合，old与缺资源将不播新背景；比较结束后释放与覆盖。");
add("L25", "skills:_zuanCHANGEHPsayskill", "global recoverAfter/damageEnd/loseHpEnd/drawAfter/dieAfter，wanqing/zuantype/new_chg、受控状态、冷却与随机门；按事件量、敌友/性别播放说话，无HP改动。", "只为LoseHp加PhaseBegin元数据，通用handler计数；未实现其它四类事件、冷却和语音选择门。", "crates/engine-native-content/src/content/host/step/l25_batch_a_ext.rs:73;crates/engine-native-content/src/content/host/step/l25_batch_a_ext.rs:337", "触发范围缩错且语音条件/随机/冷却缺失；不是简单“无规则效果”即可签署。", "同角色分别受伤、失血、回复、摸牌、死亡，开启/关闭new_chg及短间隔重复比对实际语音与冷却，静态表仅作为预期。");
add("L25", "skills:_zuan_damageSendEgg", "global damageEnd，emotion开、非本人控制，友好受害者且其当前回合，随机20%向来源掷egg/shoe；无伤害修改或选择。", "Damage钩子仅owner为伤害来源/目标时可入通用计数，未表达友好/当前回合/emotion/20%及蛋鞋类型随机。", "crates/engine-native-content/src/content/host/step/l25_batch_a_ext.rs:62;crates/engine-native-content/src/content/host/step/l25_batch_a_ext.rs:337", "门控与随机表情缺失，局外手动发一个cue不能代替自动受伤反应。", "emotion开且友好受害者当前回合，以固定种子多次受伤记录蛋/鞋与概率流；敌方受伤、自己控制和别人的回合均不播。");
add("L25", "skills:_zuanidentityshowskill", "phaseBefore firstDo，自己非game.me且storage.Hidentity；按Sidentity、zhuAtt、邻座与随机实际重写identity，并在公开阈值清Hidentity，非简单身份贴图。", "下划线被通用UI接走，未读取这些存储或变更GameMode::Identity roles，仅计数。", "crates/engine-native-content/src/content/host/step/l25_batch_a_ext.rs:11;crates/engine-native-content/src/content/host/step/l25_batch_a_ext.rs:337", "动态身份规则完全落成展示桩；身份用于胜负/阵营判断，属于规则差异。", "非me角色Hidentity=true/Sidentity=nei，phaseBefore后identity必为nei；me与Hidentity=false不变；再固定随机源比较忠/反分支，核实际roles。");
add("L25", "ui_hooks:_extraturnoverskill", "上游触发名实际拼作trunOverBefore，audio>=4时播turnover音效；不能擅自改成turnOverBefore后称源规则如此。", "native登记TurnOvered/TurnOverBefore/PhaseBegin，前端dispatch任意调用均生成sfx:_extraturnoverskill@seat；未读audio门。", "crates/engine-native-content/src/content/host/step/l25_batch_a_ext.rs:82;apps/web-client/src/engine/surfaces/zuanSurface.ts:113", "原生修正了源拼写并扩大触发，且缺audio>=4条件；应登记实际源行为差异，不能虚构上游必播。", "正常turnOver前后实际看上游是否因拼错不触发；audio3/4各跑，另记录源手动合法emit该拼写事件是否播。native需保留其更正边界。");
add("L25", "ui_hooks:_haoyunlai", "phaseDrawBegin，先音效；game.me且goodluck开启、堆>1时，取前至多20张按get.value分层随机选优牌移顶，会实际改牌序；不是只闪光。", "前端dispatch只设luckyDrawShimmer/lastAudioCue；下划线host通用分支不访问牌堆。", "apps/web-client/src/engine/surfaces/zuanSurface.ts:159;crates/engine-native-content/src/content/host/step/l25_batch_a_ext.rs:337", "牌堆搜索/随机择优置顶规则缺失，被错误当纯展示。", "goodluck开，顶牌低价值且前20含高价值桃/无中，进入摸牌应按源分层随机移牌；关配置/非me/堆1不重排。比较完整deck与RNG，而非shimmer旗。");
add("L25", "ui_hooks:_zuanattacklinecheck", "gameDrawBefore/enterGame且player==game.me，读取extension_祖安设置_zhishixian并同步config、十周年UI和game.linexy；moren还原原线，其余按所选函数，不修改对局数值。", "dispatch总返回attackLineStyle='guanjie'；未读取zhishixian所选类型，host走默认事件计数。", "apps/web-client/src/engine/surfaces/zuanSurface.ts:101;crates/engine-native-content/src/content/host/step/l25_batch_a_ext.rs:97", "配置同步被固定为关节线，缺本人/开局事件门和按所选类型切换线函数；透明度不是这个源钩子的要求，不据它判差异。", "入场前选moren或另一有效线型，由game.me入场后实际linexy应匹配配置；他人入场不得覆盖本人线设置。截图线型，不能只看attackLineStyle字段。");
add("L25", "ui_hooks:_zuanboss", "dieBegin，exwallpaper开且boss模式死者为主公时递增zuanboss并依阶段换壁纸覆盖；其它死亡/模式不生效，无玩家选择。", "dispatch任意调用只置bossDefeatedBanner=true，缺模式/主公/开关门、阶段计数与壁纸分派。", "apps/web-client/src/engine/surfaces/zuanSurface.ts:135;crates/engine-native-content/src/content/host/step/l25_batch_a_ext.rs:97", "阶段壁纸被替换成无条件旗/横幅，不保留原生命周期与配置条件。", "boss主公连续阶段死亡各核背景和计数；exwallpaper关、普通身份主公死、boss普通人死都不变。截图前后壁纸而非banner布尔。");
add("L25", "skills:wumou", "phaseJieshuBegin强制，本回合未跳过出牌阶段且出牌阶段用过普通锦囊才付一次；有暴怒必选移1标记或失1HP，无标记直接失1；不是每张锦囊收费。", "metadata主CardUsed，filter普通锦囊即触发；每张在Act立刻要求暴怒/失血，未等结束阶段，也不限制isPhaseUsing。", "crates/engine-native-content/src/content/host/step/l25_batch_a.rs:244;crates/engine-native-content/src/content/host/step/l25_batch_a.rs:480;crates/engine-native-content/src/content/host/step/l25_batch_a_ext.rs:373", "时机与次数完全不同，多锦囊会多付；出牌阶段外用锦囊也可能误收，当前定向测试本身预期即时扣费。", "本回合连用2普通锦囊，结束前不应付，结束时只付一次；有暴怒选A/B不同，跳出牌/只在回合外用无懈不付。l25_zuan_match.rs:1029现测即时。");
add("L25", "skills:new_guixin", "damageEnd可选，每伤害点一轮；每轮先选h/e/j优先区，对每名他人从第一非空区随机得1；得到>2才turnOver，未超过不翻；下一点再确认，可取消剩余轮。", "每轮只问确认，固定取每人hej_cards首张；始终直接切face_down并只发TurnOvered；可按剩余点数继续确认。", "crates/engine-native-content/src/content/host/step/l25_batch_a_ext.rs:440", "缺区域优先选择与随机取牌，得0/1/2张也翻面，且跳过turnOverBefore；面具组合无效。", "只1名他人有2手牌+装备，选优先e应拿装备且不翻面；3人可拿时才翻，面具可取消翻；两点伤第二轮取消不再取。l25_zuan_match.rs:1055。");
add("L25", "skills:gongxin", "phaseUse限2，目标他人有手牌且无gongxin_temp；先加临时目标锁，再在展示手牌中选红桃+获得/封红，允许取消；获得入自己手，封红授ssr_gongxin_break而牌仍归目标。", "当前实际绑定共用Gongxin，真实选人→选红桃→Tag两项；limits once_per_turn，Tag0弃红桃、Tag1放堆顶，没有目标临时锁/封红。", "crates/content-native-loader/src/host_skill_bindings.rs:87;crates/engine-native-content/src/content/host.rs:338;crates/engine-native-content/src/content/host/step/skills_cards.rs:240", "复用了另一版攻心：次数1≠2，弃/置顶≠获得/封红；已有询问不能消除结算不符，旧VM预置旗不是当前执行路径。", "真实发动选红桃A，获得后应到自己手；另选B封红应仍在目标手但其红牌被限制；能对另一目标用第二次，原目标临时锁不可再选。旧decision34_round4_l25_zuan.rs:252走debug不能作当前路径证明。");

# These entries describe an unexecuted branch or a missing screenshot, not a proven mismatch.
GAPS = {
    ("U06", "cards:kuwu"),
    ("U07", "generals:wdb_zaozhirenjun"),
    ("L22", "configs:timeKiller"),
    ("L29", "configs:shengjian"),
    ("L29", "skills:mbcmjiushi"),
    ("L29", "skills:mbcmqingzheng"),
    ("L29", "surfaces:main1.css"),
    ("L29", "surfaces:main1"),
    ("L29", "ui_hooks:_CMeffects"),
    ("L10", "configs:cardEmoji"),
    ("L10", "configs:skillEffects"),
    ("L28", "configs:timeTheme"),
    ("L28", "skills:th_jieshu"),
    ("L28", "skills:th_tiansuan2_3"),
    ("L28", "skills:th_yeyan"),
    ("L27", "configs:dianjibiaoji"),
    ("L27", "configs:shuzianniu"),
    ("L27", "ui_hooks:sbqingzheng"),
    ("L27", "ui_hooks:twxiafeng"),
    ("L19", "skills:xunmeng"),
    ("L20", "configs:xinshadoudizhu"),
    ("L20", "configs:zhijiechifa"),
    ("L20", "skills:kuanshi"),
    ("L20", "surfaces:extension.css"),
    ("L20", "surfaces:group.css"),
    ("L20", "ui_hooks:_mianshang_"),
    ("L20", "ui_hooks:_shanghaipaishiyongtexiao_"),
    ("L20", "skills:tspowei"),
    ("U08", "skills:cike"),
    ("U08", "skills:fenghuo"),
    ("U08", "skills:guifu"),
    ("U08", "skills:oldbolehuiyan2"),
    ("U08", "skills:yixin"),
    ("U08", "skills:yxs_fanji2"),
    ("U08", "skills:yxsciqiu"),
    ("L25", "generals:ssr_yuanshao"),
}
for key in GAPS:
    row = rows[key]
    row["verification_gap"] = row["finding"].removeprefix("待核验：")
    row["finding"] = ""

# Inventory provenance is retained verbatim; actual review references may be more precise.
reviewed_refs = {
    ("L29", "skills:mbcmqingzheng"): ["archive/upstream/noname-liuli/noname/character/mobile.js:17117"],
    ("L29", "skills:mbcmjiushi"): ["archive/upstream/noname-liuli/noname/character/mobile.js:17251"],
    ("L29", "skills:mbcmfangzhu"): ["archive/upstream/noname-liuli/noname/character/mobile.js:17410"],
    ("L29", "skills:mbjuejin"): ["archive/upstream/noname-liuli/noname/character/mobile.js:17510"],
    ("L29", "skills:mbqianlong"): ["archive/upstream/noname-liuli/noname/extension/龙血玄黄/extension.js:1"],
    ("L29", "ui_hooks:_CMeffects"): ["archive/upstream/noname-liuli/noname/extension/龙血玄黄/extension.js:1"],
    ("L20", "skills:mibei"): ["archive/upstream/noname-liuli/noname/extension/标记补充/extension.js:4592", "archive/upstream/noname-liuli/noname/character/shiji.js:2406"],
    ("U08", "skills:cike"): ["archive/upstream/noname/apps/core/extension/英雄杀/character/skill.js:3001", "archive/upstream/noname/apps/core/noname/library/element/player.js:7216"],
}
P1 = {
    ("U07", "skills:choudu"), ("U07", "skills:fuchou"), ("U07", "skills:zhucheng2"),
    ("L27", "skills:new_zhixi"), ("L27", "skills:new_meibu"),
    ("L19", "skills:_guandieskill"), ("L19", "skills:_jiangshi2"),
    ("L19", "skills:_relifeskill"), ("L19", "skills:wujinmoshiRE1"),
    ("L20", "skills:mibei"), ("L20", "ui_hooks:_xigua_zhuwanzhongji_"),
    ("L25", "cards:ssr_jinfan"), ("L25", "skills:_guanmodeskill"),
    ("L25", "skills:_zuanidentityshowskill"), ("L25", "ui_hooks:_haoyunlai"),
    ("L25", "skills:wumou"), ("L25", "skills:new_guixin"), ("L25", "skills:gongxin"),
}
RULE_UI = {("L27", "sbqingzheng"), ("L27", "twxiafeng"), ("L27", "twzhouzu"),
           ("L20", "_xigua_zhuwanzhongji_"), ("L25", "_haoyunlai")}


def resolve(ref):
    name = re.sub(r":\d+(?:-\d+)?$", "", ref)
    return ROOT.parent / name if name.startswith(("archive/", "noname-next/")) else ROOT / name


def main():
    pointers = json.loads((OUT / "inventory-source-pointers.json").read_text())
    candidates = json.loads((OUT / "interactive-candidates.json").read_text())
    expected = {i["key"] for pool in selected["sources"].values()
                for i in pool["random_samples"] + pool["interactive_supplements"]}
    assert len(expected) == len(rows) == 141
    assert expected == {r["key"] for r in rows.values()}
    summaries = {}
    hashes = {}
    for source, pool in selected["sources"].items():
        all_keys = sorted(i["key"] for i in pointers[source])
        n = min(len(all_keys), max(10, math.ceil(len(all_keys) * .1)))
        assert set(random.Random(2026111301).sample(all_keys, n)) == {i["key"] for i in pool["random_samples"]}
        assert len(all_keys) == pool["total_inventory_items"]
        sample_rows = [r for (s, _), r in rows.items() if s == source]
        interactive = candidates[source]["candidates"]
        assert all(c["key"] in expected for c in interactive)
        assert len(interactive) >= 3 or source in ("L06", "L22")
        assert source != "L06" or len(interactive) == 2
        assert source != "L22" or len(interactive) == 0
        for row in sample_rows:
            key = (source, row["category"] + ":" + row["id"])
            confirmed = bool(row["finding"])
            row["difference_confirmed_by_source"] = confirmed
            row["verdict"] = "源码确认差异，待真实运行复现" if confirmed else ("分支/画面待核验" if row["verification_gap"] else "静态范围未见差异，仍待运行对照")
            row["status_boundary"] = "简化近似" if confirmed else "已有原生落点"
            row["severity"] = ("P1" if key in P1 else "P3" if row["category"] in ("configs", "surfaces") else "P2") if confirmed else None
            row["upstream_refs_reviewed"] = reviewed_refs.get(key, [row["upstream_ref"]])
            row["interactive_candidate"] = any(c["key"] == row["key"] for c in interactive)
            row["functional_scope"] = "含规则，不能按界面钩子略过" if (source, row["id"]) in RULE_UI else row["category"]
            for ref in row["native_refs"] + row["upstream_refs_reviewed"]:
                path = resolve(ref)
                assert path.exists(), (row["key"], ref)
                suffix = re.search(r":(\d+)(?:-(\d+))?$", ref)
                if suffix:
                    limit = len(path.read_text().splitlines())
                    assert int(suffix.group(2) or suffix.group(1)) <= limit, (row["key"], ref, limit)
                hashes[str(path.relative_to(ROOT.parent))] = hashlib.sha256(path.read_bytes()).hexdigest()
        counts = Counter(r["status_boundary"] for r in sample_rows)
        summaries[source] = {
            "total_inventory_items": len(all_keys), "random_sample_count": n,
            "interactive_supplement_count": len(pool["interactive_supplements"]),
            "reviewed_sample_count": len(sample_rows), "interactive_candidate_count": len(interactive),
            "insufficiency": candidates[source]["insufficiency"],
            "status_counts_in_sample": {s: counts[s] for s in ["已有原生落点", "简化近似", "已复核等价", "缺失"]},
            "confirmed_difference_count": sum(r["difference_confirmed_by_source"] for r in sample_rows),
            "verification_gap_count": sum(bool(r["verification_gap"]) for r in sample_rows),
            "batch_disposition": "源码已确认差异，该来源不能签署通过；待主agent合并实跑结果" if counts["简化近似"] else "待实际触发比对",
        }
    all_rows = list(rows.values())
    report = {
        "date": "2026-11-13", "head": selected["head"], "worktree_dirty": True,
        "versions": {"rules": 333, "ngsa": 107, "content_schema": 5, "abi": 759, "ws": 8},
        "reviewer": "/root/fix_dddanzhi", "independence_scope": "只审未参与实现的12来源，不审U01；不读原实现者报告结论。",
        "scope": "独立静态源码复核；所有测试设计尚未由本reviewer执行，浏览器/原生同场景对照由主agent另存。",
        "seed": 2026111301, "sampling_algorithm": selected["algorithm"],
        "total_inventory_items": sum(p["total_inventory_items"] for p in selected["sources"].values()),
        "random_sample_count": sum(len(p["random_samples"]) for p in selected["sources"].values()),
        "interactive_supplement_count": sum(len(p["interactive_supplements"]) for p in selected["sources"].values()),
        "reviewed_sample_count": len(all_rows), "source_summaries": summaries,
        "status_counts_in_sample": dict(Counter(r["status_boundary"] for r in all_rows)),
        "confirmed_difference_count": sum(r["difference_confirmed_by_source"] for r in all_rows),
        "verification_gap_count": sum(bool(r["verification_gap"]) for r in all_rows),
        "executed_browser_count_by_this_reviewer": 0, "executed_native_test_count_by_this_reviewer": 0,
        "accepted_equivalent_count": 0, "samples": all_rows,
    }
    assert report["total_inventory_items"] == 765
    assert report["random_sample_count"] == 131 and report["interactive_supplement_count"] == 10
    assert len({r["upstream_checked"] for r in all_rows}) == len(all_rows)
    (OUT / "findings.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    (OUT / "source-hashes.json").write_text(json.dumps(hashes, ensure_ascii=False, indent=2, sort_keys=True) + "\n")
    md = ["# 12来源独立静态逐项表", "", "2026-11-13；HEAD `" + selected["head"] + "`；dirty。规则333 / NGSA107 / schema5 / ABI759 / WS8。", "", "141项保留原inventory key/category。以下均为源码比对与待执行设计，不是浏览器或Cargo实跑；本表0项因此上调已复核等价。", ""]
    for source, summary in summaries.items():
        md += ["## " + source, "", f"来源{summary['total_inventory_items']}项；随机{summary['random_sample_count']} + 交互补样{summary['interactive_supplement_count']} = 实审{summary['reviewed_sample_count']}项。确认差异{summary['confirmed_difference_count']}；待核验{summary['verification_gap_count']}。", ""]
        for row in (r for r in all_rows if r["source_id"] == source):
            md += ["### " + row["category"] + ":" + row["id"], "", "原key：`" + row["key"] + "`。", "", "上游：" + row["upstream_checked"], "", "原生：" + row["native_checked"], "", "结论：" + (row["finding"] or row["verification_gap"] or row["verdict"]) + "（" + row["status_boundary"] + "；不据待核验单独降级）。", "", "待执行：" + row["reproduction_or_next_check"], "", "源码：" + "；".join("`" + x + "`" for x in row["upstream_refs_reviewed"] + row["native_refs"]) + "。", ""]
    (OUT / "samples.md").write_text("\n".join(md))
    print(json.dumps({k: report[k] for k in ["reviewed_sample_count", "confirmed_difference_count", "verification_gap_count", "status_counts_in_sample", "accepted_equivalent_count"]}, ensure_ascii=False))


if __name__ == "__main__":
    main()
