import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import ts from '../../../../..//apps/web-client/node_modules/typescript/lib/typescript.js';
import {loadAudioDeclarations,resolveAudioDeclarations,declaredAudioAssets} from '../../../../../scripts/original-audio-resources.mjs';
import {parseJavaScript,propertyName,safeLiteral} from '../../../../../scripts/original-source-ast.mjs';
import {originalCharacterAudio} from '../../../../../apps/web-client/src/engine/originalResources.ts';
const root=process.cwd(),out=path.dirname(new URL(import.meta.url).pathname),core=path.resolve(root,'../archive/upstream/noname/apps/core');
const catalog=JSON.parse(fs.readFileSync('content/original/catalog.json'));const manifest=JSON.parse(fs.readFileSync('content/original/asset-manifest.json'));
const loaded=await loadAudioDeclarations(core);const plan=declaredAudioAssets(resolveAudioDeclarations(loaded,catalog.skins.substitutes),manifest.assets.filter(a=>!a.path.startsWith('audio-declared/')));
assert.deepEqual(plan.assets,manifest.assets.slice(12359));assert.equal(plan.assets.length,23393);
fs.writeFileSync(path.join(out,'all-mappings.json'),JSON.stringify(plan,null,2)+'\n');
const declared=[];
for(const pack of fs.readdirSync(path.join(core,'character'))){
 const file=path.join(core,'character',pack,'index.js');if(!fs.existsSync(file))continue;const sf=await parseJavaScript(file);
 const visit=node=>{
  if(ts.isPropertyAssignment(node)&&propertyName(node)==='characterSubstitute'&&ts.isObjectLiteralExpression(node.initializer)){
   for(const p of node.initializer.properties){if(!ts.isPropertyAssignment(p)||!ts.isArrayLiteralExpression(p.initializer))continue;
    for(const entry of p.initializer.elements){const parsed=safeLiteral(entry);if(!parsed.ok||!Array.isArray(parsed.value)||typeof parsed.value[0]!=='string')continue;
     declared.push({pack,character:propertyName(p),substitute:parsed.value[0],directives:Array.isArray(parsed.value[1])?parsed.value[1]:[],source:`legacy-source/character/${pack}/index.js`,line:sf.getLineAndCharacterOfPosition(entry.getStart(sf)).line+1});
    }
   }
  }
  ts.forEachChild(node,visit);
 };visit(sf);
}
const normalize=rows=>rows.map(({pack,character,substitute,directives,source,line})=>({pack,character,substitute,directives,source,line})).sort((a,b)=>JSON.stringify(a).localeCompare(JSON.stringify(b)));
assert.deepEqual(normalize(declared),normalize(catalog.skins.substitutes));
const owners=Map.groupBy(catalog.characters,c=>c.name);const pool=Map.groupBy(plan.mappings.filter(m=>m.type==='skill'),m=>m.id);const diePool=Map.groupBy(plan.mappings.filter(m=>m.type==='die'),m=>m.id);const available=new Set(manifest.assets.slice(0,12359).map(a=>a.path));const rows=[];
for(const declaration of catalog.skins.substitutes){
 const candidates=owners.get(declaration.character)??[];assert.equal(candidates.length,1);const owner=candidates[0];const source=loaded.characters.get(owner.id);assert.ok(source);let override=[];const death=[];
 for(const d of declaration.directives){if(d.startsWith('tempname:'))override=d.slice(9).split(':');if(d.startsWith('die:'))death.push(d.slice(4));}
 const names=[...(override.length?override:[declaration.substitute]),owner.name,...source.data.tempname??[]];const expected=[];
 for(const skill of owner.resolved_skills){const choices=pool.get(skill)??[];const selected=names.find(n=>choices.some(m=>m.variant===n))??'';expected.push(...choices.filter(m=>m.variant===selected).map(m=>m.source_path));}
 if(!death.length)expected.push(`audio/die/${declaration.substitute}.mp3`);
 for(const name of death){const target=owners.get(name)??[];assert.ok(target.length<=1);if(target.length)expected.push(...(diePool.get(target[0].id)??[]).map(m=>m.source_path));else expected.push(`audio/die/${name}${name.includes('.')?'':'.mp3'}`);}
 const required=[...new Set(expected)].filter(p=>available.has(p)).sort();const prefix=`audio-declared/${owner.pack}/character/${owner.name}/`;const aliases=manifest.assets.filter(a=>a.path.startsWith(prefix)&&a.path.includes(`/form-${declaration.substitute}/`));
 assert.deepEqual([...new Set(aliases.map(a=>a.source_path))].sort(),required,`${owner.id}/${declaration.substitute}`);const voices=originalCharacterAudio(manifest,owner);for(const p of required)assert.ok(voices.some(a=>a.source_path===p));
 rows.push({owner:owner.id,declaration_pack:declaration.pack,substitute:declaration.substitute,source:declaration.source,line:declaration.line,directives:declaration.directives,expected_sources:required,aliases,cross_pack:declaration.pack!==owner.pack});
}
assert.equal(rows.length,79);assert.equal(rows.filter(r=>r.cross_pack).length,8);assert.equal(rows.reduce((n,r)=>n+r.expected_sources.length,0),571);assert.equal(rows.filter(r=>r.cross_pack).reduce((n,r)=>n+r.aliases.length,0),91);
fs.writeFileSync(path.join(out,'all79-owner-audit.json'),JSON.stringify(rows,null,2)+'\n');
const identities=catalog.characters.map(c=>{const selected=originalCharacterAudio(manifest,c);for(const a of selected)assert.ok(a.path.startsWith(`audio-declared/${c.pack}/die/${c.name}/`)||a.path.startsWith(`audio-declared/${c.pack}/character/${c.name}/`));return {id:c.id,selected:selected.map(a=>({path:a.path,source_path:a.source_path}))};});
const liubei=identities.find(c=>c.id==='standard:liubei');assert.ok(liubei);assert.equal(liubei.selected.length,5);assert.ok(!liubei.selected.some(a=>/liushan|re_liubei|sbjijiang/.test(a.source_path)));
fs.writeFileSync(path.join(out,'all-catalog-identities.json'),JSON.stringify(identities,null,2)+'\n');
console.log(JSON.stringify({aliases:plan.assets.length,mappings:plan.mappings.length,missing:plan.missing.length,dynamic:plan.dynamic.length,declarations:rows.length,source_rows_exact:true,cross_pack:8,required_source_mappings:571,cross_pack_aliases:91,catalog_identities:identities.length,liubei_voices:liubei.selected.length}));
