new Promise((resolve, reject) => {
  let ticks = 0;
  const tick = () => {
    if (window.queue1Done) return resolve(JSON.stringify({ done: true,
      players: game.players.map((p, seat) => ({ seat, hp: p.hp,
        hand: p.getCards('h').map(c => ({ name: c.name, suit: c.suit, rank: c.number, wunature: c.wunature || null })),
        equips: p.getCards('e').map(c => ({ name: c.name, suit: c.suit, rank: c.number, wunature: c.wunature || null })),
      })),
    }));
    if (++ticks > 500) return reject(Error('finish timeout at ' + _status.event.name));
    setTimeout(tick, 20);
  };
  tick();
});
