import collections
import hashlib
import json
import pathlib

BASE = pathlib.Path(__file__).resolve().parent
BROWSER = BASE / 'browser-round4'
NATIVE = BASE / 'native-round2'


def canonical_face(c):
    name = c.get('upstream_name', c.get('name'))
    return {'id': c['id'], 'name': name, 'suit': c['suit'], 'rank': c['rank'],
            'nature': c.get('nature') or ('thunder' if name == 'shandian' else None)}


def projection(state, native, skip):
    def cards(rows):
        return sorted((canonical_face(c) for c in rows), key=lambda c: c['id'])
    def flag(seat, name):
        return next((x['value'] for x in state.get('flags', []) if x['seat'] == seat and x['key'] == name), 0)
    players = []
    for p in state['players']:
        skill_used = {x['name']: x.get('once_used', False) for x in p['skills']}
        pred = flag(p['seat'], 'bolehuiyan_pred') if native else {'能命中': 1, '不能命中': 2}.get(p.get('bolehuiyan'), 0)
        players.append({'seat': p['seat'], 'hp': p['hp'], 'max_hp': p['max_hp'], 'face_down': p['face_down'],
                        'hand': cards(p['hand']), 'equips': cards(p['equips']), 'judge': cards(p['judge']),
                        'yixin_used': bool(flag(p['seat'], 'yixin_used') or skill_used.get('yixin')) if native else p['yixin_used'],
                        'guifu_used': bool(skill_used.get('guifu')) if native else bool(p['stat']['skill'].get('guifu', 0)),
                        'bolehuiyan_used': bool(flag(p['seat'], 'bolehuiyan_used_turn')) if native else p['bolehuiyan4'],
                        'bolehuiyan_pred': pred})
    return {'players': players,
            'deck_top_first': [canonical_face(c) for c in state['deck_top_first' if native else 'deck']],
            'discard': cards([c for c in state['discard'] if c['id'] not in skip]),
            'in_resolution': cards(state['table' if native else 'ordering'])}


def differences(a, b, path=''):
    if isinstance(a, dict) and isinstance(b, dict):
        return [d for k in sorted(a.keys() | b.keys()) for d in differences(a.get(k), b.get(k), path + '/' + k)]
    if isinstance(a, list) and isinstance(b, list):
        if len(a) != len(b):
            return [{'path': path, 'native': a, 'upstream': b}]
        return [d for i, (x, y) in enumerate(zip(a, b)) for d in differences(x, y, path + '/' + str(i))]
    return [] if a == b else [{'path': path, 'native': a, 'upstream': b}]


rows = []
for b in json.loads((BROWSER / 'results.json').read_text()):
    name = b['case']
    n = json.loads((NATIVE / f'native-{name}.json').read_text())
    final = b.get('final')
    if final is None:
        rows.append({'case': name, 'runner_error': b.get('runner_error'), 'projection_equal': None})
        continue
    skip = {c['id'] for c in final['skippedInitialDiscard']}
    np = projection(n['direct']['final'], True, skip)
    bp = projection(final, False, skip)
    source_prediction = [s for s in b['stages'] if s['snapshot']['choice']['name'] == 'chooseControl' and s['label'].startswith('actual_event_')]
    native_prediction = [c for c in n['direct']['pending_archive_checks'] if c['prompt'] == 'bolehuiyan:predict']
    all_source_ids = []
    for p in final['players']:
        for zone in ['hand', 'equips', 'judge']:
            all_source_ids.extend(c['id'] for c in p[zone])
    for zone in ['deck', 'discard', 'ordering']:
        all_source_ids.extend(c['id'] for c in final[zone])
    all_native_ids = []
    for p in n['direct']['final']['players']:
        for zone in ['hand', 'equips', 'judge']:
            all_native_ids.extend(c['id'] for c in p[zone])
    for zone in ['deck_top_first', 'discard', 'table']:
        all_native_ids.extend(c['id'] for c in n['direct']['final'][zone])
    expected = [c['id'] for c in json.loads((NATIVE / 'native-catalog.json').read_text())]
    diffs = differences(np, bp)
    prediction_count_equal = len(source_prediction) == len(native_prediction)
    rows.append({'case': name, 'native': str((NATIVE / f'native-{name}.json').relative_to(BASE)),
                 'upstream': str((BROWSER / f'upstream-{name}.json').relative_to(BASE)),
                 'source_event_completed': b['upstream_completed'], 'intentional_forced_pending': b.get('intentional_forced_pending', False),
                 'native_final_choice': n['direct']['final']['choice'],
                 'native_projection': np, 'upstream_projection': bp,
                 'projection_equal': not diffs, 'differences': diffs,
                 'prediction_windows': {'native': len(native_prediction), 'upstream': len(source_prediction)},
                 'prediction_window_counts_equal': prediction_count_equal,
                 'complete_initial_control_context_comparable': name != 'b_under_control',
                 'cross_engine_full_state_equivalence_claimed': False,
                 'native_all_catalog_entity_conservation': sorted(all_native_ids) == expected,
                 'upstream_mapped_entity_conservation': sorted(all_source_ids) == [i for i in expected if i not in skip],
                 'untagged_or_foreign_pile_observed': bool(final['untaggedDeck'] or final['foreignPileMutations']),
                 'native_direct_restore_state_projection_equal': n['direct']['final'] == n['restored']['final'],
                 'native_raw_bytes_equal': n['final_ngsa_bytes_equal'],
                 'native_pending_count_direct': len(n['direct']['pending_archive_checks'])})
summary = {'cases': len(rows), 'actual_source_completed': sum(r.get('source_event_completed', False) for r in rows),
           'intentional_forced_pending': sum(r.get('intentional_forced_pending', False) for r in rows),
           'runner_errors': sum(bool(r.get('runner_error')) for r in rows),
           'projection_equal': sum(r.get('projection_equal', False) for r in rows),
           'projection_different': sum(r.get('projection_equal') is False for r in rows),
           'native_prediction_window_counts_different': sum(not r.get('prediction_window_counts_equal', True) for r in rows)}
report = {'summary': summary,
          'comparison_projection': {'matched_card_face': 'native catalog id mapped one-to-one to real upstream DOM card, upstream_name, suit, rank, nature; Lightning inherent thunder is normalized explicitly',
                                    'compared': ['all eight HP/maxHP/turnedOver', 'hand/equip/judge exact mapped ids and faces (order ignored)', 'deck exact mapped ids/faces/order', 'mapped discard membership and in-resolution membership', 'yixin limited marker', 'guifu use marker', 'bolehuiyan once/prediction markers', 'number of real prediction windows'],
                                    'excluded': ['21 initial generic armor/horse native entities lack a truthful upstream card name, so no upstream placeholder entities were added', 'hand/discard order', 'upstream DOM/cardid and native internal id encoding', 'full JS event tree vs Rust machine stack and detailed timing', 'unrecorded general turn/phase transitions and all RNG state', 'UI/input latency and animation', 'unsupported native controlled-seat relation: b_under_control is deliberately labelled non-identical control context']},
          'rows': rows}
(BASE / 'comparison.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
print(json.dumps(summary, ensure_ascii=False))
for row in rows:
    if row.get('differences') or not row.get('prediction_window_counts_equal', True) or row.get('runner_error'):
        print(row['case'], 'prediction', row.get('prediction_windows'), 'diff_paths', [x['path'] for x in row.get('differences', [])], 'error', row.get('runner_error'))
