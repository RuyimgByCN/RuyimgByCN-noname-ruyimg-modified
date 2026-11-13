# U06 与 L29 独立复核（2026-11-14）

核验时间：2026-11-14 02:04–02:15（Asia/Taipei）。HEAD `605201f3f49e10cea61c31d52a61bbb66c85599d`，工作区 dirty；本复核者没有修改产品、测试、清单、公共文档或索引。产品差异仅检查 `asks_wudian.rs` 与 `l29_longxue_eight.rs`；其他 agent 的文档变动不作为行为依据。规则 333、NGSA 107、schema 5、ABI 759；本次差异没有修改这些版本或 WS8。

结论：酒诗确认窗的恢复定义修复正确，未发现本次小改动新增的阻断点。它保留当前运行时确认窗的格式，恢复真实旧档，并继续用完整选择规格拒绝损坏候选。U06 的十三个有效样本中，初态投影十三个一致，指定终态投影十一个一致、两个不一致。清正和酒诗各有可以定位到真实上游挂起记录的规则差异，旧整技能等价应撤回。本报告新增整技能等价签署为零。

## 证据方法

- 上游和原生的实际源码、选择生产/恢复校验、probe 与浏览器控制脚本是判断依据。没有用采样者的 README、解释或比较器作为正确性证明。
- 输入副本与 SHA256 见 [initial-inputs.json](initial-inputs.json)、[review-inputs.json](review-inputs.json)。前者在本复核首次检查时冻结 U06 原始输入及关键产品源码；后者补充 L29 原始输入和额外源码。没有改写初次证据。
- [independent-recompute.py](independent-recompute.py) 独立定义投影和原始记录对应，不导入采样者比较脚本。输出见 [independent-recomputation.json](independent-recomputation.json) 与 [实际输出](independent-recomputation-02.raw.txt)。首次运行因本复核脚本误用冻结副本路径失败，保留在 [首跑输出](independent-recomputation.raw.txt)；修正路径后全部断言通过，该失败不是产品失败。
- U06 将显式别名 `duel → juedou`、三张欢乐装备名和六个中英文技能名归一；花色转小写。比较手牌、装备与弃牌的牌面多重集，未比较手牌顺序。只剔除并单独登记本次实见的三个 `player_when_*` 清理技能；实际 `addTempSkills` 的 additionalSkills 与清理机制由上游 `player.js:11905` 起源码核对，没有将任意未知技能静默删除。
- 仅核对已采样投影，不宣称上下游完整状态、完整事件流、RNG 或整个扩展等价。具体未验范围列于末尾。

## 酒诗恢复小改动

实际产品 diff 仅在 `crates/engine-native-content/src/content/host/asks_wudian.rs:724` 的 `MbcmJiushi` 两个恢复派生分支，把 `[Confirm, Pass] / allow_cancel=false` 改成 `[Confirm] / allow_cancel=true`，min/max 仍为一。实时机器在 `step/skills_targets.rs:474`、`:504` 原本就调用 `ChoiceSpec::confirm`；该构造器在 `crates/engine-native-kernel/src/choice.rs:73` 明确定义 `[Confirm]`、允许空取消。

因此，这次没有改变正常窗口的候选或回答规则，而是让恢复派生规格与旧运行时真实创建的规格一致。没有增加 Pass 的兼容通道，没有修改存档字段、版本、内容锁、事件时机或正常效果。

`crates/engine-native/src/engine/snapshot.rs:150` 的唯一挂起询问匹配仍逐项比较 observer、prompt、kind、完整且有序的 options、min/max 与 allow_cancel；不一致仍报 `OpenChoiceMismatch`。实时回答另外由 `choice.rs:88` 检查成员、数量、重复及合法空取消。仅允许 `[Confirm]` 或空选择，损坏档追加 Pass 仍不等于派生规格。

新增回归 `crates/engine-native/tests/l29_longxue_eight.rs:905` 经真实主动发动和真实 Sha 伤害触发，共五种确认/取消组合，检查七个窗口：每窗保存、编码、解码、恢复，比较状态哈希和 open_choice；给存档 options 追加 Pass 必须拒绝；直接与恢复回答后比较 Accepted/Pending、状态哈希、日志后缀、下一窗口和最终朝向/得牌数。测试没有用新规格重写旧档，也没有只断言函数名或文档。

采样者原始测试日志另行核对：`restore-fix/02-jiushi-pending-regression-red-compiled.log` 为 0 passed / 1 failed / 0 ignored，真实恢复报 `OpenChoiceMismatch`；`restore-fix/04-jiushi-l29-tests-final.log` 为 5 passed / 0 failed / 0 ignored，77 filtered。它们是他人执行，本复核没有冒称独立重跑这五个 Cargo 测试。

### 独立执行的旧档与轨迹检查

