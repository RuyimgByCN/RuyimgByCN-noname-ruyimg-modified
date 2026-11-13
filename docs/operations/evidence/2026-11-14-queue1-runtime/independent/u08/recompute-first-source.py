#!/usr/bin/env python3
"""Independently derive U08 projections and CDP provenance from frozen raw inputs."""
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import sys

BASE = Path(__file__).resolve().parent
ORIGINAL = BASE.parents[1] / 'U08'
LABEL = sys.argv[1]


def read(relative):
    return json.loads((ORIGINAL / relative).read_text())


def write(name, value):
    with (BASE / f'{name}-{LABEL}.json').open('x') as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write('\n')


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(',', ':'))


def face(card, omit_nature=False):
    row = {'id': card.get('id'), 'name': card.get('upstream_name', card.get('name')),
           'suit': card['suit'], 'rank': card['rank']}
    if not omit_nature:
        row['nature'] = card.get('nature') or None
    return row


def faces(cards, ordered=False, omit_nature=False):
    rows = [face(c, omit_nature) for c in cards]
    return rows if ordered else sorted(rows, key=canonical)


def ids(cards):
    return [c.get('id') for c in cards]


def difference(path, native, upstream):
    return [] if native == upstream else [{'field': path, 'native': native, 'upstream': upstream}]


native_rows = read('native-round2/native-results.json')
upstream_rows = read('browser-round4/results.json')
native = {r['case']: r for r in native_rows}
upstream = {r['case']: r for r in upstream_rows}
assert len(native) == len(native_rows) == len(upstream) == len(upstream_rows) == 31
assert list(native) == list(upstream)
requests = [json.loads(line) for line in (ORIGINAL / 'session-u08-1794594672096-requests.jsonl').read_text().splitlines()]
replies = [json.loads(line) for line in (ORIGINAL / 'session-u08-1794594672096-replies.jsonl').read_text().splitlines()]
assert len(requests) == len(replies)
assert all(a['at'] == b['at'] for a, b in zip(requests, replies))


def decoded(reply):
    value = reply.get('result', {}).get('result', {}).get('value')
    if isinstance(value, str):
        try:
            return json.loads(value)
        except json.JSONDecodeError:
            pass
    return value


values = [decoded(r) for r in replies]
assignments = []
decoder = json.JSONDecoder()
for index, request in enumerate(requests):
    expression = request['expression']
    if expression.startswith('window.u08Case='):
        case, end = decoder.raw_decode(expression[len('window.u08Case='):])
        remainder = expression[len('window.u08Case=') + end:]
        assert remainder.startswith(';window.u08Fixture=')
        fixture, _ = decoder.raw_decode(remainder[len(';window.u08Fixture='):])
        assignments.append({'case': case, 'index': index, 'fixture': fixture})
