#!/usr/bin/python
import glob
import os
import re
import subprocess
import sys
from termcolor import colored
import time

# Test command line programs

# TODO: tags.  File:--tag foo bar;  Cmdline: -tag 'foo&(!bar|baz)'
# TODO: -dots=10,7 # space after 10 passes in a row, newline after 10*7
# TODO: docs

# file format:
# -- -- <file comments>
# --cmd <args> OR --arg <arg> OR --par <arg>
# -- -- <command comment>
# # --tag <tag> <tag> ... # nyi
# --rv <rv> # int return value # if omitted, assumes 0
# --out (has|re)?
# <stdout>
# ...
# --err (has|re)?
# <stderr>
# ...
# --cmd ... # next command
# ...

def show_lines(lines):
    for i, line in enumerate(lines):
        print(f"{i}: {line}")

def match_exact(alines, xlines):
    na = len(alines)
    nx = len(xlines)
    n = min(na, nx)
    for i in range(0, n):
        aline = alines[i]
        xline = xlines[i]
        if aline != xline:
            return line_diff(i+1, aline, xline)
    while n < na and 0 == len(alines[n].strip()):
        n = n + 1
    while n < nx and 0 == len(xlines[n].strip()):
        n = n + 1
    if n < na:
        return f"{n+1}: actual continues: {alines[n]}"
    if n < nx:
        return f"{n+1}: expected continues: {xlines[n]}"
    None

def match_has(alines, xlines):
    for i, xline in enumerate(xlines):
        ifound = False
        for aline in alines:
            if xline in aline:
                ifound = True
                break
        if not ifound:
            return f"{i+1}: expected not found: {xline}"
    return None

def match_re(alines, xlines):
    for i, xline in enumerate(xlines):
        xre = re.compile(xline)
        ifound = False
        for aline in alines:
            if xre.search(aline) is not None:
                ifound = True
                break
        if not ifound:
            return f"{i+1}: expected not matched: {xline}"
    return None


class TextPect:
    def __init__(self, kind=None, default='exact'):
        if '' == kind or kind is None:
            kind = default

        assert kind in ['exact', 'has', 're'], 'Bad kind'

        self.kind = kind
        self.lines = []

    def add(self, line):
        self.lines.append(line);

    def cf(self, actual):
        alines = actual.split("\n")
        xlines = self.lines
        if 'exact' == self.kind:
            return match_exact(alines, xlines)
        if 'has' == self.kind:
            return match_has(alines, xlines)
        if 're' == self.kind:
            return match_re(alines, xlines)

    def show(self, name):
        print(f"vvv {name} TextPect {self.kind} vvv")
        show_lines(self.lines)
        print(f"^^^                             ^^^")

class Expect:
    def __init__(self, rv, out, err):
        if rv is None:
            rv = '0'
        if out is None:
            out = TextPect()
        if err is None:
            err = TextPect()

        rv = int(rv)

        self.rv = rv
        self.out = out
        self.err = err

    def cf(self, arv, aout, aerr):
        out = self.out.cf(aout)
        if out is not None:
            return f"out:{out}"
        err = self.err.cf(aerr)
        if err is not None:
            return f"err:{err}"
        xrv = self.rv;
        if arv != xrv:
            return f"rv: {arv} not {xrv}"
        None

    def show(self):
        print(f"rv: {self.rv}")
        self.out.show('out')
        self.err.show('err')

class RTest:
    def __init__(self, filename, lno, args, ckind, expect=None):
        if expect is None:
            expect = Expect()
        self.filename = filename
        self.name = f"{filename}:{lno}:{args}"
        self.args = args
        self.ckind = ckind;
        self.expect = expect;

    def run(self, binary, file_first):
        parts = []
        if binary is not None:
            parts.append(binary)
        if file_first:
            base1 = os.path.basename(self.filename)
            (base2, _) = os.path.splitext(base1)
        else:
            base2 = ''
        args = self.args
        if 'par' == self.ckind:
            args = '(' + args + ')'
        if 'arg' == self.ckind or 'par' == self.ckind:
            parts.append(shell_escape(base2 + args))
        else:
            if file_first:
                parts.append(base2)
            parts.append(args)
        cmd = " ".join(parts);
        r = subprocess.run(cmd, shell=True, stdout=subprocess.PIPE,
            stderr=subprocess.PIPE, encoding="utf-8")
        arv = r.returncode
        aout = r.stdout
        aerr = r.stderr
        bad = self.expect.cf(arv, aout, aerr)
        if bad is not None:
            return f"{self.name}:{cmd}:{bad}"
        None

# TODO: better
def shell_escape(s):
    return '"' + s + '"'

