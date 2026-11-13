(() => {
  const fixture = window.l29Fixture;
  const name = window.l29Case;
  if (!fixture || !name) throw Error('fixture/case required');
  game.pause();
  _status.paused2 = false; _status.over = false; _status.auto = false;
  _status.gameStarted = true; _status.gameDrawed = true;
  game.phaseNumber = 1;
  lib.config.auto_confirm = false; lib.config.touchscreen = false;
  lib.config.errstop = true; lib.config.auto_skill = false;
  lib.config.autoskilllist = ['mbcmjiushi_gain'];
  for (const control of ui.controls.slice()) control.close();
  if (ui.dialog) ui.dialog.close();
  if (ui.confirm) ui.confirm.close();
  ui.selected.cards.length = 0; ui.selected.targets.length = 0; ui.selected.buttons.length = 0;
  lib.character.l29_dummy = ['male', 'wei', 4, []];
  for (const [seat, p] of game.players.entries()) {
    if (!p.playerid) p.playerid = 'l29_' + seat;
    for (const skill of [...p.skills]) p.removeSkill(skill);
    for (const card of p.getCards('hej')) card.remove();
    p.init('l29_dummy');
    p.storage = {}; p.additionalSkills = {}; p.tempSkills = {}; p.disabledSkills = {};
    p.initedSkills = [];
    p.stat = [{ card: {}, skill: {} }];
    p.classList.remove('turnedover', 'linked');
    p.identity = ['zhu','zhong','fan','nei'][seat];
    p.maxHp = fixture.players[seat].max_hp; p.hp = fixture.players[seat].hp;
    if (fixture.players[seat].face_down) p.classList.add('turnedover');
    p.update();
  }
  game.zhu = game.players[0];
  const fromFace = c => {
    if (!c.upstream_name) throw Error('unmapped fixture face: ' + JSON.stringify(c));
    const card = game.createCard(c.upstream_name, c.suit, c.rank, c.nature || undefined);
    card.l29NativeId = c.id;
    return card;
  };
  for (const [seat, p] of game.players.entries()) {
    const specs = fixture.players[seat];
    if (specs.hand.length) p.directgain(specs.hand.map(fromFace));
    for (const skill of specs.skills) {
      const upstream = skill.name === '马术' ? 'mashu' : skill.name;
      if (!lib.skill[upstream]) throw Error('missing actual upstream skill ' + upstream);
      p.addSkill(upstream);
    }
  }
  // Align hands and the first native trick used by 酒诗. Remaining native deck
  // cards are retained in the fixture record; unsupported faces are outside the
  // projected cross-engine comparison, not silently treated as identical.
  const usable = fixture.deck_top_first.filter(c => c.upstream_name && lib.card[c.upstream_name]);
  ui.cardPile.innerHTML = ''; ui.discardPile.innerHTML = ''; ui.ordering.innerHTML = '';
  for (const c of usable) ui.cardPile.appendChild(fromFace(c));
  window.l29UnmappedDeck = fixture.deck_top_first.filter(c => !c.upstream_name || !lib.card[c.upstream_name]);
  window.l29SkippedDiscard = fixture.discard;
  for (const c of fixture.discard.filter(c => c.upstream_name && lib.card[c.upstream_name])) ui.discardPile.appendChild(fromFace(c));
  if (name === 'f_last_target') game.players[0].storage.mbcmfangzhu = game.players[1];
  game.me = game.players.find(p => p.node.handcards1 === ui.handcards1) || game.players[0];
  game.swapControl(game.players[0]);
  window.l29Trace = [];
  for (const p of game.players) {
    if (p.l29Observed) continue;
    for (const method of ['logSkill', 'discard', 'turnOver', 'damage', 'useCard', 'gain']) {
      const original = p[method];
      p[method] = function (...args) {
        const brief = a => {
          if (Array.isArray(a)) return a.map(brief);
          if (game.players.includes(a)) return { seat: game.players.indexOf(a) };
          if (a && typeof a === 'object') return a.name ? { name:a.name,suit:a.suit,rank:a.number,id:a.l29NativeId } : String(a);
          return a;
        };
        window.l29Trace.push({method,seat:game.players.indexOf(this),args:args.map(brief),event:_status.event?.name,step:_status.event?.step});
        const emitted = original.apply(this,args);
        if (method === 'damage') window.l29Trace.push({method:'damage_created',seat:game.players.indexOf(this),
          source:game.players.indexOf(emitted.source),amount:emitted.num,nature:emitted.nature});
        return emitted;
      };
    }
    p.l29Observed = true;
  }
  if (!window.l29Instrumented) {
    for (const method of ['logSkill', 'discard', 'turnOver', 'damage', 'useCard', 'gain']) {
      const old = lib.element.player[method];
      if (typeof old !== 'function') continue;
      lib.element.player[method] = function (...args) {
        const simplify = a => {
          if (a === null || a === undefined || typeof a !== 'object') return a;
          if (Array.isArray(a)) return a.map(simplify);
          if (game.players.includes(a)) return { seat: game.players.indexOf(a) };
          if (a.name) return { name: a.name, suit: a.suit, rank: a.number, nature: a.nature, id: a.l29NativeId };
          return String(a);
        };
        window.l29Trace.push({ method, seat: game.players.indexOf(this), args: args.map(simplify),
          event: _status.event?.name, step: _status.event?.step });
        return old.apply(this, args);
      };
    }
    window.l29Instrumented = true;
  }
  window.l29Snapshot = () => {
    const card = c => ({ id: c.l29NativeId, name: c.name, suit: c.suit, rank: c.number, nature: c.nature || null });
    let event = _status.event;
    const stack = [];
    for (let i = 0; event && i < 12; i++, event = event.parent) stack.push({ name: event.name, step: event.step, skill: event.skill });
    const cur = _status.event;
    return {
      case: window.l29Case, done: window.l29Done, paused: _status.paused, stack,
      choice: { name: cur?.name, skill: cur?.skill, prompt: cur?.prompt, forced: cur?.forced, controls: cur?.controls,
        confirm: ui.confirm?.str, skills: cur?._skillChoice,
        visibleSkills: ui.skills?.skills, selectablePlayers: game.players.filter(p => p.classList.contains('selectable')).map(p => game.players.indexOf(p)),
        uiControls: ui.controls.map(n=>Array.from(n.children).map(c=>({link:c.link,text:c.textContent}))),
        buttons: ui.dialog?.buttons?.map((b,index) => ({ index, link: b.link?.name ? card(b.link) : b.link, selectable: b.classList.contains('selectable'), selected: b.classList.contains('selected') })) },
      players: game.players.map((p, seat) => ({ seat, hp: p.hp, max_hp: p.maxHp,
        face_down: p.isTurnedOver(), hand: p.getCards('h').map(card), equips: p.getCards('e').map(card),
        skills: p.getSkills(null, false, false).map(s => ({ name:s, disabled:p.isDisabled(s),
          blocked_by_fangzhu:p.hasSkill('mbcmfangzhu_baiban') && lib.skill.mbcmfangzhu_baiban.skillBlocker(s,p) })),
        last_target: game.players.indexOf(p.storage.mbcmfangzhu),
        ban: p.storage.mbcmfangzhu_jin,
        enabled: { sha: lib.filter.cardEnabled({name:'sha',isCard:true},p), jiu:lib.filter.cardEnabled({name:'jiu',isCard:true},p), wuzhong:lib.filter.cardEnabled({name:'wuzhong',isCard:true},p) },
        jiu_mark: p.countMark('jiu') })),
      discard: Array.from(ui.discardPile.childNodes).map(card), ordering: Array.from(ui.ordering.childNodes).map(card),
      deck: Array.from(ui.cardPile.childNodes).map(card), unmappedDeck: window.l29UnmappedDeck, trace: window.l29Trace,
    };
  };
  window.l29Done = false;
  const root = game.createEvent('l29Probe', false);
  root.parent = null;
  root.setContent(function () { window.l29Done = true; event.finish(); game.pause(); });
  _status.event = root;
  _status.currentPhase = name.includes('damage') ? game.players[1] : game.players[0];
  const initial = window.l29Snapshot();
  window.l29Initial = initial;
  if (name.includes('damage')) {
    const attack = game.players[1].getCards('h').find(c => c.name === 'sha');
    if (!attack) throw Error('no native Sha face for damage case');
    const use = game.players[1].useCard(attack,[game.players[0]],false); use.animate = false;
  } else {
    game.players[0].phaseUse();
  }
  game.resume();
  return JSON.stringify({ started: true, initial });
})();
