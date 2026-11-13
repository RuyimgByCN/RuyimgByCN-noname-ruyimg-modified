"""Real upstream event/DOM decisions. Fresh page and audited fixture per case."""
from pathlib import Path
import datetime, hashlib, json, os, subprocess, time, urllib.request
BASE=Path(__file__).resolve().parent
ROOT=BASE.parents[5]
META=json.loads((BASE/'active-session-wudian.json').read_text())
def evaluate(expr):
 req=urllib.request.Request('http://127.0.0.1:'+str(META['controlPort']),data=expr.encode(),method='POST')
 reply=json.loads(urllib.request.urlopen(req,timeout=30).read())
 if reply.get('result',{}).get('exceptionDetails'):raise RuntimeError(json.dumps(reply,ensure_ascii=False))
 value=reply['result']['result'].get('value')
 return json.loads(value) if isinstance(value,str) else value

def click(expr,kind):
 return evaluate("(()=>{q1._status.clicked=false;q1.ui.click."+kind+".call("+expr+");q1._status.clicked=false;return true;})()")
def ok():return evaluate('q1._status.clicked=false;q1.ui.click.ok();true')
def cancel():return evaluate('q1._status.clicked=false;q1.ui.click.cancel();true')

STATE="""(()=>{const {game,ui,_status}=q1,e=_status.event;const d=typeof e?.dialog==='number'?ui.dialogs.find(x=>x.videoId===e.dialog):e?.dialog||ui.dialog;return JSON.stringify({done:runtimeDone,error:runtimeError,paused:_status.paused,event:e?.name,skill:e?.skill,player:game.players.indexOf(e?.player),prompt:e?.prompt,parent:e?.getParent()?.name,step:e?.getParent()?.step,forced:e?.forced,confirm:ui.confirm?.str,controls:e?.controls,skillButtons:ui.skills?.skills,selectedCards:ui.selected.cards.map(c=>c.__nativeId),selectedTargets:ui.selected.targets.map(p=>game.players.indexOf(p)),selectableCards:game.players.flatMap(p=>p.getCards('he').filter(c=>c.classList.contains('selectable')).map(c=>c.__nativeId)),selectableTargets:game.players.filter(p=>p.classList.contains('selectable')).map(p=>game.players.indexOf(p)),buttons:d?.buttons?.map((b,i)=>({i,id:b.link?.__nativeId,name:b.link?.name,text:b.innerText})),caption:d?.content?.innerText?.slice(0,180)});})()"""

