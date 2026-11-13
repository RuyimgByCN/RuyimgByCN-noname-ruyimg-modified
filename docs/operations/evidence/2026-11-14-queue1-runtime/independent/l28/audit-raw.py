#!/usr/bin/env python3
"""Independent integrity/fixture audit, without collector comparison code."""
from pathlib import Path
import collections, datetime, hashlib, json, subprocess

OUT = Path(__file__).resolve().parent
BASE = OUT.parent.parent / 'L28'
EXPECTED = {
    'raw-input-index.json': 'a91b0c8463c4352e89d11bf33150bf4391aca2d72c5d695226244c4aaa51b75c',
    'frozen-file-manifest.json': '7f9120cd3c349d5bc4c346e50df6aadffc63b20dc4a4d8aea3377920b20a9136',
    'browser/round8-clean/case-results.jsonl': 'a73c790e6894c811d7cc52093b6be4a836e6576fbb53f38e544061eb13cb752c',
    'native/probe-02.rs': '5384093a99da622ca7e19d3595a96dafbd41b50aa9ef6d95f25e5ffdb32b45d6',
    'native/round2.jsonl': '3fe8850398c92045a0771fb2f068b1a5cf46cd70bf84b57bc8741eb2a872dc6d',
}
def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def readj(p): return json.loads(p.read_text())
def readl(p): return [json.loads(x) for x in p.read_text().splitlines()]
def value(row):
    result=row.get('result',{})
    if result.get('exceptionDetails'): return {'__exception__':result['exceptionDetails']}
    v=result.get('result',{}).get('value')
    if isinstance(v,str):
        try: return json.loads(v)
        except json.JSONDecodeError: pass
    return v

def face_b(c): return (c.get('id'),c.get('name'),c.get('suit'),c.get('rank'),c.get('nature'))
def face_n(c): return (c.get('id'),c.get('upstream_name'),c.get('suit'),c.get('rank'),c.get('nature'))
def projected_b(s):
    return {'players':[{'seat':p['seat'],**{k:p[k] for k in ['hp','max_hp','alive','linked','face_down','hujia']},
        **{k:[face_b(c) for c in p[k]] for k in ['hand','equips','judge']},
        'skills':sorted(p['skills']),'awakened':sorted(p['awakened']),
        'expansions':[face_b(c) for c in p['expansions']]} for p in s['players']],
        **{k:[face_b(c) for c in s[k]] for k in ['deck_draw_order','discard','table']}}
def projected_n(s):
    return {'players':[{'seat':p['seat'],**{k:p[k] for k in ['hp','max_hp','alive','linked','face_down','hujia']},
        **{k:[face_n(c) for c in p[k]] for k in ['hand','equips','judge']},
        'skills':sorted(x['name'] for x in p['skills'] if x['enabled']),
        'awakened':sorted(x['name'] for x in p['skills'] if x['once_used']),
        'expansions':[face_n(c) for pile in p['piles'] if pile['key']=='th_liujing' for c in pile['cards']]} for p in s['players']],
        **{k:[face_n(c) for c in s[k]] for k in ['deck_draw_order','discard','table']}}
def ids(s,browser):
    cards=[c for k in ['deck_draw_order','discard','table'] for c in s[k]]
    for p in s['players']:
        cards.extend(c for k in ['hand','equips','judge'] for c in p[k])
        if browser:cards.extend(p['expansions'])
        else:cards.extend(c for pile in p['piles'] for c in pile['cards'])
    return collections.Counter(c.get('id') for c in cards)

manifest=readj(BASE/'frozen-file-manifest.json'); index=readj(BASE/'raw-input-index.json')
file_errors=[]
for row in manifest['files']:
    p=BASE/row['path']
    if not p.is_file(): file_errors.append({'path':row['path'],'error':'missing'});continue
    if p.stat().st_size!=row['bytes'] or digest(p)!=row['sha256']:file_errors.append({'path':row['path'],'error':'hash/size'})
for rel,expected in EXPECTED.items():
    if digest(BASE/rel)!=expected:file_errors.append({'path':rel,'error':'announced hash'})
listed={x['path'] for x in manifest['files']}|{'frozen-file-manifest.json'}
actual={str(p.relative_to(BASE)) for p in BASE.rglob('*') if p.is_file()}
file_errors.extend({'path':x,'error':'unmanifested'} for x in sorted(actual-listed))
requests=readl(BASE/'session-l28-1794595037613-requests.jsonl')
replies=readl(BASE/'session-l28-1794595037613-replies.jsonl')
browser=readl(BASE/'browser/round8-clean/case-results.jsonl'); native=readl(BASE/'native/round2.jsonl')
transcript_errors=[]
if len(requests)!=len(replies):transcript_errors.append('request/reply count differs')
for i,(q,r) in enumerate(zip(requests,replies),1):
    if q['at']!=r['at']:transcript_errors.append({'line':i,'error':'timestamp'})
