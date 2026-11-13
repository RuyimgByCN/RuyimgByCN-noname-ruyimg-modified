(async()=>{
 const n=await import('/noname.js');const {game,lib,ui,_status,get}=n;
 // U07 character skill module has unqualified noname names; provide the real exports.
 Object.assign(window,{game,lib,ui,_status,get,ai:n.ai});
 const {default:EventManager}=await import('/noname/library/element/GameEvent/GameEventManager.ts');
 const {default:PauseManager}=await import('/noname/game/PauseManager.ts');
 const {default:skills}=await import('/extension/玩点论杀/character/skill.js');
 const {default:cards}=await import('/extension/玩点论杀/card/card.js');
 const {default:cardSkills}=await import('/extension/玩点论杀/card/skill.js');
 const {default:translate}=await import('/extension/玩点论杀/character/translate.js');
 const {default:cardTranslate}=await import('/extension/玩点论杀/card/translate.js');
 const f=window.runtimeFixture;window.q1=n;window.runtimeCase=f.case;window.runtimeDone=false;window.runtimeError=null;window.runtimeChoices=[];
 _status.eventManager=new EventManager();_status.pauseManager=new PauseManager();
 const root=game.createEvent('runtimeProbe',false);root.player=game.players[f.actor];root.name=f.case.startsWith('choudu')?'phaseUse':'phase';
 _status.auto=false;_status.imchoosing=false;_status.gameStarted=true;_status.gameDrawed=true;_status.clicked=false;
 _status.currentPhase=game.players[f.actor];game.phaseNumber=1;lib.config.auto_confirm=false;lib.config.ignore_error=false;lib.config.touchscreen=false;
 Object.assign(lib.card,cards);Object.assign(lib.skill,skills,cardSkills);Object.assign(lib.translate,translate,cardTranslate);
 for(const k of [...Object.keys(skills),...Object.keys(cardSkills)])game.finishSkill(k);game.finishCards();
 lib.character.runtime_dummy=get.convertedCharacter(['male','wei',8,[]]);game.playerMap??={};
 const old=[];
 for(const [seat,p]of game.players.entries()){
  old.push({seat,virtualEquips:p.getVCards('e').map(c=>c.name),virtualJudges:p.getVCards('j').map(c=>c.name)});
  for(const c of p.getVCards('e').slice())p.removeVirtualEquip(c);
  for(const c of p.getVCards('j').slice())p.removeVirtualJudge(c);
  p.removeSkill(p.getSkills());p.uninit();p.init('runtime_dummy');for(const c of p.getCards('hejsx'))c.remove();
  p.storage={};p.stat=[{card:{},skill:{}}];p.hp=f.initial.players[seat].hp;p.maxHp=f.initial.players[seat].max_hp;p.update();
  if(!p.playerid)p.playerid='u07_'+seat;game.playerMap[p.playerid]=p;
 }
 window.runtimeCards={};
 const aliases={archery:'wanjian',armor:'bagua',bing:'bingliang',chain:'tiesuo',dismantle:'guohe',duel:'juedou',horse_minus:'chitu',horse_plus:'jueying',le:'lebu',nullify:'wuxie',savage:'nanman',shandian_safe:'shandian',snatch:'shunshou',weapon2:'qinggang',weapon2b:'cixiong',weapon3:'qinglong',fire_sha:'sha',leisha:'sha'};
 const mapped=name=>aliases[name]||(name.startsWith('wd_')?name.slice(3).replace(/_\d+$/,''):name);
 window.runtimeAliases=aliases;
 const create=d=>{const c=game.createCard(mapped(d.name),d.suit,d.rank);c.__nativeId=d.id;runtimeCards[d.id]=c;return c;};
 ui.cardPile.innerHTML='';ui.discardPile.innerHTML='';ui.ordering.innerHTML='';ui.special.innerHTML='';
 for(const d of f.initial.deck_draw_order)ui.cardPile.appendChild(create(d));
 for(const row of f.initial.players){const p=game.players[row.seat];for(const d of row.hand)p.directgain([create(d)]);for(const d of row.equips)p.directequip(create(d));for(const k of row.skills){if(!['wd_sanjian'].includes(k))p.addSkill(k);}}
 const fc=c=>({id:c.__nativeId??null,name:c.name,suit:c.suit,rank:c.number});
 window.runtimeSnapshot=()=>({players:game.players.map((p,seat)=>({seat,hp:p.hp,max_hp:p.maxHp,alive:p.isAlive(),linked:p.isLinked(),face_down:p.isTurnedOver(),hujia:p.hujia||0,hand:p.getCards('h').map(fc),equips:p.getCards('e').map(fc),judge:p.getCards('j').map(fc),virtualEquips:p.getVCards('e').map(c=>c.name),virtualJudges:p.getVCards('j').map(c=>c.name),skills:p.getSkills().slice(),fuchou_marked:(p.storage.fuchou2||[]).map(x=>game.players.indexOf(x))})),deck_draw_order:[...ui.cardPile.childNodes].filter(c=>c.name).map(fc),discard:[...ui.discardPile.childNodes].filter(c=>c.name).map(fc),table:[...ui.ordering.childNodes].filter(c=>c.name).map(fc)});
 // Control routing only: every original choice constructor/event still runs.
 for(const p of game.players)for(const method of ['chooseToUse','chooseToRespond','chooseCard','chooseToDiscard','chooseButton','chooseTarget','chooseBool','chooseControl']){
  const original=p[method];p[method]=function(...args){game.swapControl(this);const e=original.apply(this,args);runtimeChoices.push({created:method,player:game.players.indexOf(this),prompt:e.prompt,at:Date.now()});return e;};
 }
 game.swapControl(game.players[f.case.startsWith('fuchou')?1:0]);window.runtimeHistoryStart=game.getGlobalHistory('everything').length;
 window.runtimeInitial=runtimeSnapshot();window.runtimeOld=old;
 root.setContent(async function(event){
  if(runtimeCase.startsWith('choudu')){window.runtimeActivation=await q1.game.players[0].chooseToUse().set('type','phase').forResult();}
  else{await q1.game.players[runtimeFixture.actor].useCard(runtimeCards[runtimeFixture.cards.attack],[q1.game.players[runtimeFixture.target]],false).set('animate',false);}
  window.runtimeAfterAttack=runtimeSnapshot();
  if(runtimeCase.includes('settle'))await event.trigger('phaseAfter');
  window.runtimeDone=true;
 });
 window.runtimeRoot=root;root.start().catch(e=>{window.runtimeError=e.stack;});
 return JSON.stringify({case:f.case,initial:runtimeInitial,old:runtimeOld,skillReady:['choudu','fuchou','fuchou2','youzhan'].every(k=>!!lib.skill[k]),playerCount:game.players.length});
})();