def run(case,tag):
 name=case['case'];record={'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'case':name,'run':tag,'choices':[]}
 try:
  evaluate('setTimeout(()=>location.reload(),0);true');time.sleep(.2)
  for i in range(120):
   try:
    r=evaluate("(async()=>{const n=await import('/noname.js');return n.game.players.length===8&&!!n.lib.card.sha&&n.game.players.every(p=>!!p.node)})()")
    if r:break
   except Exception:pass
   time.sleep(.1)
  else:raise RuntimeError('fresh page readiness timeout')
  record['readiness']={'players':8,'lib_card_sha':True,'player_nodes':True,'name_requirement':False}
  evaluate('window.runtimeFixture='+json.dumps(case,ensure_ascii=False)+';true')
  record['setup']=evaluate((BASE/'start-case.js').read_text())
  assert record['setup']['playerCount']==8 and record['setup']['skillReady']
  def proj(p,native=False):
   return {'seat':p['seat'],'hp':p['hp'],'max_hp':p['max_hp'],'alive':p['alive'],'hand':sorted(c['id'] for c in p['hand']),'equips':sorted(c['id'] for c in p['equips']),'judge':sorted(c['id'] for c in p['judge'])}
  record['initial_player_projection_matches']=([proj(p) for p in record['setup']['initial']['players']]==[proj(p) for p in case['initial']['players']])
  record['initial_draw_order_matches']=[c['id'] for c in record['setup']['initial']['deck_draw_order']]==[c['id'] for c in case['initial']['deck_draw_order']]
  assert record['initial_player_projection_matches'] and record['initial_draw_order_matches'],'initial projection mismatch'
  assert all(len(p['virtualEquips'])==len(p['equips']) and not p['virtualJudges'] for p in record['setup']['initial']['players']),'virtual equip/judge residue'
  activated=False;choudu_cancelled=False;swap_done=False;counter_done=False;youzhan_seen=0
  last=None;repeats=0
  for tick in range(1500):
   s=evaluate(STATE)
   if s.get('error'):raise RuntimeError(s['error'])
   if s['done']:break
   if not s['paused']:time.sleep(.03);continue
   marker=json.dumps(s,sort_keys=True)
   repeats=repeats+1 if marker==last else 0;last=marker
   if repeats>5:raise RuntimeError('unchanged paused state after decisions: '+marker)
   record['choices'].append(s);ev=s['event'];seat=s['player']
   if name.startswith('choudu') and ev=='chooseToUse' and not activated:
    if name.endswith('empty') or choudu_cancelled:
     cancel();activated=True
    elif s.get('skill')!='choudu':
     evaluate("q1._status.clicked=false;q1.ui.click.skill('choudu');true")
    elif name.endswith('cancel-card') or name.endswith('cancel-target'):
     if name.endswith('cancel-target'):click('runtimeCards['+str(case['cards']['cost'])+']','card')
     cancel();choudu_cancelled=True
    else:
     cost=case['cards']['weapon' if 'equip' in name else 'cost'];target=2 if 'target2' in name else 1
     click('runtimeCards['+str(cost)+']','card');click('q1.game.players['+str(target)+']','target');ok();activated=True
   elif ev=='chooseButton':
    if name.startswith('choudu') and seat==1 and ('swap-a' in name or 'swap-b' in name) and not swap_done and '置于牌堆顶' not in (s.get('caption') or ''):
     index=1 if 'swap-b' in name else 0
     evaluate('q1._status.clicked=false;q1.ui.click.button.call(q1.ui.dialog.buttons['+str(index)+']);q1._status.clicked=false;q1.ui.click.ok();true');swap_done=True
    else:cancel()
   elif ev=='chooseCard' and name.startswith('choudu'):
    card=case['cards']['other' if 'swap-b' in name else 'hand'];click('runtimeCards['+str(card)+']','card');ok()
   elif ev=='chooseCard' and name.startswith('fuchou'):
    if name.endswith('cancel'):cancel()
    else:
     card=case['cards']['weapon' if 'equip' in name else 'hand'];click('runtimeCards['+str(card)+']','card');ok()
   elif ev=='chooseToDiscard' and name.startswith('youzhan'):
    youzhan_seen+=1
    if name.endswith('cancel') or youzhan_seen>1:cancel()
    else:
     card=case['cards']['weapon' if 'equip-' in name else 'armor'];click('runtimeCards['+str(card)+']','card');ok()
   elif ev in ['chooseToRespond','chooseToUse']:
    if name=='youzhan-counter-use' and ev=='chooseToUse' and seat==1 and not counter_done:
     click('runtimeCards['+str(case['cards']['spare'])+']','card')
     selected=evaluate(STATE)
     if 0 not in selected['selectedTargets']:click('q1.game.players[0]','target')
     ok();counter_done=True
    else:cancel()
   elif ev=='chooseBool':cancel()
   else:raise RuntimeError('unhandled '+json.dumps(s,ensure_ascii=False))
   time.sleep(.06)
  else:raise RuntimeError('case timeout '+json.dumps(evaluate(STATE),ensure_ascii=False))
  record['result']=evaluate("JSON.stringify({case:runtimeCase,settled:runtimeDone,initial:runtimeInitial,after_attack:runtimeAfterAttack,final:runtimeSnapshot(),choiceConstructors:runtimeChoices,activation:window.runtimeActivation||null,skillEvents:q1.game.getGlobalHistory('everything').slice(runtimeHistoryStart).filter(e=>e.skill&&['choudu','fuchou','fuchou2','youzhan'].includes(e.skill)).map(e=>({name:e.name,skill:e.skill,trigger:e.triggername}))})")
  record['success']=True
 except Exception as e:
  record['success']=False;record['error']=str(e)
  try:record['failure_state']=evaluate(STATE)
  except Exception:pass
 with (BASE/f'case-results-{tag}.jsonl').open('a')as out:out.write(json.dumps(record,ensure_ascii=False)+'\n')
 print(name,'passed' if record['success'] else 'FAILED '+record['error'][:250],flush=True)
 return record['success']

if __name__=='__main__':
 tag=os.environ.get('U07_RUN_TAG','first')
 src=BASE.parent/'native'/os.environ.get('U07_NATIVE_LOG','05-fifth.log')
 cases=[json.loads(l)for l in src.read_text().splitlines()if l.startswith('{')]
 if os.environ.get('U07_ONLY'):cases=[c for c in cases if c['case'].startswith(os.environ['U07_ONLY'])]
 for c in cases:
  if not run(c,tag) and not os.environ.get('U07_CONTINUE'):break
