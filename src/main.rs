#[derive(Debug, PartialEq, Clone)]
pub struct Solve {
    pub time: u32,
}

#[derive(Debug, PartialEq)]
pub enum Average {
    Time(u32),
    Dnf,
    Incomplete,
}

#[derive(Debug, PartialEq, Default)]
pub struct Session {
    pub solves: Vec<Solve>,
}

impl Session {
    pub fn add_time(&mut self, time: u32) {
        let solve = Solve { time };
        self.solves.push(solve);
    }

    pub fn add_times(&mut self, times: Vec<u32>) {
        for time in times {
            self.add_time(time);
        }
    }

    pub fn best_solve(&self) -> Option<Solve> {
        self.solves.iter().min_by_key(|solve| solve.time).cloned()
    }

    pub fn latest_ao(&self, num_solves: u32) -> Average {
        let latest = self.solves[self.solves.len() - num_solves as usize..].to_vec();
        Average::Time(trimmed_mean(latest, 0.05))
    }
}

fn trimmed_mean(mut solves: Vec<Solve>, trim_amount: f64) -> u32 {
    solves.sort_by_key(|solve| solve.time);
    let trim_number = (solves.len() as f64 * trim_amount).ceil();
    let trimmed = solves[trim_number as usize..solves.len() - trim_number as usize].to_vec();
    let mut sum = 0;
    for solve in &trimmed {
        sum += solve.time;
    }
    sum / trimmed.len() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_time_fn_adds_time() {
        let mut session = Session::default();
        let _ = session.add_time(1);
        let solve = Solve { time: 1 };
        assert_eq!(session.solves[0], solve, "did not add time");
    }

    #[test]
    fn add_times_fn_adds_times() {
        let mut session = Session::default();
        session.add_times(vec![1, 2, 3, 4, 5]);
        assert_eq!(session.solves.len(), 5, "did not add times")
    }

    #[test]
    fn best_solve_fn_returns_best_solve() {
        let mut session = Session::default();
        let _ = session.add_time(1);
        let _ = session.add_time(3);
        let _ = session.add_time(2);
        let best_solve = session.best_solve().unwrap();
        assert_eq!(best_solve, Solve { time: 1 }, "incorrect time")
    }

    #[test]
    fn latest_ao5_fn_returns_correct_time() {
        let mut session = Session::default();
        session.add_times(vec![1, 2, 3, 4, 5, 6, 50]);
        let average: Average = session.latest_ao(5);
        assert_eq!(average, Average::Time(5));
    }
}
