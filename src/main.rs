use anyhow::Result;

#[derive(Debug, PartialEq)]
pub struct Solve {
    time: u32,
}

impl Solve {
    pub fn time(&self) -> Option<u32> {
        Some(self.time)
    }
}

pub fn add_time(solves: &mut Vec<Solve>, input: &str) -> Result<()> {
    let time: u32 = input.parse()?;
    let solve = Solve { time };
    solves.push(solve);
    Ok(())
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_time_adds_time() {
        let mut solves: Vec<Solve> = Vec::new();
        let _ = add_time(&mut solves, "31415");
        let solve = Solve { time: 31415 };
        assert_eq!(solves[0], solve, "did not add time");
    }

    #[test]
    fn test_add_time_rejects_invalid_time() {
        let mut solves: Vec<Solve> = Vec::new();
        let result = add_time(&mut solves, "abc");
        assert!(result.is_err())
    }

    #[test]
    fn test_time_returns_time() {
        let solve = Solve { time: 31415 };
        let time = solve.time().unwrap();
        assert_eq!(time, 31415, "incorrect time");
    }
}
