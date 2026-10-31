from pathlib import Path
import json,hashlib,collections,subprocess,os
R=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-efhmbq7e/evidence/phase5');S=Path('/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-c331-phase5-parent-9_4z200c/snapshot');P=Path('/Users/ruyimg/Documents/Claude/sanguosha/noname-next');O=Path('/tmp/noname-parent-c331-phase5-retained');O.mkdir(exist_ok=True)
def load(p):return json.loads(p.read_bytes())
def sha(b):return hashlib.sha256(b).hexdigest()
def identity(p):b=p.read_bytes();return {'path':str(p),'bytes':len(b),'sha256':sha(b)}
def norm(p):
 prefix='/Users/ruyimg/Documents/Claude/sanguosha/'
 return p[len(prefix):] if p.startswith(prefix) else p
A=load(R/'inputs/laneA-ordering.json');B=load(R/'inputs/laneB-ordering-evidence.json');v=load(R/'validated-ordering44.json');current=load(R/'current-bindings44.json');remaining=load(R/'remaining28.json');reg=load(R/'registration-supplement.json')
lanes={row['variant']:('A',row) for row in A['variants']};assert len(lanes)==36
for row in B['rows']:assert row['variant'] not in lanes;lanes[row['variant']]=('B',row)
assert len(lanes)==72
curr={row['variant']:row for row in current['rows']};assert len(curr)==44
rows=[];body_lines=[];caller_records=[];private_callers=[];corrected=[]
for row in v['rows']:
 name=row['variant'];lane,original=lanes[name];assert original['status']=='proven-order';assert row['expected']==curr[name]['tuple'];srcs=[]
 for source in row['sources']:
  matches=[x for x in original['sources'] if norm(x['path'])==norm(source['path']) and x['name']==source['name']];assert len(matches)==1
  original_source=matches[0];assert original_source['body_sha256']==source['body_sha256'];assert sha(original_source['body'].encode())==source['body_sha256']
  srcs.append({'name':source['name'],'path':source['path'],'line':source['line'],'file_sha256':source['sha256'],'body_sha256':source['body_sha256'],'top_level_ordering':source['ordering_properties'],'trigger':source['actual_trigger'],'registration':source.get('registration'),'translation':source.get('translation'),'silent_determines_unselectable':source.get('silent_determines_unselectable'),'lane_parent':original_source.get('parent'),'lane_inheritance':original_source.get('inheritance_audit'),'lane_translation':original_source.get('translation'),'lane_translation_evidence':original_source.get('translation_evidence'),'lane_translation_resolution':original_source.get('translation_resolution')})
  if row['tuple_correction']:
   body_lines.extend([f'## {name} | lane{lane} | {source["name"]} | {source["path"]}:{source["line"]}',f'body SHA256 {source["body_sha256"]}',original_source['body'],''])
 for call in row['lane_caller_reconciliation']:
  assert call['same_file_bytes'] is True and call['private_sha256']==call['lane_sha256'];caller_records.append({'variant':name,**call})
 private_callers.extend({'variant':name,**x} for x in row['private_callers'])
 if row['tuple_correction']:assert row['native_before']['tuple']!=row['expected'];corrected.append((lane,name))
 else:assert row['native_before']['tuple']==row['expected']
 rows.append({'variant':name,'lane':lane,'lane_index':original.get('remaining_index',original.get('index')),'before':row['native_before']['tuple'],'expected':row['expected'],'current':curr[name]['tuple'],'correction':row['tuple_correction'],'sources':srcs,'current_binding':curr[name]['native_binding'],'reconciliations':len(row['lane_caller_reconciliation']),'all_reconciliations_match':True,'functional_gaps':row['functional_gaps'],'whole_source_equivalence':row['whole_source_equivalence']})
assert len(rows)==44 and len(corrected)==17 and collections.Counter(x[0] for x in corrected)=={'A':14,'B':3}
assert len(set(x['variant'] for x in rows))==44 and len(remaining['rows'])==28
assert set(curr).isdisjoint(x['variant'] for x in remaining['rows']) and set(curr)|{x['variant'] for x in remaining['rows']}==set(lanes)
unique={}
for call in caller_records:
 path=call['private_path'];digest=call['private_sha256'];assert path not in unique or unique[path]['sha256']==digest
 unique[path]={'path':path,'sha256':digest}
