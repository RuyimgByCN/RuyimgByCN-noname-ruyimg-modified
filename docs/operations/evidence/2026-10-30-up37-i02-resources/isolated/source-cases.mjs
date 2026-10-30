import fs from 'node:fs';
import path from 'node:path';
import ts from '../worktree/apps/web-client/node_modules/typescript/lib/typescript.js';
const repo=path.resolve(process.argv[2]);
const core=path.resolve(process.argv[3]);
const catalog=JSON.parse(fs.readFileSync(path.join(repo,'content/original/catalog.json')));
const manifest=JSON.parse(fs.readFileSync(path.join(repo,'content/original/asset-manifest.json')));
const names=['ol_jie_zhuran','ol_zhuhuan','tangtang','tw_dm_zhangliang','ol_sunhanhua','ol_chitu','ol_le_menghuo','ol_xushao','star_dongyun','dc_sb_zhouyu','huan_zhouyu','ol_sb_zhouyu','hm_yudu','dc_zhugejun','tw_zhugejun','wufan'];
const properties=(file,name)=>{
 const text=fs.readFileSync(file,'utf8');const ast=ts.createSourceFile(file,text,ts.ScriptTarget.Latest,true,ts.ScriptKind.JS);let found;
 const visit=node=>{if(ts.isPropertyAssignment(node)&&node.name.getText(ast).replace(/^['"]|['"]$/g,'')===name&&ts.isObjectLiteralExpression(node.initializer))found=node.initializer;else ts.forEachChild(node,visit);};visit(ast);
 return found?Object.fromEntries(found.properties.filter(p=>ts.isPropertyAssignment(p)&&['audio','audioname','audioname2','inherit','img','dieAudios'].includes(p.name.getText(ast))).map(p=>[p.name.getText(ast),p.initializer.getText(ast)])):{};
};
const cases=[];
for(const name of names){const c=catalog.characters.find(c=>c.name===name);if(!c)continue;
 const direct=manifest.assets.filter(a=>a.kind==='audio'&&(a.path===`audio/die/${c.name}.mp3`||c.skills.some(s=>a.path.startsWith(`audio/skill/${s}`)&&/^(?:[1-9][0-9]*)?\.mp3$/.test(a.path.slice(`audio/skill/${s}`.length)))));
 cases.push({id:c.id,def_id:c.def_id,status:c.migration_status,portrait:c.portrait,character_metadata:properties(path.join(core,'character',c.pack,'character.js'),name),skills:c.resolved_skills.map(id=>{const s=catalog.skills.find(s=>s.id===id);return {id,source:s.source,metadata:properties(path.join(core,s.source.replace('legacy-source/character/','')),s.name)};}),direct_audio:direct});
}
console.log(JSON.stringify({scope:'Available direct audio preview; metadata aliases are evidence only, not implemented bindings',source:manifest.sources,cases,skin_counts:catalog.counts,remaining_missing_portraits:catalog.missing_portraits},null,2));
