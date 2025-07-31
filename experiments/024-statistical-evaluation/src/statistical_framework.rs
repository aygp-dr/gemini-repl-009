//! Statistical framework for rigorous evaluation of function calling
//! 
//! Implements proper statistical analysis including:
//! - Sample size calculation with power analysis
//! - Confidence intervals (95% and 99%)
//! - P-values and hypothesis testing
//! - Effect size calculations (Cohen's d)
//! - Bootstrap methods for non-parametric analysis

use anyhow::Result;
use statrs::distribution::{Normal, StudentsT, Binomial, ContinuousCDF};
use statrs::statistics::Statistics;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use ndarray::prelude::*;
use ndarray_stats::QuantileExt;
use rand::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalResult {
    pub sample_size: usize,
    pub success_rate: f64,
    pub confidence_interval_95: (f64, f64),
    pub confidence_interval_99: (f64, f64),
    pub p_value: f64,
    pub power: f64,
    pub effect_size: f64,
    pub standard_error: f64,
    pub z_score: f64,
    pub critical_value: f64,
    pub is_significant: bool,
    pub bootstrap_ci: Option<(f64, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    pub min_sample_size: usize,
    pub desired_power: f64,
    pub alpha: f64,
    pub expected_effect_size: f64,
    pub baseline_success_rate: f64,
    pub bootstrap_iterations: usize,
}

impl Default for ExperimentConfig {
    fn default() -> Self {
        Self {
            min_sample_size: 100,
            desired_power: 0.80,
            alpha: 0.05,
            expected_effect_size: 0.5,
            baseline_success_rate: 0.5,
            bootstrap_iterations: 10000,
        }
    }
}

pub struct StatisticalAnalyzer {
    config: ExperimentConfig,
}

impl StatisticalAnalyzer {
    pub fn new(config: ExperimentConfig) -> Self {
        Self { config }
    }

    /// Calculate required sample size for desired power
    pub fn calculate_sample_size(&self, effect_size: f64, power: f64) -> usize {
        let normal = Normal::new(0.0, 1.0).unwrap();
        let z_alpha = normal.inverse_cdf(1.0 - self.config.alpha / 2.0);
        let z_beta = normal.inverse_cdf(power);
        
        let n = 2.0 * (z_alpha + z_beta).powi(2) / effect_size.powi(2);
        n.ceil() as usize
    }

    /// Perform power analysis
    pub fn power_analysis(&self, sample_size: usize, effect_size: f64) -> f64 {
        let normal = Normal::new(0.0, 1.0).unwrap();
        let z_alpha = normal.inverse_cdf(1.0 - self.config.alpha / 2.0);
        
        let z_beta = effect_size * (sample_size as f64 / 2.0).sqrt() - z_alpha;
        normal.cdf(z_beta)
    }

    /// Calculate confidence intervals for proportion
    pub fn confidence_interval_proportion(
        &self,
        successes: usize,
        total: usize,
        confidence: f64,
    ) -> (f64, f64) {
        let p = successes as f64 / total as f64;
        let n = total as f64;
        
        // Wilson score interval (better for small samples)
        let z = Normal::new(0.0, 1.0).unwrap()
            .inverse_cdf((1.0 + confidence) / 2.0);
        
        let denominator = 1.0 + z.powi(2) / n;
        let center = (p + z.powi(2) / (2.0 * n)) / denominator;
        let margin = z * ((p * (1.0 - p) / n + z.powi(2) / (4.0 * n.powi(2))).sqrt()) / denominator;
        
        (center - margin, center + margin)
    }

    /// Perform two-proportion z-test
    pub fn two_proportion_test(
        &self,
        successes1: usize,
        total1: usize,
        successes2: usize,
        total2: usize,
    ) -> (f64, f64, bool) {
        let p1 = successes1 as f64 / total1 as f64;
        let p2 = successes2 as f64 / total2 as f64;
        let n1 = total1 as f64;
        let n2 = total2 as f64;
        
        // Pooled proportion
        let p_pool = (successes1 + successes2) as f64 / (total1 + total2) as f64;
        
        // Standard error
        let se = (p_pool * (1.0 - p_pool) * (1.0 / n1 + 1.0 / n2)).sqrt();
        
        // Z-score
        let z = (p1 - p2) / se;
        
        // P-value (two-tailed)
        let normal = Normal::new(0.0, 1.0).unwrap();
        let p_value = 2.0 * (1.0 - normal.cdf(z.abs()));
        
        let is_significant = p_value < self.config.alpha;
        
        (z, p_value, is_significant)
    }

    /// Calculate Cohen's d effect size for proportions
    pub fn cohens_d_proportion(&self, p1: f64, p2: f64) -> f64 {
        let h1 = 2.0 * p1.sqrt().asin();
        let h2 = 2.0 * p2.sqrt().asin();
        h1 - h2
    }

    /// Bootstrap confidence interval
    pub fn bootstrap_confidence_interval(
        &self,
        data: &[bool],
        confidence: f64,
        iterations: usize,
    ) -> (f64, f64) {
        let mut rng = thread_rng();
        let n = data.len();
        let mut bootstrap_means = Vec::with_capacity(iterations);
        
        for _ in 0..iterations {
            let mut sample_sum = 0;
            for _ in 0..n {
                if data[rng.gen_range(0..n)] {
                    sample_sum += 1;
                }
            }
            bootstrap_means.push(sample_sum as f64 / n as f64);
        }
        
        bootstrap_means.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let lower_idx = ((1.0 - confidence) / 2.0 * iterations as f64) as usize;
        let upper_idx = ((1.0 + confidence) / 2.0 * iterations as f64) as usize;
        
        (bootstrap_means[lower_idx], bootstrap_means[upper_idx.min(iterations - 1)])
    }

    /// Comprehensive analysis of experiment results
    pub fn analyze_results(&self, successes: usize, total: usize) -> StatisticalResult {
        let success_rate = successes as f64 / total as f64;
        
        // Confidence intervals
        let ci_95 = self.confidence_interval_proportion(successes, total, 0.95);
        let ci_99 = self.confidence_interval_proportion(successes, total, 0.99);
        
        // Standard error
        let se = (success_rate * (1.0 - success_rate) / total as f64).sqrt();
        
        // Z-test against baseline
        let z_score = (success_rate - self.config.baseline_success_rate) / se;
        let normal = Normal::new(0.0, 1.0).unwrap();
        let p_value = 2.0 * (1.0 - normal.cdf(z_score.abs()));
        
        // Critical value
        let critical_value = normal.inverse_cdf(1.0 - self.config.alpha / 2.0);
        
        // Effect size
        let effect_size = self.cohens_d_proportion(success_rate, self.config.baseline_success_rate);
        
        // Power
        let power = self.power_analysis(total, effect_size);
        
        StatisticalResult {
            sample_size: total,
            success_rate,
            confidence_interval_95: ci_95,
            confidence_interval_99: ci_99,
            p_value,
            power,
            effect_size,
            standard_error: se,
            z_score,
            critical_value,
            is_significant: p_value < self.config.alpha,
            bootstrap_ci: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub group1_stats: StatisticalResult,
    pub group2_stats: StatisticalResult,
    pub relative_improvement: f64,
    pub odds_ratio: f64,
    pub chi_squared: f64,
    pub chi_squared_p_value: f64,
    pub is_significant: bool,
}

impl StatisticalAnalyzer {
    /// Compare two groups (e.g., different models or approaches)
    pub fn compare_groups(
        &self,
        group1_successes: usize,
        group1_total: usize,
        group2_successes: usize,
        group2_total: usize,
    ) -> ComparisonResult {
        let group1_stats = self.analyze_results(group1_successes, group1_total);
        let group2_stats = self.analyze_results(group2_successes, group2_total);
        
        let p1 = group1_stats.success_rate;
        let p2 = group2_stats.success_rate;
        
        // Relative improvement
        let relative_improvement = if p1 > 0.0 {
            (p2 - p1) / p1 * 100.0
        } else {
            f64::INFINITY
        };
        
        // Odds ratio
        let odds1 = p1 / (1.0 - p1);
        let odds2 = p2 / (1.0 - p2);
        let odds_ratio = odds2 / odds1;
        
        // Chi-squared test
        let expected1_success = group1_total as f64 * (group1_successes + group2_successes) as f64 
            / (group1_total + group2_total) as f64;
        let expected1_fail = group1_total as f64 - expected1_success;
        let expected2_success = group2_total as f64 * (group1_successes + group2_successes) as f64 
            / (group1_total + group2_total) as f64;
        let expected2_fail = group2_total as f64 - expected2_success;
        
        let chi_squared = 
            (group1_successes as f64 - expected1_success).powi(2) / expected1_success +
            ((group1_total - group1_successes) as f64 - expected1_fail).powi(2) / expected1_fail +
            (group2_successes as f64 - expected2_success).powi(2) / expected2_success +
            ((group2_total - group2_successes) as f64 - expected2_fail).powi(2) / expected2_fail;
        
        // Chi-squared p-value (df = 1)
        let chi_dist = statrs::distribution::ChiSquared::new(1.0).unwrap();
        let chi_squared_p_value = 1.0 - chi_dist.cdf(chi_squared);
        
        ComparisonResult {
            group1_stats,
            group2_stats,
            relative_improvement,
            odds_ratio,
            chi_squared,
            chi_squared_p_value,
            is_significant: chi_squared_p_value < self.config.alpha,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_size_calculation() {
        let analyzer = StatisticalAnalyzer::new(ExperimentConfig::default());
        
        // Medium effect size (0.5), 80% power
        let n = analyzer.calculate_sample_size(0.5, 0.80);
        assert!(n >= 64); // Standard sample size for these parameters
        
        // Large effect size (0.8), 80% power
        let n_large = analyzer.calculate_sample_size(0.8, 0.80);
        assert!(n_large < n); // Larger effect sizes need smaller samples
    }

    #[test]
    fn test_confidence_intervals() {
        let analyzer = StatisticalAnalyzer::new(ExperimentConfig::default());
        
        // 95% CI for 80/100 success rate
        let (lower, upper) = analyzer.confidence_interval_proportion(80, 100, 0.95);
        assert!(lower > 0.7 && lower < 0.75);
        assert!(upper > 0.85 && upper < 0.9);
        
        // CI should widen with higher confidence
        let (lower_99, upper_99) = analyzer.confidence_interval_proportion(80, 100, 0.99);
        assert!(lower_99 < lower);
        assert!(upper_99 > upper);
    }

    #[test]
    fn test_statistical_significance() {
        let analyzer = StatisticalAnalyzer::new(ExperimentConfig::default());
        
        // Significant difference: 90/100 vs 60/100
        let (z, p_value, is_sig) = analyzer.two_proportion_test(90, 100, 60, 100);
        assert!(p_value < 0.05);
        assert!(is_sig);
        assert!(z.abs() > 1.96); // Critical value for alpha=0.05
        
        // Non-significant difference: 52/100 vs 48/100
        let (z2, p_value2, is_sig2) = analyzer.two_proportion_test(52, 100, 48, 100);
        assert!(p_value2 > 0.05);
        assert!(!is_sig2);
        assert!(z2.abs() < 1.96);
    }
}