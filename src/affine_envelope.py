"""
Reference implementation of the technique behind provable CU bounds:
fit an exact integer-affine cost model from measured samples, refuse to
call it a bound unless every sample matches it exactly, then price the
real worst case against a hard ceiling.

This is the general method, written small and dependency-free for
illustration. It is not Skew's production code, and the numbers in the
`__main__` block below are illustrative, not Skew's actual measured
coefficients.
"""

from dataclasses import dataclass


@dataclass(frozen=True)
class AffineEnvelope:
    slope: int
    intercept: int

    def cost_at(self, n: int) -> int:
        return self.slope * n + self.intercept


def fit_affine(samples: dict[int, int]) -> AffineEnvelope:
    """Fit an exact integer-affine model from two or more (n, cost) samples.

    Raises if the samples don't actually lie on a line — a curve fit that
    merely comes close isn't a bound, it's a guess with a trend line drawn
    through it.
    """
    ns = sorted(samples)
    if len(ns) < 2:
        raise ValueError("need at least two samples to fit a line")

    n0, n1 = ns[0], ns[1]
    delta = samples[n1] - samples[n0]
    step = n1 - n0
    if delta % step != 0:
        raise ValueError("samples don't fit an integer-affine model")

    envelope = AffineEnvelope(slope=delta // step, intercept=samples[n0] - (delta // step) * n0)
    verify_zero_residual(samples, envelope)
    return envelope


def verify_zero_residual(samples: dict[int, int], envelope: AffineEnvelope) -> None:
    """Every sample must match the fitted line exactly.

    If any measured point disagrees with the line, the series isn't affine
    over this range, and treating it as one would understate the real
    worst case somewhere you didn't sample.
    """
    for n, measured in samples.items():
        predicted = envelope.cost_at(n)
        if predicted != measured:
            raise ValueError(f"non-affine: n={n} measured={measured} predicted={predicted}")


def worst_case_cost(envelope: AffineEnvelope, n_max: int, ceiling: int) -> int:
    """Return the exact cost at n_max, and refuse to call it safe if it isn't."""
    cost = envelope.cost_at(n_max)
    if cost > ceiling:
        raise ValueError(f"over budget: {cost} > {ceiling} at n_max={n_max}")
    return cost


if __name__ == "__main__":
    # Illustrative samples only — not Skew's actual measured values.
    samples = {1: 17206, 10: 26206, 25: 41206, 40: 56206}
    envelope = fit_affine(samples)

    n_max, ceiling = 49, 1_400_000
    cost = worst_case_cost(envelope, n_max, ceiling)

    print(f"cost(n) = {envelope.slope} * n + {envelope.intercept}")
    print(f"cost at n_max={n_max}: {cost:,} CU ({cost / ceiling:.1%} of the 1.4M ceiling)")
