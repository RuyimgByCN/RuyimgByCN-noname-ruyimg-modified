import ast,hashlib,json,os,re,shutil,subprocess,tempfile
from pathlib import Path
import yaml
root=Path.cwd();evidence=(root/'../evidence/corrections').resolve()
canonical=Path('/Users/ruyimg/Documents/Claude/sanguosha/archive/upstream/noname-liuli')
action=yaml.safe_load((root/'.github/actions/prepare-original-content/action.yml').read_text())
steps=action['runs']['steps'];checkout=next(s for s in steps if s.get('with',{}).get('repository')=='Helasisy1984/Noname-Liuli-Modified')
paths=[s.removeprefix('/') for s in checkout['with']['sparse-checkout'].splitlines()]
assert len(paths)==12 and checkout['with']['ref']=='8485dbb29ff16f9b258a766b1a17ef75f883aade'
assert checkout['with']['sparse-checkout-cone-mode'] is False
original=json.loads((evidence/'liuli-seven-original.json').read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
canonical_before={n:sha((canonical/n).read_bytes()) for n in original['files']}
fixture=Path(tempfile.mkdtemp(prefix='liuli-',dir=evidence))
env={**os.environ,'GIT_CONFIG_NOSYSTEM':'1','GIT_CONFIG_GLOBAL':'/dev/null','GIT_CEILING_DIRECTORIES':str(fixture),'GIT_NO_LAZY_FETCH':'1','GITHUB_WORKSPACE':str(fixture)}
for k in ['GIT_DIR','GIT_WORK_TREE','GIT_INDEX_FILE','GIT_OBJECT_DIRECTORY','GIT_ALTERNATE_OBJECT_DIRECTORIES']:env.pop(k,None)
repo=fixture/'archive/upstream/noname-liuli';repo.mkdir(parents=True)
def git(*args):return subprocess.check_output(['git','-C',str(repo),*args],env=env).decode().strip()
try:
    # Missing source is an actual filesystem failure, before any fixture input is restored.
    try:(repo/paths[0]).read_bytes();raise AssertionError('source unexpectedly present')
    except FileNotFoundError:missing_red=True
    inputs={}
    for name in paths:
        raw=subprocess.check_output(['git','-C',str(canonical),'show',checkout['with']['ref']+':'+name],env=env)
        p=repo/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(raw)
        inputs[name]={'head_sha256':sha(raw),'bytes':len(raw),'canonical_sha256':sha((canonical/name).read_bytes())}
    # Own fake skin proves the real non-cone patterns exclude unrelated asset input.
    skin=repo/'noname/extension/千幻聆音/skin/not-selected.png';skin.parent.mkdir(parents=True);skin.write_bytes(b'fixture only')
    git('init','-q');assert (repo/'.git').is_dir();assert Path(git('rev-parse','--show-toplevel'))==repo.resolve()
    git('config','user.email','fixture@example.invalid');git('config','user.name','Isolated CI fixture')
    git('add','.');git('commit','-qm','isolated real source bytes fixture');fixture_head=git('rev-parse','HEAD')
    subprocess.run(['git','-C',str(repo),'sparse-checkout','set','--no-cone','--stdin'],input=checkout['with']['sparse-checkout'],text=True,env=env,check=True)
    assert not skin.exists();assert git('status','--porcelain')==''
    for name in paths:assert (repo/name).is_file(),name
    source_tests={
      'l10_epic_cards_eight':'noname/extension/史诗卡牌/extension.js',
      'l16_drag_read_eight':'noname/extension/拖拽读取/extension.js',
      'l18_update_guard_eight':'noname/extension/更新保护/extension.js',
      'l21_martial_eight':'noname/extension/武皇争霸/extension.js',
      'l26_fool_eight':'noname/extension/笨蛋插件/extension.js',
    }
    predicates={}
    for test,name in source_tests.items():
        rs=(root/f'crates/engine-native/tests/{test}.rs').read_text()
        count=0
        for negative,literal in re.findall(r'assert!\((!?)(?:upstream|source)\.contains\(("(?:[^"\\]|\\.)*")\)\)',rs):
            needle=json.loads(literal);expected=not bool(negative)
            for text in [(repo/name).read_text(),(canonical/name).read_text()]:assert (needle in text)==expected,(test,needle)
            count+=1
        assert count>0
        predicates[test]=count
        assert 'upstream_extension()' in rs
    loader=(root/'crates/content-native-loader/src/update_guard.rs').read_text()
    for constant in ['URC_URL','FASTGIT_URL']:
        needle=re.search(r'pub const '+constant+r': &str = "([^"]+)"',loader)[1]
        assert needle in (repo/source_tests['l18_update_guard_eight']).read_text()
    assert (repo/'noname/extension/武皇争霸/mightyFoe.js').read_text().startswith('window.mightyFoe')
    # Mirror the source-only assertions in the explicitly registered boss/Longxue tests.
    extra=(repo/'noname/character/extra.js').read_text()
    seg=extra[extra.index('hlss_xiangxing:{'):][:900]
    assert ('player.storage.hlss_xiangxing=3' in seg or 'player.storage.hlss_xiangxing = 3' in seg)
    assert 'damageEnd' in seg and 'loseHpEnd' in seg
    seg=extra[extra.index('hlss_xiangxing_info'):][:260]
    assert '随机分配给其他角色' in seg and '体力值始终为满' in seg
    plans={name:ast.literal_eval((repo/name).read_text()) for name in paths if '/data/' in name}
    assert plans['noname/extension/斗转星移/data/inputPlan/空白']==['']
    assert plans['noname/extension/斗转星移/data/outputPlan/无']==['']
    standard=plans['noname/extension/斗转星移/data/inputPlan/标准包'];volcano=plans['noname/extension/斗转星移/data/inputPlan/风林火山']
    assert standard[0]=='old_caocao' and standard[-1]=='mateng' and 'shen_diaochan' in standard
    assert volcano[-1]=='zhulu_jiaxu' and standard!=volcano
    longxue='noname/extension/龙血玄黄/extension.js'
    assert 'jsjiami' not in (repo/longxue).read_text(),'baseline must demonstrate actual existing-test red'
    assert 'mbqianlong_info' not in (repo/longxue).read_text()
    patch=root/'.github/actions/prepare-original-content/liuli-longxue.patch'
    target=fixture/'noname-next/.github/actions/prepare-original-content/liuli-longxue.patch';target.parent.mkdir(parents=True);shutil.copyfile(patch,target)
    restore=next(s for s in steps if s.get('name')=='Restore adopted Longxue source input')
    assert restore['working-directory']=='archive/upstream/noname-liuli'
    subprocess.run(['bash','-e','-c',restore['run']],cwd=repo,env=env,check=True)
    final=(repo/longxue).read_bytes()
    assert sha(final)==original['files'][longxue]['canonical_sha256']
    assert 'jsjiami' in final.decode() and 'mbqianlong_info' not in final.decode()
    rejected=subprocess.run(['bash','-e','-c',restore['run']],cwd=repo,env=env,capture_output=True,text=True)
    assert rejected.returncode!=0 and 'AssertionError' in rejected.stderr
    assert (repo/longxue).read_bytes()==final
    (evidence/'liuli-wrong-preimage-rejected.log').write_text(rejected.stderr)
    assert git('rev-parse','HEAD')==fixture_head
    dirty=subprocess.check_output(['git','-C',str(repo),'diff','--name-only','-z'],env=env).decode().strip('\0').split('\0')
    assert dirty==[longxue],dirty
    for name in paths:
        inputs[name]['final_sha256']=sha((repo/name).read_bytes())
        if name!=longxue:assert inputs[name]['head_sha256']==inputs[name]['final_sha256']
    for test in [*source_tests,'l17_douzhuan_eight','nc08_upstream_boss_zhugeliang','nc08_upstream_longxue']:
        assert any(f'../{test}.rs' in p.read_text() for p in (root/'crates/engine-native/tests/groups').glob('*.rs')),test
    audit=[]
    for name,values in original['files'].items():
        reader=[test for test,path in source_tests.items() if path==name]
        if name==longxue:reader=['nc08_upstream_longxue']
        audit.append({'path':name,**values,'raw_source_readers':reader,'ci_patch_required':name==longxue,'decision':'restore exact adopted bytes; immutable HEAD fails jsjiami assertion' if name==longxue else ('both byte variants satisfy every raw-source predicate; content hashes use tracked converted packages' if reader else 'no registered raw-source reader found; not a claim of behavioral equivalence')})
    assert canonical_before=={n:sha((canonical/n).read_bytes()) for n in original['files']}
    result={'passed':True,'missing_source_red':missing_red,'immutable_longxue_red':'jsjiami missing, actual source predicate false','adopted_patch_green':True,'wrong_preimage_rejected_before_write':True,'source_predicates_checked':predicates,'extra_boss_source_assertions':'passed','four_plan_source_assertions':'passed','registered_source_tests':8,'sparse_text_inputs':inputs,'seven_local_changes_audit':audit,'canonical_seven_bytes_unchanged':True,'fixture_head_unchanged':True,'patch_only_expected_file':dirty,'no_skin_assets':True,'note':'real source bytes in own fixture; no Rust tests, real upstream checkout or complete Actions run'}
    (evidence/'liuli-fixture.json').write_text(json.dumps(result,indent=2,ensure_ascii=False)+'\n')
    print('Liuli: missing-source red, baseline Longxue red, actual action restore green; 12 inputs, 8 registered readers, 7 canonical changes preserved')
finally:shutil.rmtree(fixture)
