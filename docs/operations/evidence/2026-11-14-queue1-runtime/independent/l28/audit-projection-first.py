#!/usr/bin/env python3
"""Explicit face normalization and official card-list audit after the strict first pass."""
from pathlib import Path
import ast,collections,datetime,hashlib,json,re
OUT=Path(__file__).resolve().parent;BASE=OUT.parent.parent/'L28';REPO=OUT.parents[5]
def readl(p):return [json.loads(l) for l in p.read_text().splitlines()]
def face(c,b):
    name=c['name'] if b else c['upstream_name']
    nature=c.get('nature')
    if name=='shandian' and nature is None:nature='thunder'
    return (c['id'],name,c['suit'],c['rank'],nature)
def initial(s,b):
    return {'players':[{'seat':p['seat'],**{k:p[k]for k in ['hp','max_hp','alive','linked','face_down','hujia']},
        **{k:[face(c,b)for c in p[k]]for k in ['hand','equips','judge']},
        'skills':sorted(p['skills'] if b else [x['name']for x in p['skills']if x['enabled']]),
        'awakened':sorted(p['awakened']if b else [x['name']for x in p['skills']if x['once_used']]),
        'classics':[face(c,b)for c in p['expansions']]if b else [face(c,b)for z in p['piles']if z['key']=='th_liujing'for c in z['cards']]}for p in s['players']],
        **{k:[face(c,b)for c in s[k]]for k in ['deck_draw_order','discard','table']}}
def events_b(s):
    return [('damage',e['source'],e['player'],e['num'],e['nature']) if e['name']=='damage' else ('loseHp',None,e['player'],e['num'],None) for e in s['event_history']if e['name']in ['damage','loseHp']]
def events_n(log):
    out=[]
    for row in log:
        m=re.match(r'DamageDealt \{ source: Some\((\d+)\), target: (\d+), amount: (-?\d+)',row)
        if m:out.append(('damage',int(m[1]),int(m[2]),int(m[3]),'fire'))
        m=re.match(r'HpLost \{ player: (\d+), amount: (\d+)',row)
        if m:out.append(('loseHp',None,int(m[1]),int(m[2]),None))
    return out
browser=readl(BASE/'browser/round8-clean/case-results.jsonl');native=readl(BASE/'native/round2.jsonl');nd={r['case']:r for r in native}
base_cards=native[0]['initial']['deck_draw_order']+native[0]['initial']['players'][0]['hand']
# Read only the literal official pile lists; no upstream code is executed.
upstream=REPO.parent/'archive/upstream/noname-liuli/noname'
known=[];source_inputs=[]
for fname in ['standard','extra']:
    p=upstream/'card'/f'{fname}.js';source=p.read_text();start=source.index('list:[');literal=source[start+len('list:'):source.index('],',start)+1]
    tuples=ast.literal_eval(literal)
    known.extend((x[2],x[0],x[1],x[3]if len(x)>3 else None)for x in tuples)
    source_inputs.append({'path':str(p),'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'literal_pile_count':len(tuples)})
# The extension defines the real Taiping Yaoshu derivation with heart/3.
known.append(('th_taipingyaoshu','heart',3,None))
missing=[c for c in sorted(base_cards,key=lambda c:c['id'])if(c['upstream_name'],c['suit'],c['rank'],c.get('nature'))not in known]
missing_ids={c['id']for c in missing}
checks=[]
for b in browser:
    n=nd[b['native_case']];s=b.get('setup',{}).get('initial')or b['saved_initial'];f=b.get('final')or b['failure_snapshot']
    bp=initial(s,True);np=initial(n['initial'],False)
    strict=all(bp[k]==np[k]for k in bp)
    hpb=[p['hp']for p in f['players']];hpn=[p['hp']for p in n['final']['players']]
    costs_b=[c['id']for c in f['discard']];costs_n=[c['id']for c in n['final']['discard']]
    classics_b=sorted(c['id']for p in f['players']for c in p['expansions']);classics_n=sorted(c['id']for p in n['final']['players']for z in p['piles']if z['key']=='th_liujing'for c in z['cards'])
    be=events_b(f);ne=events_n(n['log'])
    touched_missing=set(costs_b)|{c['id']for p in f['players']for c in p['expansions']}
    checks.append({'case':b['case'],'native_case':n['case'],'initial_equal_after_only_shandian_intrinsic_nature_normalization':strict,
        'browser_completed':b['completed'],'hp_browser':hpb,'hp_native':hpn,'hp_equal':hpb==hpn,
        'discard_ids_browser':costs_b,'discard_ids_native':costs_n,'discard_ids_equal':costs_b==costs_n,
        'classics_ids_browser':classics_b,'classics_ids_native':classics_n,'classics_ids_equal':classics_b==classics_n,
        'source_hp_event_order':be,'native_hp_event_order':ne,'hp_event_order_equal':be==ne,
        'interacted_cards_absent_from_official_pile_tuple_lists':sorted(touched_missing&missing_ids),
        'initial_projection_is_complete_runtime_state':False,'native_unsupported_same_allocation':b['case']=='yeyan-great-two',
        'native_has_no_strength_config_input':b['case'].endswith('-default')})
result={'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'normalization':'Only shandian null->thunder for the explicitly listed intrinsic lightning nature; strict first-pass mismatch preserved in raw-integrity-audit.json/audit-raw-first.stdout.',
    'initial_matches_after_that_normalization':sum(x['initial_equal_after_only_shandian_intrinsic_nature_normalization']for x in checks),
    'official_tuple_input_sources':source_inputs,'native_fixture_card_count':len(base_cards),'native_faces_absent_from_official_standard_extra_and_real_th_taipingyaoshu':missing,
    'all_fixture_cards_are_official_pile_tuples':not missing,'case_checks':checks,
    'scope':'These are explicit card/HP/event-order projections. Initial equality uses collector-provided native-to-upstream names; the official tuple audit is a separate additional restriction, not erased by that equality.'}
(OUT/'projection-audit.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'cases':len(checks),'initial_after_intrinsic_nature_normalization':result['initial_matches_after_that_normalization'],'official_tuple_missing_ids':sorted(missing_ids),'source_pile_counts':[x['literal_pile_count']for x in source_inputs],'completed_hp_equal':sum(x['browser_completed']and x['hp_equal']for x in checks),'completed_classics_mismatch':[x['case']for x in checks if x['browser_completed']and not x['classics_ids_equal']],'completed_event_order_mismatch':[x['case']for x in checks if x['browser_completed']and not x['hp_event_order_equal']]},ensure_ascii=False))
