(async () => {
  const n = await import('/noname.js');
  const { game, lib, ui, _status, get } = n;
  const { default: EventManager } = await import('/noname/library/element/GameEvent/GameEventManager.ts');
  const { default: PauseManager } = await import('/noname/game/PauseManager.ts');
  const { default: skills } = await import('/extension/欢乐卡牌/card/skill.js');
  const { default: cards } = await import('/extension/欢乐卡牌/card/card.js');
  const { default: translate } = await import('/extension/欢乐卡牌/card/translate.js');
  const fixture = window.runtimeFixture;
  const initial = fixture.initial.fixture || fixture.initial;
  const names = {
    hnl_monkey_3: 'monkey', hnl_kuwu_0: 'kuwu', hnl_xuelunyang_4: 'xuelunyang',
    马术: 'mashu', 咆哮: 'paoxiao', 武圣: 'wusheng',
  };
  window.q1 = n;
  window.runtimeNames = names;
  window.runtimeCase = fixture.case;
  window.runtimeError = null;
  window.runtimeDone = false;
  window.runtimeInitial = null;
  window.runtimeAttack = null;
  _status.eventManager = new EventManager();
  _status.pauseManager = new PauseManager();
  _status.auto = false;
  _status.imchoosing = false;
  _status.gameStarted = true;
  _status.gameDrawed = true;
  _status.clicked = false;
  lib.config.auto_confirm = false;
  lib.config.touchscreen = false;
  lib.config.ignore_error = false;
  Object.assign(lib.card, cards);
  Object.assign(lib.skill, skills);
  Object.assign(lib.translate, translate);
  for (const name of Object.keys(skills)) game.finishSkill(name);
  game.finishCards();
  lib.character.runtime_dummy = get.convertedCharacter(['male', 'wei', 8, []]);
  game.playerMap ??= {};
  for (const [seat, p] of game.players.entries()) {
    if (!p.playerid) p.playerid = 'runtime_' + seat;
    game.playerMap[p.playerid] = p;
    p.removeSkill(Object.keys(p.tempSkills));
    p.uninit();
    p.init('runtime_dummy');
    for (const c of p.getCards('hej')) c.remove();
    p.storage = {};
    p.stat = [{ card: {}, skill: {} }];
    p.hp = initial.players[seat].hp;
    p.maxHp = initial.players[seat].max_hp;
    p.update();
  }
  ui.cardPile.innerHTML = '';
  ui.discardPile.innerHTML = '';
  ui.ordering.innerHTML = '';
  for (let i = 0; i < 16; i++) ui.cardPile.appendChild(game.createCard('sha', 'diamond', 2));
  const create = face => game.createCard(names[face.name] || (face.kind === 'Duel' ? 'juedou' : face.name), face.suit, face.rank);
  window.runtimeCards = {};
  for (const row of initial.players) {
    const p = game.players[row.seat];
    for (const face of row.hand) {
      const card = create(face);
      p.directgain([card]);
      runtimeCards[face.id] = card;
    }
    for (const face of row.equips) {
      const card = create(face);
      p.directequip(card);
      runtimeCards[face.id] = card;
    }
    for (const { name } of row.skills) {
      if (names[name]) p.addSkill(names[name]);
    }
  }
  const controlled = fixture.case.startsWith('kuwu') ? 1 : 0;
  game.swapControl(game.players[controlled]);
  game.phaseNumber = fixture.case.startsWith('xuelun') ? 2 : 1;
  const root = game.createEvent('runtimeProbe', false);
  root.name = fixture.case.startsWith('xuelun') ? 'phase' : 'phaseUse';
  root.player = game.players[initial.actor];
  _status.currentPhase = root.player;
  window.runtimeHistoryStart = game.getGlobalHistory('everything').length;
  window.runtimeInitial = game.players.map((p, seat) => ({ seat, hp: p.hp, max_hp: p.maxHp,
    hand: p.getCards('h').map(c => ({ name: c.name, suit: c.suit, rank: c.number })),
    equips: p.getCards('e').map(c => ({ name: c.name, suit: c.suit, rank: c.number })),
    skills: p.getSkills().slice(), nonEquipmentSkills: p.getSkills(null, false, false).slice(),
  }));
  if (initial.attack) window.runtimeAttack = runtimeCards[initial.attack.id];
  root.setContent(async function (event) {
    if (runtimeCase.startsWith('xuelun')) {
      await event.trigger('phaseBegin');
    } else if (runtimeCase.startsWith('monkey')) {
      const player = q1.game.players[event.player === q1.game.players[0] ? 0 : 1];
      await player.useCard(runtimeAttack, [player], false).set('animate', false);
    } else {
      await q1.game.players[0].useCard(runtimeAttack, [q1.game.players[1]], false).set('animate', false);
    }
    window.runtimeDone = true;
  });
  window.runtimeRoot = root;
  root.start().catch(error => { window.runtimeError = error.stack; });
  return JSON.stringify({ case: fixture.case, playerCount: game.players.length,
    controlled, skillReady: ['monkey', 'kuwu', 'xuelunyang'].every(name => !!lib.skill[name]),
    equip0: game.players[0].getCards('e').map(c => c.name), initial: runtimeInitial });
})();
