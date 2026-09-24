# Claim Confidence: an interview simulator grounded in the candidate's own code

> Status: idea, 2026-09-23. Not built. Working names so far: "Interview Prep Engineer",
> "CV Bullshit Detector", "Claim Confidence".

## The problem

A CV is a list of claims. An interview tests whether the candidate can talk about those
claims for an hour. Neither is grounded in the one thing that would settle it: the code
the candidate actually wrote and the decisions visible in its history. Candidates prepare
by rehearsing generic questions; interviewers ask generic questions; both sides know the
signal is weak.

## The idea

Connect four sources that today never meet:

1. **The CV.** Every line becomes a claim: "designed a JSON-RPC load balancer in Rust",
   "took a fleet migration live with the SRE team".
2. **The candidate's source code.** Repositories the candidate authorises (GitHub first).
   Parsed, not just read: tree-sitter or AST per language, a dependency graph, the commit
   history (who changed what, when, how the design moved), the design documents and
   RFCs sitting next to the code.
3. **The interview itself.** An adaptive, adversarial interviewer in the role of a Staff
   engineer. It asks about the candidate's own systems, follows up where the answer is
   vague, and pushes on the places where the code and the claim disagree. Voice in and
   voice out, so it feels like an interview and not a form.
4. **The target job description.** What the role actually asks for, so the questions and
   the report are about that role and not about interviews in general.

The output is not a score. It is:

- **Claim confidence** per CV line: what the code, the commits and the answers support,
  what they contradict, and what could not be checked at all.
- **A skill-gap map** against the job description: where the candidate is strong, where
  the evidence is thin, and what to prepare.
- **A prep plan**: the questions this candidate should expect and cannot yet answer well,
  with the evidence from their own repositories to anchor the answers.

## Why it could be defensible

Every mock-interview product runs the same interview for everyone. The thing that
compounds here is the **Candidate Engineering Model**: a per-person model of what they
built, how they reason about it, and how that has changed over sessions. Each session
makes the next one more specific. A competitor can copy the interviewer prompt; it cannot
copy the model of a candidate it has never seen.

## Market, as surveyed on the day

Names found in one afternoon of looking, so verify before a pitch: DojoPrep, DeepPrep,
Praxto, VirtualInterview.ai, DeepInterview (open source), Aced and Exponent (around
$12 a month), Yoodli (around $8 to $20 a month). All of them coach delivery or run generic
question banks. None grounds the interview in the candidate's own code.

## Who pays

- **B2C first.** A candidate preparing for a specific role pays for a specific, honest
  rehearsal. Pricing signals from the survey say a low monthly fee is the norm.
- **B2B later, carefully.** A hiring team could use the same engine to prepare an
  interview panel for a candidate. This side has real consent and fairness problems:
  the candidate's code is theirs, the report must never become a hidden filter, and GDPR
  applies to every byte. Whatever ships here starts from the candidate's explicit
  authorisation and a copy of every report for the candidate.

## V1 scope

CV upload, one authorised GitHub account, one job description, a voice interview with the
adaptive interviewer, the claim-confidence and skill-gap report. Nothing else.

## Risks stated plainly

- **Consent and privacy of code.** Source code is the most sensitive input a person can
  hand over. It is processed for that person, kept only as long as they say, and never
  used to train anything shared.
- **Hallucinated judgment.** An LLM will confidently misjudge a claim. Every verdict must
  cite the evidence it rests on, and the eval pipeline in [personal-llm.md](personal-llm.md)
  exists so the judgment quality is measured, not assumed.
- **Gaming.** A candidate can rehearse against the tool until the answers are fluent. That
  is the product working as intended for B2C, and a reason the B2B report must show
  evidence rather than a score.
- **Being used against candidates.** See B2B above. The design constraint is that the
  candidate always sees what an employer would see.

## Where it starts

On the owner. The first candidate is a twenty-year engineering history and the
repositories this platform lives in; the interviewer is built and tuned on that, and the
results become the case studies listed in [personal-llm.md](personal-llm.md). Nothing is
pitched before the tool has caught the owner out at least once.
