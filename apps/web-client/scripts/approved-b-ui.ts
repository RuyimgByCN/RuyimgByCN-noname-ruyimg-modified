// Actual BattleView + SeatCard with real Worker/IPC archive restoration; no engine substitute.
import {createApp, nextTick} from "vue";
import {createRouter, createMemoryHistory} from "vue-router";
import BattleView from "../src/views/BattleView.vue";
import SeatCard from "../src/components/battle/SeatCard.vue";
import {isTauriShell} from "../src/engine/desktopHost";
import {EngineClient} from "../src/engine/client";
import {choicePromptLabel, choiceKindLabel} from "../src/engine/observerMetadata";
import type {EngineEnvelope, SeatView} from "../src/types/engine";

type Fixture = {name:string;archiveHex:string;packsJson:string;seats:Array<{view:EngineEnvelope["view"];choice:EngineEnvelope["choice"]}>; legacy?:boolean; extensionPrompt?:string};
function check(value:unknown, message:string):asserts value {if(!value)throw new Error(message);}
const originalStart=EngineClient.prototype.start;
let selected:Fixture;
let ready:()=>void;
let actual:EngineEnvelope;
let observer=1;
const clients:EngineClient[]=[];
// Fixture injection uses the production start and restore requests and the real selected transport.
EngineClient.prototype.start=async function(){
 clients.push(this);
 await originalStart.call(this,selected.seats.length,0,observer,selected.packsJson);
 actual=await this.restoreHex(selected.archiveHex,selected.packsJson);
 if(selected.legacy){
  actual={...actual,view:{...actual.view,seats:actual.view.seats.map(({generals,...seat})=>seat)},choice:actual.choice?((({prompt,native_kind,...choice})=>choice)(actual.choice)):null};
 }
 if(selected.extensionPrompt&&actual.choice)actual={...actual,choice:{...actual.choice,prompt:selected.extensionPrompt}};
 ready();
 return actual;
};
const wait=async (test:()=>boolean)=>{for(let i=0;i<300;i++){if(test())return;await new Promise(resolve=>setTimeout(resolve,20));}throw new Error("UI did not render expected state");};

async function run(){
 history.replaceState(null,"",isTauriShell()?"/?engineTransport=native":"/?engineTransport=wasm");
 localStorage.setItem("noname-next-skip-anims","1");
 const fixtures:Fixture[]=await fetch("/metadata-fixtures.json").then(response=>response.json());
 const response=fixtures.find(fixture=>fixture.name==="respond_card")!;
 fixtures.push({...response,name:"legacy-choice-flow",legacy:true},{...response,name:"extension-prompt-flow",extensionPrompt:"<img src=x onerror=alert(1)>constructor"});
 const kinds=new Set<string>();const cases=[];
 for(const fixture of fixtures){
  selected=fixture;
  observer=fixture.seats.findIndex(seat=>seat.choice!==null);
  if(observer<0)observer=1;
  const mounted=new Promise<void>(resolve=>{ready=resolve;});
  const root=document.createElement("div");document.body.append(root);
  const router=createRouter({history:createMemoryHistory(),routes:[{path:"/",component:BattleView}]});
  await router.push("/");await router.isReady();
  const app=createApp(BattleView);app.use(router);app.mount(root);
  try{
   await mounted;
   const expected=actual.view.seats.flatMap(seat=>seat.generals??[null,null]);
   await wait(()=>root.querySelectorAll(".general-slot").length===expected.length);
   const slots=[...root.querySelectorAll(".general-slot")];
   for(const [index,general] of expected.entries()){
    const text=slots[index].textContent??"";
    if(general){check(text.includes(general.name),`${fixture.name}: visible general missing`);check(text.includes(general.revealed?"明置":"暗置"),`${fixture.name}: reveal flag missing`);}
    else check(text.includes(actual.view.seats[Math.floor(index/2)].generals===undefined&&index%2===0?"#":"暗将／未配置"),`${fixture.name}: hidden slot was disclosed`);
   }
   if(actual.choice){
    const choice=actual.choice;if(choice.native_kind)kinds.add(choice.native_kind);
    await wait(()=>Boolean(root.querySelector(".choice h2")?.textContent?.includes(choicePromptLabel(choice))));
    check(root.querySelector(".choice")?.textContent?.includes(choiceKindLabel(choice)),`${fixture.name}: readable kind missing`);
    if(choice.prompt)check(!root.querySelector(".choice")?.textContent?.includes(choice.prompt),`${fixture.name}: internal prompt displayed`);
    check(root.querySelector(".choice img[onerror]")===null,`${fixture.name}: extension prompt executed as markup`);
    if(observer===1 && ["confirm","select_players","select_cards","respond_card","pick_tag-fanjian"].includes(fixture.name)){
     const button=root.querySelector<HTMLButtonElement>(".choice button");check(button&&!button.disabled,`${fixture.name}: no real answer control`);
     button.click();
     await wait(()=>!root.querySelector(".choice h2")?.textContent?.includes(choicePromptLabel(choice)) || root.textContent?.includes("rev "+(actual.revision+1))===true);
     check(!root.textContent?.includes("WrongPlayer"),`${fixture.name}: UI answered as wrong seat`);
    }
   }
   const client=clients.at(-1)!;
   const hex=await client.saveHex();const restored=await client.restoreHex(hex,fixture.packsJson);
   check(JSON.stringify(restored.view.seats.map(seat=>seat.generals))===JSON.stringify((await client.observer()).seats.map(seat=>seat.generals)),`${fixture.name}: restored metadata differs`);
   cases.push({name:fixture.name,slots:slots.length,prompt:actual.choice?.prompt??null,transport:client.getTransportKind(),restored:true});
  }finally{app.unmount();root.remove();}
 }
 check(kinds.size===5,"not all five real choice kinds reached BattleView");
 // Old network/service payload remains renderable, and extension text never becomes markup/paths.
 for(const [name,seat] of [["old-payload",{character_def:17}],["unknown-extension",{character_def:0,generals:[{def:0,name:"<img src=x onerror=alert(1)>",faction:"unknown",revealed:false},null]}]] as const){
  const root=document.createElement("div");document.body.append(root);
  const props={seat:{player:0,seat:0,hp:4,max_hp:4,alive:true,turned_over:false,hand_count:0,identity:null,skills:[],...seat} as SeatView,self:true,identityLabel:"未知",isTargetMode:false,isSelectableTarget:false,isDropTarget:false,equipCards:[]};
  const app=createApp(SeatCard,props);app.mount(root);await nextTick();
  check(root.textContent?.includes(name==="old-payload"?"#17":"<img src=x onerror=alert(1)>"),`${name}: fallback missing`);
  check(root.querySelector("img[onerror]")===null,`${name}: untrusted markup`);
  check([...root.querySelectorAll("img")].every(image=>!image.src.includes("onerror")),`${name}: untrusted path`);
  app.unmount();root.remove();
 }
 return {passed:true,cases,kinds:[...kinds].sort(),oldPayloadFallback:true,extensionEscaped:true};
}
try{const result=await run();await fetch("/report",{method:"POST",body:JSON.stringify(result)});document.body.textContent=JSON.stringify(result);}
catch(error){const result={passed:false,error:String(error),stack:error instanceof Error?error.stack:null};await fetch("/report",{method:"POST",body:JSON.stringify(result)});document.body.textContent=JSON.stringify(result);}
finally{EngineClient.prototype.start=originalStart;for(const client of clients)client.dispose();}
