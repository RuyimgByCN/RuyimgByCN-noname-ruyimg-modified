(async () => {
  const n = await import('/noname.js');
  const { game, lib, ui, _status, get } = n;
  const { default: EventManager } = await import('/noname/library/element/GameEvent/GameEventManager.ts');
  const { default: PauseManager } = await import('/noname/game/PauseManager.ts');
  const { default: skills } = await import('/extension/3D精选/character/skill.js');
  const { default: translate } = await import('/extension/3D精选/character/translate.js');
  const skill = window.queue1Skill || 'dddyuanzhen';
  const fixture = window.queue1Fixture;
  const create = (face, fallback) => face
    ? game.createCard(({ Duel: 'juedou', HorseMinus: 'chitu' })[face.kind] || face.name, face.suit.toLowerCase(), face.rank)
    : game.createCard(...fallback);
  if (!['dddyuanzhen', 'dddjuxian', 'dddxuanlun'].includes(skill)) throw Error('unknown probe skill');
  window.q1 = n;
  delete window.queue1Error;
  // Keep the original tutorial event parked; each probe uses a fresh upstream event queue.
  _status.eventManager = new EventManager();
  _status.pauseManager = new PauseManager();
  const root = game.createEvent('queue1Probe', false);
  window.queue1HistoryStart = game.getGlobalHistory('everything').length;
  _status.auto = false;
  _status.imchoosing = false;
  _status.gameStarted = true;
  _status.gameDrawed = true;
  _status.clicked = false;
  game.phaseNumber = 1;
  lib.config.auto_confirm = false;
  lib.config.touchscreen = false;
  lib.config.ignore_error = false;
  Object.assign(lib.skill, skills);
  Object.assign(lib.translate, translate);
  for (const name of Object.keys(skills)) game.finishSkill(name);
  lib.character.queue1_dummy = get.convertedCharacter(['male', 'wei', 8, []]);
  game.playerMap ??= {};
  for (const p of game.players) {
    if (!p.playerid) p.playerid = 'queue1_' + game.players.indexOf(p);
    game.playerMap[p.playerid] = p;
    p.removeSkill(Object.keys(p.tempSkills));
    p.uninit();
    p.init('queue1_dummy');
    for (const c of p.getCards('hej')) c.remove();
    p.storage = {};
    p.stat = [{ card: {}, skill: {} }];
  }
  const owner = game.players[0], target = game.players[2];
  game.swapControl(skill === 'dddyuanzhen' ? target : owner);
  owner.addSkill(skill);
  ui.cardPile.innerHTML = '';
  for (const face of fixture?.top || []) ui.cardPile.appendChild(create(face));
  for (let i = 0; i < 12; i++) ui.cardPile.appendChild(game.createCard('sha', 'diamond', 2));
  const first = create(fixture?.first, ['sha', 'club', 7]), second = create(fixture?.second, ['tao', 'heart', 3]);
  if (skill !== 'dddxuanlun') target.directgain(skill === 'dddjuxian' && !fixture?.second ? [first] : [first, second]);
  if (skill === 'dddjuxian' && fixture?.second) {
    target.node.handcards1.appendChild(first);
    target.node.handcards1.appendChild(second);
    if (target.getCards('h')[0] !== first || target.getCards('h')[1] !== second) throw Error('fixture hand order');
  }
  const attack = create(fixture?.attack, ['juedou', 'spade', 1]);
  const paid = create(fixture?.paid, ['sha', 'spade', 7]);
  if (skill === 'dddyuanzhen') owner.directgain([attack]);
  if (skill === 'dddxuanlun') target.directgain([attack]);
  if (skill === 'dddjuxian') {
    owner.directgain([paid]);
    target.directequip(create(fixture?.equip, ['chitu', 'heart', 5]));
  }
  root.name = 'phaseUse';
  root.player = skill === 'dddxuanlun' ? target : owner;
  _status.currentPhase = root.player;
  window.queue1Cards = { first, second, attack, paid };
  window.queue1Skill = skill;
  window.queue1Done = false;
  root.setContent(async function () {
    if (queue1Skill === 'dddyuanzhen') {
      await q1.game.players[0].useCard(queue1Cards.attack, [q1.game.players[2]], false).set('animate', false);
    } else if (queue1Skill === 'dddjuxian') {
      await q1.game.players[0].useSkill('dddjuxian_top', [queue1Cards.paid]);
    } else {
      await q1.game.players[2].useCard(queue1Cards.attack, [q1.game.players[0]], false).set('animate', false);
    }
    window.queue1Done = true;
  });
  window.queue1Root = root;
  root.start().catch(error => { window.queue1Error = error.stack; });
  return JSON.stringify({ skill, playerCount: game.players.length, cardReady: !!lib.card.juedou,
    distance: get.distance(owner, target), skillLoaded: owner.hasSkill(skill) });
})();
