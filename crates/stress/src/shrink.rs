//! Shrink a finding's trace to the shortest sequence that still breaks the
//! same rule: delta debugging (Zeller's ddmin) over the steps, each candidate
//! replayed on fresh subjects. Steps another kept step depends on (a cursor, a
//! `recorded_at`) are pinned back in before a candidate is tried.

use std::{collections::BTreeSet, sync::Arc, time::Duration};

use crate::{client::LedgerClient, finding::Finding, replay::replay, trace::Step};

/// How much a shrink may spend.
#[derive(Debug, Clone, Copy)]
pub struct Budget {
    /// Replays.
    pub attempts: u32,
    /// Wall time.
    pub timeout: Duration,
}

/// Close `keep` under the dependencies of its steps.
fn with_dependencies(all: &[Step], keep: &BTreeSet<usize>) -> Vec<Step> {
    let mut wanted: BTreeSet<usize> = keep.clone();
    loop {
        let before = wanted.len();
        let deps: Vec<usize> = all
            .iter()
            .filter(|s| wanted.contains(&s.index))
            .flat_map(|s| s.request.depends_on())
            .collect();
        wanted.extend(deps);
        if wanted.len() == before {
            break;
        }
    }
    all.iter()
        .filter(|s| wanted.contains(&s.index))
        .cloned()
        .collect()
}

/// ddmin over a predicate on step sets; `test` says whether a candidate still
/// fails. Generic so a unit test can drive it without a ledger.
pub async fn ddmin<F, Fut>(all: &[Step], mut test: F, budget: Budget) -> (Vec<Step>, u32, bool)
where
    F: FnMut(Vec<Step>) -> Fut,
    Fut: Future<Output = bool>,
{
    let started = std::time::Instant::now();
    let mut attempts = 0u32;
    let mut current: Vec<usize> = all.iter().map(|s| s.index).collect();
    let mut n = 2usize;
    let mut exhausted = false;
    while current.len() >= 2 {
        if attempts >= budget.attempts || started.elapsed() >= budget.timeout {
            exhausted = true;
            break;
        }
        let chunk = current.len().div_ceil(n);
        let chunks: Vec<Vec<usize>> = current.chunks(chunk).map(<[usize]>::to_vec).collect();
        let mut reduced = false;
        // Try each complement: everything but one chunk.
        for i in 0..chunks.len() {
            if attempts >= budget.attempts || started.elapsed() >= budget.timeout {
                exhausted = true;
                break;
            }
            let keep: BTreeSet<usize> = chunks
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .flat_map(|(_, c)| c.iter().copied())
                .collect();
            let candidate = with_dependencies(all, &keep);
            if candidate.len() >= current.len() {
                continue;
            }
            attempts += 1;
            let indexes: Vec<usize> = candidate.iter().map(|s| s.index).collect();
            if test(candidate).await {
                current = indexes;
                n = (n - 1).max(2);
                reduced = true;
                break;
            }
        }
        if exhausted {
            break;
        }
        if !reduced {
            if n >= current.len() {
                break;
            }
            n = (n * 2).min(current.len());
        }
    }
    let keep: BTreeSet<usize> = current.into_iter().collect();
    (with_dependencies(all, &keep), attempts, !exhausted)
}

