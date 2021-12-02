// common I/O

#[cfg(not(windows))]
mod link;
#[cfg(not(windows))]
pub(super) use self::link::inkey;

#[cfg(windows)]
mod wink;
#[cfg(windows)]
pub(super) use self::wink::inkey;

// stdin / stdout
use std::io;
use std::io::Write;

#[cfg(not(tarpaulin_include))] // input
pub(super) fn inline() -> Result<String, String> {
    let mut buffer = String::new();
    let result = io::stdin().read_line(&mut buffer);
    inline_process(buffer, result)
}

#[cfg(not(tarpaulin_include))] // output
pub(super) fn flush_stdout() {
    io::stdout().flush().unwrap();
}


fn inline_process(buffer: String, result: io::Result<usize>) ->  Result<String, String> {
    match result {
        Ok(_) => Ok(chomp(buffer)),
        Err(e) => Err(format!("inp: failed to read line: {}", e)),
    }
}

fn is_newline(nl: &str) -> bool { matches!(nl, "\n"|"\r") }

fn chomp(line: String) -> String {
    let n = line.len();
    if 0 < n && is_newline(&line[n-1..]) {
        if 1 < n && is_newline(&line[n-2..n-1]) {
            return line[..n-2].to_string();
        }
        return line[..n-1].to_string();
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn inlp1() {
        let msg = "Badness 10000";
        let err = Err(io::Error::new(io::ErrorKind::Other, msg.to_string()));
        let amsg = inline_process("".to_string(), err).unwrap_err();
        let xmsg = format!("inp: failed to read line: {}", msg);
        assert_eq!(amsg, xmsg);
    }
    #[test] fn inlp2() {
        let line = "Hello, line";
        let aline = inline_process(line.to_string(), Ok(0)).unwrap();
        assert_eq!(aline, line);
    }
    #[test] fn chomp1() { assert_eq!(chomp("foo\n".to_string()), "foo"); }
    #[test] fn chomp2() { assert_eq!(chomp("foo\n\r".to_string()), "foo"); }
    #[test] fn chomp3() { assert_eq!(chomp("foo\r\n".to_string()), "foo"); }
    #[test] fn chomp4() { assert_eq!(chomp("foo\r".to_string()), "foo"); }
    #[test] fn chomp5() { assert_eq!(chomp("foo\r\r".to_string()), "foo"); }
    #[test] fn chomp6() { assert_eq!(chomp("foo\n\n".to_string()), "foo"); }
    #[test] fn chomp7() { assert_eq!(chomp("foo".to_string()), "foo"); }
    #[test] fn chomp8() { assert_eq!(chomp("foo\n\n\n".to_string()), "foo\n"); }
    #[test] fn chomp9() { assert_eq!(chomp("foo\nx".to_string()), "foo\nx"); }
    #[test] fn chomp10() { assert_eq!(chomp("foo ".to_string()), "foo "); }
}
