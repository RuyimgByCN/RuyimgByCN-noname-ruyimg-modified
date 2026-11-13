# 2026-11-14：L29 三技能真实上游运行与原生对照

本证据属于队列 1 的运行补课，执行者 `/root/fix_dddanzhi`；实现本轮场景者不负责独立签署。开工 HEAD 为 `605201f3f49e10cea61c31d52a61bbb66c85599d`，tracked/index 均 clean；规则 333／NGSA 107／schema 5／ABI 759／WS8。没有修改产品、清单、公共文档、Git index/refs，没有提交或推送。开工环境和输入 SHA256 见 [preflight.json](preflight.json)。

## 01:45–02:01 初始产品快照上的运行

[commands-before-product-fix.json](commands-before-product-fix.json) 保留实际 cwd、完整命令和每轮结果。所有 Cargo 命令使用现有共享 `target`，没有清理它，没有私有 Cargo target。原生执行入口是 [native-probe/src/main.rs](native-probe/src/main.rs)，通过真实 `NextPhase`、`ActivateSkill`、`PlayCard`、`Answer` 结算；`debug_*` 仅用于初态布阵，没有 `debug_execute_effect`。桥接的实际包输入见 [native-round4/native-pack-input.json](native-round4/native-pack-input.json)。运行时包名 `longxue-upstream-liuli`、版本 1、6 host／0 VM／0 扩展卡；content lock 为 `14312970378986227901`，存档绑定 ruleset 为 `14312970378986228208`，见 [原生输入元数据](native-round4/native-input-metadata.json)。

浏览器入口 [launch.mjs](launch.mjs) 按现有 localhost/CDP 做法创建独占 profile；最终会话见 [active-session-l29b.json](active-session-l29b.json)。最终采用 [browser-round8/results.json](browser-round8/results.json)：**22 个场景均经真实上游事件跑到预定窗口／结算，22 个运行器完成、0 个运行器失败、0 个忽略**。每个 case 先 `location.reload`，重新等待 4 个真实上游座位对象及技能库，再初始化场景。每场景记录加载页初态及布阵后的四玩家 HP/maxHP、全部手牌/装备牌面、primary skills、翻面、放逐旧目标与关键 storage。四座位的 HP/maxHP、牌面 ID/name/suit/rank、装备、技能、翻面断言全部通过；没有靠前场清理维持隔离。第 7 轮也是 22/22，但第 8 轮增加只读的 `damage` 返回事件来源观察，故作为最终输入。

上游执行的是 `archive/upstream/noname-liuli/noname/character/mobile.js` 中实际注册的三项定义（清正 17117、酒诗 17251、放逐 17410）。L29 包包装器引用这些名字，原始引用和哈希见 [source-provenance.json](source-provenance.json)。**没有执行完整龙血玄黄包装器、道心解锁、动画或配置面**。选花色、目标、模式、确认均点上游 UI 的实际 `ui.click` 路径，未替换技能 content/结算；场景初态直接授予技能。服务器在读文件前拒绝所有含 `spine` 的 URL，原始 HTTP 清单保留；未读、复制或加载官方 Spine 运行库。

最终投影比较见 [comparison-before-product-fix.json](comparison-before-product-fix.json)：**22/22 的最终 HP/maxHP、翻面、手牌与装备 ID＋牌名＋花色＋点数＋属性多重集相同**。这不代表整个状态、轨迹或规则相同。每个浏览器初始手牌都和其原生 fixture 精确对应；native 原始牌堆有 17 张未选定上游映射，初始弃牌为 native 16／upstream 10（6 张未映射），完整两者都保存在场景 JSON。只对实际参与本场技能的实体牌比较：酒诗获得的都是相同 ID=105 的红桃 8【无中生有】，清正弃牌也按对应实体 ID 比。没有把相同牌名当作完整初态一致。

### 22 个实际场景