本复核另行执行当前 probe，实际命令：

```bash
L29_EVIDENCE=/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-11-14-queue1-runtime/independent/u06-l29/l29-own-run L29_REPO=/Users/ruyimg/Documents/Claude/sanguosha/noname-next L29_RESTORE_INPUT_DIR=/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-11-14-queue1-runtime/L29/native-round4 CARGO_TARGET_DIR=/Users/ruyimg/Documents/Claude/sanguosha/noname-next/target cargo run --offline --manifest-path docs/operations/evidence/2026-11-14-queue1-runtime/L29/native-probe/Cargo.toml
```

退出码 0，[独立原始输出](l29-own-run.raw.txt)，[独立旧档恢复结果](l29-own-run/old-ngsa-restore-results.json)。共享 target 已释放，没有清理他人缓存。此 probe 是证据执行，不是完整三 crate 门禁。

另校验并执行修复前保留的可执行文件 `/private/tmp/noname-l29-runtime-preserved-kr0fipes/probe-before-jiushi-fix`，环境为 `L29_EVIDENCE=<本目录>/l29-own-before-run`、`L29_REPO=<仓库绝对路径>`，没有 `L29_RESTORE_INPUT_DIR`。它的 SHA256 `f562f543921040b9c6afa078ecd7db0f44ca6faf084ff6663f407bcc8e3abcc1` 与事先保留的 before manifest 相符。见 [校验元数据](l29-own-before-executable.json) 与 [独立 before 输出](l29-own-before-run.raw.txt)。

独立重算结果：

| 检查 | 实际结果 |
| --- | --- |
| 修复前 binary 生成的对应旧 pending NGSA 与 `L29/native-round4/` 字节逐份相等 | 68 / 68 |
| 修复前挂起恢复 | 58 成功 / 10 `OpenChoiceMismatch` |
| 当前挂起恢复 | 68 成功 / 0 失败 |
| 旧档恢复后完整 snapshot（仅归一 auth_generation） | 68 / 68 相等 |
| 继续执行原记录的 request / reply / log_delta | 136 / 136 相等 |
| 续行最终完整 snapshot（仅归一 auth_generation）及状态哈希 | 68 / 68 相等 |
| 修复前后直接运行的 initial / commands / final / final_archive_hex | 22 / 22 相等 |
| 包元数据、content lock、绑定规则版本 | 前后相等 |

修复前十个失败来自主动得牌两案各两次、受伤翻回并接受得牌四次、取消翻回两次，包含 direct/restored 两路。当前重复执行的 old-ngsa 结果逐对象等于采样者 after 记录。新恢复引起的 auth_generation 增加是安全代际变化；原始 NGSA 字节因此不要求与恢复后档案完全相同。完整比较只归一该字段，未剔除 RNG、挂起栈或其他字段。

这些证明限于已保存的同版本真实旧档，不涵盖任意旧规则版本或跨引擎档案互通。

## U06 三技能的十三个有效样本

只纳入 `U06/browser/case-results.jsonl` 中 `run_tag=fresh-nodes` 的十三条。其余十三条 `first` 原样保留，不因结果是否一致而选择性剔除。独立脚本把每条有效 setup、choice、final 逐对象对应到原始 CDP reply，核对对应请求确实是保存的 `start-case.js`、`await-choice.js` 和 `finish-case.js`，以及注入 fixture 就是同案 native JSON。重复相等的旧结果索引也一并保存。

有效 cohort 的 CDP setup 索引依次为 186、194、202、207、215、223、228、233、244、255、266、274、285（均为零起算）；最终索引为 190、198、203、211、219、224、229、240、251、262、270、281、286。原始会话为 `U06/browser/session-modern-1794592005610-{requests,replies}.jsonl`。两端八个座位的初态 HP/maxHP、手牌、装备和语义技能十三案全部相等。

| 样本 | 真实选择与结算 | 指定终态投影 |
| --- | --- | --- |
| monkey-confirm | 0 号确认；弃猴子，0 号得 1 号的红桃三桃，1 号仍 HP2 | 一致 |
| monkey-cancel | 0 号取消；两端 1 号 HP2→3，原生仍持桃，上游桃进弃牌堆 | **不一致** |
| monkey-self | 装备者自用桃，无他人偷桃选择，桃消耗并恢复 | 一致 |
| kuwu-hand | Sha 造成一伤害；1 号强制弃桃，原生先尝试空回答并被拒绝 | 一致，另有原生重复日志 |
| kuwu-equip | 同样伤害；1 号改弃猴子，失装备技，保留桃 | 一致 |
| kuwu-empty | 目标 he 为空，Sha 伤害后没有弃牌窗口 | 一致 |
| kuwu-duel | 实际决斗造成伤害，苦无不触发 | 一致 |
| xuelun-skill-a | 选 1 号，再选马术；原持有者保留，0 号临时获得 | 一致 |
| xuelun-skill-b | 同目标改选咆哮，所得技能与 A 不同 | 一致 |
| xuelun-target-b | 改选 2 号，再选武圣 | 一致 |
| xuelun-target-cancel | 第一个选目标窗口取消，不得技能，正常结束 | 一致 |
| xuelun-skill-cancel | 已选 1 号，在第二个技能窗口取消；上游结束，原生拒绝空回答并保持挂起 | **不一致** |
| xuelun-no-skills | 其他人无技能，不触发选择 | 一致 |

