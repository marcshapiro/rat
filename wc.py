#!/usr/bin/python
import re
import subprocess
import sys

# wc for a rust project

# run command, return stdout as list of non-empty lines of text
def command_output(command):
    result = subprocess.run(command, shell=True,
        stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    raw = result.stdout
    text = raw.decode('utf-8')
    output = text.split('\n')
    clean = [line for line in output if 0 < len(line)]
    return clean

# cargo test
test_cmd = 'cargo test --quiet --color never'
type_out = command_output('ls src/lib.rs src/main.rs')
if 1 == len(type_out) and "src/lib.rs" == type_out[0]:
    test_cmd = 'cargo test --lib --quiet --color never'
test_all = command_output(test_cmd)
test_sum = test_all[-1]
test_match = re.match('test result: (ok|FAILED). (\d+) passed; (\d+) failed; ', test_sum)
if not test_match:
    print("Failed to match test\n")
    sys.exit(1)
tests_passed = test_match.group(2)
tests_failed = test_match.group(3)
if '0' == tests_failed:
    tests_count = tests_passed
else:
    tests_total = int(tests_failed) + int(tests_passed)
    tests_count = f"{tests_passed}/{tests_total}"
summary = [f"{tests_count} tests"]

# wc
wc_all = command_output('wc -l `find src -name *.rs`')
if 1 < len(wc_all):
    wc_total = wc_all[-1]
    wc_total_match = re.match('\s*(\d+)\s+total$', wc_total)
    if not wc_total_match:
        print("Failed to match wc total\n")
        sys.exit(2)
    wc_lines = wc_total_match.group(1)
    summary.append(f"{wc_lines} lines")
    wc_files = wc_all[:-1]
else:
    wc_files = wc_all

# add each file from wc to summary
for wc_file in wc_files:
    pat = '\s*(\d+)\s+src/(.*)\\.rs$'
    file_match = re.match(pat, wc_file)
    if not file_match:
        print(f"Failed to match wc file: '{wc_file}'")
        sys.exit(3)
    file_lines = file_match.group(1)
    file_name = file_match.group(2)
    summary.append(f"{file_name}:{file_lines}")

# format in 80-ish character lines
short = summary[0]
for count in summary[1:]:
    if len(short) + len(count) <= 77:
        short = f"{short}; {count}"
    else:
        print(f"{short};")
        short = f"    {count}"
print(f"{short}")
