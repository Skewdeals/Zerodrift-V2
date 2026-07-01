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

## This isn't just a Python script

A fit-and-verify function on its own doesn't prove anything about Solana — it's just arithmetic. So `demo-program/` is a small, from-scratch Solana program (not Skew's settlement engine — a new, minimal one, written for this repo) with one instruction whose cost scales with a count `n`, and a `mollusk-svm` harness that loads the actual compiled BPF binary and measures real `compute_units_consumed` for every `n` from 1 to 49 — not a handful of spot-checked points.

```
cd demo-program/program && cargo build-sbf   # produces the real .so
cd ../harness && cargo run --release         # runs it through mollusk-svm, prints n,cu for every n
```

That run is what produced `real_measurements.csv` — 49 lines, one real measurement per line. The harness itself asserts the slope holds at every consecutive step as it measures. Then `verify_real_measurements.py` re-derives the same model independently, in a different language, from the raw CSV:

```
python3 verify_real_measurements.py
```

Two independent implementations, agreeing on the same real BPF-execution data: `cost(n) = 7 * n + 342`, zero residual across all 49 points. That's the actual technique, not a description of it — clone this, build it yourself, and check our arithmetic against your own run.

## Try the reference implementation on its own

`src/affine_envelope.py` is the same fit-and-verify logic used above, as a small dependency-free module you can read end to end:

```
python3 src/affine_envelope.py       # illustrative numbers
python3 tests/test_affine_envelope.py  # confirms it accepts a real affine series and rejects a fake one
```

We're not open-sourcing Skew's settlement engine itself here — that program is a purpose-built demo, not our production code. If you want to talk about the parts we didn't include, find us at [skew.deals](https://skew.deals).
