/// `ProcID`s are usually numeric PIDs; however, on some systems, they may be something else
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProcId<S: AsRef<str> + Ord + PartialEq + Clone> {
    PID(i32),
    Name(S),
}

impl<'a> From<&'a str> for ProcId<&'a str> {
    fn from(s: &str) -> ProcId<&str> {
        match s.parse() {
            Ok(pid) => ProcId::PID(pid),
            Err(_) => ProcId::Name(s),
        }
    }
}
