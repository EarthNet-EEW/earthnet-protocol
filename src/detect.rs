//! STA/LTA P-wave detection (shared Rust core).
//!
//! Mirrors the Python/ObsPy spec the country adapters use (recursive STA/LTA)
//! so picks agree across implementations. Reused by the node and, via
//! flutter_rust_bridge, by the mobile on-device detector (v1.1). Deterministic
//! by design — no ML in the alert path (DESIGN guardrail).
//!
//! PARITY TODO: the Python adapter now band-passes 2–8 Hz before STA/LTA (it
//! halved false positives in backtesting). This Rust detector is still
//! unfiltered; add the same band-pass before using it for on-device detection.

/// Short-term-average window (seconds). Keep in sync with the Python adapter.
pub const STA_SECONDS: f64 = 0.5;
/// Long-term-average window (seconds).
pub const LTA_SECONDS: f64 = 10.0;
/// STA/LTA ratio that declares a pick.
pub const TRIGGER_ON: f64 = 4.0;
/// Ratio below which a trigger is considered over.
pub const TRIGGER_OFF: f64 = 1.5;

/// A detected P-wave onset.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pick {
    /// Sample index of the onset.
    pub index: usize,
    /// STA/LTA ratio at the onset.
    pub sta_lta_ratio: f64,
}

/// Recursive STA/LTA characteristic function (matches ObsPy `recursive_sta_lta`).
/// The first `nlta` samples are zeroed (warm-up).
pub fn recursive_sta_lta(samples: &[f64], nsta: usize, nlta: usize) -> Vec<f64> {
    let mut cft = vec![0.0; samples.len()];
    if samples.is_empty() {
        return cft;
    }
    let csta = 1.0 / nsta as f64;
    let clta = 1.0 / nlta as f64;
    let icsta = 1.0 - csta;
    let iclta = 1.0 - clta;
    let mut sta = 0.0;
    let mut lta = 1e-99;
    for i in 1..samples.len() {
        let sq = samples[i] * samples[i];
        sta = csta * sq + icsta * sta;
        lta = clta * sq + iclta * lta;
        cft[i] = sta / lta;
    }
    for c in cft.iter_mut().take(nlta.min(samples.len())) {
        *c = 0.0;
    }
    cft
}

/// Returns the first P-wave pick in `samples`, or `None`.
///
/// `samples` is one channel; `sampling_rate` in Hz. Mirrors the adapter's
/// `detect_pick`: needs more than `LTA_SECONDS` of data, triggers on the first
/// crossing above [`TRIGGER_ON`].
pub fn detect_pick(samples: &[f64], sampling_rate: f64) -> Option<Pick> {
    if sampling_rate <= 0.0 || samples.is_empty() {
        return None;
    }
    let nsta = ((STA_SECONDS * sampling_rate) as usize).max(1);
    let nlta = ((LTA_SECONDS * sampling_rate) as usize).max(nsta + 1);
    if samples.len() <= nlta {
        return None;
    }
    let cft = recursive_sta_lta(samples, nsta, nlta);
    cft.iter()
        .enumerate()
        .find(|(_, &r)| r >= TRIGGER_ON)
        .map(|(index, &sta_lta_ratio)| Pick {
            index,
            sta_lta_ratio,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic signal: low-amplitude baseline then a high-amplitude burst.
    fn signal(rate: f64, total_s: f64, burst_start_s: f64, burst_len_s: f64) -> Vec<f64> {
        let n = (rate * total_s) as usize;
        let b0 = (rate * burst_start_s) as usize;
        let b1 = b0 + (rate * burst_len_s) as usize;
        (0..n)
            .map(|i| {
                let base = 0.1 * (i as f64 * 0.3).sin();
                if i >= b0 && i < b1 {
                    base + 6.0 * (i as f64 * 1.7).sin()
                } else {
                    base
                }
            })
            .collect()
    }

    #[test]
    fn detects_burst_onset() {
        let rate = 100.0;
        let s = signal(rate, 20.0, 15.0, 2.0);
        let pick = detect_pick(&s, rate).expect("should detect the burst");
        let onset = (15.0 * rate) as usize;
        assert!(
            (pick.index as i64 - onset as i64).abs() < (2.0 * rate) as i64,
            "onset off: got {}, expected ~{}",
            pick.index,
            onset
        );
        assert!(pick.sta_lta_ratio >= TRIGGER_ON);
    }

    #[test]
    fn no_pick_on_quiet_signal() {
        let rate = 100.0;
        let s: Vec<f64> = (0..(20.0 * rate) as usize)
            .map(|i| 0.1 * (i as f64 * 0.3).sin())
            .collect();
        assert!(detect_pick(&s, rate).is_none());
    }

    #[test]
    fn too_short_returns_none() {
        assert!(detect_pick(&[0.0; 10], 100.0).is_none());
    }

    #[test]
    fn invalid_rate_returns_none() {
        assert!(detect_pick(&[1.0; 5000], 0.0).is_none());
    }
}
