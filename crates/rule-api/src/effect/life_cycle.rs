//! Life cycle effect definitions.

macro_rules! variants {
    (EnterDying [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Enter dying.
    EnterDying {
        /// Player.
        player: PlayerId,
    },
    /// Kill player.
    KillPlayer {
        /// Victim.
        player: PlayerId,
        /// Killer if any.
        killer: PlayerId,
    },
        ] }
    };
    (RequestDyingSave [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Open the deterministic multi-rescuer save queue for a dying player.
    ///
    /// Appended so forced dying-trigger choices can suspend this request without losing its
    /// continuation metadata.
    RequestDyingSave {
        /// Player currently dying.
        player: PlayerId,
        /// Damage source used if the player dies.
        source: PlayerId,
    },
        ] }
    };
    (ContinueDyingSave [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// Continue a deterministic multi-player dying-save queue. Engine-internal and appended for
    /// serialized stability.
    ContinueDyingSave {
        /// Player currently dying.
        player: PlayerId,
        /// Damage source used if the player dies.
        source: PlayerId,
        /// Remaining responders in clockwise order.
        responders: Vec<PlayerId>,
    },
    /// Restore a living character card to its normal state: unlinked and face-up.
    ///
    /// Appended for serialized stability.
    RestoreCharacterState {
        /// Player whose character card state is restored.
        player: PlayerId,
    },
        ] }
    };
}
pub(super) use variants;
