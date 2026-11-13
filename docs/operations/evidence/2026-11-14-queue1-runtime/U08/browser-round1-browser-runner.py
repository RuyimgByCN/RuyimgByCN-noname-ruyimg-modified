import importlib.util
import json
import pathlib
import sys
import time
import traceback

BASE = pathlib.Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('u08control', BASE / 'control.py')
c = importlib.util.module_from_spec(spec)
spec.loader.exec_module(c)
OUT = BASE / sys.argv[1]
OUT.mkdir(exist_ok=False)
NATIVE = BASE / 'native-round2'
CASES = [x['case'] for x in json.loads((NATIVE / 'native-results.json').read_text())]
if len(sys.argv) > 2:
    CASES = sys.argv[2:]


def wait(names):
    return c.wait_for(names, allow_done=True)


def reload():
    c.evaluate('window.u08ReloadPending=true;setTimeout(()=>location.reload(),0);true')
    for _ in range(180):
        try:
            if c.evaluate("(async()=>{if(window.u08ReloadPending)return false;const n=await import('/noname.js');window.q1=n;return n.game.players?.length===8&&!!n.lib.card.sha})()"):
                break
        except Exception:
            pass
        time.sleep(.15)
    else:
        raise RuntimeError('new page ready timeout')
    return c.evaluate("""new Promise((resolve,reject)=>{
      const {game,lib,ui,_status}=q1;
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


def fixture_check(f, actual):
    def faces(cards):
        return sorted((x.get('id'), x.get('upstream_name', x.get('name')), x['suit'], x['rank'], x.get('nature')) for x in cards)
    checks = []
    assert len(f['players']) == len(actual['players']) == 8
    for a, b in zip(f['players'], actual['players']):
        row = {'seat': a['seat'], 'hp': a['hp'] == b['hp'], 'max_hp': a['max_hp'] == b['max_hp'],
               'face_down': a['face_down'] == b['face_down'], 'hand': faces(a['hand']) == faces(b['hand']),
               'equips': faces(a['equips']) == faces(b['equips']), 'judge': faces(a['judge']) == faces(b['judge']),
               'all_primary_skills': sorted(x['name'] for x in a['skills']) == sorted(x['name'] for x in b['skills']),
               'virtual_equips_match_physical': sorted(b['virtualEquips']) == sorted(x['upstream_name'] for x in a['equips']),
               'virtual_judges_match_physical': sorted(b['virtualJudges']) == sorted(x['upstream_name'] for x in a['judge']),
               'no_awakened_skills': not b['awakenedSkills'], 'no_prediction_storage': b.get('bolehuiyan') is None,
               'no_prediction_once_skill': not b['bolehuiyan4']}
        assert all(v for k, v in row.items() if k != 'seat'), row
        checks.append(row)
    assert faces(f['deck_top_first']) == faces(actual['deck'])
    assert [x['id'] for x in f['deck_top_first']] == [x['id'] for x in actual['deck']]
    assert not actual['untaggedDeck'], actual['untaggedDeck']
    mapped = [s for s in f['discard'] if s['upstream_name'] is not None]
    assert faces(mapped) == faces(actual['discard'])
    assert not actual['ordering']
    return {'players': checks, 'deck_exact_ids_faces_order': True, 'mapped_discard_exact': True,
            'unmapped_native_discard_excluded': actual['skippedInitialDiscard'], 'ordering_empty': True}


def card(card_id):
    return c.evaluate("(()=>{const {game,ui,_status}=q1;const b=game.me.getCards('he').find(c=>c.u08NativeId===ID);if(!b||!b.classList.contains('selectable'))throw Error('card not selectable '+ID);_status.clicked=false;ui.click.card.call(b);return true;})()".replace('ID', str(card_id)))


def target(seat):
    return c.evaluate("(()=>{const {game,ui,_status}=q1;const p=game.players[SEAT];if(!p.classList.contains('selectable'))throw Error('target not selectable '+SEAT);_status.clicked=false;ui.click.target.call(p);return JSON.stringify(ui.selected.targets.map(p=>game.players.indexOf(p)))})()".replace('SEAT', str(seat)))


def control(name):
    return c.evaluate("(()=>{const {ui,_status}=q1;const b=ui.controls.flatMap(n=>Array.from(n.children)).find(n=>n.link===NAME);if(!b)throw Error('control missing '+NAME);_status.clicked=false;ui.click.control.call(b);return true;})()".replace('NAME', json.dumps(name)))


def cancel_visible():
    s = c.snapshot()
    confirm = s['choice'].get('confirm', '')
    if 'c' in confirm:
        c.cancel()
        return 'confirm.cancel'
    nodes = [x.get('link') for group in s['choice']['uiControls'] for x in group]
    if '结束回合' in nodes:
        control('结束回合')
        return 'visible_end_button'
    raise RuntimeError('no cancel/end control available: ' + str(s['choice']))


def select_skill(skill):
    return c.evaluate("q1._status.clicked=false;q1.ui.click.skill(" + json.dumps(skill) + ");true")


def run(name):
    f = json.loads((NATIVE / f'native-{name}.json').read_text())['direct']['initial']
    row = {'case': name, 'native_fixture': f, 'stages': []}
    def stage(label, s=None, **extra):
        s = s or c.snapshot()
        row['stages'].append({'label': label, 'snapshot': s, **extra})
        return s
    try:
        row['fresh_page_loaded'] = reload()
        row['start'] = c.start(name, f)
        row['initial'] = row['start']['initial']
        row['fixture_checks'] = fixture_check(f, row['initial'])
        if name.startswith(('y_', 'g_')):
            skill = 'yixin' if name.startswith('y_') else 'guifu'
            s = stage('actual_phase_use_candidates', wait(['chooseToUse']))
            eligible = skill in s['choice'].get('skills', [])
            row['skill_offered'] = eligible
            if name.endswith('no_skill') or name == 'y_two_cards':
                assert not eligible, s['choice']
                row['negative_no_skill_candidate'] = True
                row['end_choice'] = cancel_visible()
            elif name in ['y_no_wound', 'g_no_equip', 'g_only_own']:
                if eligible:
                    select_skill(skill)
                    s = stage('actual_no_valid_target')
                    assert not s['choice']['selectablePlayers'], s['choice']
                    assert 'o' not in s['choice'].get('confirm', ''), s['choice']
                    row['cancel_unusable_skill'] = cancel_visible()
                    stage('back_to_phase_candidates', wait(['chooseToUse']))
                row['end_choice'] = cancel_visible()
            else:
                assert eligible, s['choice']
                select_skill(skill)
                s = stage('skill_selected')
                if name.startswith('y_') and name != 'y_cancel_cards':
                    start = int(name == 'y_cost_B')
                    for spec in f['players'][0]['hand'][start:start+2]:
                        card(spec['id'])
                    s = stage('two_real_cost_cards_selected')
                if name in ['y_cancel_cards', 'y_cancel_target', 'g_cancel_target']:
                    row['cancel_selection'] = cancel_visible()
                    s = stage('cancelled_back_to_phase', wait(['chooseToUse']))
                    assert skill in s['choice'].get('skills', []), s['choice']
                    select_skill(skill)
                    stage('skill_reopened_after_cancel')
                    row['cancel_reopened'] = cancel_visible()
                    wait(['chooseToUse'])
                    row['end_choice'] = cancel_visible()
                else:
                    seat = 2 if name.endswith('target_B') else 0 if name == 'y_self' else 1
                    target(seat)
                    stage('target_selected')
                    c.ok()
                    if name.endswith('reuse'):
                        s = stage('actual_second_use_candidates', wait(['chooseToUse']))
                        assert skill not in s['choice'].get('skills', []), s['choice']
                        row['reuse_rejected_by_candidates'] = True
                        row['end_choice'] = cancel_visible()
        else:
            wants_dodge = name in ['b_miss', 'b_wrong_A', 'b_wrong_B']
            prediction_count = 0
            for turn in range(20):
                s = stage(f'actual_event_{turn}', wait(['chooseControl', 'chooseToUse', 'chooseToRespond', 'chooseToDiscard']))
                if s['done']:
                    break
                kind = s['choice']['name']
                if kind == 'chooseControl':
                    prediction_count += 1
                    controls = s['choice']['controls']
                    assert controls == ['能命中', '不能命中', 'cancel'], controls
                    choice = 'cancel' if name == 'b_cancel_predict' or name == 'b_cancel_then_retry' and prediction_count == 1 else '不能命中' if name in ['b_miss', 'b_wrong_miss', 'b_cancel_discard'] else '能命中'
                    control(choice)
                    stage(f'prediction_control_{prediction_count}_clicked', choice=choice)
                elif kind in ['chooseToUse', 'chooseToRespond']:
                    options = [x for x in s['choice']['cards'] if x['seat'] == s['choice']['actor'] and x['selectable']]
                    shan = next((x for x in options if x['name'] == 'shan'), None)
                    if wants_dodge and shan:
                        card(shan['id'])
                        stage('real_shan_selected')
                        c.ok()
                    else:
                        row.setdefault('declined_shan', []).append(cancel_visible())
                elif kind == 'chooseToDiscard':
                    if name == 'b_cancel_discard':
                        row['forced_discard_cancel_visible'] = 'c' in s['choice'].get('confirm', '')
                        row['intentional_forced_pending'] = True
                        break
                    options = [x for x in s['choice']['cards'] if x['seat'] == 0 and x['selectable']]
                    card(options[int(name == 'b_wrong_B')]['id'])
                    stage('wrong_prediction_real_discard_selected')
                    c.ok()
                else:
                    raise RuntimeError('unexpected choice ' + str(s['choice']))
            else:
                raise RuntimeError('event loop exhausted')
            row['prediction_window_count'] = prediction_count
        if not row.get('intentional_forced_pending'):
            stage('actual_root_completed', wait([]))
        row['final'] = stage('final')['snapshot'] if False else c.snapshot()
        assert not row['final']['error'], row['final']['error']
        row['upstream_completed'] = bool(row['final']['done'])
        row['no_untagged_final_pile'] = not row['final']['untaggedDeck'] and not row['final']['foreignPileMutations']
    except Exception:
        row['runner_error'] = traceback.format_exc()
        try:
            row['failure_snapshot'] = c.snapshot()
        except Exception:
            pass
        row['upstream_completed'] = False
    (OUT / f'upstream-{name}.json').write_text(json.dumps(row, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'case': name, 'completed': row['upstream_completed'], 'intentional_pending': row.get('intentional_forced_pending', False), 'error': row.get('runner_error'), 'choice': (row.get('final') or row.get('failure_snapshot') or {}).get('choice')}, ensure_ascii=False), flush=True)
    return row


rows = [run(name) for name in CASES]
(OUT / 'results.json').write_text(json.dumps(rows, ensure_ascii=False, indent=2) + '\n')
