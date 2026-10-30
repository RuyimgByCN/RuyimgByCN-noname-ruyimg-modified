from pathlib import Path
import json,re,hashlib,shlex,datetime,subprocess
out=Path(__file__).resolve().parent
index=[]
for path in out.glob('b-*.json'):
 try:data=json.loads(path.read_text())
 except (ValueError,UnicodeError):continue
 if not isinstance(data,dict) or not isinstance(data.get('command'),list) or 'exit' not in data:continue
 log=path.with_suffix('.log');text=log.read_text(errors='replace') if log.exists() else ''
 rows=re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out',text)
 data['command_shell_display']=shlex.join(data['command']);data['record_path']=str(path);data['log_path']=str(log) if log.exists() else None
 if rows:data['rust_test_totals']={key:sum(int(row[position]) for row in rows) for key,position in [('passed',1),('failed',2),('ignored',3),('filtered_out',5)]}
 node=re.search(r'ℹ tests (\d+).*?ℹ pass (\d+).*?ℹ fail (\d+).*?ℹ skipped (\d+)',text,re.S)
 if node:data['node_test_totals']=dict(zip(['tests','passed','failed','skipped'],map(int,node.groups())))
 index.append(data)
index.sort(key=lambda item:(item.get('start',0),item['name']))
manual=[
 {'command':'node --import ./apps/web-client/scripts/ts-register.mjs --test apps/web-client/scripts/approved-b-metadata.test.mjs','exit':1,'log':'b-red-ts-direct.log','summary':'Initial generated native package was absent; copied existing pre-B generated package for meaningful red tests, then rebuilt current B WASM later.'},
 {'command':'node --import ./apps/web-client/scripts/ts-register.mjs --test apps/web-client/scripts/approved-b-metadata.test.mjs','exit':1,'log':'b-red-ts-bootstrap.log','summary':'Meaningful pre-implementation TS red: 1 passed, 3 failed.'},
 {'command':'python3 -B -m unittest discover -s scripts -p test_check_source_lines.py','exit':0,'log':'b-source-line-tests-final.log','summary':'2 passed.'},
 {'command':'python3 -B -m unittest discover -s scripts -p test_check_native_layering.py','exit':0,'log':'b-layering-tests-final.log','summary':'9 passed.'},
 {'command':'python3 -B scripts/check-source-lines.py','exit':0,'log':'b-source-lines-checked.log','summary':'8954 maintained files under caps.'},
 {'command':'python3 -B scripts/check-native-layering.py','exit':0,'log':'b-layering-checked.log','summary':'0 upward references.'},
 {'command':'git diff --check','exit':0,'log':'b-diff-check-final.log'},
 {'command':'git diff --cached --name-only','exit':0,'log':'b-no-staged-final.log','summary':'Empty; no staged files.'},
 {'command':'git apply --check approved-b-checked.patch (cwd isolated evidence baseline snapshot)','exit':0,'log':'b-checked-patch-check.log'},
 {'command':'git apply --reverse --check ../evidence/approved-b-checked.patch','exit':0,'log':'b-checked-reverse-check.log'},
 {'command':'python3 ../evidence/freeze-b.py (initial attempt)','exit':1,'log':'b-freeze-first-failure.txt','summary':'Cargo.lock original snapshot omitted by initial extension filter; reconstructed exact bytes from HEAD plus frozen prerequisite, verified SHA256.'},
 {'command':'python3 ../evidence/freeze-b.py (second attempt)','exit':1,'log':'b-freeze-final.log','summary':'Detected four tracked generated dist files removed by Vite; restored exact initial bytes before final pure patch.'},
 {'command':'python3 ../evidence/cleanup-b-unused.py (initial guard)','exit':1,'log':'b-unused-cache-cleanup-final.log','summary':'Over-broad process guard matched shell command text; no deletion happened. Corrected guard to exact executable names.'},
 {'command':'python3 ../evidence/cleanup-b-unused.py (corrected guard)','exit':0,'log':'b-unused-cache-cleanup-corrected.log','summary':'Removed unused own observer-created target and private incremental/tmp only; retained binary fingerprints and needed private target pending parent retirement.'},
]
(out/'b-command-index.json').write_text(json.dumps({'generated_at':datetime.datetime.now().isoformat(),'serialized_and_targeted_commands':index,'manual_lightweight_and_artifact_commands':manual,'heavy_command_policy':'Every explicit heavy command used /tmp/noname-run-serialized.py. All explicit Rust/WASM builds used exclusive Referee target and CARGO_INCREMENTAL=0; no profile/ABI changes or performance claim.','all_initial_failures_retained':True},indent=2,ensure_ascii=False)+'\n')
fixtures=json.loads((out/'b-metadata-fixtures.json').read_text())
reports={mode:json.loads((out/f'b-ui-{mode}-readable-report.json').read_text()) for mode in ['chrome','wkwebview']}
assert all(report['passed'] and len(report['cases'])==16 for report in reports.values())
matrix={
 'actual_archives': [{'name':fixture['name'],'seat_count':len(fixture['seats'])} for fixture in fixtures],
 'cross_entry':{'cases':14,'comparisons':98,'restores_per_seat':2,'entries':['authorized server projection','current generated WASM32 + actual Worker converter','actual native desktop IPC host'],'fields':['generals[0] and generals[1]: def/name/faction/revealed/null','character_def fallback','choice native_kind/prompt/kind/id/observer/options/token/index/min/max/allow_cancel/revision'],'id_normalization':'Only external one-based seat identities/Player option labels normalized once for comparison; definition zero preserved.'},
 'privacy':{'own':'Both configured slots, including definition zero.','ordinary_non_kingdoms':'Configured general metadata public.','foreign_kingdoms':'Dark slots null; primary/secondary visibility follows actual shared filtered GeneralView.','spectator':'No private choice even with forged valid seat, 0, 257 or MAX; no legal actions/submit/replay authority.','trusted_referee':'Full slots/private choice through trusted membership/reconnect only; read-only; actual WS stale fanout/reconnect/restart/replay refusal test in affected full suite.','shadowing':'Existing ordinary/AI hidden-state shadow regression included in full native suite.'},
 'compatibility':{'ws_version':5,'old_rust_new_wire':'Actual encoded Snapshot deserialized by frozen old readers; extra fields ignored.','new_rust_old_wire':'Frozen old Snapshot serialized then decoded by current protocol; general slots default null, optional prompt/native_kind None.','old_ts_new_native_payload':'Frozen functional old TS converter accepts actual new WASM payloads for all 98 comparisons.','new_ts_old_payload':'Absent metadata + character/kind fallback, including actual BattleView legacy flow.','invalid_types':'New Rust serde rejects malformed slot/string/native-kind types; Worker, desktop transport and network boundaries explicitly refuse malformed new metadata.'},
 'choice_kinds':['confirm','select_cards','select_players','respond_card','pick_tag'],
 'actual_prompt_keys':['trigger_confirm','tuxi_target','discard_to_limit','respond_shan','fanjian_suit','xunxun_arrange','jijie_give_target','gaiming_pick','haotianta_pick','liaoyuan_discard'],
 'ui': {mode:{'passed':report['passed'],'cases':len(report['cases']),'transports':sorted({case['transport'] for case in report['cases']}),'real_answer_controls':'Five real kinds exercised on actual BattleView controls where observer equals local UI seat.','legacy_and_extension':'Legacy fields absent; unknown prompt uses readable kind fallback; malicious general text escaped and no untrusted portrait paths.'} for mode,report in reports.items()},
 'presentation_guard':json.loads((out/'b-presentation-guard-final-scope.json').read_text()),
 'read_only_archive_and_continuation':'Same restored archive bytes before/after projection; decoder encode function source identical; full direct/restored snapshots (including RNG and resolution) and emitted event logs agree; genuinely unknown archive prompt refuses.',
 'unchanged':json.loads((out/'b-unchanged-bindings-outcome.json').read_text()),
 'versions':{'rules':328,'archive':105,'native_abi':753,'ws':5},
 'acceptance_boundary':'B metadata and exercised field/privacy/compatibility matrix only. Parent independent review and combined workspace/conformance/preflight remain pending; no NC03/NC09/37-source or PA09 six-gate completion claim.'}
(out/'b-all-field-matrix.json').write_text(json.dumps(matrix,indent=2,ensure_ascii=False)+'\n')
print(len(index),'command records indexed; 14 actual archives/98 comparisons; both final UI reports passed16')
