// Actual current WASM Worker → EngineClient → BattleView in a real webview.
import {createApp} from "vue";
import {createRouter,createMemoryHistory} from "vue-router";
import BattleView from "../src/views/BattleView.vue";
import {EngineClient} from "../src/engine/client";
import {isTauriShell} from "../src/engine/desktopHost";
import type {EngineEnvelope} from "../src/types/engine";
const check=(value:unknown,message:string)=>{if(!value)throw new Error(message);};
const wait=async(test:()=>boolean)=>{for(let i=0;i<300;i++){if(test())return;await new Promise(resolve=>setTimeout(resolve,20));}throw new Error("actual UI timed out");};
const original=EngineClient.prototype.start;
let actual:EngineEnvelope;let client:EngineClient;let ready:()=>void;
let mode="native";let hex="";
EngineClient.prototype.start=async function(){
 client=this;await original.call(this,3,2306,1,"[]");actual=await this.restoreHex(hex,"[]");
 check(actual.choice?.source_skill==="遗计","real Worker lost source_skill");
 if(mode==="missing"&&actual.choice){actual={...actual,choice:{...actual.choice,source_skill:undefined}};}
 if(mode==="null"&&actual.choice){actual={...actual,choice:{...actual.choice,source_skill:null}};}
 if(mode==="escaped"&&actual.choice){actual={...actual,choice:{...actual.choice,source_skill:"<img src=x onerror=alert(1)>"}};}
 ready();return actual;
};
const cases:unknown[]=[];
try {
 hex=await fetch("/actual-native-ask.hex").then(response=>response.text());
 localStorage.setItem("noname-next-skip-anims","1");history.replaceState(null,"",isTauriShell()?"/?engineTransport=native":"/?engineTransport=wasm");
 for(mode of ["native","missing","null","escaped"]){
  const mounted=new Promise<void>(resolve=>{ready=resolve;});
  const root=document.createElement("div");document.body.append(root);
  const router=createRouter({history:createMemoryHistory(),routes:[{path:"/",component:BattleView}]});await router.push("/");await router.isReady();
  const app=createApp(BattleView);app.use(router);app.mount(root);
  try{
   await mounted;
   const expected=mode==="native"?"是否发动【遗计】？":mode==="escaped"?"是否发动【<img src=x onerror=alert(1)>】？":"是否发动技能？";
   await wait(()=>root.querySelector(".choice h2")?.textContent===expected);
   check(root.querySelector(".choice img[onerror]")===null,"source became executable markup");
   check(client!.getTransportKind()===(isTauriShell()?"desktop-native":"wasm-worker"),"selected transport changed");
   const before=await client!.saveHex();await client!.observer();check(await client!.saveHex()===before,"metadata changed archive");
   if(mode==="native"){
    const button=root.querySelector<HTMLButtonElement>(".choice button");check(button&&!button.disabled,"real confirm unavailable");button!.click();
    await wait(()=>root.querySelector(".choice h2")?.textContent!==expected);
    check(!root.textContent?.includes("WrongPlayer"),"wrong owner answered");
   }
   cases.push({mode,heading:expected,transport:client!.getTransportKind(),actualWorkerSource:"遗计",archiveUnchanged:true});
  }finally{app.unmount();root.remove();client!.dispose();}
 }
 await fetch("/report",{method:"POST",body:JSON.stringify({passed:true,cases})});
}catch(error){await fetch("/report",{method:"POST",body:JSON.stringify({passed:false,error:String(error),stack:error instanceof Error?error.stack:null})});}
finally{EngineClient.prototype.start=original;}
