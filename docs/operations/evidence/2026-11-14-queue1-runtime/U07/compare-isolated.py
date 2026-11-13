"""Compare only the recorded player/card/deck projection, never whole engines."""
from pathlib import Path
import datetime,hashlib,json,os
BASE=Path(__file__).resolve().parent
native=[json.loads(l)for l in (BASE/'native/08-isolated.log').read_text().splitlines()if l.startswith('{')]
rows={x['case']:x for x in map(json.loads,(BASE/'browser/case-results-eleventh.jsonl').read_text().splitlines())}
assert len(native)==len(rows)==24

def player(p):
 return {k:p[k] for k in ['seat','hp','max_hp','alive','linked','face_down','hujia','fuchou_marked']}|{'hand':sorted(c['id']for c in p['hand']),'equips':sorted(c['id']for c in p['equips']),'judge':sorted(c['id']for c in p['judge']),'skills':sorted('sanjian_skill' if k=='wd_sanjian' else k for k in p['skills'])}
def projection(s):
 return {'players':[player(p)for p in s['players']],'deck_draw_order':[c['id']for c in s['deck_draw_order']],'discard_multiset':sorted(c['id']for c in s['discard']),'table_multiset':sorted(c['id']for c in s['table'])}
def changes(a,b,prefix=''):
 if isinstance(a,dict)and isinstance(b,dict):
  return [x for k in a for x in changes(a[k],b[k],prefix+'.'+k)]
 if isinstance(a,list)and isinstance(b,list)and len(a)==len(b)and a and isinstance(a[0],dict):
  return [x for i in range(len(a))for x in changes(a[i],b[i],prefix+f'[{i}]')]
 return []if a==b else [{'field':prefix.lstrip('.'),'upstream':a,'native':b}]
