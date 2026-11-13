new Promise((resolve, reject) => {
  let ticks = 0;
  const responses = [];
  const tick = () => {
    const { game, ui, _status } = q1;
    if (window.queue1Error) return reject(Error(window.queue1Error));
    if (queue1Done) return resolve(JSON.stringify({
      skill: queue1Skill, done: true, responses,
      players: game.players.map((p, seat) => ({ seat, hp: p.hp,
        hand: p.getCards('h').map(c => ({ name: c.name, suit: c.suit, rank: c.number })),
        equips: p.getCards('e').map(c => ({ name: c.name, suit: c.suit, rank: c.number })),
        skills: p.skills.filter(s => s.startsWith('ddd')),
        juxianBan: p.hasSkill('dddjuxian_ban'), xuanlunChoice: p.hasSkill('dddxuanlun_choose'),
      })),
      deckTop: [...ui.cardPile.childNodes].slice(0, 3).map(c => ({ name: c.name, suit: c.suit, rank: c.number })),
      skillEvents: game.getGlobalHistory('everything').slice(window.queue1HistoryStart).filter(e => e.skill === queue1Skill)
        .map(e => ({ name: e.name, skill: e.skill, trigger: e.triggername })),
    }));
    if (_status.paused && _status.event.name === 'chooseToRespond') {
      responses.push({ name: _status.event.name, player: game.players.indexOf(_status.event.player), decision: 'cancel' });
      _status.clicked = false;
      ui.click.cancel();
    }
    if (++ticks > 500) return reject(Error('probe timeout at ' + _status.event.name));
    setTimeout(tick, 20);
  };
  tick();
});
