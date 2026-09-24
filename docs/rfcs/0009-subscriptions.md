---
title: Subscriptions
status: open
date: 2026-09-24
public: false
lab: quietpager
summary: How Quiet Pager charges — Stripe Checkout, Billing and the customer portal through InOrbit d.o.o., entitlements in the studio service, EU VAT and Croatian fiscalization — and what must be decided before any of it is built.
---

## Problem

The free layer (kata 1 and the Radar) is the marketing; the paid layer pays for the time
it takes to build katas, levels and the agent. Charging brings obligations the rest of the
site does not have: the site promises no analytics, no tracking cookie and nothing embedded
from a third party; EU consumers pay VAT at their own country's rate; and in Croatia a
payment by card counts as a cash-type receipt for fiscalization purposes, which may or may
not apply to online card payments taken through a payment processor. The platform's rule
that services never verify tokens (Envoy does) also meets a processor that signs its own
webhooks.

## Proposal

- **Stripe, hosted.** Stripe Checkout for the subscription, Stripe Billing for renewals,
  the hosted customer portal for cards, invoices and cancellation. No Stripe.js on
  inorbit.hr: the visitor is sent to Stripe's pages and back, so the no-third-party promise
  holds and `/legal/` gains only a sentence naming Stripe as the processor and what it
  receives.
- **Entitlements in studio.** Stripe webhooks (`checkout.session.completed`,
  `customer.subscription.updated`, `.deleted`, `invoice.paid`, `invoice.payment_failed`)
  arrive at one public route on the API host; the handler verifies Stripe's signature over
  the raw body and stores the subscription state against the account's subject. This is
  the one exception to "services never verify tokens": it is a processor's HMAC, not one
  of our credentials, and it is documented here and in the route's comment.
- **What is paid.** Free: kata 1, all its levels, and the Radar. Paid: every other kata and
  level, the Radar archive, and later the agent. Reports are never gated: anyone may run
  `qp grade` on anything they have.
- **Keys** only in a Kubernetes Secret created by a new mise task (`stripe:secrets`), never
  in files; the webhook secret likewise.
- **Tax and receipts.** Stripe Tax computes EU VAT per customer (the OSS scheme for B2C
  digital services), and business customers with a VAT ID are reverse-charged. The
  finance service already issues Croatian e-invoices; whether online card receipts must
  also be fiscalized is the accountant's call, recorded here before launch.

## Alternatives considered

**Paddle or Lemon Squeezy as merchant of record.** They take VAT and receipts off the
company's hands for a higher fee, but invoices would come from them, not InOrbit, and the
finance service's bookkeeping is already built around InOrbit issuing. Kept as the fallback
if the fiscalization answer makes self-issuing too heavy.

**Stripe Elements on our own page.** A nicer checkout, but it loads Stripe's script on
inorbit.hr and breaks the site's promise. Rejected.

**One-time lifetime purchase.** Simpler, but it does not pay for new katas. Rejected as
the main offer; possibly a limited early-supporter tier.

## Decision

Nothing is built until these are decided and written here:

1. The price, monthly and yearly, and whether there is an early-supporter tier.
2. Stripe Tax on, and the OSS registration confirmed for InOrbit d.o.o.
3. The accountant's answer on fiscalizing online card receipts.
4. The legal and terms changes, drafted and reviewed.

Then build after studio and accounts (RFC 0007, track C) exist.

## Publication

Stays a draft until the paid tier opens. At launch, `/legal/` and `/terms/` change in the
same commit as the first paid route, and the Quiet Pager section shows the price.

## Status log

- 2026-09-24: opened with the open decisions listed.
