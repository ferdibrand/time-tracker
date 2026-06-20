use crate::{Penalty, Solve};
use std::assert_matches;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AverageError {
    #[error("need at least {expected} solves, received {found}")]
    NotEnoughSolves { expected: usize, found: usize },
}

#[derive(Debug, PartialEq)]
pub enum Average {
    Time(u32),
    Dnf,
    Incomplete,
}

#[derive(Clone)]
pub enum AverageKind {
    Mean,
    TrimmedMean(usize),
}

pub fn average(solves: &[Solve], kind: &AverageKind) -> Result<Average, AverageError> {
    let len = solves.len();
    let trim = match kind {
        AverageKind::Mean => 0,
        AverageKind::TrimmedMean(amount) => (len * amount).div_ceil(100),
    };

    if len < trim * 2 + 1 {
        return Err(AverageError::NotEnoughSolves {
            expected: trim * 2 + 1,
            found: len,
        });
    }

    let mut sorted = solves.to_vec();
    sorted.sort_by_key(|s| match s.effective_time() {
        Some(time) => (0, time),
        None => (1, 0),
    });
    let trimmed = &sorted[trim..len - trim];
    if trimmed.iter().any(|s| s.effective_time().is_none()) {
        return Ok(Average::Dnf);
    }

    let sum: u32 = trimmed.iter().map(|s| s.effective_time().unwrap()).sum();
    Ok(Average::Time(sum / trimmed.len() as u32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn average_trimmed_mean_is_correct() {
        let solves = vec![
            Solve::new(12, Penalty::None),
            Solve::new(13, Penalty::None),
            Solve::new(14, Penalty::None),
            Solve::new(16, Penalty::None),
            Solve::new(999, Penalty::Dnf),
        ];
        let average: Average = average(&solves, &AverageKind::TrimmedMean(5)).unwrap();
        assert_eq!(average, Average::Time(14), "incorrect average");
    }

    #[test]
    fn average_mean_is_correct() {
        let solves = vec![
            Solve::new(12, Penalty::None),
            Solve::new(13, Penalty::None),
            Solve::new(17, Penalty::None),
        ];
        let average: Average = average(&solves, &AverageKind::Mean).unwrap();
        assert_eq!(average, Average::Time(14), "incorrect average");
    }

    #[test]
    fn average_is_dnf_with_multiple_dnf_solves() {
        let solves = vec![
            Solve::new(12, Penalty::None),
            Solve::new(13, Penalty::None),
            Solve::new(14, Penalty::Dnf),
            Solve::new(16, Penalty::None),
            Solve::new(999, Penalty::Dnf),
        ];
        let average = average(&solves, &AverageKind::TrimmedMean(5)).unwrap();
        assert_matches!(average, Average::Dnf, "should have returned DNF");
    }

    #[test]
    fn average_errors_when_too_few_solves() {
        let solves = vec![Solve::new(12, Penalty::None), Solve::new(13, Penalty::None)];
        let result = average(&solves, &AverageKind::TrimmedMean(5));
        assert_matches!(
            result,
            Err(AverageError::NotEnoughSolves {
                expected: _,
                found: _
            }),
            "should have returned NotEnoughSolves error",
        );
    }
}