十三场实际执行不是十三个整技能证明。原生共有十五次回答尝试，每次都在本次运行保存、恢复后提交相同选择，核对 CommandOutcome、状态哈希及日志后缀；其中包含两次苦无空回答拒绝、一次写轮眼技能取消拒绝。十五次原生恢复检查均通过。U06 probe 未保存这十五份 NGSA 原始字节，也未将完整 snapshot 逐字段对比，不能把这些检查称作历史旧档验收。

### U06-R01：猴子取消后桃没有消耗

上游 `archive/upstream/noname/apps/core/extension/欢乐卡牌/card/skill.js:5` 的 `monkey` 在他人实体桃仍处处理区时可选拦截；不发动则继续普通桃的牌结算和消耗。fresh `monkey-cancel` 记录明确为：1 号 HP3、hand=[]、弃牌为红桃三桃。

原生同案为 HP3、hand=[红桃三桃]、discard=[]。原始路径 `U06/native-results-first.json` 的 `case=monkey-cancel`；独立输出 rows 同名案中完整保留两端投影。问题不是初态或桃的牌面映射。

实际调用在 `crates/engine-native/src/engine/play_card.rs:704` 至 `:744`：在从手牌移除桃之前挂起 `MonkeyStealMachine`。全部 Pass 后，`crates/engine-native-kernel/src/kernel/monkey.rs:64` 仅调用 `ctx.to_discard(self.card)` 和 recover。`machine.rs:582` 的 to_discard 对仍在任意玩家手中的牌直接返回，因此该桃留在手上。确认分支手动移牌，未暴露这个取消分支缺陷。

### U06-R02：写轮眼第二窗口不能取消

上游同文件 `:107` 起调用非强制 `chooseButton`，并在没有 links 时直接返回；fresh `xuelun-skill-cancel` 的 chooseButton `forced=false`、confirm=`c`，取消后 settled=true、未获得技能。

原生 `step/skills_targets.rs:267` 的该窗口 min=max=1、allow_cancel=false；`host/asks.rs:308` 的恢复派生也是不可取消。因此 [] 被 `Rejected(InvalidChoice)` 拒绝，pending 仍是 `huanle_xuelunyang_skill`，settled=false。这里恢复 faithfully 保留了原生偏差，并未修正规则。

源码另有候选范围剩余差别：上游用 `getSkills(null,false,false)` 并排除 charlotte，原生按目标的全部 `core.skills` 生成目标和技能列表；本次两个标准持技目标没有覆盖装备技、charlotte、封禁或重复已有技能。临时技能上游默认到全局 phaseAfter 或 phaseBeforeStart，原生 `machine.rs:866` 为拥有者回合结束移除；本次没有跨过这些清除边界，不能签署存续时长等价。

### U06-N01：苦无弃手牌写出重复日志

`kuwu-hand` 的原生日志对 card45 连续写两条 `CardDiscarded { player:1,cards:[45] }`，状态实际只移动一张。`step/skills_status.rs:1023` 先调用 `Ctx::discard_from_hand`，该函数在 `machine.rs:616` 已写同日志，随后 helper 又显式 log 一次。装备分支只有一次。

这是直接观察到的原生重复日志，不把它称作第二次弃牌效果。上游记录只采了相关 skillEvents，没有完整弃牌事件流，故本报告未声称已证明跨引擎事件数量差异。苦无四案的终态卡牌投影相同，也不足以消除这项日志问题或证明全部联动等价。

## L29 两项整技能规则反例

本节独立核对 `L29/browser-round8/` 与 `L29/native-round4/` 原始记录，并对关键六案全部 stage 对应到原始 CDP reply。每案初态四个座位的 HP/maxHP、朝向、手牌与装备的 id/牌面多重集、主技能均相同。独立脚本保留所有重复匹配，没有挑选较有利的一轮。

实际浏览器由 `L29/launch.mjs` 启动 liuli 1.10 规则本体、standard/extra 卡牌和 mobile 武将；未加载整套龙血玄黄 UI/解锁包装。包装 `archive/upstream/noname-liuli/noname/extension/龙血玄黄/extension.js:1` 只核对依赖与引用，实际执行技能定义取 `character/mobile.js:17117` 起的清正及 `:17251` 起的酒诗。没有阅读或加载 Spine 库。不能把本体依赖技能样本写成整个扩展包装验收。

