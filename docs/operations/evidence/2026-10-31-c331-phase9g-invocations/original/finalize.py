from pathlib import Path
import json,hashlib,datetime,subprocess,shutil
r=Path(Path('/tmp/c331-phase7-root').read_text().strip());e=r/'evidence/phase9g';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();now=lambda:datetime.datetime.now(datetime.timezone.utc).isoformat();write=lambda p,x:p.write_text(json.dumps(x,indent=2,ensure_ascii=False)+'\n');res=json.loads((e/'results-bound.json').read_text())
for name in ['noname-post-c9e-doc-accepted-commit.json','noname-post-c9e-doc-stage-audit.json','noname-parent-continuation-2026-10-31.json']:
 p=Path('/tmp')/name
 if p.exists():shutil.copyfile(p,e/'parent-receipts'/name)
old=json.loads((r/'evidence/phase9e-concordance-supplement/concordance.json').read_text());findings=[]
for row in old['findings']:
 f=dict(row);f['c9g_status']='partial_finite_invocation_runtime_checked_subset'if f['id']in ['F09','F12','F13','F16']else'mandatory_unimplemented_hidden_selection_privacy_compatibility'if f['id']=='F07'else'mandatory_native_identity_alliance_storage_gap_runtime_red'if f['id']=='F14'else'preserved_C9f_atomic_repair_not_full_revalidated';findings.append(f)
unproved=[dict(x,c9g_status='mandatory_unproved_unchanged')for x in old['rows']if x['status']=='unproved-boundary']
write(e/'all17-12-five-fsm-lifecycle-judge-privacy-compat.json',{'utc':now(),'findings':findings,'unproved_rows':unproved,'fiveFSM':{'Qinglong':'NOT_IMPLEMENTED; old atomic random acquisition remains mandatory defect. HiddenCard/private binding/privacy/old-reader/current UI/index controls not implemented.','Zhangwu':'Implemented optional variable HE holder/source asks, source-specific logBegin/loss/trailinglog, captured source and n damage; runtime checked holder cancel/nonfirstE+n2/source cancel/typed-byte pending asks. Self-source, successful source pay, death/removal/fault/candidate forgery controls remain mandatory.','Yuance':'Implemented actual two semantic controls/cancel, RNG AI draw, ordinary Judge and four color branches; controls publicly lowered Custom/string with omitted native_kind. Four branches/cancel/Tiandao/byte restore checked on earlier source; exhaustive lifecycle/death/fault/wire gates remain mandatory.','Jiding':'Implemented real Invocation/CardUsed awaited subtree/Dodge then exact def removal, typed builtin Rushou identity and authenticated-frame recovery helper. Positive Dodge/removal/phases/Rushou checked. Alias/wrong-parent/forgery/two-invocations/provider loss/empty virtual carrier source parity remain mandatory.','Jidian':'Implemented actual optional distance targets including owner, ordinary Judge, Judge then logBegin sibling order/deferred logAfter, black ordinary owner Thunder and no mute. Owner target/red/unrelated checked; black nested recursion, full rejudge/child/death/fault controls remain mandatory.'},'lifecycle':'Real persisted frame Before/Begin/Body/End/After/Omitted in existing scheduler, direct scalar-log suppression and source-specific log children; phase restoration checked. Omitted native hooks, forceDie/includeOut/removal/death/fault exhaustiveness NOT established; native removed/out shared facts absent. Decision30 presentation None/nine producers unchanged.','Judge':'Existing JudgementProgress extracted/reused with optional actual delayed-trick context and ordinary frame; held replacement old-card receipt awaits loss before gain/finalization. Genuine Tiandao nonfirst equipment color flip/gains old/typed-byte ask restore passes. Sole GuiCai old-discard narrow source correction inspected but not targeted/full runtime accepted; payment-boundary/death/exhausted-deck/rejudge-removal/TianDu/GaiMing332 remain mandatory.','privacy':'F07 unimplemented; no P/Q/private transfer/projection or successful hidden-index proof. No matching-error privacy claim.','compatibility':'Real NativeSession restore/Custom-string/index1 answer passes at final-freeze3; old WS6 reader, current real WASM binary/web/IPC/WS roundtrips NOT run. Server private WAL semantic control cases fixed after cutoff, NOT compiled. No ABI758/WS6 acceptance.','final_runtime':'final-freeze6 NOT full runtime/strict checked. final3 strict+NativeSession gates historical to exact source3; four postcut file deltas recorded in command-source-bindings.','remaining':'F14+F07 and all unchecked branches mandatory; genuine ChangeHp/notrigger armor0 and DeathFlow still NEXT; no source262/290 increase, golden rerecord or Main merge.'})
owned=[]
for p in e.glob('checks-*/runner.pid'):
 pid=p.read_text().strip();z=subprocess.run(['ps','-p',pid,'-o','pid=,ppid=,command='],capture_output=True,text=True);assert not z.stdout.strip(),z.stdout;owned.append({'pid':int(pid),'active':False})
