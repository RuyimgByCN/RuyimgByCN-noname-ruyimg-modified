(() => {
  game.pause();
  _status.paused2 = false;
  _status.over = false;
  _status.auto = false;
  _status.gameStarted = true;
  _status.gameDrawed = true;
  game.phaseNumber = 1;
  lib.config.auto_confirm = false;
  lib.config.touchscreen = false;
  lib.config.errstop = true;
  lib.character.queue1_dummy = ['male', 'wei', 8, []];
  for (const p of game.players) {
    if (!p.playerid) p.playerid = 'queue1_' + game.players.indexOf(p);
    p.init('queue1_dummy');
    for (const c of p.getCards('hej')) c.remove();
    p.wunature = undefined;
    p.storage = {};
  }
  const [source, owner] = game.players;
  game.me = game.players.find(p => p.node.handcards1 === ui.handcards1) || source;
  game.swapControl(owner);
  const first = game.createCard('sha', 'club', 7);
  const second = game.createCard('tao', 'heart', 3);
  owner.directgain([first, second]);
  owner.wunature = 'wood';
  owner.addSkill('_shengke');
  const attack = game.createCard('sha', 'spade', 9);
  attack.wunature = 'metal';
  source.directgain([attack]);
  const root = game.createEvent('queue1Probe', false);
  root.parent = null;
  root.setContent(function () { window.queue1Done = true; event.finish(); game.pause(); });
  _status.event = root;
  _status.currentPhase = source;
  window.queue1Done = false;
  window.queue1Cards = { first, second, attack };
  window.queue1Root = root;
  const use = source.useCard(attack, [owner], false);
  use.animate = false;
  game.resume();
  return JSON.stringify({ event: _status.event.name, step: _status.event.step, paused: _status.paused,
    targetHp: owner.hp, hand: owner.getCards('h').map(c => c.name), done: window.queue1Done });
})();
