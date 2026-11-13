import datetime
import hashlib
import json
import pathlib
import re
import sys

BASE = pathlib.Path(__file__).resolve().parent
native_dir = BASE / (sys.argv[1] if len(sys.argv) > 1 else 'native-round4')
output = BASE / (sys.argv[2] if len(sys.argv) > 2 else 'comparison-before-product-fix.json')
assert not output.exists(), output
upstream = json.loads((BASE / 'browser-round8/results.json').read_text())
native = {x['case']:x for x in json.loads((native_dir / 'native-results.json').read_text())}


def normalize_hand(cards):
    return sorted((c.get('id'), c.get('upstream_name', c.get('name')), c['suit'], c['rank'], c.get('nature')) for c in cards)


rows = []
for up in upstream:
    name = up['case']
    row = native[name]
    direct = row['direct']
    final = up['final']
    pairs = list(zip(final['players'], direct['final']['players']))
    checks = {
        'upstream_runner_completed': up['runner_completed'],
        'all_four_initial_fixtures_checked': len(up['fixture_checks']) == 4 and all(all(x[k] for k in ['hp','max_hp','face_down','hand','equips','all_primary_skills']) for x in up['fixture_checks']),
        'native_fixture_same_as_browser_input': direct['initial'] == up['fixture'],
        'final_hp': all(a['hp'] == b['hp'] for a,b in pairs),
        'final_max_hp': all(a['max_hp'] == b['max_hp'] for a,b in pairs),
        'final_face_down': all(a['face_down'] == b['face_down'] for a,b in pairs),
        'final_hand_id_name_suit_rank_nature_multiset': all(normalize_hand(a['hand']) == normalize_hand(b['hand']) for a,b in pairs),
        'final_equip_id_name_suit_rank_nature_multiset': all(normalize_hand(a['equips']) == normalize_hand(b['equips']) for a,b in pairs),
    }
    checkpoints = direct['pending_archive_checks']
    both_checks = checkpoints + row['restored']['pending_archive_checks']
    errors = [x for x in both_checks if x.get('error')]
    successful = [x for x in both_checks if not x.get('error')]
    assert all(checks.values()), (name, checks)
    assert all(x['state_hash_identical'] and x['snapshot_equal_except_auth_generation'] for x in successful), (name, successful)
    if not errors:
        left = (native_dir / f'{name}-direct-final-snapshot-debug.txt').read_text()
        right = (native_dir / f'{name}-restored-final-snapshot-debug.txt').read_text()
        full_final_same_except_auth = re.sub(r'^    auth_generation: \d+,$', '    auth_generation: NORMALIZED,', left, flags=re.M) == re.sub(r'^    auth_generation: \d+,$', '    auth_generation: NORMALIZED,', right, flags=re.M)
        assert full_final_same_except_auth, name
    else:
        full_final_same_except_auth = None
    spent = set(c['id'] for p in direct['initial']['players'] for c in p['hand'])
    differences = []
    if name in ['q_A', 'q_B', 'q_target_B']:
        up_choice = next(x['snapshot'] for x in up['stages'] if x['label'] == 'target_suit_question_after_owner_payment')
        native_choice = next(x['after'] for x in direct['commands'] if x['after']['choice'] and x['after']['choice']['prompt'] == 'mbcmqingzheng_target_suit')
        differences.append({'kind':'owner_payment_timing','upstream_owner_hand':[c['id'] for c in up_choice['players'][0]['hand']], 'native_owner_hand':[c['id'] for c in native_choice['players'][0]['hand']]})
    if name in ['q_A', 'q_target_B']:
        damage = [x for x in final['trace'] if x['method'] == 'damage_created']
        native_damage = [log for cmd in direct['commands'] for log in cmd['log_delta'] if log.startswith('DamageDealt')]
        assert damage[0]['source'] == 0 and 'source: None' in native_damage[0]
        differences.append({'kind':'damage_source','upstream':damage,'native':native_damage})
    if name == 'q_cancel_suit_attempt':
        assert direct['final']['choice']['prompt'] == 'mbcmqingzheng_suit'
        assert direct['commands'][-1]['reply'] == 'Rejected(InvalidChoice)'
        differences.append({'kind':'owner_suit_cancel','upstream_cancelled':True,'native_rejected_and_pending':True})
    if name == 'f_skill_A':
        up_skills = {x['name']:x for x in final['players'][1]['skills']}
        native_skills = {x['name']:x for x in direct['final']['players'][1]['skills']}
        assert up_skills['mashu']['blocked_by_fangzhu'] and not up_skills['mbcmjiushi']['blocked_by_fangzhu']
        assert native_skills['马术']['enabled'] and not native_skills['mbcmjiushi']['enabled']
        differences.append({'kind':'fangzhu_skill_scope','upstream':up_skills,'native':native_skills})
    if name in ['j_active_gain','j_active_cancel_gain']:
        up_gain = next(x['snapshot'] for x in up['stages'] if x['label'] == 'gain_question')
        native_gain = next(x['after'] for x in direct['commands'] if x['after']['choice'])
        assert up_gain['players'][0]['jiu_mark'] == 0 and native_gain['jiu_active']
        differences.append({'kind':'jiu_effect_timing_at_gain_question','upstream_jiu_mark':0,'native_jiu_active':True})
    rows.append({'case':name,'browser_file':f'browser-round8/upstream-{name}.json','native_file':str(native_dir.relative_to(BASE) / f'native-{name}.json'),
        'projection_checks':checks,'differences':differences,'restore':{'direct_checkpoints':len(checkpoints),'successful_attempts':len(successful),'failed_attempts':len(errors),'errors':errors,
        'all_successful_snapshots_equal_except_auth_generation':all(x['snapshot_equal_except_auth_generation'] for x in successful),
        'final_snapshot_equal_except_auth_generation_if_all_restores_succeeded':full_final_same_except_auth,
        'final_ngsa_bytes_equal':row['final_ngsa_bytes_equal'],'final_state_hash_equal':direct['final']['hash']==row['restored']['final']['hash']},
        'spent_fixture_cards_in_upstream_discard':sorted(c['id'] for c in final['discard'] if c.get('id') in spent),
        'spent_fixture_cards_in_native_discard':sorted(c['id'] for c in direct['final']['discard'] if c.get('id') in spent),
        'unmapped_initial_deck_count':len(up['start']['initial']['unmappedDeck']),
        'native_initial_discard_count':len(direct['initial']['discard']),'upstream_initial_discard_count':len(up['start']['initial']['discard'])})

