#!/usr/bin/env python3
from pathlib import Path
import collections,datetime,hashlib,json,subprocess
OUT=Path(__file__).resolve().parent;BASE=OUT.parent.parent/'L28'
def readj(p):return json.loads(p.read_text())
def readl(p):return [json.loads(x)for x in p.read_text().splitlines()]
index=readj(BASE/'raw-input-index.json');raw=readj(OUT/'raw-integrity-audit.json');proj=readj(OUT/'projection-audit.json')
browser={r['case']:r for r in readl(BASE/'browser/round8-clean/case-results.jsonl')};native={r['case']:r for r in readl(BASE/'native/round2.jsonl')}
replay=readl(OUT/'native-replay-first.jsonl');byreplay=collections.defaultdict(list)
for line,r in enumerate(replay,1):
 if r.get('case'):byreplay[r['case']].append(line)
keys={name:f'L28:skills:{name}@archive/upstream/noname-liuli/noname/extension/雷霆万钧/extension.js:{line}'for name,line in [('th_buxu',6372),('th_zhengjing',6018),('th_yeyan',8636),('th_zhengjing2',6162)]}
# Each observation below was written after independently checking the actual raw state/DOM and native log.
manual={
'buxu-basic':('有限投影一致','手牌0/30/45中弃0，实际六经按钮1取牌堆首张基本牌1（黑桃8杀），费用1且skillStat增为1。','费用选Card(0)、Tag(1)，取1至th_liujing并置基本类mask；两问恢复均核过。','类别1与本案实体结果相符；不覆盖找不到该类、弃牌堆检索或自然回合刷新。'),
'buxu-equip':('类别结算不一致','弃0后点实际按钮5，取得79，使用上游绝影的真实装备定义；79的黑桃1牌面不在原牌表。','Tag(5)仍取牌堆顶1（杀），却置装备类mask。','可证明装备类别选择未由原生保留；79不能认证为官方原实体，完整实体对照待合法元组重采。'),
'buxu-nullify':('类别结算不一致','弃0后点按钮2，跨过牌堆前面的杀，取得70（黑桃J无懈，原牌表存在）。','同费用及Tag(2)却取得1（黑桃8杀），并标记无懈类已补。','实际用到的费用0、经典70/1均有原牌表元组；类别A/B失效不是由off-path通用装备引起。'),
'buxu-cancel':('有限投影一致','进入补叙费用选择即点击真实取消；0/30/45仍在手，弃牌/扩展区空，技能统计未增加。','首问空selection被接受并退回发动；无弃牌或经典，只有一次pending。','证实首问取消的无效果投影；分类窗本来无取消按钮，不把它补造为可取消。'),
'buxu-empty':('有限投影一致','所有座位空手；本地出牌阶段没有th_buxu可用技能。','激活拒绝且没有开放询问，HP及牌区未变化。','只验证he不足1的条件；不等于所有费用保护/不能弃置情况已测。'),
'buxu-full':('有限投影一致','六张th_chengye扩展牌已在；真实filter因数量6而不提供补叙。','mask63及liujing_count6使激活无询问。','可证六张满额数量条件；其中89的通用武器映射不是原牌表元组，六张实体组合不作官方认证。'),
'buxu-twice':('类别结算不一致','第一次弃0选基本得1；同一真实phaseUse再弃30/45选装备得79，费用由1变2，stat最终2。','两次费用同为1/2，却分别把牌堆顶1、2两张杀作基本/装备；四次pending恢复可重放。','递增费用在此正常统计段相符，第二次类别查牌不同；79映射限制单列。两次发动之间还有真实ActivateSkill动作。'),
'buxu-invalid':('有限投影一致','真实DOM仅有0..5六经按钮，没有id6；随后真实点1得到经典1。','Tag(6)实际答复Rejected(InvalidChoice)且询问保持；再Tag(1)得经典1。','比较候选边界和后续合法结果；上游没有发送不存在的id6，因此不是双侧同一非法输入回包测试。'),
'zhengjing-one-self':('源入口阻塞','fresh页真正phaseUse/chooseToUse第一次调用原filter时抛ReferenceError: dcdAnim is not defined；尚无小游戏/目标窗。','Tag(1)拿顶牌0为自己的经，并授th_zhengjing2；两次pending及最终档已恢复。','只验证原生分支和上游真实入口异常，不能称上游成功整理一张并分给自己。'),
'zhengjing-three-target':('源入口阻塞','首次原filter同样因dcdAnim不存在中止；未选择目标/分配卡。','Tag(3)取0/1/2三张同名杀，0给座1作经，1/2给自己；两问恢复通过。','原生轨迹可复核；上游应按不同牌名及小游戏成功结果取牌、自由分配的结论来自逐行源码。'),
'zhengjing-zero':('源入口阻塞','原filter真实异常发生在小游戏之前；本案没有上游零成功结果。','Tag(0)仍置0为自己的经并摸1（牌1），boom旗被置；一次pending恢复。','与上游小游戏无成功bool:false即结束的具体差异由源码确认，不能把0计数当原小游戏实跑。'),
'zhengjing-cancel':('源入口阻塞','真实filter异常在取消选项可出现之前发生；本案没有上游取消操作。','原生count首问空答后结束，不取牌、不授辅助技能；一次pending恢复。','原生取消得到证明，上游取消分支仍未实际触发，不能把本案标双侧取消成功。'),
'zhengjing-no-skill':('有限投影一致','初态不持有整经，phaseUse选择列表不含整经，未碰其有副作用filter。','未持技能时激活拒绝且无pending、无牌区变化。','只验证技能未持有反例，不能代替usable1已用后的反例或小游戏条件。'),
'yeyan-small-one':('有限投影一致','真实拖一个火到座1并确认，座1由8到7；0的四张费用牌仍在手。','mode0、Player(1)产生1火伤，无三血/弃牌费用；两问恢复。','费用/目标/单次火伤投影相符；未含濒死、连环或其他触发技能。'),
'yeyan-small-three':('结算时序不一致','依次将三个火分给1/2/3；event_history实际伤害按1→2→3，每次1火伤。','mode0选1/2/3，实际DamageDealt日志按3→2→1。','三人最终HP都7仍不能作时序一致；原生栈逆序与源码push/last落点相符。'),
'yeyan-great-one':('结算时序不一致','strengthen_skill=forbid，三火给1，0/7/21/24自动选四花色费用；先0失3血，再座1受3火伤。','费用与最终HP同为0=5、1=5，但DamageDealt(1,3)先于HpLost(0,3)。','费用四张均为真实原牌表元组；该已证时序反例不受闲置通用牌映射影响。'),
'yeyan-great-split':('结算时序不一致','forbid配置，火分给3两点、1一点；实际先0失3血，再按座次1受1、3受2。','选择顺序Player(3),Player(1)保留2+1分配，但实际先3受2，再1受1，最后0失3血。','最终0=5、1=7、3=6相同，伤害顺序和费用时点均不同。'),
'yeyan-cancel':('有限投影一致','业炎真实火焰UI尚未分配时可取消，HP和四张手牌保持，未觉醒。','mode首问空答后退出并退还once_used；一问恢复。','只对取消无费用/无伤害/限定未消耗的投影作确认。'),
'yeyan-used':('有限投影一致','初态th_yeyan已觉醒，真实出牌列表不再有业炎。','初态used旗和once_used均已设，激活不产生询问或效果。','验证限定已用反例；不是自然发动后下一回合全过程测试。'),
'yeyan-no-four':('有限投影一致','仅手牌0黑桃杀，第二火拖同一人未获分配，仍只有座1一个火；确认造成1火伤且不付费用。','mode候选只有0；答1实际被InvalidChoice拒绝仍pending，随后选0及座1造成1火伤。','验证无四花色时不能叠火的条件；上游拖动失败与原生非法Tag拒绝是不同输入形态。'),
'yeyan-target-cancel':('有限投影一致','已拖一火到1但确认区仍oc，真实点击取消后没有HP/弃牌变化或觉醒。','选mode0后在目标窗空答退出，限定未耗；两问恢复。','有分配后的取消无效果投影相符，原生的分段目标窗与上游拖火UI不是同一状态结构。'),
'yeyan-great-discard-cancel':('取消交互分段不同','叠到三火时四张费用已自动选中且只有o、没有c；真实点头像撤到一火后才有oc，再取消，最终无费用/伤害。','大模式目标后独立弃牌窗允许空selection直接取消，原有手牌/HP不变；三问恢复。','两边最终取消无效果相符；不能写成双方都能在自动选四牌时直接取消。源实际取消前仍留一火。'),
'yeyan-great-one-default':('配置维度不一致','真实localStorage默认null，原getStrength(true,false)=false；付四牌、1受3火伤，0保持8。','复用已跑great-one原生分支，原生无该配置输入且0总失3血至5。','不是完全相同的配置初态；证明上游默认配置维度及原生缺少对应结果，未新增一次native档。'),
'yeyan-great-split-default':('配置维度不一致','默认null配置下0不失血，座次1受1后3受2，HP为0=8、1=7、3=6。','复用原生great-split：3受2、1受1、0再失3血至5。','费用配置和结算顺序都不同；native没有独立strengthen_skill输入，不能称同完整初态。'),
'yeyan-great-two':('分配能力不具备','真实只把两个火拖给座1，第三火未用，forbid配置付四牌/3血，座1只受2至6。','最近可表达的great-one分支对单人固定3火伤至5，候选/stage没有二火计数。','这不是双侧同一已可表达的命令；实际源分支加原生源码/候选证明缺少只分2火的表达。'),
}
case_rows=[]
for entry,projection,integrity in zip(index['cases'],proj['case_checks'],raw['case_checks']):
    name=entry['case'];b=browser[name];n=native[entry['fixture_case_in_native']];status,source_obs,native_obs,limit=manual[name]
    skill=n['skill'];same_native=name!=n['case']
    case_rows.append({'case':name,'inventory_key':keys[skill],'evidence_level':'双侧真实技能分支/限定投影'if b['completed']and not same_native else '上游真实入口失败+原生实际运行+源码替代'if not b['completed']else '上游真实分支+原生最近可表达分支/源码候选',
      'review_result':status,'upstream_observation':source_obs,'native_observation':native_obs,'limit':limit,
      'raw':{'browser_jsonl':entry['browser_jsonl'],'browser_line':entry['browser_line'],'native_jsonl':entry['native_jsonl'],'native_line':entry['native_line'],'native_case':n['case'],'native_reused_by_extra_browser_case':same_native,'cdp_request_reply_lines':[x['request_line']for x in entry['request_reply_rows']],'independent_native_replay_jsonl_lines':byreplay[n['case']]},
      'initial_assertions':{'eight_seats':integrity['seats_checked']==8,'source_snapshot_raw_binding':not integrity['snapshot_binding_errors'],'initial_projection_equal_after_only_intrinsic_lightning_nature_normalization':projection['initial_equal_after_only_shandian_intrinsic_nature_normalization'],'initial_unique125_native_ids':integrity['physical_ids_count']==125 and integrity['initial_unique_ids'],'final_no_null_or_foreign_pile_cards':not integrity['final_has_null_ids']and not integrity['foreign_pile_mutations'],'browser_and_native_physical_ids_conserved':integrity['browser_physical_ids_conserved']and integrity['native_physical_ids_conserved']},
      'projection':projection,'unique_native_pending_answers':len(n['choices']),'additional_native_run_for_extra_browser_case':False if same_native else None})
