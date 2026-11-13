import json
import pathlib

BASE = pathlib.Path(__file__).resolve().parent
rows = json.loads((BASE / 'browser-round4/results.json').read_text())
audit = []
for row in rows:
    name = row['case']
    initial = row['initial']
    native = row['native_fixture']
    controls = []
    for p in initial['players']:
        seat = p['seat']
        expected_true_me = 0 if name == 'b_under_control' and seat == 2 else -1
        expected_under_control = name == 'b_under_control' and seat == 2
        check = {'seat': seat, 'true_me_expected': p['trueMe'] == expected_true_me,
                 'is_under_control_expected': p['isUnderControl'] == expected_under_control,
                 'stat_card_and_skill_empty': p['stat'] == {'card': {}, 'skill': {}},
                 'no_yixin_used': not p['yixin_used'], 'no_bolehuiyan_prediction': p.get('bolehuiyan') is None,
                 'no_bolehuiyan_once': not p['bolehuiyan4'], 'alive': p['alive'],
                 'no_awakened_skills': p['awakenedSkills'] == []}
        assert all(v for k, v in check.items() if k != 'seat'), (name, check)
        controls.append(check)
    assert native['flags'] == [], (name, native['flags'])
    assert native['once'] == '{}', (name, native['once'])
    assert all(not s['once_used'] for p in native['players'] for s in p['skills']), name
    assert not initial['foreignPileMutations'] and not initial['untaggedDeck'], name
    audit.append({'case': name, 'existing_fixture_checks': row['fixture_checks'],
                  'additional_key_storage_and_control_checks': controls,
                  'native_flags_empty': True, 'native_aux_once_empty': True,
                  'native_skill_once_unused': True, 'no_foreign_pile_mutation': True,
                  'controlled_seat_relation_comparable_to_native': name != 'b_under_control'})
report = {'cases': len(rows), 'seat_checks': sum(len(r['additional_key_storage_and_control_checks']) for r in audit),
          'all_assertions_satisfied': True,
          'control_relation_fixture_scope': 'b_under_control sets the original _trueMe field before the real Sha event. It does not obtain possession through a separate original skill. Native has no corresponding relation in this fixture. All other cases have no _trueMe relation.',
          'rows': audit}
(BASE / 'initial-validation-audit.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({k: report[k] for k in ['cases', 'seat_checks', 'all_assertions_satisfied']}))
