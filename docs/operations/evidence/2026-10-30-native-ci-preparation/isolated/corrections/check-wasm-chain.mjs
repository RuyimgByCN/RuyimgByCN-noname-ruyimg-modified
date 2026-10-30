import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import vm from 'node:vm';
import {execFileSync as realExec} from 'node:child_process';
const workflow=JSON.parse(realExec('python3',['-c',"import json,yaml;print(json.dumps(yaml.safe_load(open('.github/workflows/ci.yml'))))"],{encoding:'utf8'}));
const gates=workflow.jobs.gates.steps;
const install=gates.findIndex(s=>s.name==='Install wasm-pack');
const cross=gates.findIndex(s=>s.name?.startsWith('Gate 3'));
assert(install>=0&&install<cross);
const command=gates[install].run;
assert.equal(command,'cargo install wasm-pack --version 0.15.0 --locked');
assert.equal(command,workflow.jobs['native-web'].steps.find(s=>s.name==='Install wasm-pack').run);
const chain=readFileSync('scripts/cross-target-hash.sh','utf8');
assert(chain.includes('pnpm --dir apps/web-client build:wasm'));
const lock=readFileSync('Cargo.lock','utf8');
assert.match(lock,/name = "wasm-bindgen"\nversion = "0\.2\.126"/);
const source=readFileSync('apps/web-client/scripts/build-wasm.mjs','utf8');
const executable=source.replace(/^import .*;\n/gm,'').replace('import.meta.url',JSON.stringify(new URL('apps/web-client/scripts/build-wasm.mjs',`file://${process.cwd()}/`).href));
function run(installed){
  const calls=[];
  vm.runInNewContext(executable,{
    execFileSync:(cmd,args)=>{calls.push({cmd,args});return Buffer.from('');},
    spawnSync:(cmd,args)=>{calls.push({probe:cmd,args});return {status:cmd==='wasm-pack'&&installed?0:1};},
    existsSync:()=>true,mkdirSync:()=>{},cpSync:()=>{},rmSync:()=>{},homedir:()=>'/fixture/home',
    delimiter:path.delimiter,dirname:path.dirname,join:path.join,fileURLToPath,
    console:{log:()=>{},warn:()=>{},error:()=>{}},process:{env:{PATH:'/fixture/bin'},exit:code=>{throw new Error(`unexpected exit ${code}`);}}
  });
  return calls;
}
const red=run(false),green=run(true);
assert(red.some(c=>c.cmd==='cargo'&&c.args.join(' ')==='install wasm-bindgen-cli --version 0.2.100 --locked'));
assert(green.some(c=>c.cmd==='wasm-pack'&&c.args[0]==='build'));
assert(!green.some(c=>c.cmd==='cargo'||c.cmd==='wasm-bindgen'));
const result={passed:true,actual_build_script_executed_with_stubbed_process_and_fs:true,red_missing_pack_selects_incompatible_cli:'0.2.100',lock_wasm_bindgen:'0.2.126',gate_installs_existing_version:command,ordered_before_cross_target_gate:true,green_prefers_wasm_pack:true,red_calls:red,green_calls:green,note:'No Cargo/Rust/WASM compilation or actual tool installation; this proves command routing only'};
writeFileSync('../evidence/corrections/wasm-chain.json',JSON.stringify(result,null,2)+'\n');
console.log('WASM chain: missing wasm-pack red -> configured installed-pack routing green (all tool/fs effects stubbed)');
