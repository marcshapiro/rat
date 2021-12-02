// not(windows)-specific input

use std::io;
use std::io::Read;

use std::os::unix::io::AsRawFd;
use termios::{Termios, TCSANOW, ECHO, ICANON, tcsetattr};

#[cfg(not(tarpaulin_include))] // input
pub fn inkey(echo: bool) -> Result<char, String> {
    // Based on: https://stackoverflow.com/questions/26321592/how-can-i-read-one-character-from-stdin-without-having-to-hit-enter

    // prepare stdin/terminal for read
    let stdin_fid = io::stdin().as_raw_fd();
    let termios = match Termios::from_fd(stdin_fid) {
        Ok(t) => t,
        Err(e) => return Err(format!("inkey: cannot get stdin: {}", e)),
    };
    let mut new_termios = termios;
    let flags = if echo { ICANON } else { ICANON | ECHO }; // optionally disable echo
    new_termios.c_lflag &= !flags; // disable canonical mode
    if let Err(e) = tcsetattr(stdin_fid, TCSANOW, &new_termios) {
        return Err(format!("inkey: cannot modify stdin: {}", e));
    }

    // read up to 8 bytes
    let mut reader = io::stdin();
    let mut buffer = [0; 8];
    let result = reader.read(&mut buffer);

    // reset stdin/terminal
    if let Err(e) = tcsetattr(stdin_fid, TCSANOW, &termios) {
        println!("inkey: failed to reset stdin: {}", e);
    }

    inkey_process(&buffer, result)
}

fn inkey_process(buffer: &[u8], result: std::io::Result<usize>) -> Result<char, String> {
    let n = match result {
        Ok(n) => n,
        Err(e) => return Err(format!("Error reading: {}", e)),
    };

    // convert up to n bytes of buffer to char
    for i in 0..n {
        if let Ok(ss) = std::str::from_utf8(&buffer[..i+1]) {
            if i+1 < n {
                let display_unmapped_sequences = false;
                if let Some(c) = cmap(&buffer[0..n], display_unmapped_sequences) {
                    return Ok(c)
                }
            }
            let cs: Vec<char> = ss.chars().collect();
            return Ok(cs[0]);
        }
    }

    Err(format!("inkey: not Unicode char: {:?}", buffer))
}

fn cmapn<const N:usize>(map: &[([u8; N], char)], buf: &[u8], disp: bool) -> Option<char> {
    'outer: for (clist, c) in map {
        for i in 0..N {
            if buf[i] != clist[i] { continue 'outer; }
        }
        return Some(*c);
    }
    if disp { disp_sequence(buf) }
    None
}

fn cmap(buf: &[u8], disp: bool) -> Option<char> {
    match buf.len() {
        2 => cmapn(CMAP2, buf, disp),
        3 => cmapn(CMAP3, buf, disp),
        4 => cmapn(CMAP4, buf, disp),
        5 => cmapn(CMAP5, buf, disp),
        6 => cmapn(CMAP6, buf, disp),
        7 => cmapn(CMAP7, buf, disp),
        _ => None,
    }
}

fn disp_sequence(buf: &[u8]) {
    let nstr = buf.iter()
        .map(|b| format!("{}", b))
        .collect::<Vec<String>>().join(", ");
    let cstr = buf.iter()
        .map(byte_to_string)
        .collect::<Vec<String>>().join("");
    println!("cmap: unmapped key sequence: [{}] = '{}'", nstr, cstr);
}

fn byte_to_string(bhi: &u8) -> String {
    let (blo, shi) = if *bhi < 128 {
        (*bhi, "")
    } else {
        (bhi-128, "\u{207a}")
    };
    let clo = if blo <= 32 {
        char::from_u32(0x2400 + (blo as u32)).unwrap()
    } else if 127 == blo {
        '\u{2421}'
    } else {
        blo as char
    };
    format!("{}{}", shi, clo)
}

