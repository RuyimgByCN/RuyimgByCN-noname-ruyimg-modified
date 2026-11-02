import assert from "node:assert/strict";
import { test } from "node:test";
import { choiceMetadata,choicePromptLabel } from "../src/engine/observerMetadata.ts";
import { DesktopEngineTransport } from "../src/engine/desktopEngineTransport.ts";
const choice={choice_id:1,observer:1,prompt:"trigger_confirm",native_kind:"confirm",source_skill:"遗计",options:[{token:0,label_key:"confirm"}],deadline_unix_ms:12345};
test("source metadata is optional but strict when present",()=>{
 assert.equal(choiceMetadata(choice).source_skill,"遗计");
 for(const source_skill of [null,undefined]) assert.equal(choiceMetadata({source_skill}).source_skill,undefined);
 for(const source_skill of ["",1,{},[],true]) assert.throws(()=>choiceMetadata({source_skill}),/source_skill/);
 assert.equal(choicePromptLabel(choice),"是否发动【遗计】？");
 assert.equal(choicePromptLabel({...choice,source_skill:null}),"是否发动技能？");
 const text="<img src=x onerror=alert(1)>";
 assert.equal(choicePromptLabel({...choice,source_skill:text}),`是否发动【${text}】？`);
});
test("desktop normal and chooseGeneral envelopes preserve source and deadline",async()=>{
 for(const type of ["observer","chooseGeneral"]){
  const envelope={view:{seats:[]},choice:{...choice},events:[],legal_actions:[],result:null};
  const transport=new DesktopEngineTransport(async(cmd,args)=>cmd==="native_engine_open"?27:cmd==="native_engine_close"?null:{id:args.request.id,ok:true,data:type==="chooseGeneral"?{envelope}:envelope});
  const reply=new Promise(resolve=>{transport.onmessage=e=>resolve(e.data);});
  transport.postMessage({id:1,type});
  const value=await reply;const result=type==="chooseGeneral"?value.data.envelope:value.data;
  assert.equal(result.choice.source_skill,"遗计");assert.equal(result.choice.deadline_unix_ms,12345);transport.terminate();
 }
});


test("frozen WS6 TypeScript reader ignores source while current reader preserves it",async()=>{
 const old=await import("./ws6-observer-metadata-before-source.ts");
 assert.equal(old.choiceMetadata(choice).source_skill,undefined);
 assert.equal(old.choicePromptLabel(choice),"是否发动技能？");
 assert.equal(choiceMetadata(choice).source_skill,"遗计");
});

test("C order mapping and strict source forwarding coexist",()=>{
 const result=choiceMetadata({native_kind:"pick_tag",prompt:"trigger_order",source_skill:"遗计"});
 assert.equal(result.kind,"OrderTriggers");assert.equal(result.source_skill,"遗计");
 assert.equal(choiceMetadata({native_kind:"pick_tag",prompt:"other"}).kind,"Custom");
 assert.throws(()=>choiceMetadata({native_kind:"pick_tag",prompt:"trigger_order",source_skill:""}),/source_skill/);
});
