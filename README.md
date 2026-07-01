# Provable compute-unit ceilings on Solana

Most programs find their worst-case compute cost the hard way: in production, when a batch is bigger than whatever the test suite happened to cover. We didn't want to find out that way, so while building Skew's settlement engine we stopped treating CU as something you benchmark and hope, and started treating it as something you prove.

This repo is our writeup of how, and what it's worth to anyone else shipping performance-sensitive programs on Solana.

## The problem with benchmarking CU

The standard approach: run your instruction against a few representative account graphs, note the CU, add a margin, ship it. That works until someone submits a batch at the edge of what you tested — a bigger fill, a fuller order book, a longer leg count — and the real cost turns out to be a curve you never sampled.

The failure mode isn't rare. Cost in a lot of on-chain settlement logic scales with *something* — order count, leg count, account count — and if you only spot-checked the middle of that range, you have no idea what happens at the edge, right where mainnet traffic actually stresses the system.

## What we did instead

For every instruction whose cost scales with a count, we derived the exact relationship instead of sampling it. If settlement cost is a straight line in the number of orders in a batch, we don't estimate the slope from a few points — we measure enough points to prove it's a line, get the exact coefficients, and then check every single point up to our real operational cap, not just the ones that looked convenient.

The result is a closed-form bound, not an estimate: for a given instruction, we know its exact worst-case cost before it ever runs on mainnet, because we've already run it at the actual worst case and verified the formula holds with zero residual across the whole range — not against a synthetic benchmark, against the deployed binary.

## What that buys you

- **Our worst-case settlement path** — a full batch clear at our maximum supported order count — costs about 73k compute units. That's roughly 5% of everything Solana gives a single transaction.
- **Our worst-case cross-margin re-escrow**, maxed out at our leg cap, costs under 20k CU — about 1.5% of the ceiling.

Neither of those numbers is a "should be fine" estimate. Both are the actual measured cost at the actual worst-case input, with a proof that nothing past that point gets more expensive in a way we didn't already account for.

That matters because the alternative — finding out your real worst case during a liquidation cascade, a congested network, or a batch bigger than your test suite — is exactly when you can least afford the surprise.

## The general pattern, if you're building on Solana too

1. If an instruction's cost scales with a count, don't spot-check it — derive the closed form and verify it holds across the *entire* reachable range, especially right up against whatever cap you enforce.
2. Your real cap is whatever your worst-case formula says is safe at your maximum input, not whatever number felt conservative in a meeting.
3. "Verified against the deployed binary" is the bar. A number from a synthetic benchmark that never touched your actual compiled program isn't a bound, it's a guess with better production values.

## Try it

`src/affine_envelope.py` is a small, dependency-free reference implementation of the fit-and-verify step above — the same shape we run internally, with illustrative numbers instead of Skew's actual measured coefficients:

```
python3 src/affine_envelope.py
```

`tests/test_affine_envelope.py` checks the part that actually matters: that it accepts a genuinely affine series and rejects one that only looks like it is.

```
python3 tests/test_affine_envelope.py
```

We're not open-sourcing the settlement engine itself here — this is the methodology, not the implementation. If you want to talk about the parts we didn't include, find us at [skew.deals](https://skew.deals).
