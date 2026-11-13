import json
import pathlib

BASE = pathlib.Path(__file__).resolve().parent
comparison = json.loads((BASE / 'comparison.json').read_text())
rows = {r['case']: r for r in comparison['rows']}
notes = {
'y_target_A': ('医心选两张牌后选座1（2/4）', '双方座1回到4/4并取得无中106、桃49；座0弃9、42，限定已用。'),
'y_target_B': ('医心改选座2（1/6，缺5血）', '双方只回复4点到5/6，不摸牌；没有把缺5血当成回复5点。'),
'y_cost_B': ('医心费用改为手牌第二、三张', '双方弃42、46，座0保留杀9；座1仍回2血摸两张，与A费用不同。'),
'y_self': ('医心选择受伤的自己', '双方座0从2/4到4/4并摸两张；只对所选的自己结算。手牌排列未纳入比较。'),
'y_cancel_cards': ('点医心后在未选牌时点可见取消，再次打开取消', '双方未弃牌、未回血、未消耗限定；上游同一选择事件可重新点医心。原生两次激活的两个牌窗都保存恢复。'),
'y_cancel_target': ('上游选好两张牌但不选目标；点击仅剩的“结束回合”', '双方实体牌/体力/限定标记不变。上游结束本次chooseToUse，未使用隐藏取消；原生是可取消的独立目标窗，且重开过牌窗。该交互分段不相同。'),
'y_two_cards': ('座0恰好两张he牌', '上游不列医心候选；原生激活拒绝、没有询问。验证的是>2条件，不是>=2。'),
'y_no_wound': ('全八座满血', '上游可点医心但两卡后无可选受伤目标、不能确认；原生激活直接拒绝。牌与体力不变；入口问询顺序有差别。'),
'y_reuse': ('座0五手牌，第一次用医心后仍三手牌且有受伤目标，再试', '双方第二次不可发动；上游第二次真实chooseToUse不列医心，原生拒绝第二次ActivateSkill。'),
'y_no_skill': ('不给座0医心', '上游技能候选没有医心；原生激活拒绝。原有三张牌和体力不变。'),
'g_target_A': ('鬼斧选座1的青釭114', '双方弃青釭114，座0摸无中106、座1摸桃46；座2诸葛118保留。'),
'g_target_B': ('鬼斧改选座2的诸葛118', '双方弃诸葛118，座0摸无中106、座2摸桃46；座1青釭114保留。只覆盖每目标一件装备。'),
'g_cancel_target': ('点鬼斧，在目标选择取消，重开再取消', '双方两件装备不动、无人摸牌、使用次数未消耗；原生两个目标窗均恢复续答。'),
'g_no_equip': ('所有人无装备', '上游可点鬼斧但没有可选目标、无确定；原生直接拒绝激活。末态无变化，入口层次有差别。'),
'g_only_own': ('仅自己有青釭114', '上游不会把自己列为目标，原生拒绝激活；自有青釭没有被弃、无人摸牌。'),
'g_reuse': ('鬼斧先拆座1，再试拆仍有诸葛的座2', '双方本出牌阶段只成功一次；第二次上游不列鬼斧，原生激活拒绝。未实测下一回合刷新。'),
'g_no_skill': ('场上有两件装备，但不给鬼斧', '上游无鬼斧候选，原生激活拒绝；装备及牌堆不变。'),
'b_hit': ('座1杀座2，第三人座0预言能命中，座2不出闪', '双方目标4→3血，预言者取得无中106，挂上已预言标记；只有一次预测窗。'),
'b_miss': ('相同杀，预言不能命中；座2真实选择实体闪42', '双方座2不掉血，闪42与杀进入弃牌，座0摸无中106。原生预测与响应两个挂起点均恢复。'),
'b_wrong_A': ('预言能命中但座2出闪，失败弃座0的杀4', '双方弃杀4、保留桃46且不摸牌；目标不掉血。预测、响应、失败弃牌三窗均有原生恢复。'),
'b_wrong_B': ('同一预言失败，改弃桃46', '双方保留杀4、弃桃46，与A分支实际手牌不同；目标仍不掉血。'),
'b_wrong_miss': ('预言不能命中，目标无闪而命中', '双方目标掉1血，预言者弃杀4、不摸牌；验证另一种错误预言。'),
'b_cancel_predict': ('真实慧眼chooseControl点击DOM“取消”', 'HP/手牌同；上游result为{confirm:cancel,bool:false}，仍添加bolehuiyan4并logSkill(undefined)，原生空答不置used。已预言标记不同。'),
'b_cancel_discard': ('预言不能命中却命中，停在强制失败弃牌窗', '上游forced窗无可见取消；原生空答Rejected(InvalidChoice)仍pending。双方HP/手牌同，但杀9在上游ordering、原生discard；不能当完整状态相同或完成事件。'),
'b_empty_target': ('杀目标座2无手牌', '双方完全没有慧眼预测窗；杀仍命中使座2掉1血，座0不奖惩。'),
'b_owner_attacker': ('慧眼持有者自己出杀', '双方不提供自己的慧眼预测；座1掉1血，持有者不摸牌。'),
'b_owner_target': ('慧眼持有者自己成为杀目标', '双方不提供慧眼预测；持有者掉1血，原有两张牌不变。'),
'b_under_control': ('用上游原有_trueMe关系令座2受座0控制，再真实用杀', '上游isUnderControl为true，预测0窗且不摸牌；原生无对应控制关系字段，预测1窗并摸无中106。控制初态不完全相同，仅能作条件支持边界。'),
'b_repeat': ('座1持真实诸葛，两张实体杀连续命中同一目标；第一次已预言', '双方只出现第一次预测窗，座0只摸无中106一张，座2最终2/4血。没有跳过用杀计数。'),
'b_cancel_then_retry': ('诸葛连续两杀：第一张点DOM取消，第二张再尝试预言', '上游取消后已挂bolehuiyan4，只1窗、不摸牌；原生2窗，第二次选能命中后取得无中106。两边座2都到2/4血，牌堆/座0手牌不同。'),
'b_no_skill': ('第三人不授予慧眼，其他杀初态相同', '双方预测0窗，目标掉1血，座0未摸弃牌。'),
}
lines = ['# U08 逐场实际观察（2026-11-14）', '',
'输入为 HEAD `605201f3f49e10cea61c31d52a61bbb66c85599d` 加主 agent 已授权的酒诗恢复定义/测试变更；U08 产品输入哈希与开工一致。规则333／NGSA107／schema5／ABI759／WS8。最终上游 cohort 是 `browser-round4`，原生是 `native-round2`。这是运行证据表，没有新增等价签署。', '',
'31场中30个上游根事件结束，1场故意停在不可取消弃牌窗；runner异常0。表中“投影相同”仅指 comparison.json 明列的字段；21个初始弃牌堆泛型装备实体没有虚构上游映射。native pending列为直接路径数量，恢复路径同数，共76次恢复核验。', '',
'| case | 实际入口/选择 | 观察结果与限制 | 比较投影 | native pending | 原始上游记录 |',
'|---|---|---|---|---:|---|']
for name, r in rows.items():
    scenario, result = notes[name]
    outcome = '字段相同' if r['projection_equal'] else '字段不同'
    if name == 'b_under_control': outcome += '；控制初态不可完整比'
    if name == 'b_cancel_discard': outcome += '；仍挂起'
    lines.append(f"| `{name}` | {scenario} | {result} | {outcome} | {r['native_pending_count_direct']} | [JSON]({r['upstream']}) |")