cohort = assignments[-31:]
assert [x['case'] for x in cohort] == list(native)
setup_source = (ORIGINAL / 'browser-round4-upstream-setup.js').read_text()
raw_index = read('raw-transcript-index.json')
indexed = {r['case']: r for r in raw_index['cases'] if r['cohort'] == 'browser-round4'}
provenance = []
cases = []
archives = []
candidate_rows = []
for position, assignment in enumerate(cohort):
    name = assignment['case']
    n = native[name]['direct']
    u = upstream[name]
    ni, ui = n['initial'], u['initial']
    nf, uf = n['final'], u['final']
    start = assignment['index']
    end = cohort[position + 1]['index'] if position + 1 < len(cohort) else len(requests)
    assert requests[start + 1]['expression'] == setup_source
    assert assignment['fixture'] == ni == u['native_fixture']
    assert values[start + 1] == u['start']
    assert u['start']['initial'] == ui
    assert indexed[name]['fixture_assignment_request_line'] == start + 1
    stage_matches = []
    previous = start + 1
    for stage in u['stages']:
        matches = [i for i in range(previous + 1, end) if values[i] == stage['snapshot']]
        assert matches, (name, stage['label'], previous + 1, end)
        previous = matches[0]
        stage_matches.append({'label': stage['label'], 'reply_line': previous + 1,
                              'request_line': previous + 1, 'choice': stage['snapshot']['choice'],
                              'stack': stage['snapshot']['stack']})
    final_matches = [i for i in range(previous + 1, end) if values[i] == uf]
    assert final_matches, (name, 'final')
    final_line = final_matches[-1] + 1
    operations = []
    for i in range(start + 2, final_line):
        expression = requests[i]['expression']
        kind = None
        if 'ui.click.card.call' in expression:
            kind = 'real_selectable_card_click'
        elif 'ui.click.target.call' in expression:
            kind = 'real_selectable_target_click'
        elif 'ui.click.skill(' in expression:
            kind = 'real_skill_click'
        elif 'b.click()' in expression:
            kind = 'real_dom_control_click'
        elif 'ui.click.cancel()' in expression:
            kind = 'ui_cancel'
        elif 'ui.click.ok()' in expression:
            kind = 'ui_ok'
        if kind:
            operations.append({'request_line': i + 1, 'reply_line': i + 1, 'kind': kind,
                               'expression': expression, 'returned_value': values[i]})
    provenance.append({'case': name, 'assignment_line': start + 1, 'setup_line': start + 2,
                       'final_snapshot_reply_line': final_line, 'stages': stage_matches,
                       'operations': operations, 'raw_index_assignment_agrees': True,
                       'fixture_assignment_equals_native_initial': True, 'setup_reply_equals_saved_initial': True})

    initial_differences = []
    player_checks = []
    order_differences = []
    for a, b in zip(ni['players'], ui['players']):
        seat = a['seat']
        checks = {field: a[field] == b[field] for field in ('seat', 'hp', 'max_hp', 'face_down')}
        for zone in ('hand', 'equips', 'judge'):
            checks[zone] = faces(a[zone]) == faces(b[zone])
            if ids(a[zone]) != ids(b[zone]):
                order_differences.append({'seat': seat, 'zone': zone, 'native': ids(a[zone]), 'upstream': ids(b[zone])})
        checks['primary_skills'] = sorted(s['name'] for s in a['skills']) == sorted(s['name'] for s in b['skills'])
        checks['virtual_equipment_exact'] = sorted(b['virtualEquips']) == sorted(c['upstream_name'] for c in a['equips'])
        checks['virtual_judges_exact'] = sorted(b['virtualJudges']) == sorted(c['upstream_name'] for c in a['judge'])
        checks['no_prior_awakened_skill'] = b['awakenedSkills'] == []
        checks['no_prior_prediction'] = b.get('bolehuiyan') is None and not b['bolehuiyan4']
        player_checks.append({'seat': seat, **checks})
        for field, matches in checks.items():
            if not matches:
                initial_differences.append({'seat': seat, 'field': field})
    initial_deck_equal = faces(ni['deck_top_first'], True) == faces(ui['deck'], True)
    native_discard_by_id = {c['id']: c for c in ni['discard']}
    upstream_discard_by_id = {c['id']: c for c in ui['discard']}
    omitted = sorted(set(native_discard_by_id) - set(upstream_discard_by_id))
    extra = sorted(set(upstream_discard_by_id) - set(native_discard_by_id), key=str)
    mapped_discard_differences = []
    for cid in sorted(set(native_discard_by_id) & set(upstream_discard_by_id)):
        a, b = native_discard_by_id[cid], upstream_discard_by_id[cid]
        if face(a) != face(b):
            mapped_discard_differences.append({'id': cid, 'native': face(a), 'upstream': face(b),
                                              'same_except_exported_nature': face(a, True) == face(b, True)})

    final_differences = []
    for a, b in zip(nf['players'], uf['players']):
        for field in ('hp', 'max_hp', 'face_down'):
            final_differences += difference(f'players[{a["seat"]}].{field}', a[field], b[field])
        for zone in ('hand', 'equips', 'judge'):
            final_differences += difference(f'players[{a["seat"]}].{zone}', faces(a[zone]), faces(b[zone]))
    final_differences += difference('deck_top_first', faces(nf['deck_top_first'], True), faces(uf['deck'], True))
    nnew = [c for c in nf['discard'] if c['id'] not in native_discard_by_id]
    unew = [c for c in uf['discard'] if c['id'] not in upstream_discard_by_id]
    final_differences += difference('new_discard_entities_unordered', faces(nnew), faces(unew))
    final_differences += difference('table_or_ordering_entities', faces(nf['table']), faces(uf['ordering']))
    n_old_final = [c for c in nf['discard'] if c['id'] in native_discard_by_id]
    u_old_final = [c for c in uf['discard'] if c['id'] in upstream_discard_by_id]
    preserved_native_discard = faces(n_old_final) == faces(ni['discard'])
    preserved_upstream_discard = faces(u_old_final) == faces(ui['discard'])
    flags = {(r['seat'], r['key']): r['value'] for r in nf['flags']}
    semantic_flags = []
    if name.startswith('y_') and name != 'y_no_skill':
        semantic_flags += difference('yixin_limited_used', bool(flags.get((0, 'yixin_used'), 0)), uf['players'][0]['yixin_used'])
    if name.startswith('g_') and name != 'g_no_skill':
        skill = next(s for s in nf['players'][0]['skills'] if s['name'] == 'guifu')
        semantic_flags += difference('guifu_used_this_turn', skill['once_used'], bool(uf['players'][0]['stat'].get('skill', {}).get('guifu', 0)))
    if name.startswith('b_') and name != 'b_no_skill':
        semantic_flags += difference('bolehuiyan_used_this_turn', bool(flags.get((0, 'bolehuiyan_used_turn'), 0)), uf['players'][0]['bolehuiyan4'])

    all_native_choices = []
    previous_state = ni
    for command_index, command in enumerate(n['commands']):
        if 'action: Answer ' in command['request']:
            all_native_choices.append({'command_index': command_index, 'request': command['request'],
                                       'choice_before': previous_state['choice'], 'reply': command['reply']})
        previous_state = command['after']
    upper_predictions = [s for s in u['stages'] if s['label'].startswith('prediction_control_')]
    prediction_results = [{'label': s['label'], 'visible_clicked_choice': s['choice'],
                           'actual_returned_result': s['dom_control_reply']['result']} for s in upper_predictions]
    native_predictions = [x for x in all_native_choices if x['choice_before']['prompt'] == 'bolehuiyan:predict']
    native_attacks = []
    for command in n['commands']:
        match = re.search(r'action: PlayCard \{ card: (\d+), targets: \[([^\]]*)\]', command['request'])
        if match:
            native_attacks.append({'card': int(match.group(1)), 'targets': match.group(2)})
    upper_attacks = [{'card': t['args'][0].get('id'), 'targets': [a['seat'] for a in t['args'][1]]}
                     for t in uf['trace'] if t['method'] == 'useCard' and t['event'] == 'phase']
    candidates = {'case': name, 'native_choices_and_submitted_answers': all_native_choices,
                  'upstream_choice_stages': [{'label': s['label'], 'choice': s['snapshot']['choice'], 'stack': s['snapshot']['stack']}
                                             for s in u['stages']],
                  'native_predictions_submitted': len(native_predictions), 'upstream_prediction_controls_clicked': len(upper_predictions),
                  'prediction_results': prediction_results, 'native_attack_sequence': native_attacks, 'upstream_attack_sequence': upper_attacks}
    if name.startswith(('y_', 'g_')):
        main_skill = 'yixin' if name.startswith('y_') else 'guifu'
        first = next(s['snapshot'] for s in u['stages'] if s['label'] == 'actual_phase_use_candidates')
        candidates['upstream_skill_offered'] = main_skill in first['choice'].get('skills', [])
        candidates['native_initial_activation_reply'] = n['commands'][0]['reply']
        card_stage = next((s['snapshot'] for s in u['stages'] if s['label'] == 'skill_selected'), None)
        target_stage = next((s['snapshot'] for s in u['stages'] if s['label'] == ('two_real_cost_cards_selected' if name.startswith('y_') else 'skill_selected')), None)
        ncost = next((c['after']['choice'] for c in n['commands'] if c['after']['choice'] and c['after']['choice']['prompt'] == 'yixin:discard_two'), None)
        ntarget = next((c['after']['choice'] for c in n['commands'] if c['after']['choice'] and c['after']['choice']['prompt'] in ('yixin:choose_target', 'guifu:choose_target')), None)
        if ncost and card_stage:
            a = sorted(int(re.fullmatch(r'Card\((\d+)\)', x).group(1)) for x in ncost['options'] if x.startswith('Card('))
            b = sorted(c['id'] for c in card_stage['choice']['cards'] if c['seat'] == 0 and c['selectable'])
            candidates['cost_candidates'] = {'native': a, 'upstream': b, 'equal': a == b}
        if ntarget and target_stage:
            a = sorted(int(re.fullmatch(r'Player\((\d+)\)', x).group(1)) for x in ntarget['options'] if x.startswith('Player('))
            b = sorted(target_stage['choice']['selectablePlayers'])
            candidates['target_candidates'] = {'native': a, 'upstream': b, 'equal': a == b}
    candidate_rows.append(candidates)

    archive_modes = []
    for mode in ('direct', 'restored'):
        run = native[name][mode]
        pending = []
        for entry in run['pending_archive_checks']:
            path = ORIGINAL / 'native-round2' / entry['file']
            raw = path.read_bytes()
            pending.append({'file': entry['file'], 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest(),
                            'ngsa_magic': raw[:4].decode(), 'format_version_little_endian': int.from_bytes(raw[4:6], 'little'),
                            'prompt': entry['prompt'], 'auth_before': entry['auth_before'], 'auth_after': entry.get('auth_after'),
                            'error': entry.get('error'), 'snapshot_equal_except_auth_generation': entry.get('snapshot_equal_except_auth_generation'),
                            'state_hash_identical': entry.get('state_hash_identical')})
        debug = (ORIGINAL / 'native-round2' / f'{name}-{mode}-final-snapshot-debug.txt').read_text()
        archive_modes.append({'mode': mode, 'pending': pending, 'final_debug': debug,
                              'final_archive_hex_agrees_with_file': bytes.fromhex(run['final_archive_hex']) == (ORIGINAL / 'native-round2' / f'{name}-{mode}-final.ngsa').read_bytes()})
    a, b = archive_modes
    ad = re.sub(r'(\bauth_generation: )\d+', r'\1<NORMALIZED>', a.pop('final_debug'))
    bd = re.sub(r'(\bauth_generation: )\d+', r'\1<NORMALIZED>', b.pop('final_debug'))
    archives.append({'case': name, 'modes': archive_modes, 'full_final_debug_equal_except_auth_generation': ad == bd,
                     'full_exported_final_states_equal': native[name]['direct']['final'] == native[name]['restored']['final'],
                     'command_records_equal': native[name]['direct']['commands'] == native[name]['restored']['commands'],
                     'raw_final_archive_bytes_equal': native[name]['direct']['final_archive_hex'] == native[name]['restored']['final_archive_hex']})
    cases.append({'case': name, 'initial': {'players': player_checks, 'player_projection_differences': initial_differences,
                                         'deck_ids_names_suits_ranks_natures_order_equal': initial_deck_equal,
                                         'hand_equipment_judge_order_differences': order_differences,
                                         'omitted_native_discard_ids': omitted, 'extra_upstream_discard_ids': extra,
                                         'mapped_discard_export_differences': mapped_discard_differences,
                                         'control_context': [{'seat': p['seat'], 'true_me': p['trueMe'], 'is_under_control': p['isUnderControl']} for p in ui['players']],
                                         'upper_untagged_deck': ui['untaggedDeck'], 'upper_ordering': ui['ordering']},
                  'final_specified_projection_differences': final_differences,
                  'semantic_usage_flag_differences': semantic_flags,
                  'final_hp_native': [p['hp'] for p in nf['players']], 'final_hp_upstream': [p['hp'] for p in uf['players']],
                  'final_hands_native': [ids(p['hand']) for p in nf['players']], 'final_hands_upstream': [ids(p['hand']) for p in uf['players']],
                  'final_deck_native': ids(nf['deck_top_first']), 'final_deck_upstream': ids(uf['deck']),
                  'new_discard_order_native': ids(nnew), 'new_discard_order_upstream': ids(unew),
                  'initial_discard_entities_preserved_within_native': preserved_native_discard,
                  'initial_discard_entities_preserved_within_upstream': preserved_upstream_discard,
                  'final_foreign_pile_mutations': uf['foreignPileMutations'], 'final_untagged_deck': uf['untaggedDeck'],
                  'native_pending_prompt': nf['choice']['prompt'] if nf['choice'] else None,
                  'upstream_final_choice': uf['choice'], 'upstream_final_stack': uf['stack'],
                  'upstream_root_done': uf['done'], 'upstream_error': uf['error'],
                  'upstream_trace': uf['trace'], 'native_command_log_deltas': [c['log_delta'] for c in n['commands']],
                  'pending_counts': {mode: len(native[name][mode]['pending_archive_checks']) for mode in ('direct', 'restored')}})

