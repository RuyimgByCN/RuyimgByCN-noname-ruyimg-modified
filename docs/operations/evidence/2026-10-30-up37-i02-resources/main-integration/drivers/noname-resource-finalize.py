import pathlib,subprocess,json,hashlib,os,shutil,time
r=pathlib.Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');o=r/'docs/operations/evidence/2026-10-30-up37-i02-resources/main-integration'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def save(n,x):(o/n).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
# Restore only this run's generated dist files, after byte-identified products have been retained.
products=json.loads((o/'production-products-sha256.json').read_text());tracked=json.loads((o/'dist-before.json').read_text());removed=[]
for relative,h in products.items():
 p=r/'apps/web-client/dist'/relative
 if str(p.relative_to(r)) not in tracked:
  assert p.is_file() and sha(p)==h,p
  p.unlink();removed.append(str(p.relative_to(r)))
for name,entry in tracked.items():
 assert entry['current']==entry['head'],name
 p=r/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(subprocess.check_output(['git','show','255bbfbd7f6916ee77bd29799826a225bd10871f:'+name],cwd=r));assert sha(p)==entry['head'],name
assert subprocess.check_output(['git','diff','--name-only','--','apps/web-client/dist'],cwd=r)==b''
# Keep one actual validated frontend, remove redundant first identical copy only.
first=o/'happy/validated-dist';second=o/'retry-real-http/happy/validated-dist'
assert second.is_dir()
for relative,h in products.items():assert sha(second/relative)==h,relative
if first.is_dir():shutil.rmtree(first)
source='''/tmp/noname-resource-main-hashes.py /tmp/noname-resource-main-run.py /tmp/noname-resource-url-audit.mjs /tmp/noname-resource-browser-audio.mjs /tmp/noname-resource-browser-errors.mjs /tmp/noname-resource-helper-review.mjs /tmp/noname-resource-real-http.py /tmp/noname-resource-real-http-retry.py /tmp/noname-resource-main-gates.py /tmp/noname-resource-finalize.py'''.split()
(o/'drivers').mkdir(exist_ok=True)
for name in source:shutil.copy2(name,o/'drivers'/pathlib.Path(name).name)
m=json.loads((r/'content/original/asset-manifest.json').read_text());old=json.loads(subprocess.check_output(['git','show','255bbfbd7f6916ee77bd29799826a225bd10871f:content/original/asset-manifest.json'],cwd=r));olds=old['assets'];assert len(olds)==2707;assert m['assets'][:len(olds)]==olds;assert m['trees'][:len(old['trees'])]==old['trees'];assert m['sources']==old['sources'];assert m['source']==old['source'];audio=[a for a in m['assets'] if a['kind']=='audio'];assert len(audio)==9652;assert sum(a['size'] for a in audio)==651011056
c=json.loads((r/'content/original/catalog.json').read_text());meta={'verified_utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=r,text=True).strip(),'versions':{'rules':329,'archive':105,'ABI':758,'WS':5,'backend':'WASM default'},'version_source_hashes':{name:sha(r/name) for name in ['crates/engine-native-content/src/content/standard.rs','crates/engine-native-kernel/src/archive.rs','crates/engine-api/src/version.rs','crates/engine-protocol/src/lib.rs','apps/web-client/package.json']},'manifest_schema':m['schema_version'],'old_rows_preserved_in_order':len(olds),'audio_rows':len(audio),'audio_bytes':sum(a['size'] for a in audio),'catalog_source':c['source'],'asset_sources':m['sources'],'CARGO_INCREMENTAL':'0','no_compile_of_rust_or_wasm':True,'npm_test_warnings':['2 ExperimentalWarning localStorage unavailable without --localstorage-file'],'adapters_failed':['initial guard preceded deliberate fault injection','first retry used wrong admission parent directory'],'dist_default_build_transient_write':True,'dist_restored_count':len(tracked),'removed_own_generated_dist_paths':removed,'retained_product_bundle':str(second.relative_to(r)),'asset_directory_retained_reason':'Canonical ignored resources are required by the real main product HTTP service; do not treat as unused build cache','ignored_assets_bytes':sum(p.stat().st_size for p in (r/'content/original/assets').rglob('*') if p.is_file()),'disk_free_bytes':shutil.disk_usage(r).free,'staged_files':subprocess.check_output(['git','diff','--cached','--name-only'],cwd=r,text=True).splitlines()}
assert not meta['staged_files'];save('cleanup-and-versions.json',meta)
(o/'git-status-final.txt').write_text(subprocess.check_output(['git','status','--short'],cwd=r,text=True));(o/'maintained-diff-final.txt').write_text(subprocess.check_output(['git','diff','--stat'],cwd=r,text=True))
print(json.dumps({'dist_restored':len(tracked),'own_new_dist_removed':len(removed),'old_manifest_rows_preserved':len(olds),'audio_rows':len(audio),'audio_bytes':meta['audio_bytes'],'no_staged_files':True,'disk_free_bytes':meta['disk_free_bytes']}))