### L29-R01：清正的伤害来源丢失

`browser-round8/upstream-q_A.json`：0 号弃黑桃两张（id114、85），选 1 号并弃其梅花一张（id8）；最终 trace `damage_created` 为 `{seat:1,source:0,amount:1}`。原生 `native-round4/native-q_A.json` 的 `direct.commands[4].log_delta` 为 `DamageDealt { source:None,target:1,amount:1,... }`。两端目标同降为 HP3，但来源相关伤害技能或记录会有不同条件。

上游 `character/mobile.js:17248` 调用 `target.damage()`，本体 `game/game.js:26343` 在未指定 nosource 时从当前事件的 player 取来源；当前事件正是 owner0 的清正。原生 `step/skills_damage.rs:558` 显式用 `damage_data_with(None,target,1,...)`，没有 owner 来源。该路径不受本次恢复改动影响。

原始 CDP 会话 `L29/session-l29b-1794592604612-replies.jsonl` 的零起算索引 288/289 为 resolved/final；JSON 中 trace 不是推断出来的 HP。

### L29-R02：清正先询问目标花色，后弃自己的牌

同案 `target_suit_question_after_owner_payment`：上游 owner.hand=[51]，原生在 `direct.commands[3].after` 的同类 `mbcmqingzheng_target_suit` 窗口 hand=[85,114,51]。`q_B` 也重现：上游只剩两张黑桃，原生尚有全部三张。

上游 `mobile.js:17212` 起在目标确认后先 `player.discard(cards)`，然后才询问目标手牌花色；原生 `skills_damage.rs:506` 的 TargetChoice 只建立下一问，直至 CompareTargetPick 才从双方弃牌。失牌/弃牌联动发生的时点不同。另见上游最初花色可取消，原生多一个 trigger_confirm 后才进入不可取消的花色问答；本报告不把不同窗口的空回答当成完全相同的输入序列。

### L29-R03：酒诗翻面得牌窗口之前已提前施加酒效果

`browser-round8/upstream-j_active_gain.json` 的 `gain_question`：owner 已背面朝上，hand=[]，jiu_mark=0；回答得牌后才 jiu_mark=1、获得 id105 无中。`j_active_cancel_gain` 同样在该窗口 mark=0，取消得牌后酒仍正常生效，mark=1。

原生两案 `native-round4/native-j_active_*.json` 的 `direct.commands[0].after`，在 `mbcmjiushi_gain_confirm` 挂起时已经 `jiu_active=true`、`jiu_used=true`。`skills_targets.rs:440` 起先改 face_down 和酒的四处状态，再 emit TurnOvered；上游 `mobile.js:17277` 起依次建立 turnOver 和 useCard(jiu) 子事件，翻面后的得牌询问发生在真实酒效果之前。只看最终朝向、得牌和酒标记会漏掉这一顺序差异。

原始 CDP 同会话零起算索引 376 为接受案 gain_question，390 为取消案 gain_question。受伤翻回两个样本的 HP、翻面与得牌结果对应相同，但没有覆盖 damageBegin3 时的背面标记与中途翻面变化、原生额外排除某些强化杀/连环伤害等组合。因此不签整技能触发条件等价。

## 本批结论的边界

- 小修复：同版本真实旧档恢复已独立验证；当前直接轨迹不变；损坏候选追加 Pass 的拒绝由未放宽的精确校验和新增回归支持。完整产品门禁由主 agent 另记，本报告不代签。
- 规则反例：猴子取消、写轮眼技能取消、清正来源/付牌时点、酒诗酒效果时点均有实际记录或直接源码因果；具体清单键和字段见 [findings.json](findings.json)。旧状态不可据本次恢复绿测上调。
- U06 选择候选：原始采样器的 selectableTargets 为 []，没有可靠记录真实 DOM 目标候选全集。标准场景的实际 A/B 和取消已观察；装备技/charlotte/禁用技、完整候选顺序/可见性、非法隐藏选择仍需补样本。
- U06 上游牌堆是十六张固定牌，原生完整牌堆未匹配；L29 也有十七张无法映射牌面、原生初始弃牌十六张对上游十张的范围差异。只确认所用牌、相应得牌和已比较投影，未宣称完整 deck/discard/RNG 同态。
- U06 没有越过临时技能清除点；L29 没有验收整个包装、解锁、配置、UI 或所有伤害联动；原生 NGSA 恢复不是上游存档恢复。
- 本报告独立新执行：当前 L29 probe 一次、保留 before binary 一次、纯 Python 重算两次（首次路径错误失败，第二次通过）；没有新跑浏览器或完整 Cargo 门禁，没有新增等价签署。
