from pathlib import Path
import hashlib,json,re,struct,datetime,os,subprocess
E=Path('/tmp/i02-main-typed-wire-functional-33d21ba5');M=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next')
def j(p):return json.loads(p.read_bytes())
def h(p):
 z=hashlib.sha256()
 with p.open('rb') as f:
  for b in iter(lambda:f.read(1024*1024),b''):z.update(b)
 return z.hexdigest()
manifest=j(E/'manifest.json');art=j(E/'artifact-sha256.json')['files'];sources=j(E/'source36.json');commands=j(E/'commands.json');product=j(E/'product-binding.json');entry=j(E/'gate-entry.json');exit=j(E/'gate-exit.json');seal=j(E/'seal-exit.json');before=j(E/'before.json')
assert len(art)==1054 and len(commands)==37 and len(sources)==36
for rel,r in art.items():assert (E/rel).stat().st_size==r['bytes'] and h(E/rel)==r['sha256'],rel
assert h(Path(manifest['reportPath']))==manifest['reportSha256']
assert h(E/'integrated-I02-source30.patch')==manifest['patchSha256']=='1b154aa7573fff3fc5b2fdfb54e5dbd743206c4c2b91f1e384050f38b5cdae94'
for p,sha in sources.items():assert h(M/p)==h(E/'materialized-source'/p)==sha,p
parsed=[]
for c in commands:
 name=c['name'];assert c['exit']==0
 assert c['bindingBefore']==c['bindingAfter']==manifest['binding'];assert c['bindingBefore']['source36']==sources
 assert h(E/(name+'.stdout.log'))==c['stdoutSha256'] and h(E/(name+'.stderr.log'))==c['stderrSha256']
 out=(E/(name+'.stdout.log')).read_text(errors='replace');rows=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',out);counts=[sum(int(r[i]) for r in rows) for i in range(3)] if rows else None
 assert counts==c['rustCounts'],(name,counts,c['rustCounts']);assert c['environment']['CARGO_INCREMENTAL']=='0' and c['environment']['CARGO_BUILD_JOBS']=='2';assert c['freeDiskBytesAfter']>=3*1024**3
 parsed.append({'name':name,'exit':0,'rust_counts':counts,'node_pass':c['nodePass']})
