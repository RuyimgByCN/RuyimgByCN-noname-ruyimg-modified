# U01 清单应用与运行对照方法独立复核

2026-11-14 00:28–00:30（Asia/Taipei），复核人 `/root/queue0_independent_review`。HEAD `bd58ef06d59940baed8d7f93e61ce571b152fb81`，工作区 dirty；规则333／NGSA107／schema5／ABI759／WS8。本次只读产品与证据，新增文件仅在本目录 `application-review*`，未运行 Cargo、浏览器，未提交。

限定结论：U01 的清单应用与本次局部投影对照可保留。61 个旧等价撤回、暗织维持近似、两个潜在缺口与31将保留历史状态，均与独立逐项审查范围相符。本轮新增等价签署0。实际重算覆盖13个分支、24次上游记录的指定投影；没有发现该队列中省略失败样本或删除同结果重复样本。此结论不证明整技能、全事件、随机流或完整对局轨迹等价。

依据是当前清单、实际脚本、原始CDP请求/回包、原生命令日志、冻结源码与输入哈希；没有用实现者README或说明文档证明行为。先前完整64技能与31将源码表仍保存在本目录，本次核验其结果如何应用及新增运行证据的方法，未把旧源码审查改记成实跑。

清单应用核验：

| 范围 | 独立核验结果 |
| --- | --- |
| 精确key | 64技能＋31将＝95；与当前U01键集合完全相同，无替代ID、增删或漏项。逐key明细见 [application-review-inventory.json](/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-11-13-queue1-review/independent/u01/application-review-inventory.json)。 |
| 61项降级 | 实际状态变化集合逐字等于 `findings.json.strict_downgrade_candidates`；不是按测试缺口或事件名统一下调。 |
| `dddanzhi` | 已是简化近似；旧reviewer、日期、历史note前缀保留，未重新上调；Prepare／整回合phaseBegin及其余既有边界继续存在。 |
| `dddyuanzhen`、`dddqice` | 两项potential_gap的status、reviewer、日期、上游引用、测试名及原生落点等metadata保持旧值；差别文字标明待核验，不是本轮重新认证。 |
| 31将 | 保留旧状态及metadata。correction04只在difference/diff前补具体性别、势力、起始/上限体力、技能列表；逐项与实际上游及loader常量相符。张闿3/4另核 `general_assembly.rs` 与 `general_select.rs` 专门分支；未签主公资格、姓名规则、替换表或资源显示。 |
| 来源外范围 | 原application有378条记录、226次撤回；其他复核者后续纠正非U01条目。本次不对这些来源作语义背书，也不把其变化算作U01差异。 |

原清单before SHA `4792c24d1855c8c156345ce5d013a348b849236aa779973bd51107e236605033` 等于HEAD内清单字节；最初application-after SHA为 `45629cc212b1a89ee91bdaaff7764bf522e9029a4bd339dc656d1e21aea27b96`。00:28快照的95项与该after逐项相同。correction04之后，31将文字具体化，所以最新95项不再全部逐字等于最初after：64技能仍相同，31将仅上述两字段新增精确前缀，状态及其他字段不变。最终增量见 [application-review-correction04.json](/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-11-13-queue1-review/independent/u01/application-review-correction04.json)。00:30捕获的全清单SHA为 `2658808bbe361ab248e53166157e25606c47155eb31fa19947cb601eb3f35c89`，全局634／3790／0仅作快照识别，不是本复核对全局状态的签署。

聚贤精确key为 `U01:skills:dddjuxian@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:244`，状态保持简化近似。修后候选私有排列由原生规则RNG产生并保存在已有续行载体，初版未随机排列的J06拒绝记录继续保留。当前difference保留“详见J01–J05”，其具体剩余边界为：

