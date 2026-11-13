"""Recompute the declared L28 projections from raw native/CDP result rows."""
import hashlib
import json
from pathlib import Path
import re

BASE = Path(__file__).resolve().parent
native_path = BASE/'native/round2.jsonl'
native = [json.loads(l) for l in native_path.read_text().splitlines()]
browser_path = BASE/'browser/round6-final/case-results.jsonl'
browser = [json.loads(l) for l in browser_path.read_text().splitlines()]
retry_path = BASE/'browser/round7-cancel-clear/case-results.jsonl'
retry = [json.loads(l) for l in retry_path.read_text().splitlines()]
assert len(native)==22 and len({r['case']for r in native})==22
assert len(browser)==25 and len({r['case']for r in browser})==25
assert len(retry)==1 and retry[0]['case']=='yeyan-great-discard-cancel'
assert retry[0]['completed']
assert not next(r for r in browser if r['case']==retry[0]['case'])['completed']
bycase={r['case']:r for r in native}


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def face(c,side):
    return {k:c.get(k) for k in ['id','suit','rank','nature']}|{'name':c['upstream_name']if side=='native'else c['name']}


def project(s,side):
    players=[]
    for p in s['players']:
        d={k:p[k]for k in ['seat','hp','max_hp','alive','linked','face_down','hujia']}
        d.update({k:[face(c,side)for c in p[k]]for k in ['hand','equips','judge']})
        if side=='native':
            d['skills']=sorted(x['name']for x in p['skills']if x['enabled'])
            d['yeyan_spent']=any(x['name']=='th_yeyan'and x['once_used']for x in p['skills'])
            flags={f['key']:f['value']for f in p['flags']}
            d['buxu_uses']=flags.get('th_buxu_used_turn',0)
            classics=[c for pile in p['piles']if pile['key']=='th_liujing'for c in pile['cards']]
        else:
            d['skills']=sorted(p['skills'])
            d['yeyan_spent']='th_yeyan'in p['awakened']
            d['buxu_uses']=p['skillStat'].get('th_buxu',0)
            classics=p['expansions']
        # Keep ordering visible as well as membership. No hidden slot regrouping.
        d['classics']=[face(c,side)for c in classics]
        players.append(d)
    return {'players':players,**{k:[face(c,side)for c in s[k]]for k in ['deck_draw_order','discard','table']}}


def diff(a,b,path=''):
    if isinstance(a,dict)and isinstance(b,dict):
        return [d for k in sorted(set(a)|set(b))for d in diff(a.get(k),b.get(k),path+'.'+k)]
    if isinstance(a,list)and isinstance(b,list):
        if len(a)!=len(b):return [{'path':path,'native':a,'upstream':b}]
        return [d for i,(x,y)in enumerate(zip(a,b))for d in diff(x,y,path+'['+str(i)+']')]
    return []if a==b else[{'path':path,'native':a,'upstream':b}]


def native_events(row):
    result=[]
    for line in row['log']:
        m=re.match(r'DamageDealt \{ source: (?:Some\((\d+)\)|(None)), target: (\d+), amount: (\d+),',line)
        if m:
            result.append({'kind':'damage','source':int(m[1])if m[1]else None,'target':int(m[3]),'amount':int(m[4]),'nature':'fire'if 'nature: Fire'in line else 'unparsed'})
        m=re.match(r'HpLost \{ player: (\d+), amount: (\d+) \}',line)
        if m:result.append({'kind':'loseHp','source':None,'target':int(m[1]),'amount':int(m[2]),'nature':None})
    return result


def browser_events(row):
    return [{'kind':e['parent'],'source':None if e['source']==-1 else e['source'],'target':e['player'],
             'amount':-e['num'],'nature':e['nature']}for e in row['final']['hp_history']]


