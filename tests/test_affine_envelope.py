import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "src"))

from affine_envelope import AffineEnvelope, fit_affine, worst_case_cost


def test_fits_a_clean_affine_series():
    samples = {1: 17206, 10: 26206, 25: 41206, 40: 56206}
    envelope = fit_affine(samples)
    assert envelope.slope == 1000
    assert envelope.intercept == 16206


def test_rejects_a_non_affine_series():
    # n=25 breaks the line on purpose. A fit that silently averages this
    # away as "close enough" would understate the real worst case.
    samples = {1: 17206, 10: 26206, 25: 41300, 40: 56206}
    try:
        fit_affine(samples)
        assert False, "a non-affine series must not fit cleanly"
    except ValueError:
        pass


def test_worst_case_within_budget():
    envelope = AffineEnvelope(slope=1000, intercept=16206)
    cost = worst_case_cost(envelope, n_max=49, ceiling=1_400_000)
    assert cost == 65206


def test_worst_case_over_budget_is_refused():
    envelope = AffineEnvelope(slope=1000, intercept=16206)
    try:
        worst_case_cost(envelope, n_max=2000, ceiling=1_400_000)
        assert False, "a genuinely over-budget case must not report GO"
    except ValueError:
        pass


if __name__ == "__main__":
    test_fits_a_clean_affine_series()
    test_rejects_a_non_affine_series()
    test_worst_case_within_budget()
    test_worst_case_over_budget_is_refused()
    print("4/4 passed")
