from pathlib import Path
import json,hashlib,datetime,subprocess,os
r=Path(Path('/tmp/c331-phase7-root').read_text().strip());w=r/'worktree';e=Path(__file__).parent;sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();now=lambda:datetime.datetime.now(datetime.timezone.utc).isoformat();write=lambda p,x:p.write_text(json.dumps(x,indent=2,ensure_ascii=False)+'\n');clock=json.loads((e/'clock.json').read_text());result=json.loads((e/'result.json').read_text());manifest=json.loads((e/'source-manifest.json').read_text());assert all(sha(w/x['path'])==x['sha256']for x in manifest['files']);assert datetime.datetime.now(datetime.timezone.utc)<datetime.datetime.fromisoformat(clock['seal_due_utc']);fs=json.loads((e/'filesystem-before.json').read_text());assert sha(Path(fs['private_index']['path']))==fs['private_index']['sha256'];assert sha(w/'Cargo.lock')==fs['private_lock']['sha256'];g=r/'evidence/phase9g';idx=json.loads((g/'artifact-index.json').read_text());assert sha(g/'artifact-index.json')=='0ea6e4254dff56593edd6bf87f5ca1a7c26b7b2afcc9b1d1f329afb04604be92';assert all(sha(g/x['path'])==x['sha256']for x in idx['files']);assert sha(Path(idx['external_report']['path']))==idx['external_report']['sha256'];env=dict(os.environ,GIT_OPTIONAL_LOCKS='0');assert not subprocess.check_output(['git','--no-optional-locks','diff','--cached','--name-only'],cwd=w,env=env).strip()
checks=json.loads((e/'command-results.json').read_text());write(e/'after.json',{'utc':now(),'final_source_manifest_sha256':sha(e/'source-manifest.json'),'all239_source_bytes_unchanged':True,'g1858_entries_and_external_report_unchanged':True,'private_index_sha256':fs['private_index']['sha256'],'private_index_mtime_ns':Path(fs['private_index']['path']).stat().st_mtime_ns,'private_lock_sha256':sha(w/'Cargo.lock'),'private_staged_empty':True,'all9_commands_before_after239_bound':True,'source_test_fixture_catalog_edits':False,'old_baseline_not_refreshed':True,'active_owned_producers':0,'queued_owned_producers':0})
report=Path('/Users/ruyimg/.pi/agent/sessions/--Users-ruyimg-Documents-Claude-sanguosha--/subagent-artifacts/outputs/67d42bcc-1b0d-44bc-b041-8fdc6a38ebe9/validation/c331-phase9g-final6-before-repair.md');report.parent.mkdir(parents=True,exist_ok=True)
text=f'''# C9g final6 before repair — validation-only, BLOCK
Verified2026-10-31 on original final6 bytes; no source/test/fixture/catalog/lock/index/Main edits or repairs.
New phase start19:16:31.253321Z; heavy cutoff19:36:31.253321Z; seal due19:41:31.253321Z.
All nine requested producers completed19:18:39.929424Z; no active/queued owned work, no signals, fallback or reruns.
Evidence root: `{e}`.

## Authoritative source and protection
Full239manifest `69eeebab43d3ccea5f0b48e003cecd8a339f2fe1df8cf8e8d75e118a5309087b`.
Increment46manifest `9557423705b52ebdfa158004bdeee9e2ab54398f3fa13696cf43249cc225ea00`.
Every producer has complete239hash manifests before AND after; all equal original source; HEAD5fc/private staging empty.
Pinned helperd7f/global lock dev16777231 inode620205209; offline/jobs2/incremental0/default profiles/free disk>3GiB.
Warm target is private originalT/noname-c331-efhmbq7e/target; strict uses separate owned target-strict/-Dwarnings.
`protected-after.json` rechecks original31867 baseline with exact Own6/I02/CI/generated exceptions, source37/products/schema4/hooks/C6/old reader.
Parent prephase4406docs/143909122B receiptSHA600343... plus explicit write hold; fresh post-producer4406hashes verified separately.
Main HEAD9f4, staging empty/rawindexe0de unchanged; privateindexb107 bytes+mtime and Cargo.lock unchanged.
C9g1858/index/report, C9f4167, C9e7333 and separate41audit5 entries rehashed unchanged; no old bytes overwritten.
Older4258 receipt enumeration was not separately retained; this is not C drift or a CI input baseline.
Parent docs-only CI ten-green remains separate from C runtime acceptance.

## Actual command results
`cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native --no-fail-fast`: **1848passed/9failed/76ignored**, exit101.
This full native result remains FAILED despite subsequent focused passes; all test binaries compiled on actual final6.
Independent `RUSTFLAGS=-Dwarnings cargo check --offline -p engine-native --all-targets`: PASS.
`cargo check --offline -p game-server --all-targets`: PASS;345warning-line occurrences including repeated diagnostics/summaries retained.
`cargo test --offline -p engine-native-wasm --test c331_invocation_controls -- --nocapture`:1PASS;18diagnostics plus warning-summary retained.
`cargo test --offline -p engine-native --test nc_rounds c331_ -- --nocapture`:98PASS/1ignoredF14/153filtered.
Separate original-expectation golden test: FAIL, exit101, same12disputed cases; no expected hashes changed.
Source-cap unit2 PASS; caps9045 PASS (physical2000/Rust effective1000); layering/F7 zero.
Counts above are separate command scopes; focused/WASM/server/Python are not added to a unique native grand total.
`diagnostics-and-counts.json` retains every warning/error occurrence and ignored name; all nine requested commands ran.

## Nine full-native failures — observed assertions, no repair classification assumed
1. `a0_lib_tests::standard_skill_ids_append_only` — lib.rs67: frozen count230 versus actual231.
2. `boss_guojia_qizuo_trick_reuse` — boss_guojia.rs138: play result did not match Accepted.
3. `boss_guojia_yuance_judge_branches` — boss_guojia.rs78: targetHP3 versus expected2.
4. `boss_liubei_zhangwu_discard_or_damage` — boss_liubei.rs97: play result did not match Accepted.
5. `boss_mingxingzhu_qingzhu_jiazu_jiding` — boss_mingxingzhu.rs118: Rejected(IllegalTargets).
6. `boss_zhangjiao_jidian_chain_guard` — boss_zhangjiao.rs173: HP4 versus expected3.
7. `boss_zhangjiao_leiji_shan_thunder` — boss_zhangjiao.rs145: answer result did not match Accepted.
8. `simayi_guicai_rewrites_with_any_suit_card` — simayi.rs60: old judgment not owned by rewriter as asserted.
9. `replay_scenarios_match_golden` — replay_scenarios.rs1229: original12hash mismatches.
Full exact failure sections and names are in `full-native-failures.json` and raw full-native.log.

## Fresh observations and capture limits
Current golden producer captured actual inputs/outcomes/state; original positive C8 producer/data reused WITHOUT redrive.
All362baseline JSON/snapshot/archive bindings match immutable9f bindings; all12actual/expected hashes are identical to retained9f.
Six complete business fields compared:11trajectories equal; chain0007-after differs only events, own DamageDealt before own unlink.
Difference exactly matches prior pinned Core content.ts12092–12120/Liuli game.js34767–34810 disposition, retained without normalization.
Outcomes187none/139Accepted/32Pending/4Rejected: discriminants/full Pending/reasons exact; only Accepted32-byte hash separated and retained.
Existing focused hooks emitted202JSON/snapshot/archive triples;15snapshots contain new invocation frames.
**Export limitation:** C9g control tests call in-memory restore/same_final but no capture function; separate complete C9g branch dumps unavailable.
No forbidden instrumentation/test edit was introduced to manufacture those exports; actual test statuses and existing captures are retained.

## Remaining gate
Final6 now compiles, but full regression is red; concurrent source review must classify failures before the next repair workflow.
F14 ignored P1, F07 old random/unimplemented, lifecycle/Judge/opaque privacy/compatibility/persistence and conditional wide-cost controls remain mandatory.
No all37/ABI/WS6/privacy/fuel/archive equivalence from these passes or equal errors; no golden rerecord or new protocol/version.
Whole C331/Main remains BLOCK: source262/290 remaining28 +0; true ChangeHp/DeathFlow still next, not relabelled DamageEnd.
'''
assert len(text.splitlines())<=60;report.write_text(text);(e/'report.md').write_text(text);write(e/'report-reference.json',{'path':str(report),'sha256':sha(report),'lines':len(text.splitlines())})
files=[{'path':str(p.relative_to(e)),'sha256':sha(p),'bytes':p.stat().st_size}for p in sorted(e.rglob('*'))if p.is_file()and p.name not in ['artifact-index.json','artifact-index.sha256']];index={'sealed_utc':now(),'phase':'phase9h-baseline-validation','source_manifest_sha256':sha(e/'source-manifest.json'),'source_files':239,'status':'VALIDATION_ONLY_COMPLETE_WITH_CAPTURE_EXPORT_LIMIT_FULL_NATIVE_FAILED','files':files,'external_report':{'path':str(report),'sha256':sha(report)},'no_staged_files':True};write(e/'artifact-index.json',index);(e/'artifact-index.sha256').write_text(sha(e/'artifact-index.json')+'  artifact-index.json\n');assert all(sha(e/x['path'])==x['sha256']for x in files);print(json.dumps({'sealed_utc':index['sealed_utc'],'artifact_files':len(files),'artifact_index_sha256':sha(e/'artifact-index.json'),'report_sha256':sha(report),'report_lines':len(text.splitlines()),'native_full':[1848,9,76],'no_staged_files':True}))