const CMAP2: &[([u8; 2], char)] = &[
    ([27, 8], '\u{2326}'), // ctrl- alt- backspace
    ([27, 9], '\u{2b7f}'), // ctrl- alt- tab
    ([27, 27], '\u{203c}'), // alt- escape
    ([27, 127], '\u{232b}'), // alt- backspace
];

const CMAP3: &[([u8; 3], char)] = &[
    ([27, 79, 80], '\u{2460}'), // (win-) F1
    ([27, 79, 81], '\u{2461}'), // F2
    ([27, 79, 82], '\u{2462}'), // F3
    ([27, 79, 83], '\u{2463}'), // F4
    ([27, 91, 65], '\u{2191}'), // up arrow
    ([27, 91, 66], '\u{2193}'), // down arrow
    ([27, 91, 67], '\u{2192}'), // right arrow
    ([27, 91, 68], '\u{2190}'), // left arrow
    ([27, 91, 69], '\u{2b94}'), // keypad 5
    ([27, 91, 70], '\u{24d4}'), // end
    ([27, 91, 72], '\u{24d7}'), // home
    ([27, 91, 90], '\u{2b7e}'), // shift- ctrl- tab
];

const CMAP4: &[([u8; 4], char)] = &[
    ([27, 91, 50, 126], '\u{24d8}'), // insert
    ([27, 91, 51, 126], '\u{24d3}'), // delete
    ([27, 91, 53, 126], '\u{219f}'), // page up
    ([27, 91, 54, 126], '\u{21a1}'), // page down
];

const CMAP5: &[([u8; 5], char)] = &[
    ([27, 91, 49, 53, 126], '\u{2464}'), // F5
    ([27, 91, 49, 55, 126], '\u{2465}'), // F6
    ([27, 91, 49, 56, 126], '\u{2466}'), // F7
    ([27, 91, 49, 57, 126], '\u{2467}'), // F8
    ([27, 91, 50, 48, 126], '\u{2468}'), // F9
    ([27, 91, 50, 49, 126], '\u{2469}'), // (win-) F10
    ([27, 91, 50, 51, 126], '\u{246a}'), // (win-) F11
    ([27, 91, 50, 52, 126], '\u{246b}'), // F12
];