| case | 真实动作与观察 | 原生对照及边界 |
|---|---|---|
| q_A | 选自身黑桃两张→座位1→梅花一张；目标1降至3，主人剩红桃牌 | 最终牌/HP相同；上游目标花色窗前已付款，原生尚未付款；伤害来源上游0、原生None |
| q_B | 选自身红桃一张→座位1→红桃两张；不伤害 | 最终相同；目标花色窗前的付款时点仍不同 |
| q_target_B | 选黑桃两张→座位2→方片一张；目标2降至3 | 最终相同；付款时点、伤害来源仍不同 |
| q_cancel_initial | 上游首个花色窗取消，双方不弃牌不伤害 | 原生额外 trigger_confirm 取消，同样无牌/HP变化；不是相同询问轨迹 |
| q_cancel_target | 选花色后在目标窗取消，双方牌/HP不变 | 原生目标窗允许空取消，双方牌/HP也不变 |
| q_cancel_suit_attempt | 上游花色窗有取消控件，取消后返回出牌阶段 | 原生先确认发动再到不可取消花色窗；空回答 InvalidChoice，仍挂起 |
| q_no_hand | 主人无手牌，phaseUseBegin不出现清正询问 | 原生 NextPhase 无清正询问，HP/牌不变 |
| j_active_gain | 出牌阶段真选酒诗→翻背→确认获得锦囊；获得ID105，最终酒标记1 | 原生翻背/得牌/酒生效相同；得牌窗时原生酒已生效，上游尚未生效；旧恢复失败 |
| j_active_cancel_gain | 真发动酒诗，翻背后的得锦囊窗取消；最终翻背且酒标记1、不获牌 | 原生取消也不获牌、酒生效；中间生效时点不同；旧恢复失败 |
| j_active_facedown | 初始已背面，实际候选没有酒诗主动子技能 | 原生 ActivateSkill 拒绝 InvalidChoice，不再翻面 |
| j_no_skill | 无酒诗技能，出牌候选没有酒诗 | 原生拒绝 NoSuchSkill |
| j_damage_back_accept_gain | 相邻座位1实际使用实体杀；主人背面受1伤，确认翻回，再确认获得ID105 | 原生真实杀结算后HP3、正面、同牌；旧翻回窗和得牌窗恢复均失败 |
| j_damage_back_cancel | 同样真实杀受伤，在翻回确认窗取消；HP3，保持背面，不得牌 | 原生取消分支相同；旧翻回窗恢复失败 |
| j_damage_faceup | 初始正面，实际受杀后没有翻回/得牌询问，HP3 | 原生也不询问、不翻面、不得牌 |
| f_skill_A | 放逐目标1→“技能失效”；上游酒诗不被 blocker 封锁、马术被封锁 | 原生 enabled 恰相反：酒诗false、马术true；本回合再次发动被拒绝 |
| f_ban_B | 放逐目标1→“无法用牌”；实际 storage为trick，cardEnabled 查询杀/酒false、无中true | 原生目标1写 ban=1；未跑完每类普通/虚拟用牌与响应合法性，不能认证整个禁牌规则 |
| f_target_B | 选目标2→“无法用牌”；目标2生效、目标1未获该状态 | 原生相同目标选择及ban落点；同上，后续全用牌规则未验 |
| f_cancel_target | 选主动技能后取消目标，再次点该技能可重新选目标 | 原生空取消返还本次机会，再 Activate 开新目标窗 |
| f_last_target | 初始旧目标为1，实际目标UI只允许2/3；点击1不被选中 | 原生选项仅2/3；回答1被 InvalidChoice拒绝，仍挂起 |
| f_self_attempt | 实际目标UI排除自身0；点击0不被选中 | 原生回答0被拒绝，仍挂起 |
| f_cancel_mode_attempt | 真选目标后，模式窗只有两模式，没有取消控件 | 原生模式窗allow_cancel=false，空回答被拒绝且仍挂起 |
| f_no_skill | 未授予放逐，出牌候选没有放逐 | 原生拒绝 NoSuchSkill |

### 初始快照上实际发现的差异

