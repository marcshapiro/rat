// run files for Rat
mod cmd;

use self::cmd::{process_arguments, Opt, OutFormat};
use crate::bi::register_bi;
use crate::brat::BRat;
use crate::bst::{Bst, BstFile};
use crate::cab::Cab;
use crate::repl::run_repl;

use std::env;
use std::time::Instant;

#[cfg(not(tarpaulin_include))] // env, output
pub fn standard_main() -> Result<(), String> {
    let args = env::args().skip(1).collect();
    call_file_arg(args)
}

fn call_file_arg(args: Vec<String>) -> Result<(), String> {
    match process_arguments(args)? {
        Some(opt) => { call_file(&opt, true, true)?; },
        None => run_repl(),
    }
    Ok(())
}

fn add_use_last_call(bf: &mut BstFile, path: &str) -> Result<(), String> {
    let n = bf.file.len();
    if 0 == n { return Err(format!("-{} requires non-empty script", path)); }
    match &bf.file[n-1] {
        Bst::Call(nm, _) => match &**nm {
            Bst::Name(name) => {
                bf.uses.push(build_use(path, name, name));
            },
            _ => return Err(format!("-{} requires script ending call to name", path)),
        },
        _ => return Err(format!("-{} requires script ending call", path)),
    }
    Ok(())
}

fn prep_cab(bf: &BstFile, load_auto: bool, load_use: bool) -> Result<Cab, String> {
    let mut cab = Cab::new();
    register_bi(&mut cab)?;
    if load_auto {
        cab.load_autos()?;
    }
    cab.postload()?;
    cab.load_file_rec("_main", "_main", bf, load_use, false)?;
    Ok(cab)
}

fn build_use(path: &str, func: &str, name: &str) -> (String, String, String) {
    let nm = if name.is_empty() { name } else { func };
    (path.to_owned(), func.to_owned(), nm.to_owned())
}

fn build_call_func(path: &str, func: &str, args: Vec<Bst>) -> Bst {
    let efc = Bst::Func(path.to_owned(), func.to_owned());
    Bst::Call(Box::new(efc), args)
}

fn call_file(opt: &Opt, load_auto: bool, load_use: bool) -> Result<Bst, String> {
    let t0 = Instant::now();
    let mut bf = Bst::from_str_file(&opt.script, "_main")?;

    // process -sys -std -usr -my // convert to 'use'
    match &opt.path {
        None => {},
        Some(p) => add_use_last_call(&mut bf, p)?,
    }

    let mut cab = prep_cab(&bf, load_auto, load_use)?;
    let ecall = build_call_func("_main", "_main", vec![]);
    let t1 = Instant::now();

    // call wrapper
    let res = cab.eval(&None, &ecall)?;

    let t2 = Instant::now();
    if opt.time {
        let t_load = t1.duration_since(t0).as_secs_f64() * 1000.0;
        let t_run = t2.duration_since(t1).as_secs_f64() * 1000.0;
        println!("Time: {:>.3} msec load; {:>.3} msec run", t_load, t_run);
    }

    out_result(&res, &opt.fmt, load_auto, load_use)?;
    Ok(res)
}

fn out_result(res: &Bst, fmt: &OutFormat, load_auto: bool, load_use: bool) -> Result<(), String> {
    match fmt {
        OutFormat::General => { println!("{}", res); },
        OutFormat::None => {},
        OutFormat::Text => {
            match res.d_list_text("_out") {
                Ok(text) => { println!("'{}'", text); },
                Err(_) => { println!("{}",  res); },
            }
        },
        OutFormat::Decimal(sdigits) => match BRat::from_str(sdigits) {
            None => { println!("-dec: cannot parse '{}'; result is {}",  sdigits, res); },
            Some(ndigits) => {
                let deccall = build_call_func("sys", "as_decimal", vec![res.clone(), Bst::Rat(ndigits)]);
                let saycall = build_call_func("auto", "say", vec![deccall]);
                let uses = vec![build_use("sys", "as_decimal", "")];
                let outbf = BstFile { uses, decl: None, file: vec![saycall] };
                let mut dcab = prep_cab(&outbf, load_auto, load_use)?;
                let dcall = build_call_func("_main", "_main", vec![]);
                dcab.eval(&None, &dcall)?;
            },
        },
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // cfu() does a call_file without I/O, unless it is in the script
    // Only builtins are loaded.  'sys' builtins still need a 'use'.
    fn cfu(spath: &str, sscript: &str) -> Result<Bst, String> {
        let path = if spath.is_empty() { None } else { Some(spath.to_owned()) };
        let script = sscript.to_owned();
        let opt = Opt { path, script, fmt:OutFormat::None, time:false };
        call_file(&opt, false, false)
    }
    fn cf(spath: &str, sscript: &str, xres: &str) {
        assert_eq!(format!("{}",  cfu(spath, sscript).unwrap()), xres);
    }
    fn xf(spath: &str, sscript: &str, xerr: &str) {
        assert_eq!(cfu(spath, sscript).unwrap_err(), xerr);
    }

    #[test] fn cf1() { cf("", "1 + 2", "3"); }
    #[test] fn cf2() { cf("sys", "is_char(-1)", "0"); }
    #[test] fn cf3() { cf("", "", "0"); }
    #[test] fn cf4() { cf("", "1+2", "3"); }
    #[test] fn cf5() { cf("", "-3^2", "-9"); }
    #[test] fn xf1() { xf("sys", "let x = is_char(-1)",
        "-sys requires script ending call"); }
    #[test] fn xf2() { xf("usr", "", "-usr requires non-empty script"); }
    #[test] fn buselast1() {
        let func = Bst::Func("a".to_owned(), "b".to_owned());
        let call = Bst::Call(Box::new(func), vec!());
        let mut bf = BstFile { uses: vec![], decl: None, file: vec![call] };
        let e = add_use_last_call(&mut bf, "path").unwrap_err();
        assert_eq!(e, "-path requires script ending call to name");
    }
    #[test] fn cfu1() {
        let b = cfu("sys", "unparse(if a { 1 })").unwrap();
        let t = b.d_list_text("_t").unwrap();
        assert_eq!(t, "when {\n  a => {\n    1\n  }\n} else {\n    \n}\n");
    }
}