lines += ['',
'初态逐座断言、关键storage/stat/控制关系核验在 [initial-validation-audit.json](initial-validation-audit.json)。完整原生请求/回包/日志在 [直接路径](native-round2/native-direct-commands.jsonl) 与 [恢复路径](native-round2/native-restored-commands.jsonl)；每场 JSON 还保留 initial、commands、final 与每个挂起点的 NGSA 文件名。', '',
'本轮没有运行医心装备费用、鬼斧同目标多装备选择、慧眼多目标/元素杀/防止伤害/额外伤害/结束边界和原生控制关联。未比较完整JS事件树与Rust机器栈、双方随机流、手牌或弃牌顺序、UI时延、原始DOM cardid和全产品阶段流转。牌堆顺序及映射实体ID/牌面已比较。']
(BASE / 'cases.md').write_text('\n'.join(lines) + '\n')
findings = {
 'date': '2026-11-14', 'source': 'U08', 'skills': ['yixin', 'bolehuiyan', 'guifu'],
 'implementation_agent': '/root/fix_dddanzhi', 'independent_review_performed_by_this_agent': False,
 'product_changed': False, 'new_equivalence_signoff': False,
 'summary': comparison['summary'],
 'findings': [
  {'id':'U08-RUNTIME-01','skill':'bolehuiyan','classification':'observed_rule_state_and_subsequent_outcome_difference',
   'cases':['b_cancel_predict','b_cancel_then_retry'],
   'observation':'真实DOM取消按钮调用上游通用control分支，返回confirm=cancel,bool=false且无control字段。技能只比较result.control !== cancel，故仍加bolehuiyan4、logSkill但storage是undefined。原生空答不置本回合used。第二张杀上游不再提问，原生可预言并摸id106。',
   'source_chain':[{'path':'../archive/upstream/noname/apps/core/extension/英雄杀/character/skill.js','lines':[2429,2460,2465,2469]}, {'path':'../archive/upstream/noname/apps/core/noname/ui/click/index.js','lines':[2982,3020,3167,3202]}],
   'native_chain':[{'path':'crates/engine-native-content/src/content/host/step/u08_batch_d.rs','lines':[607,609]}, {'path':'crates/engine-native-content/src/content/host/step/u08_batch_d_asks.rs','lines':[105,853,856]}],
   'runtime_evidence':['browser-round4/upstream-b_cancel_predict.json','browser-round4/upstream-b_cancel_then_retry.json','native-round2/native-b_cancel_predict.json','native-round2/native-b_cancel_then_retry.json'],
   'reproduction':'python3 browser-runner.py NEW_DIRECTORY b_cancel_predict b_cancel_then_retry （需先启动独占launch.mjs；新目录必须不存在）。使用已保留native-probe读取对应场景；不更改规则预期。',
   'limit':'这是实际源UI行为与原生的差别；本记录不把它改述为源意图，不改产品，不签等价。'},
  {'id':'U08-RUNTIME-02','skill':'bolehuiyan','classification':'observed_upstream_filter_and_native_control_context_gap',
   'cases':['b_under_control'],
   'observation':'上游原有_trueMe=持有者时isUnderControl返回true，真实shaBegin没有预测。原生CardUsed过滤没有控制关联输入，仍开预测并可摸牌。',
   'source_chain':[{'path':'../archive/upstream/noname/apps/core/extension/英雄杀/character/skill.js','lines':[2415]}, {'path':'../archive/upstream/noname/apps/core/noname/library/element/player.js','lines':[13304]}],
   'native_chain':[{'path':'crates/engine-native-content/src/content/host/step/u08_batch_d.rs','lines':[607,616]}],
   'runtime_evidence':['browser-round4/upstream-b_under_control.json','native-round2/native-b_under_control.json'],
   'limit':'原生没有对应控制关系，此场完整初态不可比。上游关系在初态直接设置既有_trueMe字段，未通过额外附身/控人技能产生。没有stub isUnderControl；只能说明这条条件与原生表达的边界，不能称同一完整对局的等价反例。'},
  {'id':'U08-RUNTIME-03','skill':'bolehuiyan','classification':'observed_pending_card_zone_timing_difference',
   'cases':['b_cancel_discard'],
   'observation':'预言错误的强制弃牌窗都不可取消；原生空答拒绝并保持pending。但该时刻源杀9仍在ordering，原生杀9已在discard。双方全部映射实体仍守恒、HP和手牌同，不能称完整挂起状态一致。',
   'runtime_evidence':['browser-round4/upstream-b_cancel_discard.json','native-round2/native-b_cancel_discard.json','native-round2/b_cancel_discard-direct-pending-1.ngsa','native-round2/b_cancel_discard-restored-pending-1.ngsa'],
   'limit':'没有运行对此区位差异敏感的第三方触发；不推断其他技能结果。该场没有结束事件，也没有点击隐藏取消。'},
  {'id':'U08-RUNTIME-04','skill':'yixin','classification':'observed_interaction_staging_boundary',
   'cases':['y_cancel_target'],
   'observation':'源在同一chooseToUse中合并费用与目标。选两牌后无确认取消，仍有结束回合控件；实际DOM点击后本次选择/root结束，牌和限定未消耗。原生单独费用窗后目标窗可空答取消，还允许重开激活。',
   'runtime_evidence':['browser-round3/upstream-y_cancel_target.json','browser-round4/upstream-y_cancel_target.json','native-round2/native-y_cancel_target.json'],
   'limit':'只比较实际可见控件；未测试反选已选卡/右键等其它UI退出方式，未运行完整自然phaseUse循环。不据此单独认定医心结算规则错误。'},
  {'id':'U08-RUNTIME-05','skills':['yixin','guifu'],'classification':'observed_activation_entry_staging_boundary',
   'cases':['y_no_wound','g_no_equip','g_only_own'],
   'observation':'源保留技能候选，点入后没有合法目标且不能确认；原生ActivateSkill直接拒绝，没有pending。实体牌和HP投影均无变化。',
   'limit':'入口UI/问询分段差别，与成功结算的效果相同是两件事；本轮不签这些技能或整来源等价。'}
 ],
 'native_restore':{'cases':31,'checkpoint_attempts':76,'restore_errors':0,'state_hash_equal':76,'complete_snapshot_equal_except_auth_generation':76,'final_complete_debug_snapshots_equal_except_auth_generation':31,'command_request_reply_log_equal':31,'raw_final_ngsa_equal_cases':10,'raw_final_ngsa_different_due_auth_generation_cases':21,'evidence':'native-validation.json'},
 'unrun_scope':['U08其余技能和整来源签署','医心装备费用、恰缺4血、非正常弃置保护和嵌套触发','鬼斧多装备卡牌选择、装备弃置第三方触发、下一阶段次数刷新','慧眼多目标/元素杀、伤害免疫与额外伤害、真实phaseAfter到期、复杂控人技能形成关系','双方完整事件树/随机流/完整产品阶段、AI策略、现代UI视觉对照、安装包/真机'],
 'earlier_rounds':{'native-round1':'原始main多余括号编译失败；源码native-first-source.rs与raw保留。native-round2只修probe语法，产品未改。','browser-round1':'4场初态闪电nature核验失败；原始返回保留，未冒充成功。','browser-round2':'四个代表正例真实执行；确认显式闪电雷属性归一化后再做完整cohort。','browser-round3':'31场；医心目标取消后runner错误地预期回同一选择，实为可见结束控件结束root。原记录保留。慧眼取消差异首次确认。','browser-round4':'31场最终cohort；真实HTMLElement.click控制按钮，医心按实际控件记录，不再假设重开。30结束/1有意强制挂起，0runner错误。'}
}
(BASE / 'findings.json').write_text(json.dumps(findings, ensure_ascii=False, indent=2) + '\n')
