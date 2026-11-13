# L28 源码核对点（2026-11-14，运行输入冻结前）

HEAD `5c73d1d9158145e33f98f2018013914667d13964`，333/107/5/759/WS8。此时采集者已告知原lib.init.onfree延迟追加牌，正在保留旧轮重采；本记录未读采集者README、结论、比较器或可变运行结果。以下由上游、实际原生调用链和内核源码独立取得，实际运行结论要等最终manifest。

补叙 th_buxu（上游显示名是“补叙”）：源码extension.js第6372行的主动条件为he张数≥本阶段使用次数+1、承业实体扩展牌少于6。选择费用是he且可先取消。进入本地content后六个按钮只为尚缺六经出现，按th_chengye.liujing_filter对应伤害锦囊/基本/无懈/无中/乐不思蜀/装备检索。原get.cardPile在game.js第63172行先查整个牌堆，再查弃牌堆；不是无条件牌堆顶。找不到时仍付费用但回退这次stat次数。非本地分支随机选缺项，定时器也会随机选缺项。原生HostBehavior由host_skill_bindings.rs第509行附近明确映射L28ThunderBatchA；激活在l28_batch_a.rs第558行检查mask和递增费用。l28_batch_a_asks.rs第702行步骤弃费用后让玩家选缺失Tag，但第736行附近无条件deck.pop，按所选Tag置mask。待核实际A/B牌面、缺项但无该类牌、弃牌堆有该类牌、第二次费用、取消和恢复。费用首窗可取消，分类窗不可取消由build_ask决定，不能仅看步骤函数的默认分支推断能取消。

整经 th_zhengjing：extension.js第6018行的filter首次就调用dcdAnim.loadSpine；若实际入口因依赖被阻塞，应保留错误并仅做逐行替代，不能置zhengjingxiao旗、伪造小游戏结果或加载Spine库绕过。原content从第6043行附近按牌名互异且非毒取牌，数量受斗地主/其它模式和原随机分支约束，再进入真实切水果/炸弹小游戏；第475–548行显示只有已切完牌名进入结果，无成功时仍为bool:false而结束。成功后强制选目标，再用chooseToMove自由分配，至少留一张作“经”，并非必然第一张作经。原生l28_batch_a_asks.rs第581行把小游戏改成0..5的可取消count询问，正count取前N张，目标窗后固定第一张th_jing、其余自己得；0走boom旗、移顶牌到th_jing再摸1。待核实际源入口是否阻塞，以及原生count、目标窗NGSA是否完整恢复；不能把原生0/3选择称为上游真实同场景小游戏完成。关联th_zhengjing2的准备阶段得经/跳判定摸牌另行检查，不因主技能差别自动为辅助项定级。

业炎 th_yeyan：extension.js第8636行为限定主动且forceDie。真实本地UI有三个火焰拖动到任意角色（包括自己），可留未分配火焰；同一角色≥2火要求四种花色手牌。第8889行后先按座次排序目标，弃牌，window.getStrength(true,false)为真才失3血，然后awaken；下一步骤依目标上的实际火焰数依次伤害。game.js第249行是真实getStrength：strengthen_skill='forbid'或含&off_strength时取from=true；默认null时取to=false，因此默认强化开与关闭时HP费用不同。原生l28_batch_b_asks.rs第327行把分配改为小/大模式、目标、四牌分段；大模式一目标固定3伤、两目标按选择顺序2+1，且无条件发LoseHp3。需核源“只给一人2火”表达及配置维度。第399行后先emit LoseHp再按座次emit多次Damage，而Ctx.emit与resolution::push_frame都直接push栈，resolution.rs第632行推进last，存在实际结算逆序的源码依据，必须以全事件/日志核对而不是最终HP。

业炎取消需经过真实公用入口：l28_batch_b.rs第62行ABORT_ON_CANCEL_ASKS包含mode、targets、discard；第343行handle在空答时先结束并refund_once_used。因此不能只读step_th_yeyan的TargetChoice分支，误报“取消仍消耗限定”或“弃牌取消永远重新问”。恢复询问由现有ask_l28_batch_b调用同一build_ask派生，后续仍须实际NGSA复核。

未验边界：上游完整event/RNG及Spine小游戏不能由静态一致代替；牌名映射和未映射实体、延迟加牌、控座切换、费用保护、有效花色修改、死亡/濒死顺序和自然回合次数刷新要据实际cohort分开写。此时未运行Cargo或浏览器，没有对任何项作新等价签署。


## 2026-11-14 源码补充（运行输入冻结前）

前段“本阶段使用次数”表述收窄为实际 `getStat('skill')` 当前统计段：game.js:28929 取 stat 最后一段，正常回合入口在 game.js:15872、34557 为各角色追加空 card/skill 统计。原生 l28_batch_a.rs:197–211 以 round/current 的 turn_stamp 计算递增费用；本次尚未用真实额外阶段/回合实跑，因此不据先前口语表述宣称刷新时机不同。

原版 game.js:41695 的子事件队列使用 `event.next.shift()`；上游业炎 extension.js:8911 附近的按座次 damage 调用依此逐个执行。原生 Ctx.emit（machine.rs:403）与 push_frame（resolution.rs:130）都直接 push，主推进读取 stack.last（resolution.rs:632）；是否产生倒序以及LoseHp延后，必须从冻结后的实际事件序列核算，不能仅凭两边HP终态相同通过。

已核正式包调用链：content-native-loader/src/locked_content.rs:33 的 bridge_single_frozen 先桥接冻结包；第109行附近调用 decision34_pack9::attach_formal_package_upstream_rules。后者第729行的 upstream-liuli/thunder 分支添加真实太平要术以及宿主技能/武将。源码下 package.npack.json 是旧生成模板（schema3/ABI753），此模板不等于当前运行协议版本；运行是否走正式冻结包和实际版本，须核probe命令与原生锁。未新增或伪造任何CardDef。

原生的业炎目标窗使用 alive_all，包含自己；小业炎最多三人、大业炎最多两人，三窗都允许取消。实际取消从 l28_batch_b.rs:343 的公用入口退还使用次数并结束，不能根据步骤函数的空答默认分支误判。整经首问为0..5 Tag且可取消，目标窗强制；补叙费用首窗可取消，分类窗强制。恢复派生由各 ask_l28_batch_* 重用 build_ask；实际NGSA/应答续行仍待冻结输入，不把源码派生相同等同于已经实测。

本补充只核源码和包数据。当前未运行L28 Cargo/浏览器，未读取采集者README、比较器、结论或早轮结果。已知真实onfree补牌污染的旧轮不会作为最终跨端终态比较输入。
