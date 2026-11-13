# U07 三技能原始运行与源码独立复核

复核日期：2026-11-14（Asia/Taipei）。复核者：/root/queue0_independent_review。HEAD：605201f3f49e10cea61c31d52a61bbb66c85599d，工作区 dirty；规则 333、NGSA 107、content schema 5、ABI 759、WS8。本目录只新增独立证据，未修改产品、测试、清单、共享文档或索引，未提交。未读取采样者 README 或其比较结论作为依据。

结论：最终 24 个 case 的指定初态投影全部相同；终态投影 12 同、12 异。筹度、复仇及诱战均有可定位反例，不能签整技能等价。复仇子技能 fuchou2 的实际清算也有反例；不能因为主技能已经下调就保留子技能的旧等价。43 份挂起档案有原始字节与成功续行断言，但这些断言只证明所列原生检查点内的恢复，不证明规则与上游相同。

## 输入与独立核验方法

最终运行输入固定为 [probe-09.rs](../../U07/native/probe-09.rs)、[09-formal.log](../../U07/native/09-formal.log)、[case-results-twelfth.jsonl](../../U07/browser/case-results-twelfth.jsonl)、[start-case.js](../../U07/browser/start-case.js)、[run-cases.py](../../U07/browser/run-cases.py)、[正式目录](../../U07/native/formal-catalog-without-placeholder.json)及 43 份 formal-archives。逐案原始 CDP 索引是 [final-input-index.json](../../U07/final-input-index.json)，SHA256 为 4bf0d611ca74bda600a1f1a3f7006fc75bd244447ad232acff656909762cda7c。

本复核自写 [recompute.py](recompute.py)，未导入采样者比较器。实际执行命令：

    python3 -B docs/operations/evidence/2026-11-14-queue1-runtime/independent/u07/recompute.py

[原始输出](recompute-01.raw.txt) exit 0；首次执行成功，无忽略。它逐案核对原生与浏览器各 24 行、各 24 唯一 case、行序一致、全部 twelfth/success；没有去重，也没有删除不相同的分支。原始 fixture 请求必须逐字段等于对应原生 JSON；setup、每个选择观察及终态必须能在同一次 CDP 会话找到原始请求/回复，不能只采信汇总字段。实际 start-case 请求必须等于保存脚本全文。原始请求和回复的行号、时间和内容均交叉核对，详见 [独立重算结果](recomputed.json)与[请求/回复摘录](raw-cdp-excerpts.jsonl)。

指定初态投影包括全部 8 座位的 hp/max_hp/alive/linked/face_down/hujia、技能、复仇标记、手牌/装备/判定区的 ID/name/suit/rank，以及整条牌堆的 ID/name/suit/rank 顺序；区域内部作为多重集比较。唯一技能别名为 wd_sanjian → sanjian_skill，牌名别名逐项定义在独立脚本。初态及终态还核对弃牌、桌面区域；浏览器 virtualEquips 数量、virtualJudges 为空和无外来延迟补牌亦核对。

这不是完整初态或整副牌语义相同：原生 ID87、88 为雷杀，94–97 为火杀；探针未输出性质，浏览器将它们作为普通 sha 创建，快照也没有性质字段。六张牌未参与这些 case 的执行。本结论只适用于所列 ID/name/suit/rank 投影及本次实际使用的普通牌，不扩展到性质、整副牌、RNG、所有修正器、暗牌顺序、完整事件流或完整回合状态。筹度以真实 chooseToUse/phaseUse 事件启动；另两技能以真实 useCard 启动；浏览器选择经实际 DOM click/ok/cancel。fixture 会设置角色和牌堆、切换控制者；这是规则运行夹具，不是完整产品牌桌验收。清算使用原生三次 NextPhase，对应浏览器显式触发 phaseAfter，未比较两边完整阶段状态。

07/08 含人为追加调兵实体的旧轮只作为历史尝试，原始文件和 [初始输入快照](initial-inputs.json)保留。最终 09 走正式 bridge_pack + attach_formal_package_upstream_rules，未追加实体；正式目录有 138 张牌、扩展实体 14 张，调兵遣将物理 ID 为零。上游 card/list.js:25–26 有红桃4、方片4两张，原生 assets/cards.json:101–105 仍列 deferred；loader decision34_pack9.rs:549 仅在原包没有任何牌时补一张，因此正式包已有 14 张时不会补。最终没有虚构一个物理调兵入口来取得通过结果。

## 24 个 case 的实际选择与终态

