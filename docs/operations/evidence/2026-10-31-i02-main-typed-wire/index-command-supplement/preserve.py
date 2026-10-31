import datetime, hashlib, json
from pathlib import Path

E = Path('/tmp/i02-main-index-command-provenance-599614a0')
OLD = Path('/tmp/i02-main-typed-wire-functional-33d21ba5')
SESSION = Path('/Users/ruyimg/.pi/agent/sessions/--Users-ruyimg-Documents-Claude-sanguosha--/2026-10-29T12-17-50-921Z_01a18798-5949-73c1-8642-9d0f55dc485a/df47e19c-73c6-4cdd-a45e-5437618a62f2/run-0/session.jsonl')
REPORT = Path('/Users/ruyimg/.pi/agent/sessions/--Users-ruyimg-Documents-Claude-sanguosha--/subagent-artifacts/outputs/599614a0-9b12-4abe-a51d-3a9113f87c3c/verification/i02-main-index-command-provenance-supplement.md')
sha = lambda b: hashlib.sha256(b).hexdigest()
old_paths = [OLD / n for n in ['index-parse-original-command.sh', 'index-mask-and-js-load-original-command.sh', 'gate-comparison.json', 'gate-to-final-seal-comparison.json', 'i02-main-typed-wire-functional.md', 'artifact-sha256.json', 'manifest.json', 'seal-root.json']]
before = {str(p): sha(p.read_bytes()) for p in old_paths}
snapshot = SESSION.read_bytes()
lines = snapshot.splitlines(keepends=True)
records = [(i, json.loads(raw), raw) for i, raw in enumerate(lines, 1)]
recovered = []
for name, definition in [('index-parse', 'def parse(p):'), ('index-mask-and-js-load', 'def masked(p):')]:
    candidates = []
    for line_no, record, raw in records:
        m = record.get('message', {})
        if m.get('role') != 'assistant' or not isinstance(m.get('content'), list):
            continue
        for call in m['content']:
            if call.get('type') != 'toolCall' or call.get('name') != 'bash':
                continue
            command = call.get('arguments', {}).get('command', '')
            if definition in command.splitlines():
                candidates.append((line_no, record, raw, call, command))
    assert len(candidates) == 1, (name, len(candidates))
    line_no, record, raw, call, command = candidates[0]
    results = [(n, r, b) for n, r, b in records if r.get('message', {}).get('role') == 'toolResult' and r['message'].get('toolCallId') == call['id']]
    assert len(results) == 1
    result_line, result_record, result_raw = results[0]
    assert result_line > line_no
    assert not result_record['message'].get('isError', False)
    command_bytes = command.encode('utf-8')
    (E / (name + '.raw.sh')).write_bytes(command_bytes)
    (E / (name + '.assistant-record.raw.jsonl')).write_bytes(raw)
    (E / (name + '.tool-result-record.raw.jsonl')).write_bytes(result_raw)
    result_content = result_record['message'].get('content', [])
    result_text = ''.join(c.get('text', '') for c in result_content if c.get('type') == 'text')
    (E / (name + '.tool-result.raw.txt')).write_bytes(result_text.encode('utf-8'))
    recovered.append({'name': name, 'selection': 'Exact whole command line equals ' + repr(definition) + '; assistant bash toolCall only; unique match; no substring or last-match overwrite', 'toolCallId': call['id'], 'assistantSessionLine': line_no, 'assistantRecordId': record.get('id'), 'assistantTimestamp': record.get('timestamp'), 'assistantRawRecordSha256': sha(raw), 'resultSessionLine': result_line, 'resultRecordId': result_record.get('id'), 'resultTimestamp': result_record.get('timestamp'), 'resultRawRecordSha256': sha(result_raw), 'resultIsError': result_record['message'].get('isError', False), 'fullCommandSha256': sha(command_bytes), 'commandBytes': len(command_bytes), 'archivalOnlyDoNotExecute': True})
    assert (E / (name + '.raw.sh')).read_bytes() == command_bytes

fields = []
for name in ['gate-comparison.json', 'gate-to-final-seal-comparison.json']:
    raw = (OLD / name).read_bytes()
    obj = json.loads(raw)
    keys = list(obj['fields'])
    assert len(keys) == 12
    fields.append({'file': str(OLD / name), 'sha256': sha(raw), 'recordedComparisonCategories': len(keys), 'keys': keys})
assert set(fields[0]['keys']) == set(fields[1]['keys'])
old_index = json.loads((OLD / 'artifact-sha256.json').read_bytes())['files']
miscaptured = []
for name in ['index-parse-original-command.sh', 'index-mask-and-js-load-original-command.sh']:
    p = OLD / name
    digest = sha(p.read_bytes())
    assert old_index[name]['sha256'] == digest
    assert 'def parse(p):' not in p.read_text().splitlines()
    assert 'def masked(p):' not in p.read_text().splitlines()
    miscaptured.append({'path': str(p), 'sha256': digest, 'matchesOriginalSealedArtifactIndex': True, 'label': 'MISCAPTURED: later extractor/seal-patching command, not defining parser/mask command; retained unchanged'})
