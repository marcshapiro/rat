// parse command line arguments for main
#[derive(Debug, PartialEq)]
pub(super) enum OutFormat { // how to format result
    General,
    Text,
    Decimal(String),
    None,
}

#[derive(Debug)]
pub(super) struct Opt {
    pub path: Option<String>,
    pub script: String,
    pub fmt: OutFormat,
    pub time: bool,
}

pub(super) fn process_arguments(args: Vec<String>) -> Result<Option<Opt>, String> {
    let mut options = true; // --
    let mut path = None; // -sys -std -usr -my
    let mut sfmt = None; // -quiet -text -decNNN
    let mut stime = None; // -time
    let mut stmts = None; // statements to execute

    for (iarg, arg) in args.iter().enumerate() {
        let jarg = 1 + iarg;
        let nchar = arg.len();
        if options && 0 < nchar && "-" == &arg[0..1] {
            let opt = &arg[1..];
            match opt {
                "-" => { options = false; },
                "sys" | "std" | "usr" | "my" => { path = set_string(path, opt, jarg)?; },
                "time" => { stime = set_string(stime, opt, jarg)?; }
                "quiet" | "text" => { sfmt = set_string(sfmt, opt, jarg)?; },
                "h" => {
                    sfmt = set_string(sfmt, "quiet", jarg)?;
                    path = set_string(path, "sys", jarg)?;
                    stmts = set_string(stmts, "rat_help()", jarg)?;
                },
                "v" => {
                    sfmt = set_string(sfmt, "text", jarg)?;
                    path = set_string(path, "sys", jarg)?;
                    stmts = set_string(stmts, "rat_version()", jarg)?;
                },
                _ => if opt.starts_with("dec") {
                    sfmt = set_string(sfmt, opt, jarg)?;
                } else {
                    return Err(format!("Argument {} '{}': unknown option", jarg, arg));
                },
            }
        } else {
            stmts = set_string(stmts, arg, jarg)?;
            options = false;
        }
    }

    let script = match stmts {
        None => {
            if None == path && None == sfmt && None == stime {
                return Ok(None);
            }
            return Err("No script specified".to_owned());
        },
        Some(s) => s,
    };
    let fmt = match sfmt {
        None => OutFormat::General,
        Some(s) => match s.as_str() {
            "text" => OutFormat::Text,
            "quiet" => OutFormat::None,
            _ => if let Some(sp) = s.strip_prefix("dec") {
                OutFormat::Decimal(sp.to_owned())
            } else {
                panic!("Unreachable: bad sfmt");
            },
        },
    };
    let time = stime.is_some();

    Ok(Some(Opt{path, script, fmt, time}))
}

fn set_string(old: Option<String>, value: &str, jarg: usize) -> Result<Option<String>, String> {
    match old {
        None => Ok(Some(value.to_owned())),
        Some(ref prev) => Err(format!("Argument {}: '{}' overwrites existing value '{}'", jarg, value, prev)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pargs(inargs: Vec<&str>) -> Result<Option<Opt>, String> {
        let mut sargs = vec![];
        for inarg in inargs.iter() {
            sargs.push((*inarg).to_owned());
        }
        process_arguments(sargs)
    }
    fn args(inargs: Vec<&str>, xpath: &str, xfunc: &str, xfmt: Option<OutFormat>, xtime: bool) {
        let opt = pargs(inargs).unwrap().unwrap();
        let xp = if xpath.is_empty() {
            None
        } else {
            Some(xpath.to_owned())
        };
        let xf = match xfmt {
            None => OutFormat::General,
            Some(f) => f,
        };
        assert_eq!(opt.path, xp);
        assert_eq!(opt.script, xfunc);
        assert_eq!(opt.fmt, xf);
        assert_eq!(opt.time, xtime);
    }
    fn bargs(inargs: Vec<&str>, xerr: &str) {
        assert_eq!(pargs(inargs).unwrap_err(), xerr);
    }

    #[test] fn arg1() { args(vec!["foo()"], "", "foo()", None, false); }
    #[test] fn arg2() { args(vec!["-sys", "bar()"], "sys", "bar()", None, false); }
    #[test] fn arg3() { args(vec!["-std", "foo()"], "std", "foo()", None, false); }
    #[test] fn arg4() { args(vec!["-usr", "foo()"], "usr", "foo()", None, false); }
    #[test] fn arg5() { args(vec!["-my", "foo()"], "my", "foo()", None, false); }
    #[test] fn arg6() { args(vec!["-time", "foo()"], "", "foo()", None, true); }
    #[test] fn arg7() { args(vec!["-time", "-sys", "bar()"], "sys", "bar()", None, true); }
    #[test] fn arg8() { args(vec!["-sys", "-time", "bar()"], "sys", "bar()", None, true); }
    #[test] fn arg9() { args(vec!["-quiet", "foo()"], "", "foo()", Some(OutFormat::None), false); }
    #[test] fn arg10() { args(vec!["-text", "foo()"], "", "foo()", Some(OutFormat::Text), false); }
    #[test] fn arg11() { args(vec!["-dec20", "foo()"], "", "foo()", Some(OutFormat::Decimal("20".to_owned())), false); }
    #[test] fn arg12() { args(vec!["-decimal", "foo()"], "", "foo()", Some(OutFormat::Decimal("imal".to_owned())), false); }
    #[test] fn arg13() { args(vec!["-v"], "sys", "rat_version()", Some(OutFormat::Text), false); }
    #[test] fn arg14() { args(vec!["-h"], "sys", "rat_help()", Some(OutFormat::None), false); }
    #[test] fn arg15() { args(vec!["--", "-5"], "", "-5", None, false); }
    #[test] fn arg16() { assert!(pargs(vec![]).unwrap().is_none()); }
    #[test] fn barg1() { bargs(vec!["foo()", "-my"], "Argument 2: '-my' overwrites existing value 'foo()'"); }
    #[test] fn barg2() { bargs(vec!["-sys", "bar()", "-time"], "Argument 3: '-time' overwrites existing value 'bar()'"); }
    #[test] fn barg3() { bargs(vec!["-sys", "-sys", "bar()"], "Argument 2: 'sys' overwrites existing value 'sys'"); }
    #[test] fn barg4() { bargs(vec!["-sys", "-usr", "bar()"], "Argument 2: 'usr' overwrites existing value 'sys'"); }
    #[test] fn barg5() { bargs(vec!["-time", "-time", "bar()"], "Argument 2: 'time' overwrites existing value 'time'"); }
    #[test] fn barg6() { bargs(vec!["-quiet", "-quiet", "bar()"], "Argument 2: 'quiet' overwrites existing value 'quiet'"); }
    #[test] fn barg7() { bargs(vec!["-quiet", "-text", "bar()"], "Argument 2: 'text' overwrites existing value 'quiet'"); }
    #[test] fn barg8() { bargs(vec!["-foo", "foo()"], "Argument 1 '-foo': unknown option"); }
    #[test] fn barg9() { bargs(vec!["-5"], "Argument 1 '-5': unknown option"); }
    #[test] fn barg11() { bargs(vec!["-quiet", "-sys"], "No script specified"); }
}
