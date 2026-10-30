import hashlib,json,os,re,subprocess
from pathlib import Path
import yaml
root=Path.cwd();out=(root/'../evidence/corrections').resolve()
paths=['.github/workflows/ci.yml','.github/actions/prepare-original-content/action.yml','docs/operations/native-ci.md','.github/actions/prepare-original-content/liuli-longxue.patch']
old=yaml.load((out/'original'/paths[0]).read_text(),Loader=yaml.BaseLoader)
new=yaml.load(Path(paths[0]).read_text(),Loader=yaml.BaseLoader)
action=yaml.load(Path(paths[1]).read_text(),Loader=yaml.BaseLoader)
assert old['on']==new['on'];assert set(old['jobs'])==set(new['jobs'])
for name,job in old['jobs'].items():
    assert job['runs-on']==new['jobs'][name]['runs-on']
    assert job.get('strategy')==new['jobs'][name].get('strategy')
    oldruns=[s['run'] for s in job['steps'] if 'run' in s]
    newruns=[s['run'] for s in new['jobs'][name]['steps'] if 'run' in s]
    assert all(c in newruns for c in oldruns),(name,oldruns,newruns)
    for s in new['jobs'][name]['steps']:
        if s.get('uses')=='./noname-next/.github/actions/prepare-original-content':
            assert s.get('with',{}).get('integration-sources','false')==('true' if name in ['gates','native-web'] else 'false')
for step in action['runs']['steps']:
    if 'run' in step:
        assert step['shell']=='bash'
        subprocess.run(['bash','-n'],input=step['run'],text=True,check=True)
checkout=[s['with'] for s in action['runs']['steps'] if s.get('uses')=='actions/checkout@v4']
assert len(checkout)==2
assert checkout[0]['ref']==json.loads(Path('content/original/asset-manifest.json').read_text())['sources']['core']['head']
assert all(not s['path'].startswith(('/', '../')) for s in checkout)
assert not any('--skin-source' in s.get('run','') for s in action['runs']['steps'])
assert sum(len(j.get('strategy',{}).get('matrix',{}).get('os',['single'])) for j in new['jobs'].values())==10
sha=lambda b:hashlib.sha256(b).hexdigest()
full=b'';correction=b'';files={}
for name in paths:
    p=Path(name);final=p.read_bytes();base=subprocess.run(['git','show','HEAD:'+name],capture_output=True)
    original=out/'original'/name
    files[name]={'base_sha256':sha(base.stdout) if base.returncode==0 else None,'previous_reviewed_sha256':sha(original.read_bytes()) if original.exists() else None,'final_sha256':sha(final),'final_bytes':len(final)}
    if base.returncode==0:full+=subprocess.check_output(['git','diff','--binary','--',name])
    else:
        diff=subprocess.run(['git','diff','--no-index','--binary','--','/dev/null',name],capture_output=True)
        assert diff.returncode==1;full+=diff.stdout
    if original.exists():
        diff=subprocess.run(['git','diff','--no-index','--binary','--',str(original),name],capture_output=True)
        assert diff.returncode==1
        text=diff.stdout.decode().splitlines(keepends=True)
        text[0]=f'diff --git a/{name} b/{name}\n'
        for i,line in enumerate(text):
            if line.startswith('--- '):text[i]=f'--- a/{name}\n';break
        correction+=''.join(text).encode()
    else:
        diff=subprocess.run(['git','diff','--no-index','--binary','--','/dev/null',name],capture_output=True)
        assert diff.returncode==1;correction+=diff.stdout
(out/'native-ci-only.patch').write_bytes(full)
(out/'correction-only.patch').write_bytes(correction)
(out/'ci-file-sha256.json').write_text(json.dumps(files,indent=2)+'\n')
checks=[]
for patch in ['native-ci-only.patch','correction-only.patch']:
    c=['git','apply','--reverse','--check',str(out/patch)]
    r=subprocess.run(c,capture_output=True,text=True);assert r.returncode==0,r.stderr
    checks.append({'command':c,'exit':r.returncode,'sha256':sha((out/patch).read_bytes())})