下表行号同时对应正式原生 log 与 twelfth JSONL（均为 1-based）。CDP 列是 session-wudian-1794592764804-requests/replies.jsonl 的终态行；具体点击和中途问答行均在 recomputed.json。所有行的指定初态投影相同。表中“同”只指上述终态投影及 settled，未签事件或整技能等价。

| 行 | case / 实际分支 | 终态 | 关键观察 | CDP终态行 |
|---|---|---|---|---|
| 1 | choudu-top-return：弃0、指定1；上游置顶3 | 异 | 原生将3弃置，牌堆首为7；上游牌堆首为3，弃牌少3；玩家区域仍相同 | 4359 |
| 2 | choudu-target1：弃0、指定1、各处取消换牌/置顶 | 同 | 0被消耗，1摸45，市场3/4/5/6弃置；上游另询问2，原生没有 | 4469 |
| 3 | choudu-target2：弃0、指定2、取消换牌/置顶 | 同 | 使用者改为2；上游仍按完整目标表询问1/2，原生仅2 | 4577 |
| 4 | choudu-swap-a：1用30换市场首张3 | 同 | 双方相同手牌与弃牌；只覆盖原生固定首张恰与上游所选相同 | 4719 |
| 5 | choudu-swap-b：1用46换市场第三张5 | 异 | 原生只能给3；上游选中5。原生弃5、上游弃3 | 4849 |
| 6 | choudu-equip-target1：弃装备136、指定1、取消换牌 | 同 | 装备与绑定技能移除；上游还有0/1/2的换牌窗口，原生仅1 | 4948 |
| 7 | choudu-cancel-card：取消发动选牌 | 同 | 未消耗0/136，未摸牌、未开市场；未测随后再次发动的次数状态 | 4960 |
| 8 | choudu-cancel-target：已选0但取消目标 | 同 | 原生尚未支付0，上游取消整次选择；未测再次发动 | 4973 |
| 9 | choudu-empty：无手牌/装备 | 同 | 原生激活拒绝/无询问，上游没有筹度技能按钮；未支付成本 | 4981 |
| 10 | fuchou-hand：被杀者1给来源0手牌30 | 异 | 取消杀、给牌、标记相同；原生又将原杀1退回0手，上游将1弃置 | 4999 |
| 11 | fuchou-equip：给装备136 | 异 | 136交给0且原装备技能移除；原生额外退回原杀1 | 5016 |
| 12 | fuchou-cancel：拒绝给牌，随后不闪 | 同 | 原杀1消耗、目标1由8降7，无复仇标记 | 5026 |
| 13 | fuchou-empty：被杀者无he牌 | 同 | 无复仇给牌问答，普通杀命中目标1 | 5034 |
| 14 | fuchou-settle-no-sha：清算时无杀 | 异 | 原生自动发虚拟杀，HP(0,1)=(7,8)；上游持有者失去体力，(8,7) | 5073 |
| 15 | fuchou-settle-draw-sha：此轮清算摸到杀3 | 异 | 上游本次仍未给出出杀窗口、HP=(8,7)；原生以底牌137发杀、HP=(7,8) | 5111 |
| 16 | fuchou-settle-have-sha-decline-retaliation：原持杀2但拒绝 | 异 | 上游真实 chooseToUse 可取消并失去体力；原生无对应选择，自动杀0 | 5151 |
| 17 | fuchou-settle-have-sha-use：选择真实杀2 | 异 | 两边HP同为(7,8)，但原生保留手牌2并弃底牌137；上游消耗2且原杀1也已弃置 | 5212 |
| 18 | youzhan-hand-stop：弃手中装备92，来源停止 | 异 | 同样消耗92/原杀1；上游只罚来源1，原生还让原杀命中0，HP(0,1)=(7,7)而非(8,7) | 5250 |
| 19 | youzhan-equip-stop：弃已装备136，来源停止 | 异 | 装备成本移除；原生同样额外结算原杀，0多受1伤害 | 5289 |
| 20 | youzhan-cancel：拒绝弃装备 | 同 | 92/136保留，原杀命中0，来源不受罚 | 5299 |
| 21 | youzhan-far：目标2距诱战持有者0过远 | 同 | 无诱战成本窗口，原杀正常命中2 | 5307 |
| 22 | youzhan-no-equip：持有者无he装备 | 同 | 无诱战窗口，原杀正常命中0 | 5315 |
| 23 | youzhan-counter-use：弃92、来源用真实杀2、再遇诱战时拒绝 | 异 | 上游仅反击杀2命中，0为7；原生还结算原杀1，0为6 | 5378 |
| 24 | youzhan-counter-dodge-stop：用杀2被闪30，随后来源停止 | 异 | 上游重新询问来源并罚1，HP=(8,7)；原生不重问且原杀继续命中，HP=(7,8)；消耗区域相同 | 5449 |

