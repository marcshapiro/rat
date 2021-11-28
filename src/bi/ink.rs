// stdin / stdout
use std::io;
use std::io::{stdin, Read, Write};
use std::os::unix::io::AsRawFd;
use termios::{Termios, TCSANOW, ECHO, ICANON, tcsetattr};

#[cfg(not(tarpaulin_include))] // input
pub(super) fn inkey(echo: bool) -> Result<char, String> {
    // Based on: https://stackoverflow.com/questions/26321592/how-can-i-read-one-character-from-stdin-without-having-to-hit-enter

    // prepare stdin/terminal for read
    let stdin_fid = stdin().as_raw_fd();
    let termios = Termios::from_fd(stdin_fid).unwrap(); // FIXME: Err not unwrap?
    let mut new_termios = termios;
    let flags = if echo { ICANON } else { ICANON | ECHO }; // optionally disable echo
    new_termios.c_lflag &= !flags; // disable canonical mode
    tcsetattr(stdin_fid, TCSANOW, &new_termios).unwrap(); // FIXME: Err not unwrap?

    // read up to 4 bytes
    let mut reader = io::stdin();
    let mut buffer = [0; 4];
    let result = reader.read(&mut buffer);

    // reset stdin/terminal
    tcsetattr(stdin_fid, TCSANOW, &termios).unwrap(); // FIXME: Err not unwrap?

    inkey_process(&buffer, result)
}

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

fn inkey_process(buffer: &[u8], result: std::io::Result<usize>) -> Result<char, String> {
    let n = match result {
        Ok(n) => n,
        Err(e) => return Err(format!("Error reading: {}", e)),
    };

    // println!("inkey bytes: {:?}", &buffer[0..n]);

    // convert up to n bytes of buffer to char
    for i in 0..n {
        match std::str::from_utf8(&buffer[..i+1]) {
            Err(_) => {}, // ignore prefix errors
            Ok(ss) => {
                let cs: Vec<char> = ss.chars().collect();
                return Ok(cs[0]);
                // NB: Some keys on keyboard (up arrow, F2, etc.) populate the
                //     buffer with a sequence, often starting 27 (ESC).  This
                //     will return just the first character of the sequence,
                //     and discard the rest.
            },
        }
    }

    Err(format!("inkey: not Unicode char: {:?}", buffer))
}

fn inline_process(buffer: String, result: io::Result<usize>) ->  Result<String, String> {
    match result {
        Ok(_) => Ok(chomp(buffer)),
        Err(e) => Err(format!("inp: failed to read line: {}", e)),
    }
}

fn is_newline(nl: &str) -> bool { matches!(nl, "\n"|"\r") }
/*
    match nl {
        "\n"|"\r" => true,
        _ => false,
    }
}
*/

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

    #[test] fn inkp1() {
        let msg = "Badness 10000";
        let err = Err(io::Error::new(io::ErrorKind::Other, msg.to_string()));
        let emsg = inkey_process(&[], err).unwrap_err();
        assert_eq!(emsg, format!("Error reading: {}", msg));
    }
    #[test] fn inkp2() {
        let err = inkey_process(&[], Ok(0)).unwrap_err();
        assert_eq!(err, "inkey: not Unicode char: []");
    }
    #[test] fn inkp3() {
        let c = inkey_process(&[65], Ok(1)).unwrap();
        assert_eq!(c, 'A');
    }
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