assert next(r for r in parsed if r['name']=='workspace')['rust_counts']==[6929,0,77];assert next(r for r in parsed if r['name']=='native3')['rust_counts']==[1719,0,75];assert next(r for r in parsed if r['name']=='api-protocol')['rust_counts']==[12,0,0];assert next(r for r in parsed if r['name']=='frontend-tests')['node_pass']=='441'
strict=next(c for c in commands if c['name']=='gate1b');assert strict['environment']['RUSTFLAGS']=='-Dwarnings' and strict['environment']['CARGO_TARGET_DIR']!=next(c for c in commands if c['name']=='workspace')['environment']['CARGO_TARGET_DIR']
preflight=(E/'preflight.stdout.log').read_text();assert len(re.findall(r'^✅ ',preflight,re.M))==21 and 'preflight 全部通过 ✅' in preflight
fields=j(E/'gate-comparison.json')['fields'];assert len(fields)==12 and all(fields.values())
for key in fields:assert entry[key]==exit[key]==seal[key],key
assert len(entry['maintainedAndMarkdown'])==11337 and entry['trackedPathsCount']==47913
# Verify frozen raw index directly, independently of the miscaptured producer parser scripts.
def parse_index(path):
 data=path.read_bytes();assert data[:4]==b'DIRC';version,count=struct.unpack('!II',data[4:12]);assert version==2 and count==47913;assert hashlib.sha1(data[:-20]).digest()==data[-20:];offset=12;rows=[];masked=bytearray(data[:-20])
 for _ in range(count):
  start=offset;stat=struct.unpack('!10I',data[start:start+40]);oid=data[start+40:start+60];flags=struct.unpack('!H',data[start+60:start+62])[0];assert not flags&0x4000;end=data.index(b'\0',start+62);name=data[start+62:end].decode();offset=start+((end+1-start+7)//8)*8;body=data[start:offset];rows.append({'name':name,'stat':stat,'oid':oid,'flags':flags,'bytes':body,'offset':start});masked[start:start+8]=b'\0'*8
 return data,rows,data[offset:-20],bytes(masked)
b0,r0,x0,m0=parse_index(E/'before-index.raw');bg,rg,xg,mg=parse_index(E/'gate-entry-index.raw');be,re_,xe,me=parse_index(E/'gate-exit-index.raw');bs,rs,xs,ms=parse_index(E/'seal-exit-index.raw');assert bg==be==bs and b0!=bg and x0==xg and m0==mg
changed=[]
for a,b in zip(r0,rg):
 assert a['name']==b['name'] and a['oid']==b['oid'] and a['flags']==b['flags'] and a['bytes'][8:]==b['bytes'][8:]
 assert (a['flags']>>12)&3==0
 if a['stat'][:2]!=b['stat'][:2]:changed.append(a['name'])
worker_diff=j(E/'index-raw-field-diff.json');assert len(changed)==33 and set(changed)=={r['pathBefore'] for r in worker_diff['changedEntries']};assert worker_diff['changedFields']==['ctimeNanos','ctimeSeconds'] and not worker_diff['behaviorSignificantEntryChanges']
index_logical={};head_logical={}
for record in entry['indexEntries'].split('\0'):
 if not record:continue
 header,path=record.split('\t',1);mode,oid,stage=header.split();assert stage=='0';index_logical[path]=(mode,oid)
for record in entry['headEntries'].split('\0'):
 if not record:continue
 header,path=record.split('\t',1);mode,kind,oid=header.split();assert kind=='blob';head_logical[path]=(mode,oid)
assert index_logical==head_logical and len(index_logical)==47913 and not entry['staged']
# Main's later single workflow change is a separately reviewed post-freeze input.
ci=j(Path('/tmp/noname-parent-windows-checkout-main-integration.json'))
for group,rows in exit['protectedSets'].items():
 for p,sha in rows.items():assert h(M/p)==(ci['after'] if p=='.github/workflows/ci.yml' else sha),(group,p)
assert len(exit['protectedSets']['modules'])==18 and len(exit['protectedSets']['dist'])==37
changes=[p for p,sha in before['protectedSets']['modules'].items() if sha!=exit['protectedSets']['modules'][p]];assert changes==manifest['generated18Changes'] and len(changes)==3
assert h(M/'apps/web-client/src/worker/native-pkg/engine_native_wasm_bg.wasm')==product['currentNativeWasmSha256']=='7c2a9f266c88e73e720a7b4e4073d28100eb2b6b3a18f087f721a57d881460da'
for p in ['apps/web-client/public/pkg/engine_wasm_bg.wasm','apps/web-client/src/wasm-pkg/engine_wasm_bg.wasm']:assert h(M/p)==product['currentLegacyWasmSha256']=='846351a9b844989c04d868dfb57b829abe5320ef452cf3da49494dd611e4cab5'
for b in product['binaries']:assert h(Path(b['preservedPath']))==b['sha256'] and Path(b['preservedPath']).stat().st_size==b['bytes']
load=j(E/'actual-main-js-load-binding.json');assert not load['defaultDist37Served'] and len(load['currentMainModules'])==5
for p,r in load['currentMainModules'].items():assert r['sourceSha256']==sources[p]
for runtime,r in load['actualLoadedEntryChunks'].items():
 assert len(r['actualEntryRequests'])>=1 and all(x['sha256']==load['entrySha256'] for x in r['actualEntryRequests']);assert h(Path(r['root'])/load['entry'])==load['entrySha256']
for name,count,positive in [('chrome',162,163),('tauri',161,162)]:
 r=j(E/('ingress-'+name+'-report.json'));assert r['passed'] and r['fixtureInjection'] and r['notRustProducer'] and r['sevenEventPositive'] and r['validAstralPositive'] and r['invalidCorpus']==80 and len(r['rows'])==count and r['positiveAbsent']==positive and r['networkBound8Replay'] and r['normalAbsenceSuccess']
 for row in r['rows']:
  for key in ['uncertain','pendingIdentityRetained','perfUnacknowledged','storeUnchanged','disposeClean']:assert row[key],(name,row['name'],key)
wire=j(E/'server-wire-results.json')
for key in ['passed','realOutcomePresentationAbsent','oldAndNewReaderSuccess','successfulFormalResult','actualReconnectSameKey','receiptResultEqual','noSecondExecutionRevision','noSecondExecutionHash','pendingAndStorageCleared']:assert wire[key],key
seven=j(E/'native-wasm-seven-binding.json');assert len(seven['cases'])==7 and seven['MainLockAlignedExternalCount']==30 and seven['fullReplyHashRevisionEventsEqual']
for r in seven['cases']:assert (E/('native-'+r['name']+'.ngsa')).read_bytes()==(E/('wasm-'+r['name']+'.ngsa')).read_bytes()
for stem,count in [('trajectory',40),('trajectory-package',20)]:
 a=j(E/(stem+'-native.json'));b=j(E/(stem+'-wasm.json'));assert len(a['direct'])==count
 for key in ['direct','restore']:assert a[key]==b[key]
sbefore=j(E/'tauri-schemas-before.json')['files'];safter=j(E/'tauri-schemas-after.json');assert len(sbefore)==4 and not safter['byteChanged'] and not safter['mtimeChanged'] and not safter['extraPaths'] and safter['allConfigPermissionSourcesUnchanged']
for p,r in sbefore.items():assert h(M/p)==r['sha256']==safter['after'][p]['sha256'] and (M/p).stat().st_mtime_ns==r['mtimeNs']==safter['after'][p]['mtimeNs']
v={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'PASS_BOUNDED_MAIN_PHASE1_PENDING_EVIDENCE_SUPPLEMENT','fresh_artifacts':len(art),'commands':parsed,'source36_current_and_materialized_exact':True,'gate_comparison_categories_actual':12,'gate_window':[entry['utc'],exit['utc']],'seal_exit':seal['utc'],'initial_index_to_gate_changed_entries':len(changed),'independent_index_ctime_only':True,'index_writer':'UNKNOWN, field parsing does not attribute cause','all_index_other_bytes_flags_oid_modes_extensions_exact':True,'native':product['currentNativeWasmSha256'],'legacy':product['currentLegacyWasmSha256'],'actual_IPC_browser_ingress_rows':[161,162],'full_native_WASM_cases':7,'trajectories':[40,20],'four_generated_schema_bytes_mtime_exact':True,'post_freeze_change':'CIworkflow354f->b46separatelyreviewed, notoldpreflightinput','P2':'Two sealed purported original parser command files were miscaptured extractor; additive true-command supplement pending. Originalreport13 gatecategories shouldbe12 recorded fields; raw values all equal. No product rerun required.','qualification':'Only DTO/strictconsumer/omission compatibility, nineconstructorsNone; no producer/committedspan/qualifiedassetfactory/audio/SDK37/physical/Core331acceptance. Older raw index147->f4ctimeonlyfieldproof and firstsealexit1 retained.'};Path('/tmp/noname-parent-i02-main-final-audit.json').write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:v for k,v in v.items() if k!='commands'},ensure_ascii=False,indent=2))