for path,record in unique.items():
 if (S/path).exists():actual=sha((S/path).read_bytes());origin='parent frozen135 snapshot'
 else:
  b=subprocess.run(['git','--no-optional-locks','-C',str(P),'cat-file','blob','38ab1678df9b4b7b991d3b3122b1d937fd2648a1:'+path],check=True,capture_output=True,env={**os.environ,'GIT_NO_LAZY_FETCH':'1'}).stdout;actual=sha(b);origin='retained immutable base38ab Git blob, absent from135 delta snapshot; not a fresh live-tree check'
 assert actual==record['sha256'],path;record['checked_sha256']=actual;record['identity_source']=origin
# Preserve available retained module excerpts without reading upstream/live files.
module_excerpts=[]
for m in reg['module_installations']:
 matches=[x for x in A['source_registration'] if norm(x['path'])==m['path']];module_excerpts.append({'record':m,'retained_excerpts':matches,'full_excerpt_retained':bool(matches)})
for x in module_excerpts:
 if x['retained_excerpts']:assert x['retained_excerpts'][0]['sha256']==x['record']['sha256']
(O/'corrected17-definition-bodies.raw.txt').write_text('\n'.join(body_lines))
(O/'registration18-and-modules6.json').write_text(json.dumps({'related18':reg['related_definitions_and_links'],'modules6':module_excerpts,'scope':'Retained records/excerpts only; no new upstream reads. Missing complete parent bodies remain identified by original path/bodySHA/group literal.'},ensure_ascii=False,indent=2)+'\n')
# One compact JSON row per line keeps every record readable with bounded pages.
(O/'ordering44-compact.jsonl').write_text(''.join(json.dumps(x,ensure_ascii=False,separators=(',',':'))+'\n' for x in rows))
files=[R/'pre-full-freeze/source-manifest.json',R/'checkpoint20/source-manifest.json',R/'final-freeze5/source-manifest.json'];arrays=[load(p)['files'] for p in files];assert all(x==arrays[0] for x in arrays);assert len(arrays[0])==135
canonical=json.dumps(arrays[0],sort_keys=True,separators=(',',':')).encode();tree_sha=sha(canonical);binding=load(R/'source-and-result-binding.json');assert tree_sha==binding['source_tree_identity_sha256']=='dcedfa22a6cff71adeb4a6107b911e462623bc609e37c7359b7226bdfd0e64f6'
freeze={'result':'PASS','manifests':[identity(p) for p in files],'ordered_files_arrays_equal':True,'path_sha_pairs':135,'canonicalization':'UTF8 json.dumps(files,sort_keys=True,separators=(comma,colon)); all keys ASCII; sort_keys=False gives same bytes because input keys already sorted','canonical_bytes':len(canonical),'source_tree_identity_sha256':tree_sha,'recorded_identity_equal':True,'qualification':'Read-only retained JSON/hash comparison; no producer or live-tree scan'};(O/'pre-checkpoint-post-freeze.json').write_text(json.dumps(freeze,indent=2)+'\n')
summary={'result':'PASS','validated_unique':44,'lane_counts':dict(collections.Counter(x['lane'] for x in rows)),'corrected':17,'corrected_lane_counts':dict(collections.Counter(x[0] for x in corrected)),'matching_without_override':27,'remaining_unique':28,'remaining_status_counts':dict(collections.Counter(x['status'] for x in remaining['rows'])),'remaining':[{'variant':x['variant'],'status':x['status'],'reason':x['reason']} for x in remaining['rows']],'union_disjoint_complete72':True,'lane_reconciliation_records':len(caller_records),'unique_reconciliation_paths':len(unique),'all_lane_reconciliations_match':True,'unique_caller_paths':list(unique.values()),'private_caller_reference_records':len(private_callers),'related_registration_records':len(reg['related_definitions_and_links']),'module_installations':len(reg['module_installations']),'input_identities':[identity(R/n) for n in ['validated-ordering44.json','current-bindings44.json','remaining28.json','registration-supplement.json','inputs/laneA-ordering.json','inputs/laneB-ordering-evidence.json']],'qualification':'Fresh retained-only extraction/sets/hash reconciliation; unchanged caller files missing from delta snapshot checked against retainedbaseGit blobs, not live/upstream. Source properties/registration are evidence claims for reviewer semantic assessment, not newly executed upstream runtime.'}
(O/'summary.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n');print(json.dumps({'output':str(O),'validated44':44,'corrected17':17,'remaining28':28,'lane_reconciliation_records':len(caller_records),'unique_caller_paths':len(unique),'freeze':freeze},indent=2))
