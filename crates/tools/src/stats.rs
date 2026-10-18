//! Statistics for the arena comparison (PLAN.md §6, layer 2B): the
//! two-proportion z test on win rates and the two-sample
//! Kolmogorov–Smirnov test on times to kill.
//!
//! This is tooling, not simulation: it compares our engine's outcomes with
//! the stock oracle's, so floating point is fine here.

/// The standard normal cumulative distribution function Φ(x).
///
/// Φ(x) = ½ · erfc(−x / √2), with erfc from the Chebyshev fit of Numerical
/// Recipes (`erfcc`, fractional error below 1.2 × 10⁻⁷ everywhere), far
/// finer than any decision at our significance levels needs.
pub fn normal_cdf(x: f64) -> f64 {
    0.5 * erfc(-x / std::f64::consts::SQRT_2)
}

/// The complementary error function, Numerical Recipes' `erfcc` fit.
fn erfc(x: f64) -> f64 {
    let z = x.abs();
    let t = 1.0 / (1.0 + 0.5 * z);
    let poly = -z * z - 1.265_512_23
        + t * (1.000_023_68
            + t * (0.374_091_96
                + t * (0.096_784_18
                    + t * (-0.186_288_06
                        + t * (0.278_868_07
                            + t * (-1.135_203_98
                                + t * (1.488_515_87 + t * (-0.822_152_23 + t * 0.170_872_77))))))));
    let ans = t * poly.exp();
    if x >= 0.0 { ans } else { 2.0 - ans }
}

/// The result of a two-proportion z test.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProportionTest {
    /// The z statistic (positive when the first rate is the higher).
    pub z: f64,
    /// The two-sided p-value, 2 · (1 − Φ(|z|)).
    pub p: f64,
}

/// The two-proportion z test with the pooled variance: x₁ successes in n₁
/// trials against x₂ in n₂.
///
/// p̂ = (x₁ + x₂) / (n₁ + n₂), se = √(p̂ (1 − p̂) (1/n₁ + 1/n₂)),
/// z = (x₁/n₁ − x₂/n₂) / se, p = 2 (1 − Φ(|z|)). When se is 0 (both samples
/// all successes or all failures) the rates are equal: z = 0 and p = 1.
/// `None` when either sample is empty.
pub fn two_proportion(x1: u32, n1: u32, x2: u32, n2: u32) -> Option<ProportionTest> {
    if n1 == 0 || n2 == 0 || x1 > n1 || x2 > n2 {
        return None;
    }
    let (x1, n1, x2, n2) = (f64::from(x1), f64::from(n1), f64::from(x2), f64::from(n2));
    let pooled = (x1 + x2) / (n1 + n2);
    let se = (pooled * (1.0 - pooled) * (1.0 / n1 + 1.0 / n2)).sqrt();
    if se == 0.0 {
        return Some(ProportionTest { z: 0.0, p: 1.0 });
    }
    let z = (x1 / n1 - x2 / n2) / se;
    let p = (2.0 * (1.0 - normal_cdf(z.abs()))).clamp(0.0, 1.0);
    Some(ProportionTest { z, p })
}

/// The two-sample Kolmogorov–Smirnov statistic D = sup |F_a(x) − F_b(x)|
/// over the two empirical distribution functions.
///
/// Ties are handled: at each distinct value both samples step past every
/// copy of it before the difference is taken (times to kill are whole
/// ticks, so ties are common). `None` when either sample is empty.
pub fn ks_statistic(a: &[i64], b: &[i64]) -> Option<f64> {
    if a.is_empty() || b.is_empty() {
        return None;
    }
    let mut a = a.to_vec();
    let mut b = b.to_vec();
    a.sort_unstable();
    b.sort_unstable();
    let (na, nb) = (a.len() as f64, b.len() as f64);
    let (mut i, mut j) = (0usize, 0usize);
    let mut d: f64 = 0.0;
    while i < a.len() && j < b.len() {
        let x = a[i].min(b[j]);
        while i < a.len() && a[i] == x {
            i += 1;
        }
        while j < b.len() && b[j] == x {
            j += 1;
        }
        d = d.max((i as f64 / na - j as f64 / nb).abs());
    }
    Some(d)
}

/// The Kolmogorov distribution's survival function
/// Q(λ) = 2 Σ_{k≥1} (−1)^{k−1} e^{−2k²λ²}, clamped to [0, 1]. The series
/// does not converge usefully for small λ, where Q is 1 to well within
/// any test's needs (Q(0.3) > 0.9999), so it returns 1 below λ = 0.3.
pub fn kolmogorov_q(lambda: f64) -> f64 {
    if lambda < 0.3 {
        return 1.0;
    }
    let mut sum = 0.0;
    let mut sign = 1.0;
    for k in 1..=100 {
        let k = f64::from(k);
        let term = sign * (-2.0 * k * k * lambda * lambda).exp();
        sum += term;
        if term.abs() < 1e-12 {
            break;
        }
        sign = -sign;
    }
    (2.0 * sum).clamp(0.0, 1.0)
}