report = {'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'native_directory':str(native_dir.relative_to(BASE)),
    'browser_directory':'browser-round8','case_count':len(rows),'projection_checks_passed':sum(all(x['projection_checks'].values()) for x in rows),
    'restore_attempts':sum(x['restore']['successful_attempts']+x['restore']['failed_attempts'] for x in rows),
    'restore_successful_attempts':sum(x['restore']['successful_attempts'] for x in rows),'restore_failed_attempts':sum(x['restore']['failed_attempts'] for x in rows),
    'cases_with_restore_failure':[x['case'] for x in rows if x['restore']['failed_attempts']],
    'semantic_equivalence_signed':False,'rows':rows,
    'not_compared':['full upstream/native state or event stream','full initial deck: 17 native cards have no chosen upstream face mapping','full initial discard: native16 vs upstream10; unmapped6 excluded',
      'RNG state/stream and native/upstream absolute event/choice IDs','full hand ordering and complete damageSource side effects','full extension wrapper/unlocking/visual hooks/config','target next-turn expiry and repeated real turns',
      'all native play/respond legality under fangzhu_ban; observed upstream cardEnabled query and native ban flag only', 'Cross-engine save/archive compatibility: NGSA checks are native-only']}
output.write_text(json.dumps(report, ensure_ascii=False, indent=2))
print(json.dumps({k:report[k] for k in ['case_count','projection_checks_passed','restore_attempts','restore_successful_attempts','restore_failed_attempts','cases_with_restore_failure']},ensure_ascii=False,indent=2))
