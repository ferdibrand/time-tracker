#![allow(dead_code, unused)]
use crate::average::*;
use std::fmt::{Display, Error, Formatter};
use thiserror::Error;

mod average;

#[derive(Debug, PartialEq, Clone)]
pub struct Solve {
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

impl Display for Solve {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let total_ms = self.time;
        let minutes = total_ms / 60_000;
        let seconds = (total_ms % 60_000) / 1000;
        let millis = total_ms % 1000;

        if self.penalty == Penalty::Dnf {
            write!(f, "DNF(")?;
        }

        if minutes > 0 {
            write!(f, "{}:{:02}.{:03}", minutes, seconds, millis)?;
        } else {
            write!(f, "{}.{:03}", seconds, millis)?;
        }

        match self.penalty {
            Penalty::None => {}
            Penalty::Plus2 => write!(f, "+")?,
            Penalty::Dnf => write!(f, ")")?,
        }

        Ok(())
    }
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
    fn average_from_is_correct() {
        let mut session = Session::default();
        session.add_times(vec![1, 2, 3, 4, 5, 6, 7]);
        let average = session.average_from(1, 5, &AverageKind::Mean).unwrap();
        assert_eq!(average, Average::Time(4), "incorrect average");
    }

    #[test]
    fn average_from_incomplete_when_too_few_solves() {
        let mut session = Session::default();
        session.add_times(vec![1, 2, 3, 4, 5, 6, 7]);
        let average = session.average_from(5, 5, &AverageKind::Mean).unwrap();
        assert_eq!(average, Average::Incomplete, "incorrect average");
    }

    #[test]
    fn current_average_is_correct() {
        let mut session = Session::default();
        session.add_times(vec![1, 2, 3, 4, 5, 6, 7]);
        let average = session.current_average(5, &AverageKind::Mean).unwrap();
        assert_eq!(average, Average::Time(5), "incorrect average");
    }

    #[test]
    fn current_average_incomplete_when_too_few_solves() {
        let mut session = Session::default();
        session.add_times(vec![1, 2, 3]);
        let average = session.current_average(5, &AverageKind::Mean).unwrap();
        assert_eq!(average, Average::Incomplete, "incorrect average");
    }

    #[test]
    fn best_average_is_correct() {
        let mut session = Session::default();
        session.add_times(vec![1, 7, 5, 4, 6, 3, 2]);
        let (_, average) = session.best_average(5, &AverageKind::Mean).unwrap();
        assert_eq!(average, Average::Time(4), "incorrect average");
    }
}
