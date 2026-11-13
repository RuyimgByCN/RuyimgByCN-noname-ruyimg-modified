new Promise((resolve, reject) => {
  let ticks = 0;
  const responses = [];
  const tick = () => {
    const { game, ui, _status } = q1;
    if (runtimeError) return reject(Error(runtimeError));
    if (runtimeDone) return resolve(JSON.stringify({ case: runtimeCase, settled: true, responses,
      initial: runtimeInitial,
      players: game.players.map((p, seat) => ({ seat, hp: p.hp, max_hp: p.maxHp,
        hand: p.getCards('h').map(c => ({ name: c.name, suit: c.suit, rank: c.number })),
        equips: p.getCards('e').map(c => ({ name: c.name, suit: c.suit, rank: c.number })),
        skills: p.getSkills().slice(), nonEquipmentSkills: p.getSkills(null, false, false).slice(),
        tempSkills: p.tempSkills, additionalSkills: p.additionalSkills,
      })),
      discard: [...ui.discardPile.childNodes].filter(c => c.name).map(c => ({ name: c.name, suit: c.suit, rank: c.number })),
      skillEvents: game.getGlobalHistory('everything').slice(runtimeHistoryStart)
        .filter(e => e.skill && ['monkey', 'kuwu', 'xuelunyang'].includes(e.skill))
        .map(e => ({ name: e.name, skill: e.skill, trigger: e.triggername })),
    }));
    const event = _status.event;
    if (_status.paused && ['chooseToRespond', 'chooseToUse'].includes(event?.name)) {
      responses.push({ name: event.name, player: game.players.indexOf(event.player), decision: 'cancel' });
      _status.clicked = false;
      ui.click.cancel();
    }
    if (++ticks > 600) return reject(Error('finish timeout at ' + event?.name));
    setTimeout(tick, 20);
  };
  tick();
});
