import argparse
import hashlib
import json
from pathlib import Path
import time
import traceback
import urllib.request

BASE = Path(__file__).resolve().parents[1]
META = json.loads((BASE / 'active-session-l28.json').read_text())
URL = 'http://127.0.0.1:%d/' % META['controlPort']
PARSER = argparse.ArgumentParser()
PARSER.add_argument('--output', required=True)
PARSER.add_argument('--cases', nargs='*')
ARGS = PARSER.parse_args()
OUT = BASE / 'browser' / ARGS.output
OUT.mkdir(exist_ok=False)
NATIVE = [json.loads(l) for l in (BASE/'native/round1.jsonl').read_text().splitlines()]
BYCASE = {r['case']:r for r in NATIVE}
CURRENT = None


def post(payload, label):
    raw = payload if isinstance(payload, str) else json.dumps(payload)
    req = Path(META['transcript']+'-requests.jsonl')
    line = len(req.read_text().splitlines())+1 if req.exists() else 1
    reply = json.loads(urllib.request.urlopen(URL, raw.encode(), timeout=30).read())
    with (OUT/'control-map.jsonl').open('a') as f:
        f.write(json.dumps({'case':CURRENT,'label':label,'request_line':line,
                           'expression_sha256':hashlib.sha256(raw.encode()).hexdigest()})+'\n')
    return reply['result']


def evaluate(expression, label='evaluate'):
    r = post(expression, label)
    if r.get('exceptionDetails'):
        raise RuntimeError(json.dumps(r,ensure_ascii=False))
    v = r.get('result',{}).get('value')
    if isinstance(v,str):
        try:return json.loads(v)
        except json.JSONDecodeError:pass
    return v


def snapshot():
    return evaluate('JSON.stringify(l28Snapshot())','snapshot')


def wait(predicate, limit=150):
    for i in range(limit):
        s = snapshot()
        if predicate(s):return s
        time.sleep(.12)
    raise RuntimeError('wait timeout: '+json.dumps(s.get('choice'),ensure_ascii=False))


def reload():
    evaluate('location.reload();true','reload')
    for i in range(200):
        try:
            if evaluate("typeof game!=='undefined'&&game.players?.length===8&&game.me&&_status.event?.name==='game'&&_status.paused",'ready'):
                return evaluate("JSON.stringify({version:lib.version,url:location.href,event:_status.event.name,players:game.players.length,spine:typeof dcdAnim})",'page-ready')
        except Exception:pass
        time.sleep(.1)
    raise RuntimeError('reload ready timeout')


def install():
    # Evaluate only unchanged properties; never execute extension content/precontent.
    for kind,target in [('skills','lib.skill'),('helpers','game'),('card','lib.card')]:
        source=(BASE/'upstream'/f'{kind}-original.js').read_text()
        evaluate(f'Object.assign({target},eval({json.dumps(source)}));true','install-'+kind)
    evaluate("for(const s of ['th_chengye','th_buxu','th_zhengjing','th_zhengjing2','th_yeyan'])game.finishSkill(s);Object.assign(lib.translate,{th_buxu:'补叙',th_zhengjing:'整经',th_yeyan:'业炎',th_chengye:'承业'});true",'register-original-skills')
    return evaluate("new Promise((resolve,reject)=>{const l=document.createElement('link');l.rel='stylesheet';l.href='/extension/'+encodeURIComponent('雷霆万钧')+'/thunder.css';l.onload=()=>resolve(true);l.onerror=reject;document.head.appendChild(l);})",'load-original-css')


def mouse(kind,x,y,buttons=0):
    return post({'cdp':True,'method':'Input.dispatchMouseEvent','params':{
        'type':kind,'x':x,'y':y,'button':'left' if kind!='mouseMoved' else 'none','buttons':buttons,'clickCount':1}},'mouse-'+kind)


def click_expr(expression):
    rect=evaluate('(()=>{const n='+expression+";if(!n)throw Error('missing click node');const r=n.getBoundingClientRect();return {x:r.left+r.width/2,y:r.top+r.height/2};})()",'click-rect')
    mouse('mousePressed',rect['x'],rect['y'],1)
    mouse('mouseReleased',rect['x'],rect['y'])
    time.sleep(.15)


def skill(key):
    evaluate("if(!_status.event._skillChoice?.includes("+json.dumps(key)+"))throw Error('not eligible');_status.clicked=false;ui.click.skill("+json.dumps(key)+");true",'skill-ui-handler')


def ok():
    evaluate("if(!ui.confirm?.str?.includes('o'))throw Error('no OK');_status.clicked=false;ui.click.ok();true",'confirm-ok')


def cancel():
    evaluate("if(!ui.confirm?.str?.includes('c'))throw Error('no cancel');_status.clicked=false;ui.click.cancel();true",'confirm-cancel')