subprocess.run(['git','diff','--check'],check=True)
assert subprocess.check_output(['git','diff','--cached','--name-only'])==b''
assert sha(Path(subprocess.check_output(['git','rev-parse','--absolute-git-dir']).decode().strip(),'index').read_bytes())=='f407bd0b312b7b3a171856d168666d1df774d251d3be07cda62ce3bc59ba4b2f'
overlay=json.loads(Path('../evidence/borrowed-resource-overlay.json').read_text())['not_part_of_ci_patch']
assert all(sha(Path(n).read_bytes())==h for n,h in overlay.items())
canonical=Path('/Users/ruyimg/Documents/Claude/sanguosha/archive/upstream/noname-liuli')
seven=json.loads((out/'liuli-seven-original.json').read_text())['files']
assert all(sha((canonical/n).read_bytes())==v['canonical_sha256'] for n,v in seven.items())
base='255bbfbd7f6916ee77bd29799826a225bd10871f'
# These are exact committed lists. No dist/evidence/history filtering is performed.
committed=subprocess.check_output(['git','-c','core.quotepath=false','ls-tree','-r','--full-tree',base])
(out/'baseline-255-ls-tree.txt').write_bytes(committed)
names=subprocess.check_output(['git','ls-tree','-r','--name-only','-z',base]).decode().strip('\0').split('\0')
(out/'baseline-255-tracked-paths.txt').write_text('\n'.join(names)+'\n')
(out/'ci-patch-paths.txt').write_text('\n'.join(paths)+'\n')
objects=subprocess.check_output(['git','rev-list','--objects',base],env={**os.environ,'GIT_NO_LAZY_FETCH':'1'})
(out/'baseline-255-reachable-objects.txt').write_bytes(objects)
commits=subprocess.check_output(['git','rev-list',base]);(out/'baseline-255-ancestors.txt').write_bytes(commits)
prepared=sorted(set(names)|set(paths)|set(overlay))
(out/'local-prepared-paths-not-a-commit.txt').write_text('\n'.join(prepared)+'\n')
publication={'base_commit':base,'baseline_committed_paths':len(names),'baseline_tracked_dist_paths':sum(n.startswith('apps/web-client/dist/') for n in names),'baseline_tracked_raw_evidence_paths':sum(n.startswith('docs/operations/evidence/') for n in names),'baseline_reachable_objects':len(objects.splitlines()),'baseline_reachable_commits':len(commits.splitlines()),'parent_supplied_object_audit':{'uncompressed_bytes':3349878968,'blobs_over_100MiB':0,'source':'parent follow-up and independent review; not recomputed here','not_network_transfer_size':True},'local_prepared_union_paths':len(prepared),'not_final_publication_commit':True,'unfiltered_scope':'entire final accepted tracked tree plus reachable objects absent from target remote; path lists cannot filter git push','parent_current_accepted_main_prefix':'977cdf1f','required_before_permission':'parent refresh exact final accepted CI commit git ls-tree, ancestors/object audit; fill commit ID','actual_pushes':0,'actual_actions_runs':0}
(out/'publication-scope-correction.json').write_text(json.dumps(publication,indent=2)+'\n')
result={'passed':True,'all_prior_jobs_matrices_and_gate_commands_preserved':True,'matrix_executions':10,'both_yaml_parsed':True,'all_action_shell_syntax_passed':True,'six_resource_premise_files_unchanged':True,'canonical_seven_modifications_unchanged':True,'product_index_identical_to_review':True,'no_staged_files':True,'files':files,'patch_checks':checks,'publication':publication,'limits':'no actionlint installed, Cargo, WASM build or true Actions run'}
(out/'final-static-validation.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
