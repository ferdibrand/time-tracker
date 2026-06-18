#![allow(dead_code, unused)]
use std::thread::AccessError;

use thiserror::Error;

use crate::AverageKind::TrimmedMean;

#[derive(Debug, PartialEq, Clone)]
struct Solve {
    time: u32,
    penalty: Penalty,
}

impl Solve {
    fn new(time: u32, penalty: Penalty) -> Self {
        Solve { time, penalty }
    }

    fn effective_time(&self) -> Option<u32> {
        match self.penalty {
            Penalty::None => Some(self.time),
            Penalty::Plus2 => Some(self.time + 2000),
            Penalty::Dnf => None,
        }
    }
}

#[derive(Debug, PartialEq)]
enum Average {
    Time(u32),
    Dnf,
    Incomplete,
}

#[derive(Clone)]
enum AverageKind {
    Mean,
    TrimmedMean(usize),
}

#[derive(Debug, PartialEq, Clone)]
enum Penalty {
    None,
    Plus2,
    Dnf,
}

#[derive(Debug, PartialEq, Default)]
struct Session {
    solves: Vec<Solve>,
}

impl Session {
    fn add_time(&mut self, time: u32, penalty: Penalty) {
        let solve = Solve { time, penalty };
        self.solves.push(solve);
    }

    fn add_times(&mut self, times: Vec<u32>) {
        for time in times {
            self.add_time(time, Penalty::None);
        }
    }

    fn best_solve(&self) -> Option<&Solve> {
        self.solves.iter().min_by_key(|s| match s.effective_time() {
            Some(time) => (0, time),
            None => (1, 0),
        })
    }

    fn average_from(
        &self,
        index: usize,
        size: usize,
        kind: &AverageKind,
    ) -> Result<Average, AverageError> {
        if self.solves.len() - index < size {
            return Ok(Average::Incomplete);
        }
        average(&self.solves[index..index + size], kind)
    }

    fn current_average(&self, size: usize, kind: &AverageKind) -> Result<Average, AverageError> {
        if self.solves.len() < size {
            return Ok(Average::Incomplete);
        }
        self.average_from(self.solves.len() - size, size, kind)
    }

    fn best_average(&self, size: usize, kind: &AverageKind) -> Option<(usize, Average)> {
        (0..self.solves.len())
            .map(|u| (u, self.average_from(u, size, kind)))
            .filter_map(|(u, r)| match r {
                Ok(Average::Incomplete) | Err(_) => None,
                Ok(avg) => Some((u, avg)),
            })
            .min_by_key(|(u, avg)| match avg {
                Average::Time(time) => (0, *time, *u),
                Average::Dnf => (1, 0, 0),
                Average::Incomplete => unreachable!("Incomplete filtered out above"),
            })
    }
}

fn average(solves: &[Solve], kind: &AverageKind) -> Result<Average, AverageError> {
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

#[derive(Debug, Error)]
enum AverageError {
    #[error("need at least {expected} solves, received {found}")]
    NotEnoughSolves { expected: usize, found: usize },
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::assert_matches;

    #[test]
    fn add_time_adds_time() {
        let mut session = Session::default();
        let _ = session.add_time(1, Penalty::None);
        let solve = Solve {
            time: 1,
            penalty: Penalty::None,
        };
        assert_eq!(session.solves[0], solve, "did not add time");
    }

    #[test]
    fn add_times_adds_times() {
        let mut session = Session::default();
        session.add_times(vec![1, 2, 3, 4, 5]);
        assert_eq!(session.solves.len(), 5, "did not add times")
    }

    #[test]
    fn best_solve_returns_best_solve() {
        let mut session = Session::default();
        session.add_times(vec![1, 2, 3]);
        let best_solve = session.best_solve().unwrap();
        assert_eq!(
            *best_solve,
            Solve {
                time: 1,
                penalty: Penalty::None
            },
            "incorrect time"
        )
    }

    #[test]
    fn average_trimmed_mean_is_correct() {
        let solves = vec![
            Solve::new(12, Penalty::None),
            Solve::new(13, Penalty::None),
            Solve::new(14, Penalty::None),
            Solve::new(16, Penalty::None),
            Solve::new(999, Penalty::Dnf),
        ];
        let average: Average = average(&solves, AverageKind::TrimmedMean(5)).unwrap();
        assert_eq!(average, Average::Time(14), "incorrect average");
    }

    #[test]
    fn average_mean_is_correct() {
        let solves = vec![
            Solve::new(12, Penalty::None),
            Solve::new(13, Penalty::None),
            Solve::new(17, Penalty::None),
        ];
        let average: Average = average(&solves, AverageKind::Mean).unwrap();
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
        let average = average(&solves, AverageKind::TrimmedMean(5)).unwrap();
        assert_matches!(average, Average::Dnf, "should have returned DNF");
    }

    #[test]
    fn average_errors_when_too_few_solves() {
        let solves = vec![Solve::new(12, Penalty::None), Solve::new(13, Penalty::None)];
        let result = average(&solves, AverageKind::TrimmedMean(5));
        assert_matches!(
            result,
            Err(AverageError::NotEnoughSolves {
                expected: _,
                found: _
            }),
            "should have returned NotEnoughSolves error",
        );
    }

    #[test]
    fn average_from_is_correct() {
        let mut session = Session::default();
        session.add_times(vec![1, 2, 3, 4, 5, 6, 7]);
        let average = session.average_from(1, 5, AverageKind::Mean).unwrap();
        assert_eq!(average, Average::Time(4), "incorrect average");
    }

    #[test]
    fn average_from_incomplete_when_too_few_solves() {
        let mut session = Session::default();
        session.add_times(vec![1, 2, 3, 4, 5, 6, 7]);
        let average = session.average_from(5, 5, AverageKind::Mean).unwrap();
        assert_eq!(average, Average::Incomplete, "incorrect average");
    }

    #[test]
    fn current_average_is_correct() {
        let mut session = Session::default();
        session.add_times(vec![1, 2, 3, 4, 5, 6, 7]);
        let average = session.current_average(5, AverageKind::Mean).unwrap();
        assert_eq!(average, Average::Time(5), "incorrect average");
    }

    #[test]
    fn current_average_incomplete_when_too_few_solves() {
        let mut session = Session::default();
        session.add_times(vec![1, 2, 3]);
        let average = session.current_average(5, AverageKind::Mean).unwrap();
        assert_eq!(average, Average::Incomplete, "incorrect average");
    }

    #[test]
    fn best_average_is_correct() {
        let mut session = Session::default();
        session.add_times(vec![1, 7, 5, 4, 6, 3, 2]);
        let (_, average) = session.best_average(5, AverageKind::Mean).unwrap();
        assert_eq!(average, Average::Time(4), "incorrect average");
    }
}
