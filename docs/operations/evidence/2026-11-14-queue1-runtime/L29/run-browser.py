import importlib.util
import json
import pathlib
import sys
import traceback
import time

BASE = pathlib.Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('l29control', BASE / 'control.py')
c = importlib.util.module_from_spec(spec)
spec.loader.exec_module(c)
OUTPUT = BASE / (sys.argv[1] if len(sys.argv) > 1 else 'browser-round1')
OUTPUT.mkdir(exist_ok=False)
NATIVE = BASE / 'native-round3'
CASES = [r['case'] for r in json.loads((NATIVE / 'native-results.json').read_text())]
if len(sys.argv) > 2:
    CASES = sys.argv[2:]


def evaluate(expression):
    return c.evaluate(expression)


def wait(names):
    return c.wait_for(names, allow_done=True)


def skill(name):
    evaluate("if(!_status.event._skillChoice?.includes(SKILL))throw Error('skill not eligible: '+SKILL);ui.click.skill(SKILL);true".replace('SKILL', json.dumps(name)))


def target(seat):
    return evaluate("_status.clicked=false;ui.click.target.call(game.players[SEAT]);JSON.stringify({selected:ui.selected.targets.map(p=>game.players.indexOf(p)),selectable:game.players[SEAT].classList.contains('selectable')})".replace('SEAT', str(seat)))


def control(link):
    return evaluate("(()=>{const node=ui.controls.flatMap(n=>Array.from(n.children)).find(n=>n.link===LINK);if(!node)throw Error('missing control '+LINK);_status.clicked=false;ui.click.control.call(node);return true;})()".replace('LINK', json.dumps(link)))


def reload_page():
    c.evaluate('location.reload();true')
    last_error = None
    for _ in range(180):
        try:
            ready = c.evaluate("typeof game!=='undefined'&&game.players?.length===4&&game.me&&typeof lib.skill.mbcmqingzheng?.content==='function'&&typeof lib.skill.mbcmfangzhu?.content==='function'&&typeof game.players[0].init==='function'")
            if ready:
                return c.evaluate("JSON.stringify({url:location.href,players:game.players.map((p,seat)=>({seat,hp:p.hp,max_hp:p.maxHp,he:p.getCards('he').map(c=>({name:c.name,suit:c.suit,rank:c.number})),skills:p.getSkills(null,false,false),face_down:p.isTurnedOver(),criticalStorage:{mbcmfangzhu:game.players.indexOf(p.storage.mbcmfangzhu),mbcmfangzhu_jin:p.storage.mbcmfangzhu_jin,jiu:p.storage.jiu}}))})")
        except RuntimeError as error:
            last_error = str(error)
        time.sleep(0.15)
    raise RuntimeError('reload ready timeout: ' + str(last_error))


def check_fixture(fixture, actual):
    checks = []
    def faces(cards):
        return sorted([(c.get('id'),c.get('name'),c['suit'],c['rank']) for c in cards])
    for expected, observed in zip(fixture['players'], actual['players']):
        expected_hand = [{**card,'name':card['upstream_name']} for card in expected['hand']]
        expected_equips = [{**card,'name':card['upstream_name']} for card in expected['equips']]
        expected_skills = sorted('mashu' if x['name']=='马术' else x['name'] for x in expected['skills'])
        observed_skills = sorted(x['name'] for x in observed['skills'])
        check = {'seat':expected['seat'],'hp':expected['hp']==observed['hp'],'max_hp':expected['max_hp']==observed['max_hp'],
                 'face_down':expected['face_down']==observed['face_down'],'hand':faces(expected_hand)==faces(observed['hand']),
                 'equips':faces(expected_equips)==faces(observed['equips']),'all_primary_skills':expected_skills==observed_skills,
                 'expected_skills':expected_skills,'observed_skills':observed_skills}
        assert all(check[k] for k in ['hp','max_hp','face_down','hand','equips','all_primary_skills']), check
        checks.append(check)
    return checks


