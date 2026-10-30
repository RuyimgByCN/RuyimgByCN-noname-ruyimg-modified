//! Durable room control; clocks remain outside the deterministic engine.
use super::*;
use engine_native::view::PolicyInput;
use engine_native::ChoiceView;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeTimeoutState {
    pub choice_timeout_secs: u8,
    pub managed_seats: Vec<u8>,
    pub deadline: Option<NativeChoiceDeadline>,
}
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeChoiceDeadline {
    pub choice_id: u64,
    pub observer: u8,
    pub deadline_unix_ms: u64,
}

pub(crate) fn unix_ms() -> Result<u64, String> {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| "server clock before Unix epoch")?;
    u64::try_from(elapsed.as_millis()).map_err(|_| "server clock overflow".into())
}
pub fn validate_timeout_secs(secs: u8) -> Result<(), String> {
    if [10, 15, 30, 60, 90].contains(&secs) {
        Ok(())
    } else {
        Err("choice_timeout_secs must be 10, 15, 30, 60 or 90".into())
    }
}

/// Minimum legal candidates from the seat's authorized projection, without planning.
fn baseline(input: &PolicyInput) -> Option<GameAction> {
    match &input.choice {
        Some(ChoiceView::Own {
            id, options, min, ..
        }) => {
            let mut selection = Vec::new();
            for option in options {
                if selection.len() == *min {
                    break;
                }
                if !selection.contains(option) {
                    selection.push(*option);
                }
            }
            Some(GameAction::Answer {
                choice_id: *id,
                selection,
            })
        }
        Some(ChoiceView::Other { .. }) => None,
        None => Some(GameAction::NextPhase),
    }
}

impl NativeRoomSession {
    pub fn set_choice_timeout_secs(&mut self, secs: u8) -> Result<(), String> {
        validate_timeout_secs(secs)?;
        self.timeout.choice_timeout_secs = secs;
        Ok(())
    }
    pub fn timeout_state(&self) -> &NativeTimeoutState {
        &self.timeout
    }
    pub(crate) fn sync_deadline(&mut self) -> Result<(), String> {
        let current = self.engine.open_choice();
        if current.map(|c| (c.id, c.observer))
            == self
                .timeout
                .deadline
                .as_ref()
                .map(|d| (d.choice_id, d.observer))
        {
            return Ok(());
        }
        self.timeout.deadline = match current {
            Some(c) => Some(NativeChoiceDeadline {
                choice_id: c.id,
                observer: c.observer,
                deadline_unix_ms: unix_ms()?
                    .checked_add(u64::from(self.timeout.choice_timeout_secs) * 1000)
                    .ok_or("deadline overflow")?,
            }),
            None => None,
        };
        Ok(())
    }
    pub(crate) fn validate_timeout(&self) -> Result<(), String> {
        validate_timeout_secs(self.timeout.choice_timeout_secs)?;
        if self
            .timeout
            .managed_seats
            .iter()
            .any(|s| *s >= self.max_seats)
            || self.timeout.managed_seats.windows(2).any(|s| s[0] >= s[1])
        {
            return Err("invalid managed seats".into());
        }
        if self.engine.open_choice().map(|c| (c.id, c.observer))
            != self
                .timeout
                .deadline
                .as_ref()
                .map(|d| (d.choice_id, d.observer))
            || self
                .timeout
                .deadline
                .as_ref()
                .is_some_and(|d| d.deadline_unix_ms == 0)
        {
            return Err("deadline does not match current choice".into());
        }
        Ok(())
    }
    pub(crate) fn choice_for(
        &self,
        role: RoomRole,
        player: Option<PlayerId>,
    ) -> Option<engine_api::ChoiceRequest> {
        let mut choice = crate::native_view::choice_for(&self.engine, role, player)?;
        // A trusted Referee retains complete choice metadata, but no control capability.
        choice.deadline_unix_ms = self
            .timeout
            .deadline
            .as_ref()
            .filter(|d| d.choice_id == u64::from(choice.choice_id.raw()))
            .map(|d| d.deadline_unix_ms);
        if choice.deadline_unix_ms.is_some() {
            choice.timeout_policy = engine_api::TimeoutPolicy::Standard;
        }
        Some(choice)
    }
    pub(crate) fn automatic_command(
        &self,
        now: u64,
        allow_managed: bool,
    ) -> Option<(u8, GameAction, bool)> {
        let seat = self
            .engine
            .open_choice()
            .map(|c| c.observer)
            .unwrap_or(self.engine.turn().current);
        let managed = self.timeout.managed_seats.contains(&seat);
        let expired = self
            .timeout
            .deadline
            .as_ref()
            .is_some_and(|d| d.deadline_unix_ms <= now);
        if managed && !allow_managed {
            return None;
        }
        if !expired && !managed {
            return None;
        }
        if self.engine.outcome().is_some() && self.engine.open_choice().is_none() {
            return None;
        }
        let action = baseline(&self.engine.ai_session(seat).input())?;
        let takeover = expired && !managed;
        let action = match action {
            GameAction::Answer {
                choice_id,
                selection,
            } if takeover => GameAction::TimeoutAction {
                choice_id,
                selection,
            },
            other => other,
        };
        Some((seat, action, takeover))
    }
    pub(crate) fn submit_automatic(
        &mut self,
        seat: u8,
        action: GameAction,
        takeover: bool,
    ) -> Result<CommandResult, String> {
        let result = self.submit_native(seat, action, self.engine.revision())?;
        if matches!(result, CommandResult::Accepted { .. }) && takeover {
            self.timeout.managed_seats.push(seat);
            self.timeout.managed_seats.sort_unstable();
            self.timeout.managed_seats.dedup();
        }
        Ok(result)
    }
}
