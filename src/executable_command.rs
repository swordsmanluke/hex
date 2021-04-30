use std::str;
use regex::Regex;

pub struct ExecutableCommand {
    pub id: String,
    pub command: String,
    pub working_dir: String,
    pub period: String,
    pub time_between_runs: u64
}

impl ExecutableCommand {
    pub fn new(id: String, command: String, working_dir: String, period: String) -> ExecutableCommand {
        ExecutableCommand {
            id,
            command,
            working_dir,
            period: period.clone(),
            time_between_runs: calc_time_between_runs(period.as_str()),
        }
    }

    pub fn millis_until_next_run(&self, elapsed: u64) -> u64 {
        match elapsed > self.time_between_runs
        {
            true => 0,
            false => self.time_between_runs - elapsed
        }

    }
}

impl Clone for ExecutableCommand {
    fn clone(&self) -> ExecutableCommand {
        ExecutableCommand::new(
            self.id.clone(),
            self.command.clone(),
            self.working_dir.clone(),
            self.period.clone()
        )
    }
}

fn calc_time_between_runs(period: &str) -> u64 {
    let matcher = Regex::new(r"(\d+)([smh]?)").unwrap();

    for c in matcher.captures_iter(period) {
        // Should only be one, but whatever.
        let time = &c[1].parse::<u64>().unwrap();
        let unit = &c[2];

        let mult = match unit {
            "h" => 3600000,
            "m" => 60000,
            _ => 1000 // default to milliseconds
        };

        return time * mult;
    }

    panic!("Couldn't calculate the time between runs from '{}'", period);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_between_runs_works_for_seconds() {
        assert_eq!(calc_time_between_runs("1s"), 1000);
    }

    #[test]
    fn time_between_runs_assumes_seconds() {
        assert_eq!(calc_time_between_runs("12"), 12000);
    }

    #[test]
    fn time_between_runs_works_for_minutes() {
        assert_eq!(calc_time_between_runs("1m"), 60000);
    }

    #[test]
    fn time_between_runs_works_for_hours() {
        assert_eq!(calc_time_between_runs("1h"), 3600000);
    }

    #[test]
    #[should_panic]
    fn time_between_panics_for_bad_pattern() {
        calc_time_between_runs("m");
    }

}