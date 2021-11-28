// display utilities
use std::fmt;

pub fn write_vec<T: fmt::Display>(f: &mut fmt::Formatter,
        pfx: &str, vs: &[T], sep: &str, sfx: &str) -> fmt::Result {
    let mut ss = vec![];
    for v in vs {
        ss.push(format!("{}", v));
    }
    let sj = ss.join(sep);
    write!(f, "{}{}{}", pfx, sj, sfx)
}
