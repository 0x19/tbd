//! The moves a caller may make, and what each one is worth.
//!
//! This module is the security boundary of the playground. The wire carries a
//! move and, at most, an instance name — never a duration, a rate or an
//! address — so there is nothing to clamp at runtime: the parameters below are
//! the only ones that exist. Two of the chaos behaviours are deliberately
//! unreachable from here. `Hang` parks a request on `pending()` and never
//! returns it, and `DelayedFailure` nests another behaviour inside itself
//! without bound; neither belongs on a public surface.

use std::time::Duration;

use tbd_common::fault::{Behavior, ErrorKind};
use tbd_proto::playground::v1::Move;

/// What one move does to the sandbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Play {
    /// What it costs from the shared budget.
    pub cost: u32,
    /// How long it lasts before the world heals itself.
    pub ttl: Duration,
    /// The chaos kind it applies to, or `None` for any faultable instance.
    pub kind: Option<&'static str>,
}

/// The whole move list. A `Move` outside it has no `Play` and is refused.
#[must_use]
pub fn play(mv: Move) -> Option<Play> {
    match mv {
        Move::Latency => Some(Play {
            cost: 1,
            ttl: Duration::from_secs(30),
            kind: None,
        }),
        Move::Errors => Some(Play {
            cost: 2,
            ttl: Duration::from_secs(30),
            kind: None,
        }),
        Move::Kill => Some(Play {
            cost: 3,
            ttl: Duration::from_secs(20),
            kind: None,
        }),
        // Only the ledger has a store to fail.
        Move::StallStore => Some(Play {
            cost: 2,
            ttl: Duration::from_secs(20),
            kind: Some("ledger"),
        }),
        Move::Unspecified => None,
    }
}

/// The behaviour a move injects. `Kill` has none — it stops the instance
/// instead of making it misbehave.
#[must_use]
pub fn behavior(mv: Move) -> Option<Behavior> {
    match mv {
        Move::Latency => Some(Behavior::Slow {
            latency: Duration::from_millis(400),
            jitter: Duration::from_millis(80),
        }),
        Move::Errors => Some(Behavior::Error {
            kind: ErrorKind::Unavailable,
            rate: 0.35,
            message: "injected from the playground".to_owned(),
        }),
        // The ledger is the one thing with no replica to fail over to, so this
        // is the one move nothing routes around. It is rated to sit just under
        // the error budget on its own — enough to hurt, not enough to win with
        // — because a move that always breaches alone is the whole game.
        Move::StallStore => Some(Behavior::Error {
            kind: ErrorKind::Internal,
            rate: 0.015,
            message: "the store is having a moment".to_owned(),
        }),
        Move::Kill | Move::Unspecified => None,
    }
}

/// What the move does, for a client that cannot read the enum.
#[must_use]
pub fn describe(mv: Move) -> &'static str {
    match mv {
        Move::Latency => "400 ms of latency, give or take 80",
        Move::Errors => "a third of its requests fail",
        Move::Kill => "stopped, and back on its feet shortly",
        Move::StallStore => "its store drops the odd write",
        Move::Unspecified => "nothing",
    }
}

/// The longest any name may be on the leaderboard.
pub const MAX_ACTOR: usize = 24;

/// Trim a caller-supplied name down to something safe to render and store:
/// one line, no control characters, no runs of whitespace, capped. Anything
/// left empty becomes `anonymous`, so a score always has an author.
#[must_use]
pub fn actor(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len().min(MAX_ACTOR));
    let mut space = false;
    for ch in raw.chars() {
        if out.chars().count() >= MAX_ACTOR {
            break;
        }
        if ch.is_whitespace() {
            // One space between words, never a newline or a tab.
            if !out.is_empty() {
                space = true;
            }
            continue;
        }
        if ch.is_control() {
            continue;
        }
        if space {
            out.push(' ');
            space = false;
        }
        out.push(ch);
    }
    let trimmed = out.trim_end();
    if trimmed.is_empty() {
        "anonymous".to_owned()
    } else {
        trimmed.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_real_move_has_a_play_and_unspecified_has_none() {
        for mv in [Move::Latency, Move::Errors, Move::Kill, Move::StallStore] {
            let play = play(mv).expect("a real move is playable");
            assert!(play.cost > 0, "{mv:?} must cost something");
            assert!(play.ttl.as_secs() > 0, "{mv:?} must expire");
            assert_ne!(describe(mv), "nothing");
        }
        assert!(play(Move::Unspecified).is_none());
        assert!(behavior(Move::Unspecified).is_none());
    }

    #[test]
    fn nothing_injectable_hangs_or_nests() {
        // The two behaviours that must never reach a public surface.
        for mv in [Move::Latency, Move::Errors, Move::Kill, Move::StallStore] {
            assert!(
                !matches!(
                    behavior(mv),
                    Some(Behavior::Hang | Behavior::DelayedFailure { .. })
                ),
                "{mv:?} injects an unbounded behaviour"
            );
        }
    }

    #[test]
    fn injected_latency_and_error_rates_stay_modest() {
        let Some(Behavior::Slow { latency, jitter }) = behavior(Move::Latency) else {
            panic!("latency is a slow behaviour")
        };
        assert!(latency <= Duration::from_millis(750), "{latency:?}");
        assert!(jitter <= Duration::from_millis(100), "{jitter:?}");

        for mv in [Move::Errors, Move::StallStore] {
            let Some(Behavior::Error { rate, .. }) = behavior(mv) else {
                panic!("{mv:?} is an error behaviour")
            };
            assert!((0.0..=0.5).contains(&rate), "{mv:?} injects {rate}");
        }
    }

    #[test]
    fn the_move_nothing_routes_around_stays_under_the_budget() {
        // Every other move lands on a kind with a replica behind a balancer, so
        // its damage is bounded by how fast the balancer ejects. This one lands
        // on the ledger, which has none, so the rate itself is the only bound.
        // Ledger operations are half the traffic and the objective allows 1%.
        let Some(Behavior::Error { rate, .. }) = behavior(Move::StallStore) else {
            panic!("stall store is an error behaviour")
        };
        let share_of_all_traffic = rate * 0.5;
        assert!(
            share_of_all_traffic < 0.01,
            "a move that breaches on its own ends the game in one click: {share_of_all_traffic}"
        );
    }

    #[test]
    fn only_the_ledger_has_a_store_to_stall() {
        assert_eq!(play(Move::StallStore).and_then(|p| p.kind), Some("ledger"));
        assert_eq!(play(Move::Latency).and_then(|p| p.kind), None);
    }

    #[test]
    fn actor_names_are_one_tidy_line() {
        assert_eq!(actor("  nevio  "), "nevio");
        assert_eq!(actor("two\nlines\there"), "two lines here");
        assert_eq!(actor(""), "anonymous");
        assert_eq!(actor("   \t "), "anonymous");
        assert_eq!(actor("\u{7}bell"), "bell");
        let long = actor(&"x".repeat(200));
        assert_eq!(long.chars().count(), MAX_ACTOR);
        // Multi-byte names are capped by characters, not bytes, and never split one.
        let emoji = actor(&"ø".repeat(100));
        assert_eq!(emoji.chars().count(), MAX_ACTOR);
    }
}