const CMAP6: &[([u8; 6], char)] = &[
    ([27, 91, 49, 59, 50, 65], '\u{1f861}'), // shift- up arrow
    ([27, 91, 49, 59, 50, 66], '\u{1f863}'), // shift- down arrow
    ([27, 91, 49, 59, 50, 67], '\u{1f862}'), // shift- right arrow
    ([27, 91, 49, 59, 50, 68], '\u{1f860}'), // shift- left arrow
    ([27, 91, 49, 59, 50, 80], '\u{2776}'), // shift- F1
    ([27, 91, 49, 59, 50, 81], '\u{2777}'), // shift- F2
    ([27, 91, 49, 59, 50, 82], '\u{2778}'), // shift- F3
    ([27, 91, 49, 59, 50, 83], '\u{2779}'), // shift- F4
    ([27, 91, 49, 59, 51, 65], '\u{21a5}'), // alt- up arrow
    ([27, 91, 49, 59, 51, 66], '\u{21a7}'), // alt- down arrow
    ([27, 91, 49, 59, 51, 67], '\u{21a6}'), // alt- right arrow
    ([27, 91, 49, 59, 51, 68], '\u{21a4}'), // alt- left arrow
    ([27, 91, 49, 59, 51, 69], '\u{267d}'), // alt- keypad 5
    ([27, 91, 49, 59, 51, 70], '\u{24ba}'), // alt- end
    ([27, 91, 49, 59, 51, 72], '\u{24bd}'), // alt- home
    ([27, 91, 49, 59, 51, 80], '\u{2474}'), // (win-) alt- F1
    ([27, 91, 49, 59, 51, 81], '\u{2475}'), // (win-) alt- F2
    ([27, 91, 49, 59, 51, 82], '\u{2476}'), // (win-) alt- F3
    ([27, 91, 49, 59, 51, 83], '\u{2477}'), // (win-) alt- F4
    ([27, 91, 49, 59, 52, 65], '\u{2b9d}'), // shift- alt- up arrow
    ([27, 91, 49, 59, 52, 66], '\u{2b9f}'), // shift- alt- down arrow
    ([27, 91, 49, 59, 52, 67], '\u{2b9e}'), // shift- alt- right arrow
    ([27, 91, 49, 59, 52, 68], '\u{2b9c}'), // shift- alt- left arrow
    ([27, 91, 49, 59, 52, 69], '\u{2672}'), // shift- alt- keypad 5
    ([27, 91, 49, 59, 52, 80], '\u{2488}'), // shift- alt- F1
    ([27, 91, 49, 59, 52, 81], '\u{2489}'), // (win-) shift- alt- F2
    ([27, 91, 49, 59, 52, 82], '\u{248a}'), // shift- alt- F3
    ([27, 91, 49, 59, 52, 83], '\u{248b}'), // shift- alt- F4
    ([27, 91, 49, 59, 53, 65], '\u{21d1}'), // ctrl- up arrow
    ([27, 91, 49, 59, 53, 66], '\u{21d3}'), // ctrl- down arrow
    ([27, 91, 49, 59, 53, 67], '\u{21d2}'), // ctrl- right arrow
    ([27, 91, 49, 59, 53, 68], '\u{21d0}'), // ctrl- left arrow
    ([27, 91, 49, 59, 53, 69], '\u{267c}'), // ctrl- keypad 5
    ([27, 91, 49, 59, 53, 70], '\u{00ca}'), // ctrl- end
    ([27, 91, 49, 59, 53, 72], '\u{0124}'), // ctrl- home
    ([27, 91, 49, 59, 53, 80], '\u{2160}'), // (win) ctrl- F1
    ([27, 91, 49, 59, 53, 81], '\u{2161}'), // (win) ctrl- F2
    ([27, 91, 49, 59, 53, 82], '\u{2162}'), // (win) ctrl- F3
    ([27, 91, 49, 59, 53, 83], '\u{2163}'), // (win) ctrl- F4
    ([27, 91, 49, 59, 54, 67], '\u{21dd}'), // shift- ctrl- right arrow
    ([27, 91, 49, 59, 54, 68], '\u{21dc}'), // shift- ctrl- left arrow
    ([27, 91, 49, 59, 54, 80], '\u{1d360}'), // shift- ctrl- F1
    ([27, 91, 49, 59, 54, 81], '\u{1d361}'), // shift- ctrl- F2
    ([27, 91, 49, 59, 54, 82], '\u{1d362}'), // shift- ctrl- F3
    ([27, 91, 49, 59, 54, 83], '\u{1d363}'), // shift- ctrl- F4
    ([27, 91, 49, 59, 55, 65], '\u{21c8}'), // (win-) ctrl- alt- up arrow
    ([27, 91, 49, 59, 55, 66], '\u{21ca}'), // (win-) ctrl- alt- down arrow
    ([27, 91, 49, 59, 55, 67], '\u{21c9}'), // (win-) ctrl- alt- right arrow
    ([27, 91, 49, 59, 55, 68], '\u{21c7}'), // (win-) ctrl- alt- left arrow
    ([27, 91, 49, 59, 55, 69], '\u{267a}'), // (win-) ctrl- alt- keypad 5
    ([27, 91, 49, 59, 55, 70], '\u{1f134}'), // (win-) ctrl- alt- end
    ([27, 91, 49, 59, 55, 72], '\u{1f137}'), // (win-) ctrl- alt- home
    ([27, 91, 49, 59, 56, 67], '\u{21e2}'), // shift- ctrl- alt- right arrow
    ([27, 91, 49, 59, 56, 68], '\u{21e0}'), // shift- ctrl- alt- left arrow
    ([27, 91, 49, 59, 56, 80], '\u{12415}'), // shift- ctrl- alt- F1
    ([27, 91, 49, 59, 56, 81], '\u{12416}'), // shift- ctrl- alt- F2
    ([27, 91, 49, 59, 56, 82], '\u{12417}'), // shift- ctrl- alt- F3
    ([27, 91, 49, 59, 56, 83], '\u{12418}'), // shift- ctrl- alt- F4
    ([27, 91, 51, 59, 50, 126], '\u{24b9}'), // shift- delete
    ([27, 91, 51, 59, 52, 126], '\u{1f147}'), // shift- alt- delete
    ([27, 91, 51, 59, 53, 126], '\u{1f133}'), // ctrl- delete
    ([27, 91, 51, 59, 54, 126], '\u{24e7}'), // shift- ctrl- delete
    ([27, 91, 51, 59, 55, 126], '\u{24cd}'), // (win-) ctrl- alt- delete
    ([27, 91, 53, 59, 51, 126], '\u{21de}'), // alt- page up
    ([27, 91, 53, 59, 53, 126], '\u{2bad}'), // (win-) ctrl- page up
    ([27, 91, 53, 59, 55, 126], '\u{2bac}'), // ctrl- alt- page up
    ([27, 91, 54, 59, 51, 126], '\u{21df}'), // alt- page down
    ([27, 91, 54, 59, 53, 126], '\u{2baf}'), // (win-) ctrl- page down
    ([27, 91, 54, 59, 55, 126], '\u{2bae}'), // ctrl- alt- page down
];

