from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os
BASE=Path(__file__).resolve().parent
n=BASE/'native/09-formal.log';b=BASE/'browser/case-results-twelfth.jsonl'
native=list(map(json.loads,n.read_text().splitlines()));browser=list(map(json.loads,b.read_text().splitlines()))
assert len(native)==len(browser)==24
assert len({x['case']for x in native})==len({x['case']for x in browser})==24
assert [x['case']for x in native]==[x['case']for x in browser]
meta=json.loads((BASE/'browser/active-session-wudian.json').read_text());stem=Path(meta['transcript'])
rq=Path(str(stem)+'-requests.jsonl');rp=Path(str(stem)+'-replies.jsonl')
requests=list(map(json.loads,rq.read_text().splitlines()));replies=list(map(json.loads,rp.read_text().splitlines()))
def dt(s):return datetime.fromisoformat(s.replace('Z','+00:00'))
index=[]
for i,(nr,br)in enumerate(zip(native,browser)):
 assert br['success'] and br['initial_player_projection_matches']and br['initial_draw_order_matches']
 assert not br['result']['foreignPileMutations']
 start=dt(br['at']);end=dt(browser[i+1]['at'])if i+1<len(browser)else datetime.max.replace(tzinfo=timezone.utc)
 candidates=[(j+1,r)for j,r in enumerate(requests)if start<=dt(r['at'])<end]
 fixture=[(j,r)for j,r in candidates if r['expression'].startswith('window.runtimeFixture=')]
 final=[(j,r)for j,r in candidates if r['expression'].startswith('JSON.stringify({case:runtimeCase') and 'foreignPileMutations' in r['expression']]
 assert len(fixture)==len(final)==1,(br['case'],len(fixture),len(final))
 parsed=json.loads(fixture[0][1]['expression'][len('window.runtimeFixture='):-len(';true')]);assert parsed['case']==br['case']
 answer=[]
 for j,r in enumerate(replies):
  if r['at']!=final[0][1]['at']:continue
  value=r.get('result',{}).get('result',{}).get('value')
  if isinstance(value,str):
   try:v=json.loads(value)
   except ValueError:continue
   if isinstance(v,dict)and v.get('case')==br['case']and v.get('settled')is True:answer.append(j+1)
 assert len(answer)==1,(br['case'],answer)
 index.append({'case':br['case'],'native_jsonl_line':i+1,'browser_jsonl_line':i+1,'browser_at':br['at'],'raw_fixture_request_line':fixture[0][0],'raw_final_request_line':final[0][0],'raw_final_reply_line':answer[0],'native_ngsa_windows':len(nr['choices'])})
files=[n,b,rq,rp,BASE/'native/probe-09.rs',BASE/'browser/start-case.js',BASE/'browser/run-cases.py',BASE/'native/formal-catalog-without-placeholder.json',BASE/'native/archive-manifest-formal.json',BASE/'native-09-before.json',BASE/'native-09-after.json',BASE/'browser-formal-before.json',BASE/'browser-formal-after.json']
row={'at':datetime.now(timezone.utc).isoformat(),'cohort':'native09 + browser twelfth, immutable raw inputs','unique_cases':24,'native_jsonl':os.path.relpath(n,BASE),'browser_jsonl':os.path.relpath(b,BASE),'probe_source':'native/probe-09.rs','fixture_source':'browser/start-case.js','runner_source':'browser/run-cases.py','raw_requests':os.path.relpath(rq,BASE),'raw_replies':os.path.relpath(rp,BASE),'cases':index,'sha256':{os.path.relpath(p,BASE):hashlib.sha256(p.read_bytes()).hexdigest()for p in files},'new_equivalence_signoff':False}
with (BASE/'final-input-index.json').open('x')as out:out.write(json.dumps(row,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'unique_cases':24,'native_ngsa_windows':sum(x['native_ngsa_windows']for x in index),'index':'final-input-index.json','index_sha256':hashlib.sha256((BASE/'final-input-index.json').read_bytes()).hexdigest()}))
