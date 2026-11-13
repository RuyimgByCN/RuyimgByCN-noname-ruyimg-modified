#!/usr/bin/env python3
from pathlib import Path
import datetime,hashlib,json,subprocess
OUT=Path(__file__).resolve().parent;BASE=OUT.parent.parent/'L28';REPO=OUT.parents[5]
def readl(p):return [json.loads(x)for x in p.read_text().splitlines()]
index=json.loads((BASE/'raw-input-index.json').read_text());req=readl(BASE/'session-l28-1794595037613-requests.jsonl')
checks=[]
for c in index['cases']:
    rows=c['request_reply_rows'];bylabel={}
    for row in rows:bylabel.setdefault(row['label'],[]).append(req[row['request_line']-1]['expression'])
    expected={
        'actual-phaseUse-start':(BASE/'browser/start-case.js').read_text(),
        'install-skills':'Object.assign(lib.skill,eval('+json.dumps((BASE/'upstream/skills-original.js').read_text())+'));true',
        'install-helpers':'Object.assign(game,eval('+json.dumps((BASE/'upstream/helpers-original.js').read_text())+'));true',
        'install-card':'Object.assign(lib.card,eval('+json.dumps((BASE/'upstream/card-original.js').read_text())+'));true',
        'reload':'location.reload();true',
    }
    checks.append({'case':c['case'],'exact_driver_payload_matches':{label:bylabel.get(label)==[source]for label,source in expected.items()},
        'mouse_event_count':sum(row['label'].startswith('mouse-')for row in rows),
        'original_card_bootstrap_drained_before_fixture':next(i for i,r in enumerate(rows)if r['label']=='drain-original-card-bootstrap')<next(i for i,r in enumerate(rows)if r['label']=='fixture')})
source=REPO.parent/'archive/upstream/noname-liuli/noname/extension/雷霆万钧/extension.js'
card='({\n'+''.join(source.read_text().splitlines(keepends=True)[8967:9016])+'\n})\n'
actual=(BASE/'upstream/card-original.js').read_text()
report={'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'head':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),
    'exact_card_property_reassembly_equal':card==actual,'card_source_lines':[8968,9016],'case_checks':checks,
    'runtime_scope':{'upstream_core':'original noname-liuli game/game.js 1.10.1.0.1','original_extension_properties':'five skill properties, nine original game helpers, one real card property are injected unchanged and registered by finishSkill; extension content/precontent is not executed',
    'actual_entry':'fresh location.reload per case, original onfree/cardsAsync drained, then real phaseUse/chooseToUse, ui.click.skill/card/ok/cancel and actual CDP Input.dispatchMouseEvent for custom buttons/flames',
    'not_verified':['complete extension installation/precontent/content/default interactions','full original standard+extra pile parity; native125 is a subset with21 tuples absent from original lists','complete upstream storage/event queue/history/RNG state','death/dying/chained fire damage, effective-suit modifiers, natural extra-phase/turn resets','th_zhengjing successful mini-game/target/distribution/cancellation upstream; dcdAnim ReferenceError occurs in original filter','direct UI cancellation while four-card great-Yeyan auto-selection is active; observed cancel follows undo to one flame','buxu missing-class/no-match and discard-pile fallback runtime; only source proves behavior here','th_zhengjing2 actual next Prepare resolution is outside this25-case runtime cohort']},
    'no_equivalence_signoff':True}
(OUT/'entry-and-boundary-audit.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'cases':len(checks),'payload_exact_cases':sum(all(c['exact_driver_payload_matches'].values())for c in checks),'card_property_exact':card==actual,'all_original_bootstrap_before_fixture':all(c['original_card_bootstrap_drained_before_fixture']for c in checks)},ensure_ascii=False))