notes={
'choudu-top-return':'上游选定角色在最后展示池窗口选1张放回牌堆顶；原生没有此窗口，余牌全弃。',
'choudu-target1':'选手牌成本、角色1，所有换牌和放顶均取消；终态可相同，上游另询问角色2和末尾放顶，原生只询问角色1换出牌。',
'choudu-target2':'选手牌成本、角色2；所有后续选择取消。上游仍询问其他有手牌角色及末尾放顶，原生只询问角色2。',
'choudu-swap-a':'角色1换出闪并选入首张展示杀；其他角色与末尾放顶取消。当前终态投影相同，原生缺选入展示牌窗口。',
'choudu-swap-b':'角色1换出同一桃，上游选择展示池index2的黑桃10杀；原生仅能选换出牌且固定取得index0的黑桃9杀。',
'choudu-equip-target1':'弃已装备三尖两刃刀、选角色1，后续全取消。上游还询问角色0、2和末尾放顶，原生只询问角色1。',
'choudu-cancel-card':'上游进入筹度后取消并退出本次出牌选择；原生在选成本窗口取消，均保留手牌和装备。',
'choudu-cancel-target':'上游已选成本后取消联合选牌/选目标；原生选成本后在目标窗口取消，均没有实际支付成本。',
'choudu-empty':'上游无he牌，出牌选择中没有可支付筹度成本并退出；原生ActivateSkill被InvalidChoice拒绝，无筹度结算。未据此覆盖其他filterTarget条件。',
'fuchou-hand':'交手牌闪给攻击者并取消原杀。上游原杀进入弃牌堆；原生把原杀额外加入攻击者手牌。',
'fuchou-equip':'交已装备三尖两刃刀给攻击者并取消原杀。两边卸装；上游原杀弃置，原生把原杀额外加入攻击者手牌。',
'fuchou-cancel':'取消交牌，原杀照常命中且不登记复仇来源；本次终态投影相同。',
'fuchou-empty':'目标无he牌，未出现交牌选择，原杀命中；本次终态投影相同。',
'fuchou-settle-no-sha':'交闪后触发phaseAfter，复仇者摸到桃、无杀可用。上游复仇者失1HP；原生自动虚拟杀伤攻击者1HP，未要求实际杀/未令复仇者失HP。',
'fuchou-settle-draw-sha':'交闪后初手无杀；phaseAfter调度摸到杀，但上游旧step在调度draw同一步先检查hasSha，实际未创建出杀选择并失1HP；原生自动虚拟杀伤攻击者1HP。本案不计已执行的出杀取消。',
'fuchou-settle-have-sha-cancel':'初手有备用杀，phaseAfter实际出现chooseToUse；玩家取消，复仇者失1HP并保留备用杀与新摸桃。原生自动虚拟杀伤攻击者1HP，没有相应确认窗口。',
'fuchou-settle-have-sha-use':'初手有备用杀，phaseAfter实际选择该杀攻击标记来源，来源取消闪响应。上游消耗备用杀并伤来源1HP；原生同样伤来源但保留备用杀并使用虚拟杀，且仍多返还原杀。',

'youzhan-hand-stop':'弃手牌装备；攻击者无备用杀。上游取消原杀并只令攻击者受1伤；原生还令原目标承受原杀伤害。',
'youzhan-equip-stop':'弃已装备武器；攻击者无备用杀。上游取消原杀并只令攻击者受1伤；原生还令原目标承受原杀伤害。',
'youzhan-cancel':'取消弃装备，原杀正常命中；本次终态投影相同。',
'youzhan-far':'8人局角色0持技能，角色1对2出杀，0到2距离2；诱战未触发，本次终态投影相同。',
'youzhan-no-equip':'技能拥有者he无装备牌，诱战未触发；原杀正常命中，本次终态投影相同。',
'youzhan-counter-use':'支付手牌装备后攻击者实际再出1张杀；再次诱战取消，目标无闪。按实际终态列差别，未声称原杀取消路径等价。',
'youzhan-counter-dodge-stop':'反击杀被目标手牌闪抵消，再次诱战取消。上游随后重新要求攻击者出杀，取消后攻击者受伤；原生无对应重复出杀窗口。',
}
results=[];archives=[]
for n in native:
 name=n['case'];b=rows[name]
 assert not b['result']['foreignPileMutations'],name+' foreign pile mutation'
 assert all(c['id'] is not None for zone in ['deck_draw_order','discard','table'] for c in b['result']['final'][zone]),name+' unknown final card'
 assert b['success'] and b['readiness']['players']==8 and b['initial_player_projection_matches']and b['initial_draw_order_matches'],name
 assert projection(b['setup']['initial'])==projection(n['initial']),name+' initial expanded mismatch'
 assert all(len(p['virtualEquips'])==len(p['equips'])and not p['virtualJudges']for p in b['setup']['initial']['players'])
 d=changes(projection(b['result']['final']),projection(n['final']))
 for c in n['choices']:
  assert c['resume_state_hash_log_equal']
  p=Path(c['archive_file']);assert p.exists()and p.stat().st_size==c['archive_bytes']
  archives.append({'case':name,'prompt':c['prompt'],'observer':c['observer'],'selection':c['selection'],'path':os.path.relpath(p,BASE),'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'original_restore_answer_hash_and_new_log_equal':True})
 r={'case':name,'upstream_run':b['run'],'initial_projection_equal':True,'initial_residual_virtual_equipment_or_judge':False,'upstream_original_choice_constructors':b['result']['choiceConstructors'],'upstream_manual_decision_states':b['choices'],'native_choices':[{k:c[k]for k in ['prompt','observer','min','max','allow_cancel','options','selection','archive_bytes','archive_file','resume_state_hash_log_equal']}for c in n['choices']],'native_ngsa_roundtrips':len(n['choices']),'final_projection_equal':not d,'final_projection_differences':d,'note':notes[name]}
 results.append(r)
source_checks=[]
for pre,post in [('native-08-before.json','native-08-after.json'),('browser-isolated-before.json','browser-isolated-after.json')]:
 a=json.loads((BASE/pre).read_text());z=json.loads((BASE/post).read_text());diff=[k for k in a['sha256']if a['sha256'][k]!=z['sha256'][k]]
 source=[k for k in a['sha256']if '/target/debug/'not in k]
 assert all(a['sha256'][k]==z['sha256'][k]for k in source)
 source_checks.append({'before':pre,'after':post,'head_equal':a['head']==z['head'],'head':a['head'],'entries':len(a['sha256']),'input_files':len(source),'input_hashes_all_equal':True,'changed_entries':diff,'binary_sha256_after':z['sha256'][str(BASE.parents[4]/'target/debug/queue1-u07-runtime-probe')]})
summary={'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'cases':len(results),'native_pending_ngsa_roundtrips':len(archives),'final_projection_equal_cases':sum(r['final_projection_equal']for r in results),'final_projection_different_cases':sum(not r['final_projection_equal']for r in results),'all_initial_projections_equal':True,'source_cohorts':source_checks,'new_equivalence_signoff':False,'uncompared':['full event/continuation/RNG identity across engines','initial identities/generals and AI values','hand/discard physical ordering','unused deck suffix rule semantics (generic catalog equipment aliases are harness-only)','installed extension/product/UI acceptance','all unexecuted skill branches']}
for name,obj in [('comparison-isolated.json',{'summary':summary,'cases':results}),('native/archive-manifest-isolated.json',{'count':len(archives),'archives':archives})]:
 with (BASE/name).open('x')as out:out.write(json.dumps(obj,ensure_ascii=False,indent=2)+'\n')
lines=['# U07 三交互技能实际运行对照·排除启动尾牌追加（2026-11-14）','',f"原生 {len(results)} 案；浏览器逐案刷新并实际结算 {len(results)} 案；{len(archives)} 个原生待答窗口保存 NGSA 后编码/解码恢复，同答后的 state_hash 与新增日志一致。初态核对全部通过。终态投影相同 {summary['final_projection_equal_cases']} 案、存在差别 {summary['final_projection_different_cases']} 案；相同只限下述投影，不签新增等价。",'', '| case | 初态 | 实际选择及结果 | 终态投影 | 原生 NGSA 窗口 |','| --- | --- | --- | --- | --- |']
for r in results:lines.append(f"| {r['case']} | 匹配 | {r['note']} | {'相同' if r['final_projection_equal']else'不同'} | {r['native_ngsa_roundtrips']} |")
lines+=['','投影逐玩家核对 seat、HP/maxHP、alive、linked、face_down、护甲、手牌/装备/判定牌 ID 集、技能名（wd_sanjian→sanjian_skill）与复仇来源标记；另比牌堆 ID 顺序、弃牌/处理中牌 ID 多重集。浏览器额外核对虚拟装备数量与实体装备一致、虚拟判定为空。原始 face、区域顺序、提示和选项都保留在 JSON 中。','', '未比较身份/武将、跨引擎完整事件和随机流、手牌/弃牌物理顺序、未抽到的牌堆后缀规则。原生通用装备/坐骑后缀的映射仅为夹具载入；本次实际抽到的前缀均为杀/闪/桃。armor→bagua 只作为手牌装备成本使用，没有据此证明防具效果。','', '[逐字段差异与原始选择](comparison-isolated.json)；[43份原始NGSA及SHA-256](native/archive-manifest-isolated.json)；[原生实际输出](native/08-isolated.log)；[浏览器24案原始输出](browser/case-results-eleventh.jsonl)。本表取代 preliminary comparison.md 的汇总计数；首次表保留，修正原因见 [更正记录](correction-07-initial-review.json)。','']
with (BASE/'comparison-isolated.md').open('x')as out:out.write('\n'.join(lines))
print(json.dumps(summary,ensure_ascii=False))
