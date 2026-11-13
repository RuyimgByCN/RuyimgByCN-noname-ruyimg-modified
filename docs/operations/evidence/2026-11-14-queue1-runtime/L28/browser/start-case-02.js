(() => {
  const f = window.l28Fixture;
  game.pause();
  _status.paused2 = false; _status.over = false; _status.auto = false;
  _status.gameStarted = true; _status.gameDrawed = true;
  _status.currentPhase = game.players[0]; game.phaseNumber = 1;
  lib.config.auto_confirm = false; lib.config.auto_skill = false;
  lib.config.touchscreen = false; lib.config.errstop = true;
  // These are the original extension's configuration defaults, not loader flags.
  lib.config['extension_雷霆万钧_yuqiTheme'] = 'follow';
  lib.config['extension_雷霆万钧_timeTheme'] = 'follow';
  lib.config['extension_雷霆万钧_standPath'] = 'extension/雷霆万钧/image/stand/';
  if (f.strength === 'forbid') localStorage.setItem('strengthen_skill','forbid');
  else localStorage.removeItem('strengthen_skill');
  for (const n of ui.controls.slice()) n.close();
  if (ui.dialog) ui.dialog.close();
  if (ui.confirm) ui.confirm.close();
  for (const k of ['cards','targets','buttons']) ui.selected[k].length = 0;
  for (const [seat,p] of game.players.entries()) {
    for (const s of p.getSkills(null,false,false)) p.removeSkill(s);
    p.init('caocao');
    for (const s of p.getSkills(null,false,false)) p.removeSkill(s);
    for (const c of p.getCards('hejsx')) c.remove();
    p.storage = {}; p.additionalSkills = {}; p.tempSkills = {}; p.disabledSkills = {};
    p.initedSkills = []; p.stat = [{card:{},skill:{}}];
    p.playerid = 'l28_' + seat;
    p.classList.remove('turnedover','linked');
    p.maxHp = f.initial.players[seat].max_hp; p.hp = f.initial.players[seat].hp;
    p.hujia = 0; p.update();
    p.identity = seat === 0 ? 'zhu' : 'fan';
  }
  game.zhu = game.players[0]; game.swapControl(game.players[0]);
  window.l28Cards = {};
  const create = d => {
    if (!lib.card[d.upstream_name]) throw Error('missing actual upstream card '+JSON.stringify(d));
    const c = game.createCard(d.upstream_name,d.suit,d.rank,d.nature||undefined);
    c.l28NativeId = d.id; l28Cards[d.id] = c; return c;
  };
  for (const pile of [ui.cardPile,ui.discardPile,ui.ordering,ui.special]) pile.innerHTML='';
  for (const d of f.initial.deck_draw_order) ui.cardPile.appendChild(create(d));
  for (const d of f.initial.discard) ui.discardPile.appendChild(create(d));
  for (const row of f.initial.players) {
    const p = game.players[row.seat];
    if (row.hand.length) p.directgain(row.hand.map(create));
    for (const d of row.equips) p.directequip(create(d));
    for (const info of row.skills) {
      if (!lib.skill[info.name]) throw Error('missing original skill '+info.name);
      p.addSkill(info.name);
      if (info.once_used) p.awakenSkill(info.name);
    }
    for (const pile of row.piles) {
      if (pile.key !== 'th_liujing') throw Error('unsupported initial fixture pile '+pile.key);
      for (const d of pile.cards) {
        const c=create(d); c.addGaintag('th_chengye'); p.node.expansions.appendChild(c);
      }
    }
  }
  window.l28Trace=[]; window.l28Done=false;
  window.l28HistoryStart=game.getGlobalHistory('everything').length;
  window.l28HpHistoryStart=game.getGlobalHistory('changeHp').length;
  const brief = x => Array.isArray(x) ? x.map(brief) : game.players.includes(x) ? {seat:game.players.indexOf(x)} : x&&typeof x==='object' ? x.name ? {name:x.name,id:x.l28NativeId,suit:x.suit,rank:x.number,nature:x.nature||null} : String(x) : x;
  for (const p of game.players) {
    for (const method of ['damage','loseHp','discard','gain','addToExpansion','awakenSkill','logSkill','useSkill']) {
      const original=p[method]; if (typeof original!=='function') continue;
      p[method]=function(...args) {
        const eventBefore={name:_status.event?.name,step:_status.event?.step};
        const result=original.apply(this,args);
        l28Trace.push({method,seat:game.players.indexOf(this),args:args.map(brief),event:eventBefore,
          emitted:result&&typeof result==='object'?{name:result.name,source:game.players.indexOf(result.source),player:game.players.indexOf(result.player),num:result.num,nature:result.nature||null}:null});
        return result;
      };
    }
  }
  const face=c=>({id:c.l28NativeId??null,name:c.name,suit:c.suit,rank:c.number,nature:c.nature||null});
  window.l28Snapshot=()=>{
    const e=_status.event; const stack=[]; for(let p=e,i=0;p&&i<12;p=p.parent,i++)stack.push({name:p.name,step:p.step,skill:p.skill});
    const rect=n=>n?{left:n.getBoundingClientRect().left,top:n.getBoundingClientRect().top,width:n.getBoundingClientRect().width,height:n.getBoundingClientRect().height}:null;
    return {case:f.case,done:l28Done,paused:_status.paused,stack,
      choice:{name:e?.name,skill:e?.skill,prompt:e?.prompt,skills:e?._skillChoice,confirm:ui.confirm?.str,controls:e?.controls,
        selectableCards:game.me.getCards('he').filter(c=>c.classList.contains('selectable')).map(face),selectedCards:ui.selected.cards.map(face),
        visibleSkills:ui.skills?.skills,selectedTargets:ui.selected.targets.map(p=>game.players.indexOf(p))},
      dom:{chambers:[...document.querySelectorAll('.th-mamidibu')].map(n=>({id:n.id,rect:rect(n)})),
        flames:[...document.querySelectorAll('.th-yeyanfire>.th-yeyanflame')].map(n=>({id:n.id,extinguished:n.classList.contains('extinguish'),rect:rect(n)})),
        players:game.players.map((p,seat)=>({seat,rect:rect(p),avatar:rect(p.node.avatar),fires:p._thyeyanfires?.slice()}))},
      players:game.players.map((p,seat)=>({seat,hp:p.hp,max_hp:p.maxHp,alive:p.isAlive(),linked:p.isLinked(),face_down:p.isTurnedOver(),hujia:p.hujia||0,
        hand:p.getCards('h').map(face),equips:p.getCards('e').map(face),judge:p.getCards('j').map(face),expansions:p.getExpansions('th_chengye').map(face),
        skills:p.getSkills(null,false,false),awakened:p.awakenedSkills.slice(),skillStat:{...p.getStat('skill')}})),
      deck_draw_order:[...ui.cardPile.childNodes].filter(c=>c.name).map(face),discard:[...ui.discardPile.childNodes].filter(c=>c.name).map(face),table:[...ui.ordering.childNodes].filter(c=>c.name).map(face),
      event_history:game.getGlobalHistory('everything').slice(l28HistoryStart).filter(e=>['damage','loseHp','changeHp'].includes(e.name)).map(e=>({name:e.name,source:game.players.indexOf(e.source),player:game.players.indexOf(e.player),num:e.num,nature:e.nature||null,parent:e.parent?.name})),
      hp_history:game.getGlobalHistory('changeHp').slice(l28HpHistoryStart).map(e=>({num:e.num,player:game.players.indexOf(e.player),parent:e.parent?.name,source:game.players.indexOf(e.parent?.source),nature:e.parent?.nature||null})),
      trace:l28Trace.slice(),strength:{stored:localStorage.getItem('strengthen_skill'),result:window.getStrength(true,false)},
      spine:{dcdAnim:typeof dcdAnim,zhengjingxiao:window.zhengjingxiao??null}};
  };
  const root=game.createEvent('l28Probe',false); root.parent=null;
  root.setContent(function(){window.l28Done=true;event.finish();game.pause();});
  _status.event=root; window.l28Initial=l28Snapshot();
  game.players[0].phaseUse();game.resume();
  return JSON.stringify({initial:l28Initial,origin:location.origin,version:lib.version,started:true});
})();
