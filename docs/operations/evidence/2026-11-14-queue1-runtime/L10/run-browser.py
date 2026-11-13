import importlib.util
import json
import pathlib
import sys
import time
import traceback

BASE=pathlib.Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('l10control',BASE/'control.py')
c=importlib.util.module_from_spec(spec);spec.loader.exec_module(c)
OUT=BASE/sys.argv[1];OUT.mkdir(exist_ok=False)
NATIVE=BASE/'native-round2'
CASES=[x['case'] for x in json.loads((NATIVE/'native-results.json').read_text())]
if len(sys.argv)>2:CASES=sys.argv[2:]


def wait(names):return c.wait_for(names,allow_done=True)


def reload():
    c.evaluate('window.l10ReloadPending=true;setTimeout(()=>location.reload(),0);true')
    for _ in range(180):
        try:
            if c.evaluate("!window.l10ReloadPending&&typeof game!=='undefined'&&game.players?.length===4&&game.me&&!!lib.skill.reguhuo_guess"):
                break
        except Exception:pass
        time.sleep(.15)
    else:raise RuntimeError('new page ready timeout')
    return c.evaluate("""new Promise((resolve,reject)=>{
      const before={onfree:!!lib.onfree,waiting:!!_status.waitingForCards,deck:ui.cardPile.childNodes.length};
      if(lib.onfree)lib.init.onfree();
      let ticks=0,last=-1,quiet=0;
      const test=()=>{
        let count=ui.cardPile.childNodes.length;
        quiet=count===last?quiet+1:0;last=count;
        if(!lib.onfree&&!_status.waitingForCards&&quiet>=12)return resolve(JSON.stringify({before,after:{onfree:!!lib.onfree,waiting:!!_status.waitingForCards,deck:count},quiet_ms:quiet*50,players:game.players.map((p,seat)=>({seat,hp:p.hp,max_hp:p.maxHp,skills:p.getSkills(null,false,false),face_down:p.isTurnedOver(),he:p.getCards('he').map(c=>({name:c.name,suit:c.suit,rank:c.number}))}))}));
        if(++ticks>240)return reject(Error('onfree/cardsAsync quiet timeout'));setTimeout(test,50);
      };test();
    })""")


def fixture_check(f,actual):
    def faces(cards):return sorted((x.get('id'),x.get('upstream_name',x.get('name')),x['suit'],x['rank'],x.get('nature'))for x in cards)
    checks=[]
    for a,b in zip(f['players'],actual['players']):
        row={'seat':a['seat'],'hp':a['hp']==b['hp'],'max_hp':a['max_hp']==b['max_hp'],'face_down':a['face_down']==b['face_down'],
             'hand':faces(a['hand'])==faces(b['hand']),'equips':faces(a['equips'])==faces(b['equips']),'judge':faces(a['judge'])==faces(b['judge']),
             'all_primary_skills':sorted(x['name']for x in a['skills'])==sorted(x['name']for x in b['skills'])}
        assert all(v for k,v in row.items()if k!='seat'),row;checks.append(row)
    assert faces(f['deck_top_first'])==faces(actual['deck'])
    assert [x['id']for x in f['deck_top_first']]==[x['id']for x in actual['deck']]
    assert not actual['untaggedDeck'],actual['untaggedDeck']
    return checks


def button(name):
    c.evaluate("(()=>{const b=ui.dialog.buttons.find(b=>Array.isArray(b.link)&&b.link[2]===NAME&&!b.link[3]);if(!b||!b.classList.contains('selectable'))throw Error('button not selectable '+NAME);_status.clicked=false;ui.click.button.call(b);return true;})()".replace('NAME',json.dumps(name)))


def card(card_id):
    c.evaluate("(()=>{const b=game.me.getCards('hs').find(c=>c.l10NativeId===ID);if(!b||!b.classList.contains('selectable'))throw Error('card not selectable '+ID);_status.clicked=false;ui.click.card.call(b);return true;})()".replace('ID',str(card_id)))


def target(seat):
    return c.evaluate("_status.clicked=false;ui.click.target.call(game.players[SEAT]);JSON.stringify(ui.selected.targets.map(p=>game.players.indexOf(p)))".replace('SEAT',str(seat)))


