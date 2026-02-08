//! Multi-asset geometric Brownian motion price process.

use rand::SeedableRng;
use rand_distr::{Distribution, StandardNormal};
use rand_pcg::Pcg64;

/// Generates fair prices for multiple assets in numeraire terms.
///
/// Each non-numeraire asset follows independent GBM:
/// S(t+1) = S(t) * exp((mu - 0.5*sigma^2)*dt + sigma*sqrt(dt)*Z)
pub struct MultiAssetPriceProcess {
    prices: Vec<f64>,
    numeraire_token: usize,
    drift_term: f64,
    vol_term: f64,
    rng: Pcg64,
}

impl MultiAssetPriceProcess {
    pub fn new(
        initial_prices: Vec<f64>,
        numeraire_token: usize,
        mu: f64,
        sigma: f64,
        dt: f64,
        seed: Option<u64>,
    ) -> Self {
        let rng = match seed {
            Some(s) => Pcg64::seed_from_u64(s),
            None => Pcg64::from_entropy(),
        };

        let mut prices = initial_prices;
        if numeraire_token < prices.len() {
            prices[numeraire_token] = 1.0;
        }

        Self {
            prices,
            numeraire_token,
            drift_term: (mu - 0.5 * sigma * sigma) * dt,
            vol_term: sigma * dt.sqrt(),
            rng,
        }
    }

    pub fn current_prices(&self) -> &[f64] {
        &self.prices
    }

    pub fn step(&mut self) -> &[f64] {
        for (idx, p) in self.prices.iter_mut().enumerate() {
            if idx == self.numeraire_token {
                *p = 1.0;
                continue;
            }
            let z: f64 = StandardNormal.sample(&mut self.rng);
            *p *= (self.drift_term + self.vol_term * z).exp();
            if *p <= 0.0 {
                *p = 1e-9;
            }
        }
        &self.prices
    }
}

#[cfg(test)]
mod tests {
    use super::MultiAssetPriceProcess;

    #[test]
    fn test_multi_asset_prices_deterministic() {
        let mut p1 = MultiAssetPriceProcess::new(vec![2.0, 1.0, 3.0], 1, 0.0, 0.001, 1.0, Some(42));
        let mut p2 = MultiAssetPriceProcess::new(vec![2.0, 1.0, 3.0], 1, 0.0, 0.001, 1.0, Some(42));

        for _ in 0..20 {
            assert_eq!(p1.step(), p2.step());
        }
    }

    #[test]
    fn test_numeraire_is_one() {
        let mut p = MultiAssetPriceProcess::new(vec![10.0, 20.0], 0, 0.0, 0.1, 1.0, Some(1));
        for _ in 0..20 {
            p.step();
            assert_eq!(p.current_prices()[0], 1.0);
        }
    }
}
