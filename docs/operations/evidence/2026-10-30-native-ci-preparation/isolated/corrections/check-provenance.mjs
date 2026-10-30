import assert from 'node:assert/strict';
import {readFileSync, mkdtempSync, mkdirSync, writeFileSync, rmSync, realpathSync, existsSync} from 'node:fs';
import path from 'node:path';
import {realpath} from 'node:fs/promises';
import {spawn, execFileSync} from 'node:child_process';
const fixture=mkdtempSync(path.resolve('../evidence/corrections/provenance-'));
const env={...process.env,GIT_CONFIG_NOSYSTEM:'1',GIT_CONFIG_GLOBAL:'/dev/null',GIT_CEILING_DIRECTORIES:fixture};
for(const key of ['GIT_DIR','GIT_WORK_TREE','GIT_INDEX_FILE','GIT_OBJECT_DIRECTORY','GIT_ALTERNATE_OBJECT_DIRECTORIES']) delete env[key];
try {
  const source=path.join(fixture,'archive/upstream/noname');
  const coreSourceRoot=path.join(source,'apps/core');
  const repoRoot=path.join(fixture,'noname-next');
  mkdirSync(coreSourceRoot,{recursive:true}); mkdirSync(repoRoot);
  writeFileSync(path.join(coreSourceRoot,'package.json'),'{"version":"fixture"}\n');
  writeFileSync(path.join(source,'README.md'),'fixture\n');
  const git=(...args)=>execFileSync('git',['-C',source,...args],{encoding:'utf8',env}).trim();
  git('init','-q');
  assert(existsSync(path.join(source,'.git')));
  assert.equal(git('rev-parse','--show-toplevel'),realpathSync(source));
  git('config','user.email','fixture@example.invalid');git('config','user.name','Isolated CI fixture');
  git('add','.');git('commit','-qm','isolated source fixture');
  const head=git('rev-parse','HEAD');
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
  const initial=await check();
  const adopted={category:'core',root:'archive/upstream/noname/apps/core',repository:'https://github.com/libnoname/noname.git',repository_root:'archive/upstream/noname',head,dirty:false,tracked_changes:[],tracked_change_count:0,untracked_count:0,redistribution_status:'preserved_from_source_requires_review'};
  assert.throws(()=>initial.assertJsonEqual('resource source provenance',adopted,initial.value),/resource source provenance mismatch/);
  const action=readFileSync('.github/actions/prepare-original-content/action.yml','utf8');
  const command=action.match(/run: (git remote set-url origin [^\n]+)/)[1];
  execFileSync('bash',['-e','-c',command],{cwd:source,env});
  const corrected=await check();corrected.assertJsonEqual('resource source provenance',adopted,corrected.value);
  assert.equal(git('rev-parse','HEAD'),head);assert.equal(git('status','--porcelain'),'');
  console.log(JSON.stringify({passed:true,red:'resource source provenance mismatch',green:corrected.value,actual_action_command:command,head_unchanged:true,clean_after:true,scope:'own Git fixture only; its commit is test data, not adopted upstream HEAD',cleanup:'fixture removed in finally'},null,2));
} finally {rmSync(fixture,{recursive:true,force:true});}
