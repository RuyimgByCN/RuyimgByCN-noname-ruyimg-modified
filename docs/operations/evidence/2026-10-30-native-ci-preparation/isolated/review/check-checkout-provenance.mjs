import assert from 'node:assert/strict';
import {readFileSync, mkdtempSync, mkdirSync, writeFileSync, rmSync} from 'node:fs';
import path from 'node:path';
import {realpath} from 'node:fs/promises';
import {spawn, execFileSync} from 'node:child_process';
const fixture=mkdtempSync(path.resolve('../evidence/review/checkout-provenance-'));
try {
  const source=path.join(fixture,'archive/upstream/noname');
  const coreSourceRoot=path.join(source,'apps/core');
  const repoRoot=path.join(fixture,'noname-next');
  mkdirSync(coreSourceRoot,{recursive:true}); mkdirSync(repoRoot);
  writeFileSync(path.join(coreSourceRoot,'package.json'),'{"version":"fixture"}\n');
  writeFileSync(path.join(source,'README.md'),'fixture\n');
  const git=(...args)=>execFileSync('git',['-C',source,...args],{encoding:'utf8',env:{...process.env,GIT_CONFIG_NOSYSTEM:'1',GIT_CONFIG_GLOBAL:'/dev/null'}}).trim();
  git('init','-q'); git('config','user.email','review@example.invalid');git('config','user.name','Isolated review fixture');
  git('add','.');git('commit','-qm','isolated provenance fixture');
  git('remote','add','origin','https://github.com/libnoname/noname');
  git('sparse-checkout','set','apps/core');
  const text=readFileSync('scripts/sync-original-content.mjs','utf8');
  const extract=(start,end)=>text.slice(text.indexOf(start),text.indexOf(end,text.indexOf(start)));
  const definitions=[
    extract('async function sourceProvenance(','function sanitizeRepository('),
    extract('function sanitizeRepository(','function parseGitStatus('),
    extract('function parseGitStatus(','async function syncTree('),
    extract('async function runCapture(','function sourceLabel('),
    extract('function sourceLabel(','function printResults('),
    extract('function assertJsonEqual(','async function run('),
  ].join('\n');
  const check=eval(`(async()=>{${definitions};return {value:await sourceProvenance('core',coreSourceRoot),assertJsonEqual};})`);
  const {value,assertJsonEqual}=await check();
  assert.equal(value.repository,'https://github.com/libnoname/noname');
  assert.equal(value.root,'archive/upstream/noname/apps/core');
  assert.equal(value.repository_root,'archive/upstream/noname');assert.equal(value.dirty,false);
  const adopted={...value,repository:'https://github.com/libnoname/noname.git'};
  assert.throws(()=>assertJsonEqual('resource source provenance',adopted,value),/resource source provenance mismatch/);
  git('remote','set-url','origin',adopted.repository);
  const corrected=await check(); corrected.assertJsonEqual('resource source provenance',adopted,corrected.value);
  console.log(JSON.stringify({passed:true,functions:'unmodified function bodies extracted from actual sync-original-content.mjs',sparse_checkout:'apps/core',fixture_provenance:value,rejection:'resource source provenance mismatch',minimal_origin_correction:'passed',note:'No upstream objects, pnpm, build, assets or platform execution; fixture head is its own real commit'},null,2));
} finally {rmSync(fixture,{recursive:true,force:true});}