def payments(row,side):
    if side=='native':
        return [int(c)for l in row['log']for m in re.finditer(r'^CardDiscarded \{ player: 0, cards: \[([^\]]*)\]',l)for c in m[1].split(', ')if c]
    return [c['id']for e in row['final']['trace']if e['method']=='discard'and e['seat']==0 for arg in e['args']if isinstance(arg,list)for c in arg]


results=[]
inputs=[]
for line,original in enumerate(browser,1):
    row=retry[0]if original['case']=='yeyan-great-discard-cancel'else original
    used=retry_path if row is retry[0]else browser_path
    used_line=1 if row is retry[0]else line
    n=bycase[row['native_case']]
    observed=row.get('setup',{}).get('initial',row.get('saved_initial'))
    assert observed is not None,row['case']
    initial_diff=diff(project(n['initial'],'native'),project(observed,'upstream'))
    assert not initial_diff,(row['case'],initial_diff)
    blocked=not row['completed']and 'dcdAnim is not defined'in row.get('error','')
    assert row['completed']or blocked,row['case']
    result={'case':row['case'],'native_case':n['case'],'initial_projection_equal':True,
            'browser_completed':row['completed'],'spine_entry_blocked':blocked,
            'native_choice_count':len(n['choices']),'native_settled':n['settled'],
            'same_requested_allocation':row['case']!='yeyan-great-two'}
    if not blocked:
        differences=diff(project(n['final'],'native'),project(row['final'],'upstream'))
        ne,be=native_events(n),browser_events(row)
        np,bp=payments(n,'native'),payments(row,'upstream')
        result.update(final_projection_equal=not differences,final_differences=differences,
                      native_hp_events=ne,upstream_hp_events=be,hp_event_sequence_equal=ne==be,
                      native_payment=np,upstream_payment=bp,payment_sequence_equal=np==bp,
                      native_yeyan_spent=project(n['final'],'native')['players'][0]['yeyan_spent'],
                      upstream_yeyan_spent=project(row['final'],'upstream')['players'][0]['yeyan_spent'])
        if row['case']=='yeyan-great-two':
            result['classification']='allocation_not_representable_in_native_choice_spec; paired to nearest one-target great mode, not an identical input'
        elif differences or ne!=be or np!=bp:
            result['classification']='observed_difference_in_declared_projection'
        else:result['classification']='declared_state_payment_hp_events_match_only'
    else:result['classification']='blocked_before_choice; source comparison required'
    results.append(result)
    source_line=next(i for i,r in enumerate(native,1)if r['case']==n['case'])
    control=used.parent/'control-map.jsonl'
    mappings=[json.loads(l)for l in control.read_text().splitlines()if json.loads(l)['case']==row['case']]
    inputs.append({'case':row['case'],'native_file':str(native_path.relative_to(BASE)),'native_line':source_line,
                   'browser_file':str(used.relative_to(BASE)),'browser_line':used_line,
                   'native_sha256':sha(native_path),'browser_sha256':sha(used),
                   'control_map':str(control.relative_to(BASE)),'raw_request_lines':[r['request_line']for r in mappings],
                   'retained_prior_attempt':{'file':str(browser_path.relative_to(BASE)),'line':line,'reason':'four automatic selections expose only OK; raw unavailable-cancel attempt retained; retry clears allocations through original avatar handlers'}if row is retry[0]else None})

checks=['decode_restore_ok','full_snapshot_before_equal_except_auth_generation',
        'full_snapshot_after_equal_except_auth_generation','state_hash_equal','outcome_equal','new_logs_equal']
restorations=[{'case':r['case'],'archive_file':c['archive_file'],'archive_bytes':c['archive_bytes'],
               'sha256':sha(Path(c['archive_file'])),**{k:c[k]for k in checks}}for r in native for c in r['choices']]
assert len(restorations)==39 and all(all(r[k]for k in checks)for r in restorations)
all_rounds=[]
for p in sorted((BASE/'browser').glob('*/case-results.jsonl')):
    rows=[json.loads(l)for l in p.read_text().splitlines()]
    all_rounds.append({'file':str(p.relative_to(BASE)),'sha256':sha(p),'raw_rows':len(rows),
                       'completed':sum(r['completed']for r in rows),'not_completed':sum(not r['completed']for r in rows)})