def control(name):
    return c.evaluate("(()=>{const b=ui.controls.flatMap(n=>Array.from(n.children)).find(n=>n.link===NAME);if(!b)throw Error('control missing '+NAME);_status.clicked=false;ui.click.control.call(b);return true;})()".replace('NAME',json.dumps(name)))


def cancel_visible():
    s=c.snapshot();confirm=s['choice'].get('confirm','')
    if 'c' in confirm:c.cancel();return 'confirm.cancel'
    nodes=[x.get('link')for group in s['choice']['uiControls']for x in group]
    if '结束回合'in nodes:control('结束回合');return 'visible_end_button'
    raise RuntimeError('no cancel/end control available: '+str(s['choice']))


def run(name):
    f=json.loads((NATIVE/f'native-{name}.json').read_text())['direct']['initial']
    row={'case':name,'native_fixture':f,'stages':[]}
    def stage(label,s=None,**extra):
        s=s or c.snapshot();row['stages'].append({'label':label,'snapshot':s,**extra});return s
    try:
        row['fresh_page_loaded']=reload()
        row['source_load']=c.evaluate((BASE/'load-source.js').read_text())
        try:row['start']=c.start(name,f)
        except RuntimeError:
            row['actual_trigger_error']=traceback.format_exc()
            actual=c.evaluate('JSON.stringify(window.l10Initial)')
            row['fixture_checks']=fixture_check(f,actual);row['initial']=actual
            if name.startswith('k_'):
                stage('actual_k_danggu_trigger_failed')
                try:c.evaluate('EpicFX.initSCSAnim();true')
                except RuntimeError:row['actual_animation_dependency_error']=traceback.format_exc()
                row['upstream_completed']=False;row['boundary']='Actual forced phaseBefore entered k_danggu; custom UI dependency failed, no stub or Spine runtime loaded.'
                return row
            raise
        row['initial']=row['start']['initial'];row['fixture_checks']=fixture_check(f,row['initial'])
        if name.startswith('g_'):
            s=stage('actual_use_or_response_candidates',wait(['chooseToUse','chooseToRespond']))
            eligible='reguhuo'in s['choice'].get('skills',[]);row['eligible']=eligible
            if name in ['g_no_hand','g_no_skill']:
                assert not eligible,s['choice']
            else:
                assert eligible,s['choice']
                c.evaluate("ui.click.skill('reguhuo');true");stage('skill_selected');c.ok()
                s=stage('declaration_name',wait(['chooseButton']))
                if name=='g_cancel_name':
                    row['cancel_method']=cancel_visible();stage('name_cancelled',wait(['chooseToUse','chooseToRespond']))
                else:
                    button('tao'if name=='g_tao'else'shan'if name=='g_response'else'sha');c.ok()
                    s=stage('carrier_selection',wait(['chooseToUse','chooseToRespond']))
                    if name=='g_cancel_card':
                        row['cancel_method']=cancel_visible();stage('carrier_cancelled',wait(['chooseToUse','chooseToRespond']))
                    else:
                        carrier=f['players'][0]['hand'][0]['id'];card(carrier);stage('carrier_selected')
                        if name=='g_cancel_target':
                            row['cancel_method']=cancel_visible();stage('target_cancelled',wait(['chooseToUse','chooseToRespond']))
                        else:
                            s=c.snapshot()
                            if name!='g_response':
                                wanted=0 if name=='g_tao' else 1
                                if wanted in s['choice']['selectablePlayers']:target(wanted)
                            c.ok()
                            for ask in range(12):
                                s=stage('question_or_effect_'+str(ask),wait(['chooseButton','chooseToDiscard','chooseToUse','chooseToRespond']))
                                if s.get('done'):break
                                question=s['choice'];buttons=question.get('buttons')or[]
                                if question['name']=='chooseButton'and any(isinstance(b['link'],list)and b['link'][2]=='reguhuo_ally'for b in buttons):
                                    challenge=(name=='g_sha_question_fake'and question['actor']==1)or(name.startswith('g_true_question')and question['actor']==2)
                                    button('reguhuo_betray'if challenge else'reguhuo_ally');c.ok()
                                elif question['name']=='chooseToDiscard':
                                    if name=='g_true_question_discard':card(f['players'][2]['hand'][0]['id']);c.ok()
                                    else:cancel_visible()
                                elif question['name']in['chooseToUse','chooseToRespond']:
                                    if question.get('prompt')and question['actor']!=0:
                                        cancel_visible()
                                    else:break
                                else:raise RuntimeError('unexpected guhuo choice '+str(question))
        elif name.startswith('r_'):
            s=stage('after_real_sha',wait(['chooseControl','chooseToUse','chooseToRespond']))
            if name in ['r_no_skill','r_other_target']:
                assert s.get('done')or s['choice']['name']!='chooseControl',s['choice']
            else:
                loops=2 if name=='r_double' else 1
                for turn in range(loops):
                    assert s['choice']['name']=='chooseControl',s['choice']
                    assert '获得牌'in s['choice']['controls'],s['choice']
                    if name=='r_cancel_choice':control('取消');stage('literal_cancel_selected',wait(['chooseTarget','chooseToUse']));break
                    if 'move'in name:
                        assert '移动牌'in s['choice']['controls'],s['choice'];control('移动牌')
                        s=stage('move_from_to_question',wait(['chooseTarget','choosePlayerCard','chooseButton']))
                        if s['choice']['name']=='chooseTarget':
                            target(1);stage('move_source_selected');target(2 if name=='r_move_B' else 0);stage('move_destination_selected');c.ok()
                            s=stage('move_card_or_finished',wait(['choosePlayerCard','chooseButton','chooseControl']))
                        if s['choice']['name']in['choosePlayerCard','chooseButton']and not s.get('done'):
                            c.evaluate("(()=>{const b=ui.dialog.buttons.find(b=>b.classList.contains('selectable'));if(!b)throw Error('no movable card');_status.clicked=false;ui.click.button.call(b);return true;})()");c.ok()
                        s=stage('moved',wait(['chooseControl','chooseToUse']))
                    else:
                        control('获得牌');s=stage('gain_type_after_draw_'+str(turn),wait(['chooseButton']))
                        if name=='r_cancel_type':row['cancel_method']=cancel_visible();stage('type_cancelled',wait(['chooseControl','chooseToUse']));break
                        button('equip'if name=='r_equip'else'trick'if name=='r_trick'else'basic');c.ok()
                        s=stage('gain_resolved_'+str(turn),wait(['chooseControl','chooseToUse']))
        else:
            s=stage('phase_before_filter_result',wait(['chooseToUse','chooseButton']))
            assert name in ['k_no_skill','k_after_initial_no_trigger'],s['choice']
            assert not any(x['method']=='logSkill'and x['args']and x['args'][0]=='k_danggu'for x in s['trace'])
        row['upstream_completed']=True
        return row
    except Exception:
        row['upstream_completed']=False;row['runner_error']=traceback.format_exc();return row
    finally:
        try:
            row['final']=c.snapshot();row['untagged_deck_count_final']=len(row['final']['untaggedDeck'])
            if row['untagged_deck_count_final']:
                row['comparison_excluded']='Late cardsAsync/onfree added untagged cards';row['upstream_completed']=False
        except Exception:row['final_snapshot_error']=traceback.format_exc()
        (OUT/f'upstream-{name}.json').write_text(json.dumps(row,ensure_ascii=False,indent=2))
        print(json.dumps({'case':name,'upstream_completed':row.get('upstream_completed'),'error':row.get('runner_error')or row.get('actual_trigger_error'),'stages':[x['label']for x in row['stages']],'untagged_deck':row.get('untagged_deck_count_final')},ensure_ascii=False),flush=True)


results=[run(name)for name in CASES]
(OUT/'results.json').write_text(json.dumps(results,ensure_ascii=False,indent=2))
print(json.dumps({'cases':len(results),'upstream_completed':sum(x.get('upstream_completed',False)for x in results),'failed_or_blocked':sum(not x.get('upstream_completed',False)for x in results)},ensure_ascii=False),flush=True)