write(e/'producer-completion.json',{'utc':now(),'owned_runners':owned,'no_active_or_queued_owned_work':True,'cutoff_guard_stopped_future_commands':'actual18:44:55.361409Z; copied assertion text mentions old17:31:36 but numeric guard is correct and raw retained','no_new_heavy_after_cutoff':True,'signals_sent':False,'final_source_runtime_missing':True})
report=Path('/Users/ruyimg/.pi/agent/sessions/--Users-ruyimg-Documents-Claude-sanguosha--/subagent-artifacts/outputs/7017b4ab-78bf-497f-85ce-f54dbe430bd6/implementation/c331-phase9g-genuine-invocations.md');report.parent.mkdir(parents=True,exist_ok=True)
text=f'''# C331 phase9g — PARTIAL, BLOCK, independent review required
Verified2026-10-31; exclusive private writer/base5fc; source262/290 remaining28 +0 unchanged.
Start17:54:55.361409Z; checkpoint18:14:04 before18:14:55; no NEW heavy after18:44:55; seal due18:54:55.
Evidence `{e}`; no active/queued owned producer, signals, staging/commit/push or Main code import.

## Authoritative source and preservation
Final **final-freeze6/full-source6**,46increment/239composed; prior checkpoints/freezes remain immutable historical results.
Increment manifest `{res['increment_manifest_sha256']}`.
Increment tar `{res['increment_tar_sha256']}`; patch `{res['increment_patch_sha256']}`.
Full manifest `{res['composed_manifest_sha256']}`.
Full tar `{res['composed_tar_sha256']}`; patch `{res['composed_patch_sha256']}`.
Both patches reverse-check; materialized final snapshots read-only; C private index empty.
All original9f4167,9e7333 and separate41audit entries rehashed unchanged, including reds/expected/goldens/old readers.
P2 fixed additively in `golden-dispositions-c9f-reference-correction.json`:12bad references point to actual9f19/225 manifests.
Original sealed9f report/index/comparator unchanged; next comparator points to existing final6 manifest.
`protected-after.json` preserves original31867 Main protection, Own6/I02/CI/source37, generated18/dist37/schema4/hooks/C6/old readers.
Exact Parent DOCS-only9f4b74236f97b0b9dc72388b6e08d310d2f26f64/tree88dfb549...9905paths receipt verified separately.
Main runtime remains330/105/758/WS6; historical34eCI is not C validation; no blanket baseline refresh.
Private331/106/758/WS6, public format/API1 remain; no VM operand/context reinterpretation or public number allocation.

## Implemented finite mechanisms; completion is not claimed
Real named InvocationData frame and persisted finite machine use existing stack/queue/parks/slots and actual admitted owner/def/window.
Before→Begin→Body→End→After and Before-cancel→Omitted are structural phases; logBegin/after are actual children.
C7 both native starts preserve selected def; direct units suppress generic scalar prelog; source-specific costs/logs stay inside body.
Zhangwu optional variable HE holder payment awaits log/loss/trailing log, then recomputes exact source payment or n holder-source damage.
Jiding awaits full virtual CardUsed/Dodge/damage subtree before exact-def removal; named Rushou recovery requires actual Invocation/CardUsed receipt.
Yuance has two semantic controls/cancel, actual ordinary Judge and four color HP branches; no activation log.
Jidian has actual distance target/cancel, ordinary Judge before logBegin and deferred after-log, black owner Thunder, no recursion mute.
Ordinary Judge reuses JudgementProgress with optional delayed-trick context, real Tiandao queue, held-card receipt and awaited loss.
Sole GuiCai old-card discard correction is narrow; mixed-provider/other variants and GaiMing332 are not accepted/fixed by assertion.
Internal Control lowers existing public Custom/string options with native_kind omitted; no new public enum/object/required field.
**F07 is unimplemented:** old forced random acquisition remains a mandatory source defect; no hidden-card/privacy/wire completion claim.
**F14 remains P1:** actual revealed Wei/Shu runtime counterexample returnsNone versus sourceSome(true); explicit ignored regression retained.
Native declared unknown/ye identity and mutual yexinjia_friend storage/writers are absent; faction heuristic was not negated.

## Executed evidence and material limits
Original three invocation counterexamples and two Judge-consumer counterexamples plus named-recovery red retained before fixes.
Earlier source passes11new cases/1explicitF14ignore: phase typed/byte restores, Dodge/removal, n2+equipment/source-cancel,
Yuance four colors/cancel and genuine Tiandao nonfirst equipment flipping color/gaining old card, Jidian owner-target/red/unrelated, Rushou heal.
Final-freeze3 independent `RUSTFLAGS=-Dwarnings cargo check --offline -p engine-native --all-targets` PASS.
Final-freeze3 real NativeSession archive→Custom/string controls→normal index1 answer PASS, with18existing WASM warnings retained.
**Final-freeze6 is NOT compiled/full-runtime checked.** Postcut fixes: server-private WAL semantic controls, virtual-carrier fault guard,
forged payment-count bound, and append-only internal Control discriminant to retain old ChoiceKind hash/ordinal meanings.
Two full-native attempts failed compilation at optional-trick test callers; corrected in final3 but no final full test executed.
Server all-target attempt failed missing private WAL Control match; later exhaustive fix is uncompiled. No false all-green claim.
Final3 runner stopped naturally at cutoff; server/focused/golden/full commands after guard did not run or queue.
Final cheap caps9045 PASS, source unit2 PASS, layering/F7 zero; source binding files distinguish every tested snapshot from final deltas.
No fresh9g golden comparison was produced after compilation failure; original362C8/9f pairs and complete outcomes remain preserved.
No golden rerecord/C8 producer rerun or normalization;12original disputed cases remain red historical evidence, not refreshed acceptance.

## Mandatory continuation
`all17-12-five-fsm-lifecycle-judge-privacy-compat.json` separates implemented/runtime-checked/unchecked for every finding and12unproved rows.
F07 hidden-slot/private-binding/gainability/event privacy/PQ actual index submissions and frozen WS6/current WASM/web/IPC/WS remain mandatory.
Four other FSMs still need all cancel/payment-success/self-source/death/removal/forgery/fault/alias/recursive-child/exhausted-deck controls.
Lifecycle Omitted/forceDie/includeOut/hook cleanup and Judge paid-loss/held-card/death/provider-removal/TianDu cases remain mandatory.
ABI758/WS6 compatibility, all37/source equivalence, true ChangeHp(nottrigger armor0), DeathFlow/provider/LossEnd remain unaccepted.
Next same-writer slice must compile/test final6 first, then close these source/privacy/lifecycle boundaries; no Main merge follows.
'''
assert len(text.splitlines())<=80;report.write_text(text);(e/'report.md').write_text(text);write(e/'report-reference.json',{'path':str(report),'sha256':sha(report),'lines':len(text.splitlines())})
# No producers or mutable artifacts follow this inventory.
files=[{'path':str(p.relative_to(e)),'sha256':sha(p),'bytes':p.stat().st_size}for p in sorted(e.rglob('*'))if p.is_file()and p.name not in ['artifact-index.json','artifact-index.sha256']];index={'sealed_utc':now(),'status':'PARTIAL_BLOCK_FINAL6_NOT_RUNTIME_CHECKED','files':files,'external_report':{'path':str(report),'sha256':sha(report)},'increment_files':46,'source_files':239,'no_staged_files':True};write(e/'artifact-index.json',index);(e/'artifact-index.sha256').write_text(sha(e/'artifact-index.json')+'  artifact-index.json\n');assert all(sha(e/x['path'])==x['sha256']for x in files);assert not subprocess.check_output(['git','diff','--cached','--name-only'],cwd=r/'worktree').strip();print(json.dumps({'sealed_utc':index['sealed_utc'],'artifacts':len(files),'index_sha256':sha(e/'artifact-index.json'),'report_sha256':sha(report),'report_lines':len(text.splitlines())}))