def drag(index,seat):
    geometry=evaluate("(()=>{const f=document.querySelector('#th-yeyanflames"+str(index)+"');const p=game.players["+str(seat)+"];const a=f.getBoundingClientRect(),b=p.getBoundingClientRect();return {a:{x:a.left+a.width/2,y:a.top+a.height/2},b:{x:b.left+b.width/2,y:b.top+b.height*.7},zoom:game.documentZoom};})()",'drag-geometry')
    a,b=geometry['a'],geometry['b'];mouse('mousePressed',a['x'],a['y'],1)
    mouse('mouseMoved',b['x'],b['y'],1);mouse('mouseReleased',b['x'],b['y'])
    time.sleep(.15)
    return geometry


def run(name):
    global CURRENT
    CURRENT=name
    native=BYCASE[name.removesuffix('-default')]
    f={'case':name,'initial':native['initial'],'strength':'default' if name.endswith('-default') else 'forbid'}
    row={'case':name,'native_case':native['case'],'fixture':f,'stages':[]}
    def stage(label,s=None,**extra):
        s=s or snapshot();row['stages'].append({'label':label,'snapshot':s,**extra});return s
    try:
        row['ready']=reload();install()
        evaluate('window.l28Fixture='+json.dumps(f)+';true','fixture')
        row['setup']=evaluate((BASE/'browser/start-case.js').read_text(),'actual-phaseUse-start')
        if name.startswith('zhengjing') and name!='zhengjing-no-skill':
            # The first real chooseToUse filter call may stop the loop synchronously.
            row['unexpected_no_entry_error']=True
        s=stage('phaseUse-choice',wait(lambda x:x['choice']['name']=='chooseToUse'))
        eligible=native['skill'] in (s['choice'].get('skills')or[])
        row['skill_eligible']=eligible
        if name in ['buxu-empty','buxu-full','yeyan-used','zhengjing-no-skill']:
            if eligible:raise AssertionError('unmet condition unexpectedly eligible')
        elif name.startswith('buxu'):
            rounds=2 if name=='buxu-twice' else 1
            for iteration in range(rounds):
                skill('th_buxu');s=stage('buxu-cost-selected')
                if name=='buxu-cancel':
                    cancel();stage('cancel-cost',wait(lambda x:x['choice']['name']=='chooseToUse'));break
                cost_ids=[0] if iteration==0 else [30,45]
                for cid in cost_ids:
                    evaluate("(()=>{const c=l28Cards["+str(cid)+"];_status.clicked=false;ui.click.card.call(c);return true;})()",'select-cost-card')
                stage('buxu-cost-cards');ok()
                s=stage('buxu-chambers',wait(lambda x:len(x['dom']['chambers'])>0))
                chamber=5 if name=='buxu-equip'or iteration==1 else 2 if name=='buxu-nullify'else 1
                row.setdefault('chosen_chambers',[]).append(chamber)
                if name=='buxu-invalid':
                    row['invalid_chamber_absent']=not any(x['id']=='6'for x in s['dom']['chambers'])
                click_expr("document.querySelector('.th-mamidibu[id=\""+str(chamber)+"\"]')")
                stage('buxu-resolved',wait(lambda x:x['choice']['name']=='chooseToUse'and not x['dom']['chambers']))
        elif name.startswith('yeyan'):
            skill('th_yeyan')
            stage('yeyan-flames',wait(lambda x:len(x['dom']['flames'])==3))
            if name=='yeyan-cancel':
                cancel();stage('yeyan-cancelled',wait(lambda x:x['choice']['name']=='chooseToUse'))
            else:
                allocations=[1,2,3]if name=='yeyan-small-three'else [1,1,1]if 'great-one'in name else [3,3,1]if 'great-split'in name else [1]
                for index,seat in enumerate(allocations):
                    geometry=drag(index,seat);stage('flame-'+str(index)+'-to-'+str(seat),geometry=geometry)
                if name=='yeyan-no-four':
                    geometry=drag(1,1);stage('second-flame-without-four-suits',geometry=geometry)
                ok();stage('yeyan-resolved',wait(lambda x:x['choice']['name']=='chooseToUse'))
        else:raise AssertionError('unhandled case')
        time.sleep(.8)
        row['final']=snapshot();row['completed']=True
    except Exception:
        row['completed']=False;row['error']=traceback.format_exc()
        try:row['saved_initial']=evaluate('JSON.stringify(l28Initial)','saved-initial-after-error')
        except Exception:pass
        try:row['failure_snapshot']=snapshot()
        except Exception:row['snapshot_error']=traceback.format_exc()
    (OUT/(name+'.json')).write_text(json.dumps(row,ensure_ascii=False,indent=2)+'\n')
    with (OUT/'case-results.jsonl').open('a')as log:log.write(json.dumps(row,ensure_ascii=False)+'\n')
    print(json.dumps({'case':name,'completed':row['completed'],'error':row.get('error','').splitlines()[-1][:180] if row.get('error')else None,'stages':[s['label']for s in row['stages']]},ensure_ascii=False),flush=True)


for name in ARGS.cases or list(BYCASE)+['yeyan-great-one-default','yeyan-great-split-default']:run(name)