assert before == {str(p): sha(p.read_bytes()) for p in old_paths}
manifest = {'utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'scope': 'Additive readonly metadata preservation supplement; no recovered command executed, no raw-index parse rerun, no tests/builds/network/Main/schema/index/old-E writes', 'sourceSession': {'path': str(SESSION), 'snapshotByteCount': len(snapshot), 'snapshotLineCount': len(lines), 'snapshotSha256': sha(snapshot), 'qualification': 'Hash identifies the exact bytes read at capture; append-only later session records may change the full-file hash. Exact selected raw records are preserved.'}, 'recoveredCommands': recovered, 'miscapturedSealedFiles': miscaptured, 'countErratum': {'from': 13, 'to': 12, 'meaning': 'Recorded comparison categories, not an invented separately proved thirteenth category', 'files': fields}, 'preservedOldFileHashes': before, 'oldFilesUnchanged': True, 'priorRuntimeResultsReevaluated': False, 'qualification': 'Restores command provenance, not an independent certification of original parser correctness or raw-index conclusions; parent independent parser remains separate. Current Main CI workflow after freeze is outside this supplement.'}
(E / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
report = '\n'.join([
'# I02 index command provenance supplement',
'',
'P2 corrected additively: the two old sealed command files captured the later extractor. They remain unchanged; this supplement recovers the actual executed defining commands from structured retained-session assistant tool calls.',
'New evidence root: `/tmp/i02-main-index-command-provenance-599614a0`; selection, SHA256s, raw records and result bindings are in `manifest.json`.',
'`index-parse.raw.sh`: original assistant session line ' + str(recovered[0]['assistantSessionLine']) + ', timestamp `' + recovered[0]['assistantTimestamp'] + '`, matching result line ' + str(recovered[0]['resultSessionLine']) + '.',
'`index-mask-and-js-load.raw.sh`: original assistant session line ' + str(recovered[1]['assistantSessionLine']) + ', timestamp `' + recovered[1]['assistantTimestamp'] + '`, matching result line ' + str(recovered[1]['resultSessionLine']) + '.',
'Both tool-call IDs, exact full-command hashes, assistant/result raw JSONL records and full returned text are preserved. Result records match each toolCallId and report no tool error.',
'Selection requires an entire command line exactly equal to `def parse(p):` or `def masked(p):`, within assistant bash calls, with exactly one candidate each. Quoted substring/search predicates in the later extractor do not satisfy this rule; no last-match overwrite is used.',
'Commands are archival only and were NOT executed or syntax-checked. No parser rerun, test, build, network, Main/source/schema/index write or old-E rewrite occurred.',
'Old miscaptured file SHA256s (also match the original sealed artifact index):',
'`index-parse-original-command.sh`: `' + miscaptured[0]['sha256'] + '`.',
'`index-mask-and-js-load-original-command.sh`: `' + miscaptured[1]['sha256'] + '`.',
'Number erratum: the original report’s “all13” comparison wording means **12 recorded comparison categories**, as both `gate-comparison.json` and `gate-to-final-seal-comparison.json` actually contain12 keys.',
'Those keys are `head`, `tree`, `indexSha256`, `indexEntries`, `headEntries`, `staged`, `trackedStatus`, `untrackedNames`, `sourceBinding`, `maintainedAndMarkdown`, `protectedSets`, `hooks`. No thirteenth category is claimed; original report preserved.',
'The source-session snapshot SHA/byte count and all retained old-file hashes are recorded. This is a provenance repair, not independent raw-index certification; mutation origin remains unknown and the parent’s independent parser is separate.',
'Prior21-gate/native7c/legacy846/actual IPC/WS evidence is unchanged. Later parent-authorized Main CI changes are outside this supplement.',
'Skipped execution, new acceptance checks and target cleanup. Residual risk: this supplement establishes retained-command identity, not independent correctness of their index interpretation.',
]) + '\n'
assert len(report.splitlines()) <= 25
REPORT.parent.mkdir(parents=True, exist_ok=True)
REPORT.write_text(report)
(E / 'report-copy.md').write_text(report)
artifact = {str(p.relative_to(E)): {'sha256': sha(p.read_bytes()), 'bytes': p.stat().st_size} for p in sorted(E.rglob('*')) if p.is_file() and p.name != 'artifact-sha256.json'}
(E / 'artifact-sha256.json').write_text(json.dumps({'files': artifact, 'reportPath': str(REPORT), 'reportSha256': sha(REPORT.read_bytes())}, indent=2) + '\n')
print(json.dumps({'report': str(REPORT), 'reportLines': len(report.splitlines()), 'manifestSha256': sha((E / 'manifest.json').read_bytes()), 'artifactIndexSha256': sha((E / 'artifact-sha256.json').read_bytes()), 'recovered': recovered, 'oldFilesUnchanged': True}, indent=2))