/// The result of a two-sample Kolmogorov–Smirnov test.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KsTest {
    /// The statistic D.
    pub d: f64,
    /// The asymptotic p-value.
    pub p: f64,
}

/// The two-sample Kolmogorov–Smirnov test with its asymptotic p-value and
/// Stephens' small-sample correction (as in Numerical Recipes' `kstwo`):
/// nₑ = n_a n_b / (n_a + n_b), λ = (√nₑ + 0.12 + 0.11 / √nₑ) · D, p = Q(λ).
/// `None` when either sample is empty.
pub fn ks_two_sample(a: &[i64], b: &[i64]) -> Option<KsTest> {
    let d = ks_statistic(a, b)?;
    let (na, nb) = (a.len() as f64, b.len() as f64);
    let ne = na * nb / (na + nb);
    let root = ne.sqrt();
    let lambda = (root + 0.12 + 0.11 / root) * d;
    Some(KsTest {
        d,
        p: kolmogorov_q(lambda),
    })
}

/// Mean and sample standard deviation (n − 1 in the denominator; 0 for a
/// single value). `None` for an empty sample.
pub fn mean_sd(xs: &[f64]) -> Option<(f64, f64)> {
    if xs.is_empty() {
        return None;
    }
    let n = xs.len() as f64;
    let mean = xs.iter().sum::<f64>() / n;
    if xs.len() == 1 {
        return Some((mean, 0.0));
    }
    let var = xs.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / (n - 1.0);
    Some((mean, var.sqrt()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn normal_cdf_matches_table_values() {
        // Φ(0) = 0.5 by symmetry.
        assert!(close(normal_cdf(0.0), 0.5, 1e-7));
        // Φ(1.959964) = 0.975 and Φ(−1.959964) = 0.025: the two-sided 5 % points.
        assert!(close(normal_cdf(1.959_964), 0.975, 1e-6));
        assert!(close(normal_cdf(-1.959_964), 0.025, 1e-6));
        // Φ(−1) = ½ erfc(1/√2) = 0.158655 (normal table).
        assert!(close(normal_cdf(-1.0), 0.158_655, 1e-6));
        // Φ(2.575829) = 0.995: the two-sided 1 % point.
        assert!(close(normal_cdf(2.575_829), 0.995, 1e-6));
    }

    #[test]
    fn two_proportion_by_hand() {
        // 20/32 against 12/32: p̂ = 32/64 = 0.5; se = √(0.5 · 0.5 · (1/32 + 1/32))
        // = √(0.25 · 0.0625) = √0.015625 = 0.125; z = (0.625 − 0.375) / 0.125 = 2.0;
        // p = 2 (1 − Φ(2)) = 2 · 0.0227501 = 0.0455003.
        let t = two_proportion(20, 32, 12, 32).unwrap();
        assert!(close(t.z, 2.0, 1e-12));
        assert!(close(t.p, 0.045_500_3, 1e-6));
        // The same samples the other way round: z = −2, same p.
        let r = two_proportion(12, 32, 20, 32).unwrap();
        assert!(close(r.z, -2.0, 1e-12) && close(r.p, t.p, 1e-12));
        // 16/32 against 16/32: equal rates, z = 0, p = 2 (1 − Φ(0)) = 1.
        let e = two_proportion(16, 32, 16, 32).unwrap();
        assert!(close(e.z, 0.0, 1e-12) && close(e.p, 1.0, 1e-7));
        // 32/32 against 32/32: p̂ = 1, se = 0; the rates are equal, p = 1.
        let all = two_proportion(32, 32, 32, 32).unwrap();
        assert_eq!((all.z, all.p), (0.0, 1.0));
        // 0/10 against 0/20: p̂ = 0, se = 0, p = 1.
        assert_eq!(two_proportion(0, 10, 0, 20).unwrap().p, 1.0);
        // 32/32 against 0/32: p̂ = 0.5, se = 0.125, z = 1 / 0.125 = 8, p ≈ 1.2 × 10⁻¹⁵.
        let far = two_proportion(32, 32, 0, 32).unwrap();
        assert!(close(far.z, 8.0, 1e-12) && far.p < 1e-10);
        // 10/20 against 30/40: p̂ = 40/60 = 2/3; se = √(2/9 · (1/20 + 1/40)) = √(2/9 · 3/40)
        // = √(1/60) = 0.1290994; z = (0.5 − 0.75) / 0.1290994 = −1.9364917;
        // p = 2 (1 − Φ(1.9364917)) = 2 · 0.0264066 = 0.0528132.
        let u = two_proportion(10, 20, 30, 40).unwrap();
        assert!(close(u.z, -1.936_491_7, 1e-6));
        assert!(close(u.p, 0.052_813_2, 1e-5));
        // An empty sample has no test.
        assert!(two_proportion(0, 0, 3, 4).is_none());
    }

    #[test]
    fn ks_statistic_by_hand() {
        // Disjoint samples: F_a reaches 1 before F_b leaves 0, D = 1.
        assert_eq!(ks_statistic(&[1, 2, 3], &[4, 5, 6]), Some(1.0));
        // a = {1, 2, 3, 4}, b = {3, 4, 5, 6}: at x = 1, 2, 3, 4 the steps are
        // F_a = .25, .5, .75, 1 and F_b = 0, 0, .25, .5; the largest gap is .5.
        assert_eq!(ks_statistic(&[1, 2, 3, 4], &[3, 4, 5, 6]), Some(0.5));
        // Ties: a = {1, 1}, b = {1}. Both distributions jump from 0 to 1 at x = 1,
        // so D = 0 (stepping one element at a time would wrongly report ½).
        assert_eq!(ks_statistic(&[1, 1], &[1]), Some(0.0));
        // Identical samples in any order: D = 0.
        assert_eq!(ks_statistic(&[5, 3, 3, 9], &[9, 3, 5, 3]), Some(0.0));
        // a = {10, 20}, b = {10, 10, 30}: at 10, F_a = ½ and F_b = ⅔ (gap ⅙); at 20,
        // F_a = 1 and F_b = ⅔ (gap ⅓); at 30 both are 1. D = ⅓.
        let d = ks_statistic(&[10, 20], &[10, 10, 30]).unwrap();
        assert!(close(d, 1.0 / 3.0, 1e-12));
        assert_eq!(ks_statistic(&[], &[1]), None);
    }

    #[test]
    fn kolmogorov_q_at_known_points() {
        // The Kolmogorov distribution's critical values: Q(1.3581) = 0.05 and
        // Q(1.6276) = 0.01 (tables of the asymptotic distribution).
        assert!(close(kolmogorov_q(1.3581), 0.05, 1e-5));
        assert!(close(kolmogorov_q(1.6276), 0.01, 1e-5));
        // Small λ: Q is 1.
        assert_eq!(kolmogorov_q(0.0), 1.0);
        assert_eq!(kolmogorov_q(0.1), 1.0);
        // Q decreases with λ.
        assert!(kolmogorov_q(1.0) > kolmogorov_q(1.2));
    }

    #[test]
    fn ks_two_sample_by_hand() {
        // 32 against 32 trials with D = 0.5: nₑ = 32 · 32 / 64 = 16, √nₑ = 4;
        // λ = (4 + 0.12 + 0.11 / 4) · 0.5 = 4.1475 · 0.5 = 2.07375; λ² = 4.30044;
        // Q = 2 (e^(−8.60088) − e^(−34.4035) + …) = 2 · 1.83944e−4 = 3.67888e−4.
        let a: Vec<i64> = (0..32).collect();
        let b: Vec<i64> = (16..48).collect();
        let t = ks_two_sample(&a, &b).unwrap();
        assert_eq!(t.d, 0.5);
        assert!(close(t.p, 3.678_88e-4, 1e-8));
        // 8 against 8 with D = 0.75: nₑ = 4, √nₑ = 2; λ = (2 + 0.12 + 0.055) · 0.75 = 1.63125;
        // Q(1.63125) = 0.0097664 (series to its third term).
        let a: Vec<i64> = (0..8).collect();
        let b: Vec<i64> = (6..14).collect();
        let t = ks_two_sample(&a, &b).unwrap();
        assert_eq!(t.d, 0.75);
        assert!(close(t.p, 0.009_766_4, 1e-6));
        // Equal samples: D = 0, λ = 0, p = 1.
        assert_eq!(ks_two_sample(&[3, 4], &[4, 3]).unwrap().p, 1.0);
    }

    #[test]
    fn mean_and_sd_by_hand() {
        // {2, 4, 4, 4, 5, 5, 7, 9}: mean 40/8 = 5; squared deviations 9+1+1+1+0+0+4+16 = 32;
        // sample variance 32/7, sd = √(32/7) = 2.1380899.
        let (m, s) = mean_sd(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]).unwrap();
        assert!(close(m, 5.0, 1e-12) && close(s, 2.138_089_9, 1e-6));
        assert_eq!(mean_sd(&[3.0]), Some((3.0, 0.0)));
        assert_eq!(mean_sd(&[]), None);
    }
}
