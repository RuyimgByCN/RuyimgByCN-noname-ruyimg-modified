"""Freeze a neutral raw-input map; it contains no comparison conclusions."""
from pathlib import Path
import datetime
import hashlib
import json
import shutil
import subprocess

BASE=Path(__file__).resolve().parent
REPO=BASE.parents[4]
assert (REPO/'Cargo.toml').exists()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def rel(p):return str(p.relative_to(BASE))
meta=json.loads((BASE/'active-session-l28.json').read_text())
requests_path=Path(meta['transcript']+'-requests.jsonl')
replies_path=Path(meta['transcript']+'-replies.jsonl')
requests=[json.loads(x)for x in requests_path.read_text().splitlines()]
replies=[json.loads(x)for x in replies_path.read_text().splitlines()]
assert len(requests)==len(replies)and all(a['at']==b['at']for a,b in zip(requests,replies))
native_path=BASE/'native/round2.jsonl'
native=[json.loads(x)for x in native_path.read_text().splitlines()]
browser_path=BASE/'browser/round8-clean/case-results.jsonl'
browser=[json.loads(x)for x in browser_path.read_text().splitlines()]
controls=[json.loads(x)for x in (browser_path.parent/'control-map.jsonl').read_text().splitlines()]
assert len(browser)==25 and len(set(r['case']for r in browser))==25
cases=[]
for line,row in enumerate(browser,1):
    entry={'case':row['case'],'input_note':'same initial fixture as native great-one; DOM requests two flames while native requests great mode and one target'if row['case']=='yeyan-great-two'else None,'browser_jsonl':rel(browser_path),'browser_line':line,
           'fixture_case_in_native':row['native_case'],'native_jsonl':rel(native_path),
           'native_line':next(i for i,n in enumerate(native,1)if n['case']==row['native_case']),
           'request_reply_rows':[]}
    for c in controls:
        if c['case']!=row['case']:continue
        i=c['request_line'];raw=requests[i-1]
        assert hashlib.sha256(raw['expression'].encode()).hexdigest()==c['expression_sha256']
        entry['request_reply_rows'].append({'label':c['label'],'request_line':i,'reply_line':i,
            'at':raw['at'],'request_expression_sha256':c['expression_sha256']})
    cases.append(entry)
initial=json.loads((BASE/'initial-inputs.json').read_text())
source=[]
for f in initial['inputs']:
    p=Path(f['path']);h=sha(p);assert h==f['sha256']
    source.append({'path':str(p),'sha256':h,'snapshot':f['snapshot'],'unchanged_from_initial':True})
extra=[REPO.parent/'archive/upstream/noname-liuli/noname/game/game.js',
       REPO.parent/'archive/upstream/noname-liuli/noname/card/standard.js',
       REPO.parent/'archive/upstream/noname-liuli/noname/card/extra.js',
       REPO.parent/'archive/upstream/noname-liuli/noname/extension/雷霆万钧/thunder.css',
       REPO/'crates/engine-native-kernel/src/kernel/judge.rs',
       REPO/'crates/engine-native-kernel/src/catalog.rs',
       REPO/'crates/engine-native-kernel/src/catalog/card_kind.rs',
       REPO/'crates/engine-native-content/src/content/standard/cards.rs',REPO/'Cargo.lock']
for p in extra:
    local=Path('inputs')/('upstream'/p.relative_to(REPO.parent)if p.is_relative_to(REPO.parent/'archive')else p.relative_to(REPO))
    target=BASE/local;target.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,target)
    source.append({'path':str(p),'sha256':sha(p),'snapshot':str(local)})
archives=[{'path':rel(p),'bytes':p.stat().st_size,'sha256':sha(p)}for p in sorted((BASE/'native/round2').glob('*.ngsa'))]
assert len(archives)==61
http_path=Path(meta['transcript']+'-http.jsonl')
http=[json.loads(x)for x in http_path.read_text().splitlines()]
raw_files=[requests_path,replies_path,http_path]
raw_files.extend(p for p in [Path(meta['transcript']+'-events.jsonl'),Path(meta['transcript']+'-control-errors.jsonl')]if p.exists())
history=[]
for p in sorted((BASE/'browser').glob('*/case-results.jsonl')):
    rows=[json.loads(x)for x in p.read_text().splitlines()]
    history.append({'file':rel(p),'sha256':sha(p),'raw_rows':len(rows),'cases':[r['case']for r in rows],
                    'cohort':'final selected input'if p==browser_path else 'retained earlier attempt; not selected in final cohort'})
out={'frozen_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),
     'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=REPO,text=True).strip(),
     'dirty_tracked':subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=REPO,text=True).splitlines(),
     'versions':{'rule':333,'NGSA':107,'schema':5,'ABI':759,'WS':8},
     'role':'runtime collector; separate independent reviewer will assess these raw inputs',
     'selected_cohort_rule':'Use all 25 rows of round8-clean with no deduplication or conditional filtering. All earlier rows remain retained; early cohorts did not drain the original cardsAsync/onfree before fixture setup, so their final pile could contain late original cards without native IDs.',
     'session':meta,'raw_transcripts':[{'path':rel(p),'sha256':sha(p),'bytes':p.stat().st_size,'rows':len(p.read_text().splitlines())}for p in raw_files],
     'source_inputs':source,'cases':cases,'archives':archives,'retained_browser_rounds':history,
     'native':{'probe':'native/probe-02.rs','probe_sha256':sha(BASE/'native/probe-02.rs'),
               'command':'native/round2-command.json','stdout':rel(native_path),'stderr':'native/round2.stderr',
               'exit':'native/round2-exit.json','cases':22,'pending_archives':39,'final_archives':22,
               'cargo_manifest':'native/Cargo.toml','cargo_lock':'native/Cargo.lock',
               'formal_package':'native/round2/formal-frozen-package.json'},
     'browser_setup':{'launch':'launch.mjs','fixture_and_config':'browser/start-case.js','driver':'browser/run-cases.py',
                      'selected_native_fixture':'native/round2.jsonl: each case initial object',
                      'upstream_properties':'upstream/skills-original.js','helpers':'upstream/helpers-original.js',
                      'physical_card_property':'upstream/card-original.js','source_line_map':'upstream/source-slices.json',
                      'origin':meta['url'],'version':'1.10.1.0.1','extension_content_and_precontent_executed':False,
                      'spine_runtime_loaded':False,'spine_paths_in_http_log':[r['path']for r in http if r['spineBlocked']],
                      'config_entry':'start-case.js: strengthen_skill key uses actual game.js getStrength; default and forbid recorded per fixture; no helper override',
                      'bootstrap_entry':'run-cases.py install(): call real lib.init.onfree if needed and wait original cardsAsync before replacing fixture'},
     'historical_attempt_entries':{'first_extraction':'upstream/extraction-first-error.txt','first_card_slice':'upstream/card-first-extraction-error.txt',
          'original_yeyan_extra_confirmation':'browser/round2/yeyan-small-one.json','initial_direct_cancel_unavailable':'browser/round6-final/yeyan-great-discard-cancel.json',
          'early_delayed_pile_diagnostic':'browser/round6-final/case-results.jsonl; all earlier rows retained',
          'first_comparison_assertion':'compare-first.stderr','pre_bootstrap_comparison':'compare-second.stdout'},
     'cleanup':'cleanup.json'}
(BASE/'raw-input-index.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'cases':len(cases),'native_cases':len(native),'archives':len(archives),'raw_requests':len(requests),'source_inputs':len(source),'spine_http_paths':out['browser_setup']['spine_paths_in_http_log']}))