## 逐技能条件、分支与结算审查

筹度：上游 character/skill.js:586–602 是出牌阶段、usable:1、he任意一张成本；所选使用者须通过虚拟调兵的 cardEnabled，完整存活玩家按其座位排序后作为目标。card/card.js:4–142 令使用者摸1，揭示存活人数一半向上取整的公共市场；每个可结算目标先选任一市场牌，再强制选一张手牌替换；最后使用者可按顺序置顶任意选中的市场牌，其余弃置。原生 activation/wudian.rs:320 检查有he牌；step/wudian_cards.rs:572–651 的牌/目标问答均可取消，只有选中目标后才支付成本。区别在 :604 只筛存活目标、:646–649 将目标表置为单个使用者、:722 固定市场第一张、:707/:757 将余牌全部弃置。运行第1/5行直接暴露置顶和选市场牌缺失，第2/3/6行的真实上游额外问答暴露参与者缺失。host.rs:381–395 还将次数设为 once_per_turn；上游 usable 依 phaseUse 计数，额外出牌阶段及 cardEnabled 修正器场景本轮未跑，不能从单次取消或成功补签这些条件。

复仇：上游 character/skill.js:508–546 于目标 shaBefore 检查he非空，chooseCard可取消；成功时只取消本次目标的杀，将所选牌给杀来源，并将来源加入 fuchou2 标记。原生 kernel/sha/resume.rs:12–74 内联在杀结算中提问，候选确含手牌与装备、允许空答；成功时移除所选牌及装备技能、给予来源并写标记。差异是 :59–60 将原杀从桌面移回来源手牌，实跑第10/11行确认；这不是取消的共同必然行为。标记使用 .push，区别于上游 .add，重复同来源、多目标杀与来源死亡分支未在本 cohort 实跑，未当成已复现结果。

复仇清算 fuchou2：上游 character/skill.js:548–584 是任意角色 phaseAfter，标记中有活人即强制结算；逐一移出标记、对活来源发起摸1，在当前 step 处检查 canUse/hasSha，可用时选择真实杀，拒绝或条件不满足者 loseHp，随后处理下个标记。这里不能把源码中文字顺序当成“摸牌已执行完再判断手牌”；第15行实跑摸到杀3仍未给出出杀选择。原生 filtering/common.rs:23–30 的实际代码也匹配全局 TurnEndWindow after:true，文件上方旧“自己回合”注释不作为结论。step/wudian_status.rs:124–145 先收取全部标记/安排摸牌，随后对每个来源自动调用 use_virtual_card；没有出杀/拒绝分支、没有持有者 loseHp。kernel/machine.rs:660–682 会抽牌堆底实体作为虚拟杀载体，本次是137。第14–17行分别证明无杀、拒绝以及真实杀消耗的差异；第17行HP相同也不能判通过。

诱战：上游 character/skill.js:422–447 在每个目标 shaBefore 检查持有者到该目标距离≤1且he含装备，成本为可取消的弃装备；成功由被杀目标虚拟使用诱敌。card/card.js:177–200 取消原杀，使来源反复选择对被杀目标用真实杀；仅当 card/skill.js:5–14 的 damageEnd 确认杀已对该人造成伤害才结束，闪避后必须继续问，停止则由被杀目标造成1伤害给来源。原生 filtering/common.rs:1020–1042 实际只检查 CardUsed 第一目标；factory.rs:519/632 保存来源及目标表；step/wudian_cards.rs:439–568 的装备候选/可取消成本、单次再杀问答有落点。:492–496 尝试给当前帧写 cancelled 后，正式实跑中原杀仍继续：第18/19/23行有额外目标损血，原生日志亦有原杀1的 DamageDealt。:551–556 发出反击杀就 Flow::Done，未等成功伤害后决定是否重问，第24行确认闪避后的第二次选择和惩罚丢失。多目标/后追加目标、距离修正器、无效装备成本等组合未运行。

伤害归属需分清证据层级：原生日志实测包含 defender0→attacker1 的惩罚和多出的 attacker1→defender0 原杀伤害；上游本次结果只保存 HP、选择与 skillEvents，未保存 damage/source/target 事件字段。因此，上游惩罚来源由实际上游 card.js:199 的 damage(player) 源码推出，本报告未声称两引擎伤害来源事件字段已运行对照相同。事件次数、CardPlayed/CardDiscarded 全流也没有跨引擎完整采样。