1. 清正付款延后：上游 `mobile.js:17200` 附近在目标确认后调用 `player.discard(cards)`，再提目标花色；原生 `step/skills_damage.rs:525` 附近只提问，直到 `:538` 分支才支付双方。`q_A` 的上游该窗主人只剩 ID51；原生还持 ID85/114/51。其他失牌触发与挂起时的持牌状态因此没有相同轨迹。
2. 清正伤害归属：上游真实 `target.damage()` 返回事件 `source=seat0`，见 `browser-round8/upstream-q_A.json`／`upstream-q_target_B.json` 的 `damage_created`；原生最终 `DamageDealt { source: None }`，见对应 native JSON command log。后续伤害来源技能结果尚未实跑，不能把该后果当已测，但来源差异本身已测。
3. 清正花色窗取消：上游首个实际花色窗可取消；原生额外确认后 `mbcmqingzheng_suit` 不可取消。不是完全不能放弃发动，而是取消发生的窗口不同；真实反例为 `q_cancel_suit_attempt`。
4. 放逐封锁范围：上游 `mobile.js:17478` 的 blocker 只排除 forever/charlotte，故普通锁定马术受封、forever酒诗保留；原生 `step/skills_targets.rs:633` 使用中文锁定技能白名单。`f_skill_A` 真实验证两技能的反向结果。没有以此推断未测的其他技能。
5. 酒诗主动中间时点：上游翻面触发得牌询问时，酒标记仍0；其栈上待结算实际 useCard(jiu)。原生在发 TurnOvered 前已置 jiu_active/jiu_used。最终酒状态同向，但完整事件轨迹未对齐。
6. 酒诗挂起恢复失败：真实 `ChoiceSpec::confirm` 是 `[Confirm]`、min1/max1、allow_cancel=true；初始 `asks_wudian.rs:724` 恢复派生却为 `[Confirm, Pass]`、allow_cancel=false。**34 个待回答检查点在 direct 和恢复支路各尝试一次，共68次；58成功、10失败。4个case有失败，两种提示词为 gain_confirm/turnback_confirm**。每个失败的原始 NGSA、decode/restore 错误及完整命令都保留。成功58次均完整 `StateSnapshot` 除 auth_generation 外一致、state_hash相同；原始字节均因恢复代次递增而不同。失败时运行器继续原实例以保留结算结果，不能将这些case的最终哈希相同当作恢复成功。

### 首次失败与修正保留

- `native-first.raw.txt`：外部probe误用不存在的 `ExtensionPack::to_json`，编译失败E0599；改为保存精确 cards.json 桥接输入，产品未动。
- `native-second.raw.txt`：22case执行，但3个受伤case因对座杀超距离被IllegalTargets拒绝。`native-round2` 改相邻座位1；首轮不改写、不计作伤害正例。
- `browser-round1` 14完成/8失败：Qinggang手牌未映射，以及跨 evaluate 重用全局const。`round2` 16完成/6未映射手牌失败。`native-round3` 仅补确切Qinggang牌面映射，JS局部变量用IIFE作用域。
- `browser-round3` 20完成/2失败：复用页面把 storage 清了却留下 initedSkills，第二次放逐不能初始化storage。`round4` 补fixture清理后22完成，但仍有when/其他残留风险，前四轮全部只作探索，不作最终对照。
- `round5` 尝试每case刷新，错误等待新页必须已有角色名；新页暂停在启动更新/游戏选择，场景随后才init(dummy)。中断runner产生HTTP abort，首launcher未捕获ECONNRESET崩溃。`round6` 的502仅是运行器会话失败，排除。完整first launcher及请求回包保留。
- 第二launcher用新唯一profile/端口，捕获HTTP请求abort；ready改为等待真实4座位、技能库和player.init，再在fresh JS realm布阵。`round7`、`round8` 各22/22运行器完成。没有修改上游或产品结算。

### 未验范围

未比较全状态、RNG流、绝对事件/choice ID、完整事件日志、手牌次序、原始牌堆/弃牌未映射部分、完整L29包装器与解锁、其他技能组合、目标下一回合临时状态到期、全部禁牌/虚拟用牌/响应路径、上游存档恢复或跨引擎档案。此次NGSA验证仅为原生。没有签新的“已复核等价”，没有改任何清单状态。

## 主 agent 的酒诗局部恢复修复待复跑

主 agent 独占产品 `asks_wudian.rs` 和现有 L29 测试文件；本agent不修改它们。主 agent 已获知两种真实旧NGSA及最小复现。修复前二进制位于 [native-before-product-fix-inputs.json](native-before-product-fix-inputs.json) 记录的 `/private/tmp` 独占目录，旧NGSA和原始结果永久保留在本证据目录。待收到产品稳定通知后另建目录重跑，不能覆盖本节结果，也不能以局部补跑替代主 agent 的完整门禁。

## 02:03–02:04：主 agent 酒诗恢复修复后的独立复跑

