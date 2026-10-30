import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,existsSync} from 'node:fs';
import path from 'node:path';
import {originalResourceUrl} from '/Users/ruyimg/Documents/Claude/sanguosha/noname-next/apps/web-client/src/engine/originalResources.ts';
const root='/Users/ruyimg/Documents/Claude/sanguosha/noname-next';const out=process.argv[2];
const c=JSON.parse(readFileSync(root+'/content/original/catalog.json'));
const entries=[];for(const r of c.characters)if(r.portrait)entries.push({kind:'portrait',id:r.id,asset:r.portrait});
for(const [kind,rows] of Object.entries(c.skins))if(Array.isArray(rows))for(const r of rows)if(r.asset)entries.push({kind,id:r.id,asset:r.asset});
for(const r of entries){r.url=originalResourceUrl('http://127.0.0.1:7700/original-content',r.asset);r.exists=existsSync(path.join(root,'content/original',r.asset));assert.ok(r.url,r.id);}
const yudu=c.characters.find(r=>r.name==='hm_yudu');assert.ok(yudu);assert.equal(yudu.portrait,'assets/portraits/yudu.jpg');
writeFileSync(out+'/catalog-url-audit.json',JSON.stringify({total:entries.length,rejected:entries.filter(r=>!r.url),missing:entries.filter(r=>!r.exists),nullPortraits:c.characters.filter(r=>!r.portrait).map(r=>r.id),skinCounts:Object.fromEntries(Object.entries(c.skins).map(([k,v])=>[k,Array.isArray(v)?v.length:null])),yudu,entries},null,2)+'\n');
console.log(JSON.stringify({total:entries.length,rejected:0,missing:entries.filter(r=>!r.exists).length,nullPortraits:c.characters.filter(r=>!r.portrait).length,hm_yudu:yudu.portrait}));
