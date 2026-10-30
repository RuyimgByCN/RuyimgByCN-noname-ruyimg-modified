from pathlib import Path
import json, hashlib, tempfile

root = Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-efhmbq7e')
e = root/'evidence/phase3'
v = json.loads((e/'author257-full-vector.json').read_bytes())
records = json.loads((e/'desktop-ipc-full-replies.json').read_bytes())['records']
projection = json.loads((e/'desktop-web/projection.json').read_bytes())
out = Path(tempfile.mkdtemp(prefix='noname-parent-c331-ipc-audit-'))

def equal(a,b,where):
    assert json.dumps(a,sort_keys=True,ensure_ascii=False,separators=(',',':')) == json.dumps(b,sort_keys=True,ensure_ascii=False,separators=(',',':')), where

# JSON types remain distinct, including bool versus integer.
assert json.dumps(True) != json.dumps(1)

def view(raw,revision):
    def card(cid,owner,zone):
        face=raw['cards'][str(cid)]
        return {'id':cid,'def':face['def'],'suit':face['suit'],'rank':face['rank'],'owner':owner,'zone':zone,'kind':face.get('kind','Other')}
    seats=[];cards=[];own=[]
    for p in raw['players']:
        for key,zone in [('equips','Equip'),('judge','Judge')]:
            cards.extend(card(cid,p['seat'],zone) for cid in p[key])
        seats.append({'player':p['seat'],'character_def':p.get('characterDef',0),'generals':p.get('generals',[None,None]),'seat':p['seat'],'hp':p['hp'],'max_hp':p['maxHp'],'alive':p['alive'],'turned_over':p['turnedOver'],'hand_count':p['handCount'],'identity':p.get('identity'),'skills':[{'id':0,'def':0,'enabled':True,'use_count':0,'active':False,'view_as':False,'locked':False,'name':name} for name in p['skills']]})
        if p['seat']==raw['seat']:
            own.extend(card(cid,raw['seat'],'Hand') for cid in p['hand'].get('own',[]))
    own.extend(cards)
    for key,zone in [('discard','DiscardPile'),('table','Processing')]:
        own.extend(card(cid,None,zone) for cid in raw[key])
    return {'revision':revision,'state_hash':'','observer':raw['seat'],'round':raw['round'],'current_player':raw['turn'],'phase':{'judge':'Judge','draw':'Draw','play':'Play','discard':'Discard','end':'End'}.get(raw['phase'],'Idle'),'seats':seats,'known_cards':own,'outcome':raw['outcome']}

def choice(raw,revision,seat):
    if raw is None or raw.get('hidden') is True:return None
    def label(option):
        assert isinstance(option,dict) and set(option)=={'skill'}, 'this fixture uses only typed Skill options'
        return 'skill:'+option['skill']
    result={'choice_id':raw['id'],'revision':revision,'observer':seat,'kind':'OrderTriggers','options':[{'token':i,'label_key':label(option)} for i,option in enumerate(raw['options'])]}
    for a,b in [('prompt','prompt'),('native_kind','native_kind'),('min','min_count'),('max','max_count'),('allowCancel','allow_cancel')]:
        if a in raw:result[b]=raw[a]
    return result

assert len(records)==545
assert [r['request']['id'] for r in records] == list(range(1,31))+list(range(32,547))
assert all(r['response']['id']==r['request']['id'] and type(r['response']['ok']) is bool for r in records)
assert len(set(r['resource'] for r in records[:16]))==4
assert json.loads((e/'desktop-web/vector.json').read_bytes())==v
counts={'archive':0,'view':0,'choice':0,'hash':0,'event_suffix':0,'outcome':0,'invalid':0}

def save(rec,expected):
    assert rec['request']['type']=='save' and rec['response']['ok'] is True
    equal(rec['response']['data'],{'hex':expected,'kind':'NGSA'},'full save payload')
    counts['archive']+=1

def envelope(rec,point,seat,events=False,outcome=None):
    assert rec['response']['ok'] is True
    data=rec['response']['data']
    assert set(data)=={'choice','events','finished','hash_hex','legal_actions','result','revision','view'}
    equal(data['view'],view(point['views'][seat],data['revision']),'entire projected view')
    equal(data['choice'],choice(point['choices'][seat],data['revision'],seat),'entire projected choice')
    equal(data['hash_hex'],point['hash'],'entire hash')
    assert data['finished']==(point['views'][seat]['outcome'] is not None)
    counts['view']+=1;counts['choice']+=1;counts['hash']+=1
    if events:
        codes=[event['PublicEvent']['code'] for event in data['events']]
        equal(codes,point['events'][-len(codes):] if codes else [],'every event code suffix')
        counts['event_suffix']+=1
    if outcome is not None:
        equal(data['result'],outcome,'entire submission result');counts['outcome']+=1

