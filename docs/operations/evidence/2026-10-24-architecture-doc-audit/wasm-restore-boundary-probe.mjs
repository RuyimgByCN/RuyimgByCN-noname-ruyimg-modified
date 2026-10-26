import { readFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
import { createHash } from 'node:crypto';
const root = '/Users/ruyimg/Documents/Claude/sanguosha/noname-next';
const pkg = `${root}/apps/web-client/src/worker/native-pkg`;
const {default:init, NativeEngine} = await import(pathToFileURL(`${pkg}/engine_native_wasm.js`).href);
await init({module_or_path:readFileSync(`${pkg}/engine_native_wasm_bg.wasm`)});
const which = process.argv[2];
const engine = new NativeEngine(4, 42n);
const before = engine.save_hex();
const beforeViews = Array.from({length:4}, (_, seat) => engine.view_json(seat));
const input = which === 'valid_roundtrip' ? before : ({odd_length:'0',non_ascii:'中',invalid_ascii:'gg'})[which];
if (input === undefined) throw new Error(`unknown case ${which}`);
const record = {case:which,input:which==='valid_roundtrip'?'existing valid archive':input,archive_sha256:createHash('sha256').update(before).digest('hex')};
try {engine.restore_hex(input);record.restore={accepted:true};}
catch (error) {record.restore={accepted:false,type:error?.name??typeof error,message:String(error)};}
try {
  const after=engine.save_hex();
  const a=Buffer.from(before,'hex'), b=Buffer.from(after,'hex');
  const differences=[];
  for(let i=0;i<Math.max(a.length,b.length);i++) if(a[i]!==b[i]) differences.push(i);
  record.after={readable:true,archive_unchanged:after===before,revision:engine.revision().toString(),
    all_seat_views_unchanged:beforeViews.every((v,seat)=>v===engine.view_json(seat)),
    authority_hash:'not exported by NativeEngine', before_bytes:a.length,after_bytes:b.length,
    first_differing_byte:differences[0]??null,
    payload_differing_bytes:differences.filter(i=>i<Math.min(a.length,b.length)-32),
    checksum_differing_bytes:differences.filter(i=>i>=Math.min(a.length,b.length)-32),
    first_difference_values:differences.length?{before:a[differences[0]],after:b[differences[0]]}:null};
}
catch (error) {record.after={readable:false,type:error?.name??typeof error,message:String(error)};}
console.log(JSON.stringify(record));