summary = {
    'unique_cases': len(cases), 'cdp_requests_and_replies': len(requests), 'all_fixture_assignments_in_session': len(assignments),
    'final_cohort_first_assignment_line': cohort[0]['index'] + 1,
    'final_cohort_last_snapshot_line': provenance[-1]['final_snapshot_reply_line'],
    'all_31_initial_player_projections_match': all(not r['initial']['player_projection_differences'] for r in cases),
    'all_31_initial_deck_projections_match': all(r['initial']['deck_ids_names_suits_ranks_natures_order_equal'] for r in cases),
    'specified_final_projection_match_cases': [r['case'] for r in cases if not r['final_specified_projection_differences']],
    'specified_final_projection_different_cases': [r['case'] for r in cases if r['final_specified_projection_differences']],
    'semantic_usage_flag_different_cases': [r['case'] for r in cases if r['semantic_usage_flag_differences']],
    'both_player_projection_and_usage_flags_match_cases': [r['case'] for r in cases if not r['final_specified_projection_differences'] and not r['semantic_usage_flag_differences']],
    'upstream_root_done_cases': sum(r['upstream_root_done'] for r in cases),
    'upstream_pending_cases': [r['case'] for r in cases if not r['upstream_root_done']],
    'native_final_pending_cases': [r['case'] for r in cases if r['native_pending_prompt']],
    'pending_direct': sum(r['pending_counts']['direct'] for r in cases),
    'pending_restored': sum(r['pending_counts']['restored'] for r in cases),
    'all_final_full_debug_equal_except_auth_generation': all(r['full_final_debug_equal_except_auth_generation'] for r in archives),
    'all_direct_restored_commands_equal': all(r['command_records_equal'] for r in archives),
    'all_pending_header_version_107': all(p['ngsa_magic'] == 'NGSA' and p['format_version_little_endian'] == 107 for r in archives for mode in r['modes'] for p in mode['pending']),
    'all_pending_errors_null': all(p['error'] is None for r in archives for mode in r['modes'] for p in mode['pending']),
    'all_pending_snapshot_restores_equal_except_auth_generation': all(p['snapshot_equal_except_auth_generation'] for r in archives for mode in r['modes'] for p in mode['pending']),
    'all_pending_state_hashes_equal': all(p['state_hash_identical'] for r in archives for mode in r['modes'] for p in mode['pending']),
    'all_initial_discard_entities_preserved_within_each_runtime': all(r['initial_discard_entities_preserved_within_native'] and r['initial_discard_entities_preserved_within_upstream'] for r in cases),
    'final_foreign_pile_mutation_cases': [r['case'] for r in cases if r['final_foreign_pile_mutations'] or r['final_untagged_deck']],
    'read_collector_reports_or_comparator': False,
    'new_equivalence_signoffs': 0,
}
write('case-recalculation', cases)
write('candidate-recalculation', candidate_rows)
write('cdp-provenance', {'request_reply_pairing_by_timestamp_exact': True, 'cases': provenance})
write('archive-recalculation', archives)
write('summary-recalculation', summary)
print(json.dumps(summary, ensure_ascii=False, indent=2))