(OUT/'case-review.json').write_text(json.dumps(case_rows,ensure_ascii=False,indent=2)+'\n')
findings=[
 {'id':'L28-R1','inventory_key':keys['th_buxu'],'classification':'已观察规则差异','severity':'high','recommendation':'维持或改为简化近似；不签等价','cases':['buxu-equip','buxu-nullify','buxu-twice'],
  'finding':'分类按钮未决定原生取牌类别，原生总deck.pop并设置所选类别mask。','upstream':'extension.js:6187–6197的六类过滤及6372–6514补叙；game.js:63172先全牌堆、再弃牌堆检索。','native':'l28_batch_a_asks.rs:702–750丢费用、选Tag后无条件deck.pop。','runtime_proof':'buxu-nullify源实际70黑桃J无懈（真实原牌表元组），原生1黑桃8杀；基本/无懈按钮A/B在原生得同一顶牌。','limits':'装备分支的79为原生通用映射黑桃1绝影，官方原实体不可认证；不影响无懈分支的类别反例。','unrun_repro_design':'用真实牌表的杀/无懈/桃建受控牌区，让牌堆仅有杀、弃牌堆有无懈后选礼；另测牌堆/弃牌均无该类时费用已付但原getStat次数回退。尚未运行，不纳入已测计数。'},
 {'id':'L28-R2','inventory_key':keys['th_yeyan'],'classification':'已观察规则差异','severity':'high','recommendation':'维持或改为简化近似；不签等价','cases':['yeyan-small-three','yeyan-great-one','yeyan-great-split'],
  'finding':'原生伤害顺序倒转；大业炎失去3体力晚于伤害。','upstream':'extension.js:8889–8915先discard、配置允许时loseHp，再按座次damage；game.js:41695逐个shift子事件。','native':'l28_batch_b_asks.rs:399–420依次emit而machine.rs:403/resolution.rs:130压栈，resolution.rs:632读stack.last。','runtime_proof':'源小业炎伤害1→2→3，native日志3→2→1；源forbid大分配顺序loseHp0→damage1(1)→damage3(2)，native damage3(2)→damage1(1)→loseHp0。','limits':'八座HP8，无濒死/死亡/连环/其他触发，不推断这些后果已经实测；四费用牌0/7/21/24均有真实原牌表元组。','unrun_repro_design':'以真实四花色手牌、业炎者HP3及伤害后反击/死亡相关技能测费用引发濒死先后，保留完整事件轨迹；本批未运行。'},
 {'id':'L28-R3','inventory_key':keys['th_yeyan'],'classification':'已观察配置/规则差异','severity':'high','recommendation':'与同key合并保留简化近似','cases':['yeyan-great-one-default','yeyan-great-split-default'],
  'finding':'原生无strengthen_skill维度，大业炎无条件失3血；原上游默认强化配置不付3血。','upstream':'liuli game.js:249–267的真实getStrength；extension.js:8894附近只在getStrength(true,false)为真时loseHp。','native':'l28_batch_b_asks.rs:399–407 great无条件emit LoseHp。','runtime_proof':'真实localStorage null→getStrength=false，源两场0保持8；原生近似分支均0降5。','limits':'native没有对应配置初态，因此不是两个完整配置状态相同的对答；源forbid/default真实配置两支均已采集。'},
 {'id':'L28-R4','inventory_key':keys['th_yeyan'],'classification':'已观察表达能力差异','severity':'medium','recommendation':'与同key合并保留简化近似','cases':['yeyan-great-two'],
  'finding':'原生不能只给单目标2火并留第三火未分配。','upstream':'真实火焰UI允许任意未分配火；原content以targets非空和result.bool确认，伤害数取每人的实际火焰数组长度。','native':'l28_batch_b_asks.rs:391–396大模式一个目标固定3，多目标首位2其余1；mode/targets/cost窗无2火值。','runtime_proof':'源两火拖给1且第三未用，目标8→6；最近的native单人大业炎为8→5。','limits':'无对应native命令，不能称同一命令回包不一致；原生分支缺失由候选和实际源码明确确认。'},
 {'id':'L28-R5','inventory_key':keys['th_zhengjing'],'classification':'源入口已实际阻塞；规则差异由源码确认','severity':'high','recommendation':'维持或改为简化近似；不签等价','cases':['zhengjing-one-self','zhengjing-three-target','zhengjing-zero','zhengjing-cancel'],
  'finding':'原生用人工0..5询问代替真实切水果、固定首张作经；0仍添经并摸牌。','upstream':'extension.js:6018–6161中按不同牌名且非毒检索、真实thunderFruit、无成功结束、强制目标、chooseToMove至少一张作经自由分配；thunderFruit结果代码同文件:475–548。','native':'l28_batch_a_asks.rs:581–625 count取顶N，固定first作经，其余归自己；count0添顶牌经再摸1。','runtime_proof':'四场都由真实phaseUse/chooseToUse首次filter抛dcdAnim ReferenceError；并未运行上游小游戏。native count3实取同名杀0/1/2，0固定给目标、1/2给自己；count0仍授辅助技能/取经/摸牌，全部保存恢复可核。','limits':'遵守裁决35，未读/加载官方Spine，未stub或提前置zhengjingxiao；未取得上游小游戏成功/失败/取消/自由分配的runtime证据。关联th_zhengjing2不因本结论自动签署或撤回。'},
 {'id':'L28-I1','inventory_key':keys['th_yeyan'],'classification':'已观察交互分段边界','severity':'medium','recommendation':'记录交互差异；不单独新增等价或撤回计数','cases':['yeyan-great-discard-cancel'],
  'finding':'源真实叠火自动选四牌后只有OK；撤火至一火再取消。native独立弃牌窗可直接空答取消。','runtime_proof':'source DOM确认串从o经撤火变oc，最终无费用/觉醒/伤害；nativediscard窗allow_cancel=true，直接空答退出。','limits':'最终无效果投影相符，原场景名称不能当成源直接取消四牌窗的证明。'},
]
report={'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'reviewer':'/root/fix_dddanzhi','head':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'versions':{'rule':333,'NGSA':107,'schema':5,'ABI':759,'WS':8},
 'source_id':'L28','sample_keys':[keys[x]for x in ['th_buxu','th_zhengjing','th_yeyan']],'related_key_not_runtime_reviewed':keys['th_zhengjing2'],
 'case_counts':dict(collections.Counter(r['review_result']for r in case_rows)),'raw_counts':{'browser_cases':25,'source_branch_completed':21,'source_original_filter_errors':4,'native_cases':22,'unique_native_pending':39,'native_final_archives':22},
 'findings':findings,'signed_equivalent_items':[],
 'not_a_full_source_audit':'仅复核这三技25个已采集场景和所需源链，不覆盖L28全部inventory条目；有限投影一致行不签为整个技能等价。',
 'fixture_boundaries':{'strict_initial_projection_matches_before_normalization':0,'after_only_intrinsic_lightning_nature_normalization':25,'native_catalog_fixture_count':125,'original_standard_extra_pile_count':161,'native_faces_not_original_pile_tuples':proj['native_faces_absent_from_official_standard_extra_and_real_th_taipingyaoshu'],
 'affected_interaction_cases':{'buxu-equip':'取得79黑桃1绝影，不作官方实体认证；仍可看到该真实定义属装备而native拿杀。','buxu-twice':'第二次同79，限于递增费用和类别差异观察。','buxu-full':'初态扩展89黑桃1青釭非原元组，满6反例仅核数量。'},
 'all_other_cases':'21个非原元组多为闲置牌堆元素，不能称完整牌堆同；逐案没有触及这些卡的费用/HP/伤害时序观察仍保留。'},
 'native_replay':{'pending_restore_before_full_snapshot_except_auth':39,'answer_hash_outcome_logs_same':39,'answer_after_full_snapshot_same_to_next_or_final_except_auth':38,'separate_activation_boundary':{'case':'buxu-twice','choice_index':1,'reason':'回答第一轮分类后采集者再提交第二次ActivateSkill，所以下一个pending不是直接后的完整快照；仍核当前应答hash/日志/回包，不把缺失的直接后NGSA编成已比较。'},'final_restore_full_snapshot_except_auth':22,'failures':0,'complete_upstream_state_events_rng_compared':False},
 'review_source_rule':'先源码后冻结raw；未读取采集者README、findings或比较器决定预期。完整manifest仅按字节hash核对，未将报告文字用作规则证据。'}
