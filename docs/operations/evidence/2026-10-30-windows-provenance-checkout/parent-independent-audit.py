from pathlib import Path
import json,hashlib,os,subprocess,re
R=Path('/tmp/noname-ci-777-checkout-selfcheck-nisccqfo');A=Path('/tmp/noname-ci-777-provenance-byte-checkout-h8CFic');P=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');H='777bf33969e5ecc04b63971adab26be32f12fd28'
O=Path('/tmp/noname-parent-windows-byte-audit');O.mkdir(exist_ok=True)
def sha(b):return hashlib.sha256(b).hexdigest()
def ident(p):
 b=p.read_bytes();return {'path':str(p),'bytes':len(b),'sha256':sha(b)}
def git(*args):return subprocess.run(['git','-C',str(P),*args],check=True,capture_output=True,env={**os.environ,'GIT_OPTIONAL_LOCKS':'0'}).stdout
def blob(path):return git('cat-file','blob',H+':'+path)
def oid(b):return hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
manifest=json.loads((A/'evidence-manifest.json').read_bytes());verified=[]
for row in manifest['files']:
 p=Path(row['path'])
 if 'symlink' in row:assert p.is_symlink() and os.readlink(p)==row['symlink'];verified.append({'path':str(p),'symlink':os.readlink(p)})
 else:
  v=ident(p);assert v['bytes']==row['bytes'] and v['sha256']==row['sha256'];verified.append(v)
assert len(verified)==472
s=json.loads((R/'summary.json').read_bytes());cmds=json.loads((R/'commands.json').read_bytes());assert len(cmds)==96
for c in cmds:assert c['exit']==c['expected_exit'];assert (R/c['stdout']).is_file() and (R/c['stderr']).is_file()
source=[]
assert s['source14_before']==s['source14_after']
for path,digest in s['source14_before'].items():
 b=blob(path);assert sha(b)==digest and (P/path).read_bytes()==b
 if (R/'source'/path).exists():assert (R/'source'/path).read_bytes()==b
 source.append({'path':path,'bytes':len(b),'sha256':sha(b),'main_matches':True,'git_oid':oid(b)})
original=blob('.gitattributes');candidate=(R/'candidate/.gitattributes').read_bytes();assert candidate==original+b'# Preserve the exact provenance bytes pinned by validateAudioCompat.\ncontent/original-audio-compat/provenance.json -text\n';assert (P/'.gitattributes').read_bytes()==original
assert sha(candidate)=='9dbbbde6fb0c137b5a47b3d3738b3ca9399dec80a03f9acc1399ffc1d3619388';assert sha((R/'candidate.patch').read_bytes())=='8f81c68354401bee2807cf4cd80288c6d50cbf62d12a357a5a4f3b8474e8a4ff'
imports=[]
for c in cmds:
 a=c['argv']
 if 'update-index' in a:
  mode,expected,path=a[-1].split(',',2);b=candidate if '--add' not in a else blob(path);assert oid(b)==expected
  hashcmd=next(x for x in cmds if 'hash-object' in x['argv'] and x['stdin_sha256']==sha(b));assert (R/hashcmd['stdout']).read_text().strip()==expected
  imports.append({'path':path,'git_oid':expected,'source_sha256':sha(b),'original_blob': '--add' in a})
assert len(imports)==8 and sum(x['original_blob'] for x in imports)==7
fixtures=[]
for p in (R/'source/scripts').iterdir():
 if p.is_file():
  path=str(p.relative_to(R/'source'));assert p.read_bytes()==blob(path);fixtures.append(ident(p))