/// Shrink `finding` against `client`. The finding comes back with its trace
/// replaced when a shorter one reproduces, and a note either way.
pub async fn shrink(
    mut finding: Finding,
    client: Arc<dyn LedgerClient>,
    budget: Budget,
    skew: Duration,
    tolerate: &[String],
) -> Finding {
    let invariant = finding.invariant.clone();
    let signature = finding.signature.clone();
    // The rule must break on a fresh replay of the whole trace first, else
    // shrinking has nothing to hold on to.
    let whole = replay(&finding.trace, client.as_ref(), skew, tolerate).await;
    if !whole.reproduces(&invariant, &signature) {
        finding.shrink_note = Some(match whole.first_of(&invariant) {
            Some(v) => format!(
                "not reproduced on a fresh replay (the rule broke differently: {})",
                v.message
            ),
            None => "not reproduced on a fresh replay".into(),
        });
        return finding;
    }
    let tolerate: Vec<String> = tolerate.to_vec();
    let (steps, attempts, complete) = ddmin(
        &finding.trace,
        |candidate: Vec<Step>| {
            let client = Arc::clone(&client);
            let invariant = invariant.clone();
            let signature = signature.clone();
            let tolerate = tolerate.clone();
            async move {
                replay(&candidate, client.as_ref(), skew, &tolerate)
                    .await
                    .reproduces(&invariant, &signature)
            }
        },
        budget,
    )
    .await;
    if steps.len() < finding.trace.len() {
        finding.trace = steps;
        finding.shrunk = complete;
        finding.shrink_note = Some(format!(
            "{} of {} steps after {attempts} replays{}",
            finding.trace.len(),
            finding.original_len,
            if complete { "" } else { "; budget exhausted" }
        ));
    } else {
        finding.shrunk = complete;
        finding.shrink_note = Some(format!(
            "every step is needed ({attempts} replays{})",
            if complete { "" } else { "; budget exhausted" }
        ));
    }
    finding
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trace::{Request, SubjectRef};

    fn step(index: usize, path: &str) -> Step {
        Step {
            index,
            request: Request::Retract {
                subject: SubjectRef::Own,
                path: path.into(),
                source: 2,
                origin: serde_json::json!({}),
            },
            response: None,
            error: None,
            at_ms: 0.0,
            tolerated: false,
        }
    }

    #[tokio::test]
    async fn ddmin_finds_the_two_steps_that_matter() {
        let all: Vec<Step> = (0..40).map(|i| step(i, &format!("p.{i}"))).collect();
        let failing = |c: &[Step]| {
            let has = |i: usize| c.iter().any(|s| s.index == i);
            has(7) && has(31)
        };
        let (steps, attempts, complete) = ddmin(
            &all,
            |c: Vec<Step>| std::future::ready(failing(&c)),
            Budget {
                attempts: 500,
                timeout: Duration::from_secs(5),
            },
        )
        .await;
        assert!(complete);
        assert_eq!(
            steps.iter().map(|s| s.index).collect::<Vec<_>>(),
            vec![7, 31]
        );
        assert!(attempts < 200, "{attempts}");
    }

    #[tokio::test]
    async fn dependencies_are_pinned_back_in() {
        let mut all: Vec<Step> = (0..6).map(|i| step(i, &format!("p.{i}"))).collect();
        all[5].request = Request::History {
            subject: SubjectRef::Own,
            paths: vec![],
            sources: vec![],
            scopes: vec![],
            limit: 0,
            cursor_from: None,
            at: Some(crate::trace::TimeRef::RecordedAtOf(2)),
        };
        let (steps, _, _) = ddmin(
            &all,
            |c: Vec<Step>| std::future::ready(c.iter().any(|s| s.index == 5)),
            Budget {
                attempts: 100,
                timeout: Duration::from_secs(5),
            },
        )
        .await;
        assert_eq!(
            steps.iter().map(|s| s.index).collect::<Vec<_>>(),
            vec![2, 5]
        );
    }

    #[tokio::test]
    async fn the_budget_bounds_the_work() {
        let all: Vec<Step> = (0..64).map(|i| step(i, &format!("p.{i}"))).collect();
        let (steps, attempts, complete) = ddmin(
            &all,
            |c: Vec<Step>| std::future::ready(c.iter().any(|s| s.index == 63)),
            Budget {
                attempts: 3,
                timeout: Duration::from_secs(5),
            },
        )
        .await;
        assert_eq!(attempts, 3);
        assert!(!complete);
        assert!(steps.iter().any(|s| s.index == 63));
    }
}
