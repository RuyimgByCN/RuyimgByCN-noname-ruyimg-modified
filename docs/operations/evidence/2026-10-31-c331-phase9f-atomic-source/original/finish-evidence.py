from pathlib import Path
import json,hashlib,datetime,subprocess,shutil,re,os
r=Path(Path('/tmp/c331-phase7-root').read_text().strip());e=r/'evidence/phase9f';w=r/'worktree';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();now=lambda:datetime.datetime.now(datetime.timezone.utc).isoformat();write=lambda p,x:p.write_text(json.dumps(x,indent=2,ensure_ascii=False)+'\n')
audit=r/'evidence/phase9e-concordance-supplement';assert sha(audit/'artifact-index.json')=='abb59477b0f3290729e8e98e0fd25933f8c733b4fff9ccdbf9881c8fe8038108';ai=json.loads((audit/'artifact-index.json').read_text());assert all(sha(audit/x['path'])==x['sha256']for x in ai['files']);c=json.loads((audit/'concordance.json').read_text());assert all(sha(Path(x['source']['path']))==x['source']['sha256']for x in c['rows']);orig=Path('/Users/ruyimg/.pi/agent/sessions/--Users-ruyimg-Documents-Claude-sanguosha--/subagent-artifacts/outputs/0b7ce162-d228-460e-a9c9-7fb28e076018/implementation/c331-phase9e-source-predicate-repair.md');assert sha(orig)=='8a576676e38e5c04f56d45cba004ed3abece45236a7544f5e7309d49e2ebda74'
rec=e/'parent-receipts';rec.mkdir(exist_ok=True)
receipts=[]
for p in ['/tmp/noname-parent-post-c9e-main-write-receipt.json','/tmp/noname-post34e-doc-stage-audit.json','/tmp/noname-parent-c9e-concordance-source-bindings-audit.json','/tmp/noname-parent-c9f-checkpoint-enemy-source-review.json','/tmp/noname-parent-c9f-checkpoint-audit.json','/tmp/noname-post-c9e-doc-preview-audit.json','/tmp/noname-parent-continuation-2026-10-31.json']:
 q=Path(p)
 if q.is_file():shutil.copyfile(q,rec/q.name);receipts.append({'path':p,'sha256':sha(q)})
write(e/'additional-preservation.json',{'utc':now(),'supplement_index_sha256':sha(audit/'artifact-index.json'),'supplement_entries_verified':len(ai['files']),'all41_pinned_source_sha_verified':True,'original_report_sha256':sha(orig),'parent_exact_receipts':receipts,'Main_docs_and_intentional2540_stage':'Parent owned; exact receipts retained separately, no C baseline refresh/import; source37/Own6/I02/CI/lock/hooks/approvals/generated18/dist37/schema4 are independently protected in protected-after.json'})
notes={
'F01':'Allow genuine self-source Jueqing replacement; armor remains intact and child loseHp completes.',
'F02':'Resolve effective use_as through adjacent authenticated chain parents to the actual immediate CardUsed; direct and propagated Xingjunyan/Ciqiu/Xixue tested with non-Sha carrier and full restores.',
'F03':'Source self-damage cancels in Begin1 before Begin2; target qualification retains self-source cancellation even if nature changes.',
'F04':'Reject already-hit Shuchui before admission; admitted body consumes qualification without physical-kind requery.',
'F05':'Allow genuine self-source Xixue recovery.',
'F06':'Subtract full canonical amount using u32 then narrow checked level; 255/256/257, zero/negative amount exclusion and actual VM overflow rollback tested.',
'F08':'Choose from ordered hand+equipment pool with RuleRng; normal owned-card removal, equipment revoke, loss and CardsDiscarded children; equipment-only and ordinary-discard callback red/green, full child restores. Mixed-zone sampling not independently executed.',
'F10':'Retain terra for every later hit, use ordinary RNG over alive other seats, expire at existing global turn-end/start boundaries; real boundary red/green and full random-child restores. Whole temporary-provider model remains unproved.',
'F11':'Consume actual RuleRng Fire/Thunder draw and attribute retaliation to owning victim; exact RNG/state/child restores. Both nature branches were not independently exhaustively sampled.',
'F14':'Boss ally/enemy, two-team Versus, Doudizhu, standard Identity source friendship fallback supported/tested. Only actual poison is granted; genuine typed Mushengoumang primary declaration/id draws normally, foreign same-name definition excluded, child restores tested. Kingdoms known Wei/Shu enemy plus unknown/ye/yexinjia_friend storage remain mandatory mode gaps; no all-mode acceptance.',
'F15':'Reject source owning another enabled genuinely qualified Duqu definition; admitted body uses actual source. Global/grouped Shedu provider and lifetime are not accepted by this atomic correction.',
'F17':'Allow self-source Minwan and initialize actual target set; real command restores.'}
next_ids={'F07','F09','F12','F13','F16'};rows=[]
for f in c['findings']:
 item=dict(f);item['readonly_audit_status_preserved']=item.pop('status');item['original_runtime_reproduced']=item.pop('runtime_reproduced_in_this_readonly_audit');item['c9f_status']='mandatory_next_persisted_choice_invocation_slice'if f['id']in next_ids else 'partial_mandatory_mode_gap'if f['id']=='F14'else 'atomic_repaired_in_executed_scope';item['disposition']=notes.get(f['id'],'Mandatory persisted invocation/choice/cost/judgment/card-subtree continuation; no deterministic proxy, accepted variant, early removal or extra framework introduced. F13 exact admitted def removal must await complete card subtree including dodge and grouped recovery.');item['source_admission_body_evidence']='final-freeze2/source/, full-source/source/, checks-final2/predicate-traces/, command-source-bindings.json' if f['id']not in next_ids else 'unchanged source; separately reviewed next five-FSM slice';rows.append(item)