counts={'final_cohort':len(results),'initial_projection_matches':len(results),'completed':sum(r['browser_completed']for r in results),
        'blocked':sum(r['spine_entry_blocked']for r in results),'native_cases':len(native),'pending_restorations':len(restorations)}
for key in ['declared_state_payment_hp_events_match_only','observed_difference_in_declared_projection','allocation_not_representable_in_native_choice_spec; paired to nearest one-target great mode, not an identical input']:
    counts[key]=sum(r['classification']==key for r in results)
out={'counts':counts,'projection':'All eight players: hp/max/alive/link/turnover/armor, ordered card faces incl nature and identity in hand/equipment/judgment/classics; enabled skill names, Yeyan limited use, Buxu count; ordered draw/discard/table. HP event sequence and payment sequence compared separately. Excludes other storage/events/RNG and unexecuted full card semantics. Configuration is recorded, not assumed matched.',
     'results':results,'restorations':restorations,'all_retained_browser_rounds':all_rounds,
     'limitations':['Collector result, not independent signoff. No skill or source equivalence certification.',
                    'Four fresh Zhengjing entries stop before a selectable skill: no minigame result or target/distribution completion was injected.',
                    'The three original selected skills/helpers and physical card property execute unchanged; extension installation/precontent and Spine runtime are never executed.',
                    'Named mapping of generic equipment is a fixture mapping; all deck faces/order/nature are checked, but unexecuted weapon/armor rule semantics are not certified.']}
(BASE/'comparison-results.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n')
(BASE/'final-case-index.json').write_text(json.dumps({'cases':inputs,'final_cohort':25,'raw_round6_rows':25,'retained_round7_rows':1,'all_browser_rounds':all_rounds},ensure_ascii=False,indent=2)+'\n')
lines=['# L28 逐场采集对照（2026-11-14）','','这是采集者对声明投影的重算，待另一人独立复核；不签署整技能等价。初态包含卡牌性质与区域内顺序。完整投影、差异字段和 39 次 NGSA 恢复见 [JSON](comparison-results.json)，逐场 raw/CDP 行号见 [索引](final-case-index.json)。','','| 场景 | 初态投影 | 上游执行 | 结算区域投影 | 支付序列 | HP事件顺序 | 限定技消耗（原生/上游） |','| --- | --- | --- | --- | --- | --- | --- |']
for r in results:
    if r['spine_entry_blocked']:cells=['一致','Spine入口阻塞','未验','未验','未验','未验']
    else:cells=['一致','完成'if r['same_requested_allocation']else '原生无2火单目标选项',
                '一致'if r['final_projection_equal']else '有差异','一致'if r['payment_sequence_equal']else '有差异',
                '一致'if r['hp_event_sequence_equal']else '有差异',str(r['native_yeyan_spent'])+'/'+str(r['upstream_yeyan_spent'])]
    lines.append('| '+r['case']+' | '+' | '.join(cells)+' |')
lines+=['','所有历史轮次完整保留，没有从成功样本中删除差异。round6 的大业炎直接取消尝试在四牌已自动选择时只有 OK，重试只经原始头像监听器撤回三团火再取消；该失败尝试仍在索引列明。`yeyan-great-two` 的原生配对是最近的“大模式、单目标”而非相同火数输入，不计作同输入通过。','','跨引擎不比较完整事件流或 RNG；39 次原生自身恢复则逐次比较完整快照（仅归一 auth_generation）、状态哈希、命令结果和新增日志。卡牌身份、名称、花色、点数、性质及区域顺序已进入初态投影；未执行的通用武器/坐骑语义仍未验。','']
(BASE/'comparison.md').write_text('\n'.join(lines))
print(json.dumps(counts,ensure_ascii=False))
