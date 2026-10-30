async function pcmStatistics(buffer) {
 const channels=[];
 for(let channel=0;channel<buffer.numberOfChannels;channel++){
  const data=buffer.getChannelData(channel);let square=0,peak=0,nonfinite=0,first=-1,last=-1;
  const energy32=[];
  for(let start=0;start<data.length;start+=32){let local=0;for(let i=start;i<Math.min(start+32,data.length);i++){const x=data[i];if(!Number.isFinite(x)){nonfinite++;continue;}square+=x*x;local+=x*x;peak=Math.max(peak,Math.abs(x));if(Math.abs(x)>1e-6){if(first<0)first=i;last=i;}}energy32.push(Math.sqrt(local/Math.min(32,data.length-start)));}
  const raw=data.slice().buffer;
  const sha256=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',raw)),x=>x.toString(16).padStart(2,'0')).join('');
  channels.push({channel,sha256,peak,rms:Math.sqrt(square/data.length),nonfinite,firstAbove1e_6:first,lastAbove1e_6:last,leadingAtMost1e_6:first<0?data.length:first,trailingAtMost1e_6:last<0?data.length:data.length-last-1,energy32});
 }
 return {frames:buffer.length,duration:buffer.duration,sampleRate:buffer.sampleRate,channels:buffer.numberOfChannels,channelStatistics:channels};
}
function compareInMemory(a,b){
 const channels=[];
 for(let c=0;c<Math.min(a.numberOfChannels,b.numberOfChannels);c++){
  const x=a.getChannelData(c),y=b.getChannelData(c);const length=Math.min(x.length,y.length);let xx=0,yy=0,xy=0,maxAbs=0,sumX=0,sumY=0;
  for(let i=0;i<length;i++){xx+=x[i]*x[i];yy+=y[i]*y[i];xy+=x[i]*y[i];maxAbs=Math.max(maxAbs,Math.abs(x[i]-y[i]));sumX+=x[i];sumY+=y[i];}
  const cov=xy-sumX*sumY/length;const varianceX=xx-sumX*sumX/length,varianceY=yy-sumY*sumY/length;
  channels.push({channel:c,overlapFrames:length,lag:0,maxAbsDifference:maxAbs,normalizedPCMCorrelation:cov/Math.sqrt(varianceX*varianceY)});
 }
 return {sameFrames:a.length===b.length,sameSampleRate:a.sampleRate===b.sampleRate,channels};
}
async function fetchDecode(asset,base){
 const url=originalResourceUrl(base,'assets/'+asset.path);const response=await fetch(url,{cache:'no-store'});const body=await response.arrayBuffer();
 const hash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',body)),x=>x.toString(16).padStart(2,'0')).join('');
 const http={url,status:response.status,mime:response.headers.get('content-type'),bytes:body.byteLength,contentLength:response.headers.get('content-length'),sha256:hash};
 if(response.status!==200||body.byteLength!==asset.size||Number(http.contentLength)!==asset.size||hash!==asset.sha256)throw new Error('Frozen HTTP identity mismatch '+asset.path);
 const context=new OfflineAudioContext(1,1,48000);const pcm=await context.decodeAudioData(body);return {http,pcm,statistics:await pcmStatistics(pcm)};
}
async function playMuted(asset,base){
 const audio=document.createElement('audio');audio.controls=true;audio.preload='none';audio.muted=true;audio.src=originalResourceUrl(base,'assets/'+asset.path);document.body.append(audio);
 const result={events:[],progress:[],muted:true};let timer;const start=performance.now();
 const state=()=>({elapsedMs:Math.round(performance.now()-start),currentTime:audio.currentTime,duration:Number.isFinite(audio.duration)?audio.duration:null,ended:audio.ended,readyState:audio.readyState,error:audio.error?{code:audio.error.code,message:audio.error.message}:null});
 for(const name of ['loadedmetadata','loadeddata','playing','ended','error'])audio.addEventListener(name,()=>result.events.push({event:name,...state()}));audio.ontimeupdate=()=>result.progress.push(state());
 const terminal=new Promise(resolve=>{audio.addEventListener('ended',()=>resolve('ended'),{once:true});audio.addEventListener('error',()=>resolve('error'),{once:true});timer=setTimeout(()=>resolve('timeout'),20000);});
 try{try{await audio.play();result.playPromise='resolved';}catch(error){result.playPromise='rejected';result.playError={name:error.name,message:error.message};}
 result.terminal=result.playPromise==='resolved'?await terminal:'playRejected';result.final=state();result.ok=result.terminal==='ended'&&audio.ended&&!audio.error&&audio.currentTime>0&&result.progress.some(row=>row.currentTime>0);
 }finally{clearTimeout(timer);audio.pause();audio.removeAttribute('src');audio.load();audio.remove();}
 return result;
}

async function errorRecovery(asset,base){
 const audio=document.createElement('audio');audio.muted=true;audio.preload='none';document.body.append(audio);
 const wait=()=>new Promise(resolve=>{let timer;const finish=(event)=>{clearTimeout(timer);audio.removeEventListener('ended',ended);audio.removeEventListener('error',error);resolve(event);};const ended=()=>finish('ended'),error=()=>finish('error');audio.addEventListener('ended',ended);audio.addEventListener('error',error);timer=setTimeout(()=>finish('timeout'),20000);});
 try{audio.src=originalResourceUrl(base,'assets/missing-compat/'+asset.path.split('/').at(-1));let terminal=wait();try{await audio.play();}catch{}const failed=await terminal;const errorCode=audio.error?.code;
 audio.src=originalResourceUrl(base,'assets/'+asset.path);terminal=wait();await audio.play();const recovered=await terminal;
 return {failed,errorCode,recovered,ended:audio.ended,time:audio.currentTime,ok:failed==='error'&&recovered==='ended'&&audio.ended&&!audio.error};
 }finally{audio.pause();audio.removeAttribute('src');audio.load();audio.remove();}
}


async function runProbe(config,emit){
 const report=row=>emit(JSON.stringify(row));report({event:'runtime',userAgent:navigator.userAgent,mode:'current Main derived five HTTP/full PCM/HTML only; original raw not re-decoded'});
 for(const asset of config.assets){const decoded=await fetchDecode(asset,config.base);const play=await playMuted(asset,config.base);report({event:'result',path:asset.path,http:decoded.http,statistics:decoded.statistics,play});}
 report({event:'done',count:config.assets.length,stopped:false});
}