def run(name):
    fixture = json.loads((NATIVE / f'native-{name}.json').read_text())['direct']['initial']
    row = {'case': name, 'native_fixture': str(NATIVE.relative_to(BASE) / f'native-{name}.json'), 'fixture': fixture, 'stages': []}

    def stage(label, snap=None, **extra):
        snap = snap or c.snapshot()
        row['stages'].append({'label': label, 'snapshot': snap, **extra})
        return snap

    try:
        row['page_loaded_initial'] = reload_page()
        row['page_isolation'] = 'location.reload before this case; new page JS realm'
        row['start'] = c.start(name, fixture)
        row['fixture_checks'] = check_fixture(fixture, row['start']['initial'])
        if name.startswith('q_'):
            s = stage('phaseUseBegin', wait(['chooseButton', 'chooseToUse']))
            if name != 'q_no_hand':
                assert s['choice']['name'] == 'chooseButton', s['choice']
                if name in ['q_cancel_initial', 'q_cancel_suit_attempt']:
                    assert 'c' in s['choice'].get('confirm', ''), s['choice']
                    c.cancel()
                    stage('cancel_owner_suit', wait(['chooseToUse']))
                else:
                    suit = 'heart' if name == 'q_B' else 'spade'
                    evaluate("(()=>{const b=ui.dialog.buttons.find(b=>b.link[2]===SUIT);if(!b||!b.classList.contains('selectable'))throw Error('suit not selectable');_status.clicked=false;ui.click.button.call(b);return true;})()".replace('SUIT', json.dumps('lukai_' + suit)))
                    stage('owner_suit_selected')
                    c.ok()
                    s = stage('target_question', wait(['chooseTarget']))
                    if name == 'q_cancel_target':
                        assert 'c' in s['choice'].get('confirm', ''), s['choice']
                        c.cancel()
                        stage('cancel_target', wait(['chooseToUse']))
                    else:
                        seat = 2 if name == 'q_target_B' else 1
                        result = target(seat)
                        assert result['selected'] == [seat], result
                        c.ok()
                        s = stage('target_suit_question_after_owner_payment', wait(['chooseControl']))
                        selected = 'heart' if name == 'q_B' else 'diamond' if name == 'q_target_B' else 'club'
                        assert selected in s['choice']['controls'], s['choice']
                        control(selected)
                        stage('resolved', wait(['chooseToUse']))
        elif name.startswith('j_'):
            if 'damage' in name:
                s = stage('sha_damage_window', wait(['chooseToRespond', 'chooseBool']))
                if s['choice']['name'] == 'chooseToRespond':
                    c.cancel()
                    s = stage('sha_response_passed', wait(['chooseBool', 'chooseToRespond']))
                if name == 'j_damage_faceup':
                    if s.get('done'):
                        stage('faceup_damage_no_jiushi', s)
                    else:
                        stage('faceup_unexpected_question', s)
                else:
                    assert s['choice']['name'] == 'chooseBool', s['choice']
                    assert '翻面' in s['choice']['prompt'], s['choice']
                    if name == 'j_damage_back_cancel':
                        c.cancel()
                        stage('cancel_turnback', wait([]))
                    else:
                        c.ok()
                        s = stage('gain_after_turnback', wait(['chooseBool']))
                        assert '锦囊' in s['choice']['prompt'], s['choice']
                        c.ok()
                        stage('accept_gain', wait([]))
            else:
                s = stage('phaseUse_active_candidates', wait(['chooseToUse']))
                eligible = 'mbcmjiushi_1' in s['choice'].get('skills', [])
                row['active_eligible'] = eligible
                if name in ['j_active_facedown', 'j_no_skill']:
                    assert not eligible, s['choice']
                else:
                    assert eligible, s['choice']
                    skill('mbcmjiushi_1')
                    stage('jiushi_use_selected')
                    c.ok()
                    s = stage('gain_question', wait(['chooseBool']))
                    assert '锦囊' in s['choice']['prompt'], s['choice']
                    if name == 'j_active_gain':
                        c.ok()
                    else:
                        c.cancel()
                    stage('gain_answered', wait(['chooseToUse']))
        else:
            s = stage('phaseUse_active_candidates', wait(['chooseToUse']))
            eligible = 'mbcmfangzhu' in s['choice'].get('skills', [])
            row['active_eligible'] = eligible
            if name == 'f_no_skill':
                assert not eligible, s['choice']
            else:
                assert eligible, s['choice']
                skill('mbcmfangzhu')
                s = stage('target_question')
                if name == 'f_cancel_target':
                    c.cancel()
                    s = stage('cancel_target', wait(['chooseToUse']))
                    assert 'mbcmfangzhu' in s['choice'].get('skills', []), s['choice']
                    skill('mbcmfangzhu')
                    stage('reactivate_after_cancel')
                elif name in ['f_last_target', 'f_self_attempt']:
                    seat = 1 if name == 'f_last_target' else 0
                    result = target(seat)
                    assert not result['selectable'] and not result['selected'], result
                    stage('invalid_target_click_ignored', click_result=result)
                else:
                    seat = 2 if name == 'f_target_B' else 1
                    result = target(seat)
                    assert result['selected'] == [seat], result
                    c.ok()
                    s = stage('mode_question', wait(['chooseControl']))
                    assert s['choice']['controls'] == ['技能失效', '无法用牌'], s['choice']
                    if name == 'f_cancel_mode_attempt':
                        row['cancel_mode_available'] = any(x.get('link') in ['cancel', 'cancel2'] for group in s['choice']['uiControls'] for x in group)
                        assert not row['cancel_mode_available'], s['choice']
                        stage('mode_has_no_cancel_control')
                    else:
                        control('技能失效' if name == 'f_skill_A' else '无法用牌')
                        s = stage('resolved', wait(['chooseToUse']))
                        row['same_phase_skill_reavailable'] = 'mbcmfangzhu' in s['choice'].get('skills', [])
                        assert not row['same_phase_skill_reavailable'], s['choice']
        row['final'] = c.snapshot()
        row['runner_completed'] = True
    except Exception:
        row['runner_completed'] = False
        row['runner_error'] = traceback.format_exc()
        try:
            row['failure_snapshot'] = c.snapshot()
        except Exception:
            row['snapshot_error'] = traceback.format_exc()
    (OUTPUT / f'upstream-{name}.json').write_text(json.dumps(row, ensure_ascii=False, indent=2))
    print(json.dumps({'case':name,'runner_completed':row['runner_completed'],'error':row.get('runner_error'),'stages':[x['label'] for x in row['stages']]}, ensure_ascii=False), flush=True)
    return row


results = [run(name) for name in CASES]
(OUTPUT / 'results.json').write_text(json.dumps(results, ensure_ascii=False, indent=2))
print(json.dumps({'cases':len(results),'runner_completed':sum(r['runner_completed'] for r in results),'runner_failed':sum(not r['runner_completed'] for r in results)}, ensure_ascii=False), flush=True)