canonical=blob('content/original-audio-compat/provenance.json');prov=json.loads(canonical);assert sha(canonical)=='1bd98a3e5d7ab22f6749b132d041f409e95a0dbf2f7ed9a603ec256422cc5313'
assert '1bd98a3e5d7ab22f6749b132d041f409e95a0dbf2f7ed9a603ec256422cc5313' in blob('scripts/original-audio-resources.mjs').decode()
validator=[]
for tag,num,accept in [('windows-before-red','060',False),('lf-before-green','062',True),('windows-after-green','068',True),('lf-after-green','070',True)]:
 p=R/f'commands/{num}-{tag}.stdout';v=json.loads(p.read_bytes());case=s['cases'][tag];assert case['validator']==v and v['accepted']==accept and len(v['calls'])==10
 for n,binding in enumerate(prov['bindings']):
  for call,key in [(v['calls'][n*2],'source'),(v['calls'][n*2+1],'derivative')]:
   assert sha(Path(call['file']).read_bytes())==call['sha256']==binding[key+'_sha256'];assert Path(call['file']).stat().st_size==call['bytes']
 if accept:assert v['bindings']==prov['bindings']
 else:assert v['error']=='audio compat fixed provenance/recipe/tool binding differs'
 # The red checkout was overwritten by subsequent actual Git checkouts; reconstruct only its recorded bytes hypothesis, explicitly not a second checkout experiment.
 expected=canonical.replace(b'\n',b'\r\n') if not accept else canonical;meta=case['provenance'];assert sha(expected)==meta['sha256'] and len(expected)==meta['bytes'] and expected.count(b'\r\n')==meta['crlf'];assert json.loads(expected)==prov
 validator.append({'case':tag,'output':ident(p),'accepted':accept,'callbacks':10,'provenance':meta,'historical_checkout_byte_scope':'red bytes reconstructed from canonical LF and recorded metadata; actual Git argv/transcript retained, final checkout is LF'})
for row in s['readonly_core5']:
 p=R/'core'/row['path'];assert ident(p)['sha256']==row['sha256'];assert p.read_bytes()==(Path('/Users/ruyimg/Documents/Claude/sanguosha/archive/upstream/noname/apps/core')/row['path']).read_bytes()
tap=[]
for p,passes,fails in [(R/'commands/078-original-resource17.stdout',17,0),(Path('/tmp/noname-ci-777-checkout-selfcheck-tk30ip4g/commands/077-original-resource17.stdout'),16,1)]:
 text=p.read_text();assert re.search(rf'^(?:#|ℹ) pass {passes}$',text,re.M) and re.search(rf'^(?:#|ℹ) fail {fails}$',text,re.M);tap.append({**ident(p),'passes':passes,'fails':fails})
idx=Path(git('rev-parse','--path-format=absolute','--git-path','index').decode().strip());index=ident(idx);assert index['sha256']=='2b4bee80bdba1ccecceee3a522fadd5722901f289cda98c96eb1c5ac04348b4d';assert not git('diff','--cached','--name-only');head=git('rev-parse','HEAD').decode().strip();assert head==H
out={'result':'PASS','scope':'Fresh read-only bytes/JSON/Git cat-file parser; no producer execution, tests, builds, or network','manifest_identity':ident(A/'evidence-manifest.json'),'verified_manifest_count':len(verified),'verified_manifest':verified,'command_count':len(cmds),'all_recorded_exits_match_expected':True,'imports':imports,'source14':source,'fixture_scripts':fixtures,'candidate':ident(R/'candidate/.gitattributes'),'patch':ident(R/'candidate.patch'),'canonical_provenance':{'bytes':len(canonical),'sha256':sha(canonical)},'validator_cases':validator,'TAP':tap,'head':head,'index':index,'staged_paths':[],'limitations':['Original Windows runner provenance bytes/SHA were not observed. Controlled checkout suffices to reproduce error; new platform CI still required.','No claim that current CRLF checkout file persists: overwritten by later actual LF/candidate checkouts; all transcript files remain manifest-bound.']}
(O/'independent-results.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({'result':'PASS','output':str(O/'independent-results.json'),'manifest':len(verified),'source14':len(source),'fixture_scripts':len(fixtures),'imports':len(imports),'commands':len(cmds),'index':index,'validator_cases':[{'case':v['case'],'accepted':v['accepted']} for v in validator]},indent=2))