(OUT/'findings.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
lines=['# L28 独立逐场复核（2026-11-14）','',f'HEAD `{report["head"]}`；规则333／NGSA107／schema5／ABI759／WS8。25场全部纳入，没有去重或择优；21场完成目标技能分支，4场真实filter异常。22个原生场景独立重放39个pending、22个final档。下表的“有限投影一致”只表示列出的分支结果。','', '| 场景 | 独立判断 | 上游实际观察 | 原生实际观察与限制 | 原始行（browser/native） |','| --- | --- | --- | --- | --- |']
for r in case_rows:
 lines.append('| '+ ' | '.join([r['case'],r['review_result'],r['upstream_observation'],r['native_observation']+' '+r['limit'],str(r['raw']['browser_line'])+'/'+str(r['raw']['native_line'])])+' |')
lines+=['','全部初态的八座HP/maxHP/状态、手装判、技能、限定已用、映射扩展区、完整125个native-ID牌区及顺序均核；严格逐牌首轮因两张闪电固有thunder差异为0/25相符，单独声明此归一化后25/25匹配。该投影建立在native→原牌名映射上，不消除21个非原牌表元组；装备取得79和初态89的限制已逐行列出。所有25场最终无外来牌/空ID，双方物理ID守恒；这些是夹具证据，不等于整副原牌堆或完整state/events/RNG一致。','', '原始数据及精确CDP行在 [case-review.json](case-review.json) 和 [raw-snapshot-index.json](raw-snapshot-index.json)。规则差异、精确inventory键与未测边界见 [findings.json](findings.json)。']
(OUT/'case-review.md').write_text('\n'.join(lines)+'\n')
print(json.dumps({'case_rows':len(case_rows),'counts':report['case_counts'],'findings':len(findings),'signed_equivalent_items':0},ensure_ascii=False))