产品修改归主 agent，输入 SHA 与 tracked dirty 清单见 [native-after-product-fix-inputs.json](native-after-product-fix-inputs.json)；HEAD 仍为 `605201f3f49e10cea61c31d52a61bbb66c85599d`。dirty 产品文件是主 agent 的 `asks_wudian.rs` 和既有 L29 测试；本agent只写 L29 证据目录。版本仍333/107/5/759/WS8。

实际执行（使用共享target，先等已有构建锁；没有清理target）：

```bash
mkdir docs/operations/evidence/2026-11-14-queue1-runtime/L29/native-after-jiushi-fix
L29_REPO=/Users/ruyimg/Documents/Claude/sanguosha/noname-next L29_EVIDENCE=/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-11-14-queue1-runtime/L29/native-after-jiushi-fix L29_RESTORE_INPUT_DIR=/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-11-14-queue1-runtime/L29/native-round4 cargo run --offline --manifest-path docs/operations/evidence/2026-11-14-queue1-runtime/L29/native-probe/Cargo.toml --target-dir target > docs/operations/evidence/2026-11-14-queue1-runtime/L29/native-after-first.raw.txt 2>&1
python3 -B docs/operations/evidence/2026-11-14-queue1-runtime/L29/compare.py native-after-jiushi-fix comparison-after-product-fix.json > docs/operations/evidence/2026-11-14-queue1-runtime/L29/comparison-after.raw.txt 2>&1
```

- 22case分别直接运行及在每次回答前NGSA恢复：**68次恢复成功、0失败、0忽略**。成功检查点完整 `StateSnapshot` 除 auth_generation 外一致，state_hash一致；22case直接/恢复续行后的完整最终快照（仅归一化auth_generation）均一致。真实取消与非法回答分支照旧保留，没有改为自动接受。
- 修复前 `native-round4` 的**68份旧pending NGSA全部恢复成功**，包括主动得牌、受伤翻回、翻回后得牌。每份均从其真实挂起点重放原剩余命令，**136/136请求、136/136回包、136/136 log_delta** 与修复前原始记录一致；**68/68最终完整快照（除auth_generation）和hash相同**。见 [旧档续行汇总](old-ngsa-continuation-summary.json) 和 [68份逐档原始结果](native-after-jiushi-fix/old-ngsa-restore-results.json)。
- [native-before-after-direct.json](native-before-after-direct.json) 确认22case初态、全部直接运行命令/回包/状态/log_delta、最终NGSA字节都与before相同；包元数据、content lock和存档绑定版本完全相同。这是本次局部恢复修复的验证范围，不能推广到所有历史档案。
- [comparison-after-product-fix.json](comparison-after-product-fix.json) 仍保留清正付款/取消/伤害来源、放逐技能封锁范围、酒诗中间酒生效时点的具体差异。上游最终比较复用同一真实fresh `browser-round8`，没有为变绿而重录或更改上游预期。

本agent未跑全量Cargo/产品门禁，也没有独立签署等价；主 agent 负责整合门禁，另一个agent负责独立复核新场景。本节不覆盖前节首次恢复失败和前轮运行器失败。

引用行号补充：清正上游实际付款调用为 `mobile.js:17212`，伤害调用为 `:17248`；放逐 blocker 定义为 `:17470`、forever/charlotte条件为 `:17471`。前节的“17200附近”与“17478”指向附近代码，本条给出准确行号。机器可读发现与未验范围见 [findings.json](findings.json)。

## 02:05：证据收口与清理

两次会话的请求/回包/HTTP原始日志都保留。第2会话已向自己监听controlPort的launcher发送SIGTERM，自己的Chrome退出、临时profile删除，见 [cleanup.json](cleanup.json)；第1会话孤儿Chrome/profile也已清理，见 [first-profile-cleanup.json](first-profile-cleanup.json)。没有操作其他Chrome，没有私有Cargo缓存，没有清共享target。修复前34MB二进制仍按主 agent 要求保留在明确的 `/private/tmp/noname-l29-runtime-preserved-kr0fipes/` 路径供复核，属于本批对照输入；确认无需该输入后可只删此目录。

最终文件哈希及本目录JSON/本地链接检查见 [final-evidence-manifest.json](final-evidence-manifest.json)。此次只形成3技能、22case的运行证据；保留5项实际状态/规则差异及其未验后果，酒诗恢复一项局部修复已独立复跑通过；不签等价、不调整来源计数。