case_checks=[]; raw_snapshot_index=[]
for entry in index['cases']:
    b=browser[entry['browser_line']-1]; n=native[entry['native_line']-1]
    assert b['case']==entry['case'] and n['case']==entry['fixture_case_in_native']
    raw=[]
    for item in entry['request_reply_rows']:
        q=requests[item['request_line']-1];r=replies[item['reply_line']-1]
        if q['at']!=item['at'] or r['at']!=item['at'] or hashlib.sha256(q['expression'].encode()).hexdigest()!=item['request_expression_sha256']:
            transcript_errors.append({'case':b['case'],'entry':item,'error':'index request'})
        raw.append((item['reply_line'],value(r),item['label']))
    s=b.get('setup',{}).get('initial') or b.get('saved_initial')
    assert s, b['case']+' initial not captured'
    snapshots=[('initial',s)]
    snapshots.extend(('stages.'+str(i)+'.'+x['label'],x['snapshot']) for i,x in enumerate(b['stages']))
    if 'final' in b:snapshots.append(('final',b['final']))
    if 'failure_snapshot' in b:snapshots.append(('failure_snapshot',b['failure_snapshot']))
    snapshot_errors=[]
    for label,snapshot in snapshots:
        matches=[i for i,v,l in raw if v==snapshot or isinstance(v,dict) and v.get('initial')==snapshot]
        raw_snapshot_index.append({'case':b['case'],'snapshot':label,'matching_reply_lines':matches})
        if not matches:snapshot_errors.append(label)
    initial_equal=projected_b(s)==projected_n(n['initial'])
    initial_fields_equal={k:projected_b(s)[k]==projected_n(n['initial'])[k] for k in ['players','deck_draw_order','discard','table']}
    final=b.get('final') or b.get('failure_snapshot')
    bcon=ids(s,True)==ids(final,True)
    ncon=ids(n['initial'],False)==ids(n['final'],False)
    selected_requests=[{'line':i['request_line'],'label':i['label']} for i in entry['request_reply_rows']]
    case_checks.append({'case':b['case'],'native_case':n['case'],'browser_completed':b['completed'],
        'initial_projection_equal':initial_equal,'initial_fields_equal':initial_fields_equal,
        'seats_checked':len(s['players']),'physical_ids_count':sum(ids(s,True).values()),
        'initial_unique_ids':all(c==1 and k is not None for k,c in ids(s,True).items()),
        'browser_physical_ids_conserved':bcon,'native_physical_ids_conserved':ncon,
        'final_has_null_ids':None in ids(final,True),'foreign_pile_mutations':final['foreign_pile_mutations'],
        'snapshot_binding_errors':snapshot_errors,'ready':b['ready'],'bootstrap':b['bootstrap'],
        'strength_initial':s['strength'],'strength_final':final['strength'],
        'source_entry_exception':any(l=='actual-phaseUse-start' and isinstance(v,dict) and '__exception__' in v for i,v,l in raw),
        'requests':selected_requests})
    assert b['fixture']['initial']==n['initial'], b['case']+' fixture differs from native raw initial'

source_checks=[]
for row in index['source_inputs']:
    p=Path(row['path']);q=BASE/row['snapshot']
    source_checks.append({'path':row['path'],'current_sha256':digest(p),'snapshot_sha256':digest(q),
        'announced_sha256':row['sha256'],'current_and_snapshot_match':digest(p)==digest(q)==row['sha256']})
slice_checks=[]
for row in readj(BASE/'upstream/source-slices.json'):
    lines=Path(row['source']).read_text().splitlines(keepends=True)
    block=''.join(lines[row['start_line']-1:row['end_line']])
    variants={'with_final_newline':block,'without_final_newline':block.rstrip('\r\n')}
    matches=[k for k,v in variants.items() if hashlib.sha256(v.encode()).hexdigest()==row['sha256']]
    slice_checks.append({'name':row['name'],'kind':row['kind'],'source_lines':[row['start_line'],row['end_line']],'source_hash_matches':matches})
for kind in ['skills','helpers']:
    rows=[r for r in readj(BASE/'upstream/source-slices.json') if r['kind']==kind]
    chunks=[''.join(Path(r['source']).read_text().splitlines(keepends=True)[r['start_line']-1:r['end_line']]) for r in rows]
    source='({\n'+''.join(chunks)+'\n})\n'
    actual_source=(BASE/'upstream'/f'{kind}-original.js').read_text()
    slice_checks.append({'kind':kind,'exact_object_reassembly_equal':source==actual_source,'source_len':len(source),'actual_len':len(actual_source)})
http=readl(BASE/'session-l28-1794595037613-http.jsonl')
report={'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'head':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),
    'collector_file_count_checked':len(manifest['files']),'collector_total_bytes':sum(x['bytes'] for x in manifest['files']),
    'file_errors':file_errors,'request_count':len(requests),'reply_count':len(replies),'transcript_errors':transcript_errors,
    'selected_request_count':sum(len(c['request_reply_rows']) for c in index['cases']),
    'browser_case_count':len(browser),'unique_browser_cases':len(set(x['case'] for x in browser)),
    'native_case_count':len(native),'case_checks':case_checks,'source_checks':source_checks,'slice_checks':slice_checks,
    'spine_http_paths':[r for r in http if 'spine' in r['path'].lower()],
    'narrative_or_comparison_code_used':False,
    'projection_boundary':'Initial mapping covers ordered native id/upstream name/suit/rank/nature in all captured zones, eight seats HP/maxHP/alive/link/face/hujia, enabled skills, once-used/awakened and mapped expansions. It is not full upstream storage/event/RNG equivalence. Native generic equipment face mapping requires a separate real-card audit.'}
(OUT/'raw-integrity-audit.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
(OUT/'raw-snapshot-index.json').write_text(json.dumps(raw_snapshot_index,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'manifest_checked':len(manifest['files']),'file_errors':len(file_errors),'requests':len(requests),'selected_requests':report['selected_request_count'],'transcript_errors':len(transcript_errors),'browser':len(browser),'browser_completed':sum(x['completed'] for x in browser),'native':len(native),'initial_projection_matches':sum(x['initial_projection_equal'] for x in case_checks),'bound_snapshots':sum(bool(x['matching_reply_lines']) for x in raw_snapshot_index),'snapshots':len(raw_snapshot_index),'source_files':len(source_checks),'source_hash_mismatches':sum(not x['current_and_snapshot_match'] for x in source_checks),'slice_checks':slice_checks,'spine_http_paths':len(report['spine_http_paths'])},ensure_ascii=False))
