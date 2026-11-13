from pathlib import Path
import datetime,json,re,subprocess
root=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');ev=root/'docs/operations/evidence/2026-11-13-queue0-gates'
names=['02-line-check-tests','03b-source-lines-final','04d-native-three-complete-final','05c-strict-all-targets-final','06b-wasm-check-rustup','07-workspace-complete','08-conformance','10d-frontend-tests','11-frontend-typecheck','12-build-native','13-smoke-native','14-native-wasm-cross-target','15-cross-target-hash','16-desktop-tests','17-frontend-build','19-preflight-final']
rows=[]
for name in names:
    p=ev/(name+'.json');assert p.exists(),name
    data=json.loads(p.read_text());assert data['exit_code']==0,(name,data['exit_code'])
    log=(ev/(name+'.log')).read_text(errors='replace')
    rust=[tuple(map(int,m)) for m in re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',log)]
    counts={'passed':sum(r[0] for r in rust),'failed':sum(r[1] for r in rust),'ignored':sum(r[2] for r in rust)} if rust else None
    node={k:int(v) for k,v in re.findall(r'^# (pass|fail|skipped) (\d+)$',log,re.M)}
    rows.append({'name':name,'command':data['command'],'cwd':data['cwd'],'env_overrides':data['env_overrides'],'exit_code':0,'duration_seconds':data['duration_seconds'],'rust_counts':counts,'node_counts':node or None,'log':name+'.log','metadata':name+'.json'})
record={'created_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source_head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),'tracked_dirty':True,'versions':{'rules':333,'NGSA':107,'schema':5,'ABI':759,'WS':8},'checks':rows}
out=ev/'gate-summary.json';assert not out.exists();out.write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n')
for row in rows:print(row['name'],row['rust_counts'] or row['node_counts'] or 'exit 0')
