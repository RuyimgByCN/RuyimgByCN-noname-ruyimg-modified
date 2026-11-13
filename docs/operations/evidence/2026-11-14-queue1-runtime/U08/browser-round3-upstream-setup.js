(async () => {
  const n = await import('/noname.js');
  const { game, lib, ui, get, _status } = n;
  const { default: EventManager } = await import('/noname/library/element/GameEvent/GameEventManager.ts');
  const { default: PauseManager } = await import('/noname/game/PauseManager.ts');
  const { default: skills } = await import('/extension/英雄杀/character/skill.js');
  const { default: translate } = await import('/extension/英雄杀/character/translate.js');
  const fixture = window.u08Fixture;
  const name = window.u08Case;
  window.q1 = n; window.u08Trace = []; window.u08Error = null; window.u08Done = false;
  Object.assign(lib.skill, skills); Object.assign(lib.translate, translate);
  for (const key of Object.keys(skills)) game.finishSkill(key);
  _status.eventManager = new EventManager(); _status.pauseManager = new PauseManager();
  const root = game.createEvent('u08Probe', false);
  root.name = name.startsWith('b_') ? 'phase' : 'phaseUse';
  root.player = game.players[name.startsWith('b_') && name !== 'b_owner_attacker' ? 1 : 0];
  _status.auto = false; _status.imchoosing = false; _status.gameStarted = true; _status.gameDrawed = true;
  _status.clicked = false; _status.currentPhase = root.player;
  game.phaseNumber = 1;
  Object.assign(lib.config, { auto_confirm: false, touchscreen: false, ignore_error: false, game_speed: 'vvfast', duration: 0 });
  for (const control of ui.controls.slice()) control.close();
  for (const dialog of ui.dialogs.slice()) dialog.close();
  if (ui.confirm) ui.confirm.close();
  ui.selected.cards.length = 0; ui.selected.targets.length = 0; ui.selected.buttons.length = 0;
  lib.character.u08_dummy = get.convertedCharacter(['male', 'wei', 4, []]);
  game.playerMap ??= {};
  const prior = [];
  for (const [seat, p] of game.players.entries()) {
    prior.push({ seat, virtualEquips: p.getVCards('e').map(c => c.name), virtualJudges: p.getVCards('j').map(c => c.name), skills: p.getSkills() });
    for (const card of p.getVCards('e').slice()) p.removeVirtualEquip(card);
    for (const card of p.getVCards('j').slice()) p.removeVirtualJudge(card);
    p.removeSkill(p.getSkills()); p.uninit(); p.init('u08_dummy');
    for (const card of p.getCards('hejsx')) card.remove();
    p.storage = {}; p.additionalSkills = {}; p.tempSkills = {}; p.disabledSkills = {};
    p.initedSkills = []; p.awakenedSkills = []; p.stat = [{ card: {}, skill: {} }];
    delete p._trueMe;
    p.classList.remove('turnedover', 'linked');
    p.identity = ['zhu', 'zhong', 'fan', 'nei', 'zhong', 'fan', 'fan', 'fan'][seat];
    p.maxHp = fixture.players[seat].max_hp; p.hp = fixture.players[seat].hp;
    if (!p.playerid) p.playerid = 'u08_' + seat;
    game.playerMap[p.playerid] = p; p.update();
  }
  game.zhu = game.players[0];
  const fromFace = spec => {
    if (!spec.upstream_name || !lib.card[spec.upstream_name]) throw Error('unmapped actual fixture entity ' + JSON.stringify(spec));
    const card = game.createCard(spec.upstream_name, spec.suit, spec.rank, spec.nature || undefined);
    card.u08NativeId = spec.id; return card;
  };
  ui.cardPile.innerHTML = ''; ui.discardPile.innerHTML = ''; ui.ordering.innerHTML = ''; ui.special.innerHTML = '';
  for (const spec of fixture.deck_top_first) ui.cardPile.appendChild(fromFace(spec));
  for (const spec of fixture.discard.filter(s => s.upstream_name && lib.card[s.upstream_name])) ui.discardPile.appendChild(fromFace(spec));
  for (const [seat, p] of game.players.entries()) {
    for (const spec of fixture.players[seat].hand) p.directgain([fromFace(spec)]);
    for (const spec of fixture.players[seat].equips) p.directequip(fromFace(spec));
    for (const { name } of fixture.players[seat].skills) p.addSkill(name);
  }
  game.swapControl(game.players[0]);
  if (name === 'b_under_control') game.players[2]._trueMe = game.players[0];
  window.u08ForeignPileMutations = [];
  window.u08PileObserver = new MutationObserver(ms => {
    for (const m of ms) for (const c of m.addedNodes) if (c.name && c.u08NativeId == null) u08ForeignPileMutations.push({name:c.name,suit:c.suit,rank:c.number});
  });
  u08PileObserver.observe(ui.cardPile, { childList:true });
  for (const p of game.players) for (const method of ['logSkill','discard','discardPlayerCard','damage','useCard','respond','gain','draw','chooseButton','chooseControl','chooseToUse','chooseToRespond','chooseToDiscard','chooseTarget','chooseCard']) {
    const original = p[method];
    p[method] = function (...args) {
      const brief = a => Array.isArray(a) ? a.map(brief) : game.players.includes(a) ? {seat:game.players.indexOf(a)} : a && typeof a === 'object' ? a.name ? {name:a.name,id:a.u08NativeId,suit:a.suit,rank:a.number} : String(a) : a;
      u08Trace.push({method,seat:game.players.indexOf(this),args:args.map(brief),event:_status.event?.name});
      if (['chooseButton','chooseControl','chooseToUse','chooseToRespond','chooseToDiscard','chooseTarget','chooseCard'].includes(method)) game.swapControl(this);
      return original.apply(this,args);
    };
  }
  const face = c => ({id:c.u08NativeId ?? null,name:c.name,suit:c.suit,rank:c.number,nature:c.nature || null});
  window.u08Snapshot = () => {
    const event = _status.event;
    const dialog = (typeof event?.dialog === 'object' ? event.dialog : ui.dialog);
    const stack = []; let e=event;
    for (let i=0;e && i<12;i++,e=e.parent) stack.push({name:e.name,step:e.step,skill:e.skill,triggername:e.triggername});
    return {case:name,done:window.u08Done,error:window.u08Error,paused:_status.paused,stack,
      choice:{name:event?.name,skill:event?.skill,prompt:event?.prompt,forced:event?.forced,controls:event?.controls,actor:game.players.indexOf(event?.player),
        confirm:ui.confirm?.str,skills:event?._skillChoice,visibleSkills:ui.skills?.skills,selectablePlayers:game.players.filter(p=>p.classList.contains('selectable')).map(p=>game.players.indexOf(p)),
        selectedPlayers:ui.selected.targets.map(p=>game.players.indexOf(p)),selectedCards:ui.selected.cards.map(c=>c.u08NativeId),
        cards:game.players.flatMap((p,seat)=>p.getCards('he').map(c=>({...face(c),seat,selected:c.classList.contains('selected'),selectable:c.classList.contains('selectable')}))),
        actorIsMe:event?.player===game.me,uiControls:ui.controls.map(n=>Array.from(n.children).map(c=>({link:c.link,text:c.textContent}))),
        buttons:dialog?.buttons?.map((b,index)=>({index,link:b.link?.name?face(b.link):b.link,selectable:b.classList.contains('selectable'),selected:b.classList.contains('selected')}))},
      players:game.players.map((p,seat)=>({seat,hp:p.hp,max_hp:p.maxHp,face_down:p.isTurnedOver(),alive:p.isAlive(),
        hand:p.getCards('h').map(face),equips:p.getCards('e').map(face),judge:p.getCards('j').map(face),
        virtualEquips:p.getVCards('e').map(c=>c.name),virtualJudges:p.getVCards('j').map(c=>c.name),
        skills:p.getSkills(null,false,false).map(name=>({name,disabled:p.isDisabled(name)})),awakenedSkills:p.awakenedSkills.slice(),
        yixin_used:p.awakenedSkills.includes('yixin'),bolehuiyan:p.storage.bolehuiyan,bolehuiyan4:p.hasSkill('bolehuiyan4'),
        trueMe:game.players.indexOf(p._trueMe),isUnderControl:p.isUnderControl(),stat:p.getStat()})),
      deck:Array.from(ui.cardPile.childNodes).filter(c=>c.name).map(face),discard:Array.from(ui.discardPile.childNodes).filter(c=>c.name).map(face),ordering:Array.from(ui.ordering.childNodes).filter(c=>c.name).map(face),
      skippedInitialDiscard:fixture.discard.filter(s=>!s.upstream_name || !lib.card[s.upstream_name]),
      untaggedDeck:Array.from(ui.cardPile.childNodes).filter(c=>c.name && c.u08NativeId==null).map(face),foreignPileMutations:u08ForeignPileMutations,trace:u08Trace,
      history:game.getGlobalHistory('everything').slice(window.u08HistoryStart).map(e=>({name:e.name,skill:e.skill,triggername:e.triggername,player:game.players.indexOf(e.player),card:e.card?.name,targets:e.targets?.map(p=>game.players.indexOf(p))}))};
  };
  window.u08HistoryStart = game.getGlobalHistory('everything').length;
  window.u08Initial = u08Snapshot();
  root.setContent(async function(event) {
    if (u08Case.startsWith('b_')) {
      const actor=u08Case==='b_owner_attacker'?0:1;
      const target=u08Case==='b_owner_attacker'?1:u08Case==='b_owner_target'?0:2;
      const attacks=['b_repeat','b_cancel_then_retry'].includes(u08Case)?2:1;
      for(let i=0;i<attacks;i++) {
        window.u08AttackIndex=i;
        const card=q1.game.players[actor].getCards('h').find(c=>c.name==='sha');
        if(!card)throw Error('missing real attack');
        await q1.game.players[actor].useCard(card,[q1.game.players[target]]).set('animate',false);
      }
    } else {
      const attempts=u08Case.endsWith('reuse')?2:1;
      for(let i=0;i<attempts;i++) {
        window.u08Attempt=i;
        window.u08Activation=await q1.game.players[0].chooseToUse().set('type','phase').forResult();
      }
    }
    window.u08Done=true;
  });
  window.u08Root=root;root.start().catch(error=>{window.u08Error=error.stack;});
  return JSON.stringify({started:true,initial:u08Initial,prior,sourceEntry:'/extension/英雄杀/character/skill.js',skillReady:['yixin','bolehuiyan','guifu'].every(k=>!!lib.skill[k])});
})()