## 关联实体与未验证入口

调兵遣将的市场选择/置顶机器反例同时影响筹度调用路径和该机器；正式包实体缺席由 card/list.js、原生包数据、loader 条件与目录输出共同确认。没有执行不存在的原生物理 ID，也不将“有宿主名称”算成实体可用。

实体诱敌深入有独立内核落点，不能从 host_skill_bindings.rs:51 的共用名称绑定直接判断它没有实现。kernel/sha/step.rs:708–715、874–919 允许在杀响应中使用实体，并取消原杀；resume.rs:302–335 会问一次杀，然后发出子杀即结束。与上游“造成杀伤害前持续追问”的规则存在源码确认差异。这个实体入口未在本次24案跑，不能把第24行虚拟诱战反例写成实体卡实跑；若旧实体整卡条目仍无差异，应按该具体源码差异撤回等价，并补实体杀被闪后的对照。g_youdishenru 的独立触发适配不因共用 HostBehavior 名称而自动判错，本轮未新签它。

原生另有仅日志层观察：筹度弃手牌0、诱战弃手牌装备92各记录两次相同 CardDiscarded，反击杀2有两次 CardPlayed，但实体只移动一次。对应 step/wudian_cards.rs:476/480、:634/638 及 :545/551，底层 helper 已记录事件；本轮上游未完整采对应日志，故不写成已测得的跨引擎事件次数差异。

## NGSA 与恢复范围

探针 answer():17–23 在每次实际应答前 save/encode，保存原档，再 decode/restore_with_extensions；它断言恢复前 state_hash 相同，并对相同原始 Answer 比较 CommandOutcome、应答后 state_hash、应答产生的日志切片；拒绝结果会导致探针失败。因此输出中的 true 不是无条件代写的结论。43 份原档逐字节与记录的文件/长度相符，独立检查 SHA、NGSA magic、107版本、载荷长度及头内规则字段与载荷开头一致，未只统计文件数。

| 询问 | 检查点数 | 实际范围 |
|---|---:|---|
| wudian_choudu_card | 8 | 手/装备成本及取消 |
| wudian_choudu_target | 7 | 目标1/2及取消 |
| wudian_diaobing_swap | 6 | 换不同手牌或取消；不包含缺失的市场牌/置顶选择 |
| wudian_fuchou_cancel | 7 | 手/装备给予或取消，含后续清算场景 |
| respond_shan | 6 | 普通杀/反击杀的实际闪与放弃 |
| wudian_youzhan_discard | 7 | 手牌装备、已装备成本、取消及反击再遇询问 |
| wudian_youdishenru_sha | 2 | 选真实杀2；没有闪后再次追问的原生窗口 |

这次独立工作没有重新运行 U07 Cargo/浏览器，也没有独立 Rust 解码重放 U07 的43档。它核查生产探针的实际断言、原始成功执行、保存字节和适用源码。asks_wudian.rs:579/629/664 重建这几种 host 问答；engine/snapshot.rs:144–175 比较观察者、prompt、kind、有序候选、min/max、取消许可。未在U07另造损坏候选、错观察者、错序、跨内容锁、旧版本档案，也未比较完整快照逐字段或两引擎 RNG，不能对这些边界作实跑背书。跨版本旧档的 L29 独立恢复证明属于另一报告，不能挪来覆盖U07。

## 清单建议与审查范围

[findings.json](findings.json)给出精确 inventory key、上下游位置、运行行、反例与证据等级。三主技能继续简化近似；fuchou2 的旧等价应撤回。实体调兵保持近似；实体诱敌及其“成功伤害后结束”的辅助技能按已确认源码差异处理，不能因没有跑实体入口就新签等价。g_youdishenru、其它U07规则项、整包八项签署及全来源等价均不在本轮签署范围。本复核不写清单；主agent统一应用并登记当前数量。

源/规则及初始尝试哈希见 [initial-inputs.json](initial-inputs.json)，最终原始输入与43档快照见 [final-inputs.json](final-inputs.json)，补查源码见 [review-inputs.json](review-inputs.json)。最后的[输入稳定性检查](final-input-check.json)分别核对历史尝试、最终输入与补查源码；所有已保存历史快照均保留。完整三crate、全workspace、preflight和发布/CI由主agent另记，不计为本独立报告执行结果。