for seat in range(4):
    init,start,restore,saved=records[seat*4:seat*4+4]
    assert init['request']['type']=='init' and init['response']['ok']
    assert start['request']['type']=='start' and start['request']['observer']==seat and start['response']['ok']
    assert restore['request']['hex']==v['initial'] and restore['request']['packsJson']==v['packs_json']
    envelope(restore,v['before'],seat)
    expected_projection=dict(projection['before'][seat]);expected_projection['revision']=restore['response']['data']['revision']
    equal(restore['response']['data']['view'],expected_projection,'projection fixture binding')
    save(saved,v['before']['archive'])
owner=records[4]['resource']
invalid=[(0,v['choice_id'],[256]),(1,v['choice_id'],[]),(1,v['choice_id'],[0,1]),(1,v['choice_id'],[257]),(1,v['choice_id']+1,[256])]
for i,(actor,cid,selections) in enumerate(invalid):
    rec,saved=records[16+i*2:18+i*2]
    assert rec['resource']==owner
    equal({k:x for k,x in rec['request'].items() if k!='id'},{'type':'resolveChoice','actor':actor,'choiceId':cid,'selections':selections},'exact invalid request')
    assert rec['response']['ok'] is False or rec['response']['data']['result']['outcome']=='rejected'
    save(saved,v['before']['archive']);counts['invalid']+=1
assert records[26]['request']['selections']==[256]
envelope(records[26],v['after'],1,True,v['outcome']);save(records[27],v['after']['archive'])
assert records[28]['response']['ok'] is False or records[28]['response']['data']['result']['outcome']=='rejected'
save(records[29],v['after']['archive'])
assert records[30]['request']['type']=='init' and records[30]['response']['ok']
assert records[31]['request']['type']=='start' and records[31]['response']['ok']
assert records[32]['request']['hex']==v['after']['archive'] and records[32]['request']['packsJson']==v['packs_json']
envelope(records[32],v['restart_before'],1);save(records[33],v['restart_before']['archive'])
assert len(v['continuation'])==255
for i,point in enumerate(v['continuation']):
    rec,saved=records[34+i*2:36+i*2]
    equal({k:x for k,x in rec['request'].items() if k!='id'},{'type':'resolveChoice','actor':1,'choiceId':point['choice_id'],'selections':point['answer']},'entire continuation request')
    envelope(rec,point['after'],1,True,point['outcome']);save(saved,point['after']['archive'])
save(records[-1],v['terminal']['archive'])
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
product=json.loads((e/'desktop-ipc-result.json').read_bytes())
binary=root/'target/debug/noname-next-desktop'
assert sha(binary)==product['binary_sha256']
files=[e/'desktop-ipc-full-replies.json',e/'author257-full-vector.json',e/'desktop-web/projection.json',e/'desktop-web/index.html',e/'desktop-run.py',e/'private-wasm-preload.mjs',e/'platform-commands.json',e/'final-freeze3/source-freeze.json',binary]
files.extend(sorted((e/'native-pkg').glob('*.js')));files.extend(sorted((e/'native-pkg').glob('*.wasm')))
result={'result':'PASS','record_count':545,'counts':counts,'continuation_steps':255,'json_comparison':'all keys and scalar types, object key ordering ignored only','matched_archives_include_full_rng_state':True,'source_projection':'manual independent translation of frozen native_envelope.rs snapshot/choice','binary_matches_recorded_sha':True,'unverified':['legal_actions have no Native expected in the supplied vector','five pre-fixture start envelopes only checked ok, not Native compared','event labels not retranslated, every code compared','missing closed-resource request id31 inferred from driver and id gap, no failed invoke record','binary/source build provenance depends on recorded argv/freeze review, binary hash alone does not prove reproducible source build'],'files':[{'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p)} for p in files]}
(out/'independent-results.json').write_text(json.dumps(result,indent=2)+'\n')
Path('/tmp/noname-parent-c331-ipc-audit-current.json').write_text(json.dumps({'out':str(out)},indent=2)+'\n')
print(json.dumps({'out':str(out),'result':'PASS','counts':counts,'bound_files':len(files)},indent=2))