unproved=[]
for old in c['rows']:
 if old['status']=='unproved-boundary':unproved.append(dict(old,c9f_status='mandatory_unproved_unchanged',c9f_note='No whole provider/lifetime/renewal/grouped/async acceptance from finite atomic rule repairs.'))
write(e/'all17-findings-and12-unproved-dispositions.json',{'utc':now(),'basis_index':sha(audit/'artifact-index.json'),'finding_count':len(rows),'unproved_count':len(unproved),'findings':rows,'unproved_rows':unproved,'additional_birth_fix':'At Chain child Start reject a target still present in immediate parent remaining queue; valid popped child positive typed/byte restore plus forged rejection. Later mutable stages remain governed by existing stage/cause validation. No new ledger or format.','whole_goal':'BLOCK;262/290 remaining28 +0; F14 partial, next five FSM mandatory, ChangeHp/DeathFlow not started; no golden rerecord'})
results=json.loads((e/'checks-final2/check-results.json').read_text());assert len(results)==7
bindings=json.loads((e/'command-source-bindings.json').read_text());cut=datetime.datetime(2026,10,31,17,31,36,tzinfo=datetime.timezone.utc).timestamp();times=[]
for x in results:
 p=e/'checks-final2'/(x['name']+'.log');birth=p.stat().st_birthtime;times.append({'name':x['name'],'log_open_utc':datetime.datetime.fromtimestamp(birth,datetime.timezone.utc).isoformat(),'wall_seconds':x['seconds'],'heavy':x['argv'][0]=='cargo','log_sha256':sha(p)})
 if x['argv'][0]=='cargo':assert birth<cut
bindings['producer_timing']=times;bindings['timing_note']='Log opened immediately before serialized helper launch; cargo launches explicitly guarded against cutoff. Final full-native launched17:31:24.739305Z before17:31:36 and completed naturally afterward; no new cargo after cutoff.';write(e/'command-source-bindings.json',bindings)
owned=[]
for p in e.glob('checks-*/runner.pid'):
 pid=p.read_text().strip();z=subprocess.run(['ps','-p',pid,'-o','pid=,ppid=,command='],capture_output=True,text=True);assert not z.stdout.strip(),z.stdout;owned.append({'pid_file':str(p),'pid':int(pid),'active':False})
write(e/'producer-completion.json',{'utc':now(),'owned_runners':owned,'all_final_commands_completed':True,'no_owned_producer_queued_or_active':True,'no_signals_or_cleanup':True,'no_new_heavy_after_cutoff':True})
for d in [e/'final-freeze2/source',e/'full-source/source']:
 for p in d.rglob('*'):
  if p.is_file():p.chmod(0o444)
assert all(sha(w/x['path'])==x['sha256']for x in json.loads((e/'final-freeze2/complete-source-manifest.json').read_text())['files']);assert not subprocess.check_output(['git','diff','--cached','--name-only'],cwd=w).strip()
print(json.dumps({'utc':now(),'findings':len(rows),'unproved_rows':len(unproved),'predicate_raw_capture_triples':len(list((e/'checks-final2/predicate-traces').glob('*.json'))),'owned_runners_finished':len(owned),'source_still_frozen':True}))