- J01：原生次数以turn_stamp计；上游top/hand子技能在phaseUseAfter失效，额外出牌阶段仍需对照。
- J02：原生在支付前按他人he预先禁止top并先选目标；上游先展示、置顶并等待lose，随后核gainable目标。
- J03：原生按手牌/装备枚举；上游getGainableCards、明置手牌及viewHandcard组合分支未齐。
- J04：原生支付的CardsLost子事件在获得询问之后才继续分派；上游await lose后才选目标/获得。
- J05：直接区域修改仍未覆盖完整gain/give/move/reveal事件语义。

这些边界不能因单次获得黑牌、红牌、装备成功而撤销；本轮仅核验清单没有把选择器修复扩大成整技能等价。

运行证据复算：

[application-review-runtime.json](/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-11-13-queue1-review/independent/u01/application-review-runtime.json) 对31条含完整players的结果逐条给出CDP回包文件/行、请求时间戳、所属队列、receipt分组及实际启动脚本SHA。31条分别对应31个不同的CDP成功回包；原生13条JSON逐字等于命令日志抽取结果，无额外原生结果被字典覆盖。更早的小诊断回包仍在CDP原始流中，不混入31条完整结果计数。

原final16汇总保留；新版reviewed24新增此前exact8。纳入规则是exact8＋ordered8＋blind-repeat3＋L06五条；此前7条探索fixture继续保留而明确排除。它们的牌面/牌堆输入不同，不是从同一输入队列挑出失败来丢弃。本复核独立实现了投影与分组，没有执行或import主agent比较脚本；原final16及reviewed24各自的样本标签、时间、分支集合、投影结果均与独立计算完全相同。

| 实际分支 | 上游记录数 | 原生挂起恢复点 | 被比较的可见结果 |
| --- | ---: | ---: | --- |
| U01 远振：弃杀 | 2 | 1 | 目标7血、剩红桃3桃。 |
| U01 远振：弃桃 | 2 | 2 | 目标7血、剩梅花7杀。 |
| U01 远振：取消 | 2 | 2 | 持技者摸方块6杀；目标两牌都保留，7血。 |
| U01 聚贤：实际获得黑手牌 | 3 | 4 | 获梅花7杀、ban=false；目标剩桃及装备。 |
| U01 聚贤：实际获得红手牌 | 3 | 4 | 获红桃3桃、ban=true；目标剩杀及装备。 |
| U01 聚贤：获得公开装备 | 3 | 4 | 获黑桃1赤兔、ban=false；目标保留两手牌。 |
| U01 玄论：确认 | 2 | 1 | 持技者7血、摸四张指定方块杀，后续选择flag=true。 |
| U01 玄论：取消 | 2 | 1 | 持技者7血、不摸牌、后续选择flag=false。 |
| L06 生克：弃杀 | 1 | 1 | 目标7血、剩桃。 |
| L06 生克：弃桃 | 1 | 1 | 目标7血、剩杀。 |
| L06 五行盘：杀→金 | 1 | 2 | 仅所选杀的五行属性=0。 |
| L06 五行盘：杀→土 | 1 | 2 | 仅所选杀的五行属性=4。 |
| L06 五行盘：桃→金 | 1 | 2 | 仅所选桃的五行属性=0。 |
| 合计 | 24 | 27 | U01为8分支/19次上游；L06为5分支/5次上游。 |

投影还核对每个座位的hp、手牌/装备牌面多重集合，保留花色、点数、五行属性；U01另外核对聚贤ban与玄论后续flag。名称归一仅显式映射原生horse_minus→chitu、wxl_wuxingpan_0→wuxingpan，五行字符串映射0至4；本次原生flag实际均为0/1，不依靠bool折叠异常值。未比较完整牌堆、弃牌堆、目标事件次序、全部存储、RNG、牌的唯一ID或手牌顺序。