class RSuite:
    def __init__(self, binary):
        self.tests = []
        self.binary = binary

    def run_tests(self, file_first, show_pass, prefix):
        if prefix is None:
            prefix = ''
        else:
            prefix = prefix + ': '
        n_pass = 0
        n_fail = 0
        for test in self.tests:
            res = test.run(self.binary, file_first)
            if res is None:
                if 'dot' == show_pass:
                    print('.') # FIXME - spaces/newlines
                elif 'name' == show_pass:
                    print(f"pass: {test.name}")
                n_pass = n_pass + 1
            else:
                print(f"FAIL: {res}")
                n_fail = n_fail + 1
        if 0 < n_fail:
            cfail = colored(f"{n_fail} FAIL", 'red')
            if 0 < n_pass:
                cpass = colored(f"{n_pass} pass", 'green')
                print(f"{prefix}{cfail}, {cpass}")
            else:
                print(f"{prefix}{cfail}, NONE pass")
        else:
            if 0 < n_pass:
                cpass = colored(f"{n_pass} pass", 'green')
                print(f"{prefix}{cpass}")
            else:
                print(f"{prefix}NO TESTS")

    def add(self, test):
        self.tests.append(test)

    def load_files(self, filenames):
        for filename in filenames:
            self.load_file(filename)

    def load_file(self, filename):
        with open(filename, 'r', encoding='utf-8') as f:
            text = f.read()
        lines = text.split("\n")
        out = None # TextPect
        err = None # TextPect
        rv = None # int
        clno = None # int
        command = None # string
        ckind = None # string
        state = 'start'
        for lno, line in enumerate(lines):
            if line.startswith("--//"):
                if state in ['start', 'xrv']:
                    pass
                else:
                    raise KeyError(f"{filename}:{lno+1}: [{state}] Bad //: {line}")
            elif line.startswith("--cmd") or line.startswith("--arg") or line.startswith("--par"):
                if state not in ['start']:
                    self.add(RTest(filename, clno, command, ckind, Expect(rv, out, err)))
                    out = None
                    err = None
                    rv = None
                    ckind = None
                command = line[5:].strip()
                ckind = line[2:5]
                clno = lno + 1
                if '' == command:
                    raise KeyError(f"{filename}:{clno}: [{state}] No command")
                state = 'xrv'
            elif line.startswith("--rv"):
                if state in ['xrv']:
                    assert rv is None, "'rv' reached twice"
                    rv = int(line[4:].strip())
                    state = 'xout'
                else:
                    raise KeyError(f"{filename}:{lno+1}: [{state}] Bad ret: {line}")
            elif line.startswith("--out"):
                if state in ['xrv', 'xout']:
                    assert out is None, "'out' reached twice"
                    out = TextPect(line[5:].strip())
                    state = 'out'
                else:
                    raise KeyError(f"{filename}:{lno+1}: [{state}] Bad out: {line}")
            elif line.startswith("--err"):
                if state in ['xrv', 'xout', 'out', 'xerr']:
                    assert err is None, "'err' reached twice"
                    err = TextPect(line[5:].strip(), default='has')
                    state = 'err'
                else:
                    raise KeyError(f"{filename}:{lno+1}: [{state}] Bad err: {line}")
            else:
                if state in ['xrv', 'xout']:
                    assert out is None, "'out' reached twice"
                    out = TextPect('') # out exact # if lines without --out
                    state = 'out'
                    out.add(line)
                elif state in ['out']:
                    out.add(line)
                elif state in ['err']:
                    err.add(line)
                else:
                    line = line.strip()
                    if '' != line:
                        raise KeyError(f"{filename}:{lno+1}: [{state}] Bad text: {line}")
        if state not in ['start']:
            self.add(RTest(filename, clno, command, ckind, Expect(rv, out, err)))


def line_diff(lno, actual, expect):
    na = len(actual)
    nb = len(expect)
    n = min(na, nb)
    pn = n
    cno = 0
    for i in range(0, n):
        cno = cno + 1
        act = actual[i]
        if act != expect[i]:
            pn = i
            break
    if pn < n:
        e = min(n, pn+10)
        aa = actual[pn:e]
        ee = expect[pn:e]
        return f"{lno}: char {cno}: '{aa}' not '{ee}'"
    if na < nb:
        e = min(nb, n+10)
        ee = expect[n:e]
        return f"{lno}: char {cno}: expect continues '{ee}'"
    if nb < na:
        e = min(na, n+10)
        aa = actual[n:e]
        return f"char {cno}: actual continues '{aa}'"
    assert False, 'char by char match'


# process command line
binary = None
file_first = False
show_pass = None
filenames = []
timing = False
prefix = None
args = sys.argv
nargs = len(args)
iarg = 1
while iarg < nargs:
    arg = args[iarg]
    iarg = iarg + 1
    if '-binary' == arg:
        assert iarg < nargs, "Must specify executable after -binary"
        binary = args[iarg]
        iarg = iarg + 1
    elif '-dots' == arg:
        show_pass = 'dot'
    elif '-file-first' == arg:
        file_first = True
    elif '-pass' == arg:
        show_pass = 'name'
    elif '-pfx' == arg:
        assert iarg < nargs, "Must specify prefix after -pfx"
        prefix = args[iarg]
        iarg = iarg + 1
    elif '-time' == arg:
        timing = True
    else:
        filenames.append(arg)

# load
t0 = time.perf_counter()
suite = RSuite(binary)
suite.load_files(filenames)
t1 = time.perf_counter()

# run
suite.run_tests(file_first, show_pass, prefix)
t2 = time.perf_counter()
if timing:
    t_load = t1 - t0
    t_run = t2 - t1
    print(f"Loaded in {t_load:.3} seconds, ran in {t_run:.3} seconds")