const CMAP7: &[([u8; 7], char)] = &[
    ([27, 91, 49, 53, 59, 50, 126], '\u{277a}'), // shift- F5
    ([27, 91, 49, 53, 59, 51, 126], '\u{2478}'), // alt- F5
    ([27, 91, 49, 53, 59, 52, 126], '\u{248c}'), // shift- alt- F5
    ([27, 91, 49, 53, 59, 53, 126], '\u{2164}'), // (win-) ctrl- F5
    ([27, 91, 49, 53, 59, 54, 126], '\u{1d364}'), // shift- ctrl- F5
    ([27, 91, 49, 53, 59, 56, 126], '\u{12419}'), // shift- ctrl- alt- F5
    ([27, 91, 49, 55, 59, 50, 126], '\u{277b}'), // shift- F6
    ([27, 91, 49, 55, 59, 51, 126], '\u{2479}'), // (win-) alt- F6
    ([27, 91, 49, 55, 59, 52, 126], '\u{248d}'), // shift- alt- F6
    ([27, 91, 49, 55, 59, 53, 126], '\u{2165}'), // (win-) ctrl- F6
    ([27, 91, 49, 55, 59, 54, 126], '\u{1d365}'), // shift- ctrl- F6
    ([27, 91, 49, 55, 59, 56, 126], '\u{1241a}'), // shift- ctrl- alt- F6
    ([27, 91, 49, 56, 59, 50, 126], '\u{277c}'), // shift- F7
    ([27, 91, 49, 56, 59, 51, 126], '\u{247a}'), // (win-) alt- F7
    ([27, 91, 49, 56, 59, 52, 126], '\u{248e}'), // shift- alt- F7
    ([27, 91, 49, 56, 59, 53, 126], '\u{2166}'), // (win-) ctrl- F7
    ([27, 91, 49, 56, 59, 54, 126], '\u{1d366}'), // shift- ctrl- F7
    ([27, 91, 49, 56, 59, 56, 126], '\u{1241b}'), // shift- ctrl- alt- F7
    ([27, 91, 49, 57, 59, 50, 126], '\u{277d}'), // shift- F8
    ([27, 91, 49, 57, 59, 51, 126], '\u{247b}'), // (win-) alt- F8
    ([27, 91, 49, 57, 59, 52, 126], '\u{248f}'), // shift- alt- F8
    ([27, 91, 49, 57, 59, 53, 126], '\u{2167}'), // (win-) ctrl- F8
    ([27, 91, 49, 57, 59, 54, 126], '\u{1d367}'), // shift- ctrl- F8
    ([27, 91, 49, 57, 59, 56, 126], '\u{1241c}'), // shift- ctrl- alt- F8
    ([27, 91, 50, 48, 59, 50, 126], '\u{277e}'), // shift- F9
    ([27, 91, 50, 48, 59, 51, 126], '\u{247c}'), // (win-) alt- F9
    ([27, 91, 50, 48, 59, 52, 126], '\u{2490}'), // shift- alt- F9
    ([27, 91, 50, 48, 59, 53, 126], '\u{2168}'), // (win-) ctrl- F9
    ([27, 91, 50, 48, 59, 54, 126], '\u{1d368}'), // shift- ctrl- F9
    ([27, 91, 50, 48, 59, 56, 126], '\u{1241d}'), // shift- ctrl- alt- F9
    ([27, 91, 50, 49, 59, 51, 126], '\u{24fe}'), // (win-) alt- F10
    ([27, 91, 50, 49, 59, 53, 126], '\u{2169}'), // (win-) ctrl- F10
    ([27, 91, 50, 51, 59, 50, 126], '\u{247d}'), // shift- F11
    ([27, 91, 50, 51, 59, 51, 126], '\u{247e}'), // (win-) alt- F11
    ([27, 91, 50, 51, 59, 52, 126], '\u{2492}'), // shift- alt- F11
    ([27, 91, 50, 51, 59, 53, 126], '\u{216a}'), // (win-) ctrl- F11
    ([27, 91, 50, 51, 59, 54, 126], '\u{1d36a}'), // shift- ctrl- F11
    ([27, 91, 50, 51, 59, 56, 126], '\u{1241f}'), // shift- ctrl- alt- F11
    ([27, 91, 50, 52, 59, 50, 126], '\u{24ec}'), // shift- F12
    ([27, 91, 50, 52, 59, 51, 126], '\u{247f}'), // (win-) alt- F12
    ([27, 91, 50, 52, 59, 52, 126], '\u{2493}'), // shift- alt- F12
    ([27, 91, 50, 52, 59, 53, 126], '\u{216b}'), // (win-) ctrl- F12
    ([27, 91, 50, 52, 59, 54, 126], '\u{1d36b}'), // shift- ctrl- F12
    ([27, 91, 50, 52, 59, 56, 126], '\u{12420}'), // shift- ctrl- alt- F12
];

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
    #[test] fn inkp4() {
        let c = inkey_process(&[65, 65], Ok(2)).unwrap();
        assert_eq!(c, 'A');
    }
    #[test] fn inkp5() {
        let c = inkey_process(&[27, 27], Ok(2)).unwrap();
        assert_eq!(c, '\u{203c}');
    }
    #[test] fn inkp6() {
        let c = inkey_process(&[65, 2, 3, 4, 5, 6, 7, 8], Ok(8)).unwrap();
        assert_eq!(c, 'A');
    }
    #[test] fn inkp7() {
        let c = inkey_process(&[65, 2, 3, 4, 5, 6, 7], Ok(7)).unwrap();
        assert_eq!(c, 'A');
    }
    #[test] fn inkp8() {
        let c = inkey_process(&[65, 2, 3, 4, 5, 6], Ok(6)).unwrap();
        assert_eq!(c, 'A');
    }
    #[test] fn inkp9() {
        let c = inkey_process(&[65, 2, 3, 4, 5], Ok(5)).unwrap();
        assert_eq!(c, 'A');
    }
    #[test] fn inkp10() {
        let c = inkey_process(&[65, 2, 3, 4], Ok(4)).unwrap();
        assert_eq!(c, 'A');
    }
    #[test] fn inkp11() {
        let c = inkey_process(&[65, 2, 3], Ok(3)).unwrap();
        assert_eq!(c, 'A');
    }

    #[test] fn bts1() { assert_eq!(byte_to_string(&255), "\u{207a}\u{2421}"); }
    #[test] fn bts2() { assert_eq!(byte_to_string(&27), "\u{241b}"); }
    #[test] fn bts3() { assert_eq!(byte_to_string(&65), "A"); }

}