暗槽点击必须按事后receipt解释：runner根据预定case选按钮索引0/1/2，未读取按钮牌面来决定点击。ordered的两个隐藏点击都获得红桃3桃，blind-repeat的两个都获得梅花7杀，全部保留；exact另有红/黑各一次。三次装备获得也全部保留。把black/red标签当作期望牌色会产生原first对照失败，不能据标签认定随机牌面不同就是规则不等价；相关首次投影失败和重复分组失败日志仍在原证据目录。

receipt分组仅检验已发生该获得结果后的局部结算，不验证选择概率、随机分布或跨引擎RNG相同。actual CDP request65与119的启动脚本差别是后者固定聚贤手牌DOM顺序并断言；exact的3条聚贤记录未固定此顺序。因此“同fixture”仅指相同牌面/设定角色输入，不能写成24次初始完整状态、隐藏索引或DOM排列完全一致。两版实际启动SHA分别为 `47c7bbb7f7c650e94d5d2ae25cf06b0ab7a34e9b4b59721cadcd2fedce4b5cb3`、`664750ee5c705077cf3f3d7188f19400463eeaef5a58b1dea04cd6fca526d467`。

事件入口采用真实引擎API：上游远振/玄论由useCard引出选择，聚贤从useSkill(top)进入获得；原生从PlayCard/ActivateSkill进入。上游聚贤top及五行盘的入口直接指定子技能/支付牌，未覆盖整个模式按钮、支付牌选择UI与全部取消入口；玄论确认只覆盖本次受伤摸牌及flag，不覆盖后续转化用牌。两端fixture主动清理/装配了状态并使用dummy角色，不能把这些探针称为完整产品流程或31将建局验收。

输入与存档边界：

- 对 [frozen-product-inputs.json](/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-11-13-queue1-review/frozen-product-inputs.json) 的32,259个文件逐一重新hash，00:28及00:30均无漂移，包括聚贤6文件和既有NGSA样本。原生探针源码SHA `d12cbac06b20e28f92a25d5885009bdb1f8cfb4bab645363b415643b697bd46d` 与临时example清理记录一致；实际输入清单见 [application-review-inputs.json](/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-11-13-queue1-review/independent/u01/application-review-inputs.json)。
- 实际现代上游临时副本的850个已复制文件与来源逐字相同，包括U01 skill.js、content.ts、player.js、standard.js。active-session-modern旧meta的upstream字段仍指向launcher默认liuli静态server；实际页面URL为5173，CDP实际导入现代TS模块，现代副本由modern-setup记录。溯源依实际URL、请求与副本，不能拿那个默认路径声称U01是在liuli运行。
- 27次是本次新生成挂起存档的encode/decode/restore，再对同一选择比较恢复前/回答后state_hash；确有断言，计数不是凭空填入。它们不是27份历史旧档，也未逐点比较完整事件日志/全部Snapshot字段。
- 既有 `u01-dddanzhi-pending-v107.ngsa` SHA `3158038e27eb46cffd15b4fc1eb4e378daa381cc9a01e16b96ef8f6656178736` 与HEAD字节完全相同；测试源码仍有旧档encode/decode、内容锁、目标续行断言。本次只核读，未重跑该测试，未看到真实修前聚贤gain新询问的旧档；不能由暗织旧档或27个新档推论全部历史U01存档兼容。

独立执行的检查是 `python3 -B .../application-review-check.py`：28项证据检查通过、0失败；另运行 `application-review-correction04-check.py` 核对具体31将描述及最终产品哈希。后者首跑额外断言“全清单仍等于correction04-after SHA”因其他来源的correction05并发更新而失败；这是复核脚本范围设得过宽，31将/64技能检查已先通过。首版脚本与失败保存在 `application-review-correction04-initial.py`、`application-review-correction04-first-failure.json`，收窄至授权U01范围后通过；未把首跑抹除或记成首跑全绿。

未发现本次U01清单应用或受限对照方法的新阻断；J01–J05及原逐技能表的其他已确认差异继续待做。无新增等价签署，无Cargo/浏览器重复执行，本报告不签其他来源、公共门禁、发布/CI或队列完成。
