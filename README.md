# rat
`rat` is an interpreter for a toy programming language, also called `rat`,
which uses rational numbers of unlimited precision.


### Code snippet
```rat
let pi = 3.14159265358793238462643383279502884197169399375;
say('pi is approximately', as_text(pi))
```
When run, that code prints:
```output
pi is approximately 502654824574069181540229413247204614715471039/160000000000000000000000000000000000000000000
```
The fraction is exactly equal to the literal.

### Running `rat` from the command line

There are various ways to run `rat`, some of which are listed in the
table below.  In the examples, I will use "`cargo run --`".

| Possible starts to `rat` command lines | Notes |
| ----------------------- | ----- |
| `cargo run --`          | recommended |
| `cargo run --release --` | faster |
| `./target/debug/rat` | after `cargo build` |
| `./target/release/rat` | after `cargo build --release` |
| `rat` | if a `rat` executable is on your path |

See the `auto` list below for a list of `auto` functions.  To run an
function in `auto`, such as `eval` or `op_add`:
```sh
cargo run -- "eval(3 + 5)" # 8
cargo run -- "eval('ABC')" # single quotes inside double # list[65, 66, 67]
cargo run -- -text "eval('abc')" # -t tries to format result as text # 'abc'
cargo run -- "let a = 5; let b = 10; eval(a * b)" # 50
cargo run -- "op_add(3/5, 4/3)" # 29/15
cargo run -- -dec5 "op_add(3/5, 4/3)" # 1.93333 # print to 5 decimal places
cargo run -- "op_add(3/5 * 4/9, 4/3 * 7/12)" # 47/45
cargo run -- -dec75 "op_add(3/5 * 4/9, 4/3 * 7/12)" # 1.0444... # 75 places
cargo run -- "3 + 5" # 8 # eval is rarely needed
cargo run -- "'ABC'" # list[65, 66, 67]
cargo run -- -text "'abc'" # 'abc'
cargo run -- -dec20 "3/7" # 0.42857142857142857143
cargo run -- -time -dec20 -std "root(3, 2, 1e-20)" # -time prints timing for the command before the result
cargo run -- -quiet "say('hi')" # -quiet suppresses printing the returned result
cargo run -- -dec20 -- "-3/7" # the -- indicates the next part is the script, not another option
```
Note that `eval` takes one argument, and `op_add` takes two.
The double quotes are just to keep
the shell from interpreting characters like spaces or parentheses.

As you can see, you can use `rat` as a somewhat awkward command-line
calculator.

To run a function in a different path, you specify the path as an option.
See the `sys` list below for a list of `sys` functions.
```sh
cargo run -- -sys "is_finite(nan)" # 0
cargo run -- "use sys is_finite; is_finite(nan)" # equivalent to above
cargo run -- -my "liaber()" # (Hit 'q' to quit, or 'n', 'e', 'w', 's' to move.)
```

There are some special options
```sh
cargo run -- -v # 'rat v0.0.1'
cargo run -- -text -sys "rat_version()" # equivalent to above
cargo run -- -h # brief command-line help
cargo run -- -quiet -sys "rat_help()" # equivalent to above
```

## Types in `rat`

### Numbers
The numeric type in `rat` is called `Rat`.  It is basically rational
numbers, with a minor extension.
Numeric literals in `rat` can look much like those in other programming
languages: `123`, `-123.456`, `123.456e-7`.  They can also be ratios:
`123/456`, `12.34/45.67`, `-12.34/45.67e-8`.
Numbers in `rat` are extended with three additional values: `inf`,
`-inf`, and `nan`.  This makes them closed not only under addition,
subtraction and multiplication, but also under division.
```rat
6.7/0 == inf;
-1e2/0 == -inf;
0/0 == nan;
```
Unlike `nan` in many floating point systems, in `rat`, `nan` is equal to
itself, and also less than `-inf`.

`Rat` is the only numeric type.  An integer (`Int`) is just a `Rat` which
happens to have a denominator of `1`.

### Other scalar types
There are no other scalar types in `rat`; everything is rational numbers.
Similar to `Int`, a boolean (`Bool`) value
is just a `Rat` which is `0` (false) or `1` (true).
A character (`Char`) is a just a `Rat` that is a
a Unicode scalar value.  Implementation note: the Rust type
[`char`](https://doc.rust-lang.org/std/primitive.char.html) is
used for I/O; that handles all the Unicode magic.

### `List`
The only other data type currently supported in `rat` is the `List`.
A `List` can be a list of `Rat`.  Lists can also nest.
```rat
list[1, 2, 3, list[4, 5, list[6, 7], 8, list[]], 9]
```
The `Text` type is just a `List` where every element is a `Char`.
There is a `Text` literal, but it just becomes a `List`. The following
are exactly the same:
```rat
'ABC'
list[65, 66, 67]
```

### Other types
There is a type `Function` for functions.  And there is sort of a
type of unevaluated expressions.

## The syntax and semantics of `rat`

Note: `rat` code should be UTF-8 encoded Unicode.  But, outside of
comments and `Text` literals, it is all ASCII.

### Whitespace
Spaces, tabs, newlines and returns are ignored, except for separating
other tokens.  Because newlines are ignored, code is free form; you can
break lines where you like (well, not within tokens).

### Comments
Comments start with a `#` and continue to the end of the line.  They
are also ignored.  They may contain any characters besides newlines
and returns.

### Constants and variables
Names of constants and variables (and functions) in `rat` start with
an alphabetic character, and are followed by any number of alphabetic or
numeric characters or underscores or question marks: `a`, `X`, `ab12`,
`a_long_name_71`, `confused?`, `a_8?4b_9`.  The last one may be a poor
choice stylistically, but it is permitted.  The only restriction on names
is that they cannot conflict with any `rat` keywords, or with any
previously defined name within a function.
Constants are declared with a `let` statement.  Variables are declared
with a `mutable` statement and modified with an `update` statement.
```rat
let pi = 3.14;
let pie = 'apple';
let tau = 2 * pi; # use the previously defined pi
mutable x = 3;
update x = x + 3; # use the old value of x
```
Note that you should not put a `let` or `mutable` statement in a loop;
the second time through, it will conflict with itself.  Also note that
the `each` loop, discussed below, defines a special ephemeral constant
for each iteration of the loop.

### `lazy` constants and variables
This is rarely useful, but it is possible to assign an expression to
a constant or variable without evaluating it.
```rat
let tau = lazy 2 * pi;
mutable q = lazy 1 + 3 / 4;
update y = lazy tau + 1;
```

### Conditional execution: `if`, `when`, `case`, `switch`, and `else`
#### `if`
`if` is much as in other languages:
```rat
if x < 3 {
    let y = 5;
    say('x is only', as_text(x));
};
if y < 3 {
    let z = 6;
} else {
    let z = 7;
};
```
`if` can be used as an expression:
```rat
let z = if y < 3 { 6 } else { 7 };
let q = if 3 < y { 9 };
```
An empty `then` block, or an empty `else` block or missing `else` clause,
with evaluate to `0` if executed.  Of course, the blocks are lazy; only
the selected one is evaluated.
#### `when`
`when` executes the first block with a guard that evaluates to `1` (true).
```rat
when {
   x < 3 => { say('small'); };
   x < 7 => { say('medium'); };
   x < 10 => { say('large'); };
} else { say('extra large'); };
```
The `else` clause is optional, and empty blocks evaluate to `0`, as with `if`.
In addition to the blocks being lazy, any guard expressions after the first
one which is `1` are not evaluated.
#### `case`
`case` executes the first clause where the guard is equal to the specified value.
```rat
case response { # response is a variable containing Text (or not)
   'yes' => { say('Thank you'); };
   'no' => { say('Please'); };
   'maybe' => { say('Let me know'); };
} else { say('Tell me more'); };
```
`case` uses the `==` operator to check for equality.  Again, the `else`
is optional, and empty blocks evaluate to `0`.
#### `switch`
`switch` uses the specified comparison operator or function to match
each guard against the specified value.  Only the first match is executed.
```rat
switch < 10 {
    age => { say('young'); }; # if age < 10
    height => { say('short'); }; # if height < 10
} else { say('Everything is at least 10'); };

switch (myfunc) 10 {
    age => { say('ok'); }; # if myfunc(age, 10)
};
```
Again, the `else` is optional, and empty blocks evaluate to `0`.
So `case` is the same as `switch ==`.  The comparison function in
the `switch` clause can, within the parentheses, be an expression
which evalues to a `Function`.

### Loops: `loop`, `break`, `each`
`loop` loops forever, or until a `break` statement is executed.  It
also stops when a `return` statement is executed, as discussed below.
```rat
mutable x = 1;
loop { # count to 10
    say(as_text(x));
    if 10 < x { break; };
};
```
`each` traverses a list.
```rat
each element in list[2, 4, 6] {
   say(as_text(element));
};
```
The variable (`element` in the example above) is constant inside the loop
(it cannot be modified with `update`), but does not exist outside the loop.

### Function calls
Function calls are similar to many other languages.
```rat
say('The', as_text(count), 'best features');
```
The first word can be a function name, or in parentheses,
a constant or variable that
evaluates to a `Function`.  The parentheses are required, but the
arguments are optional: `foo()` calls a function without arguments.
The arguments are expressions; and are separated by commas.

### Operators, precedence, associativity
Parentheses can override precedence and associativity of operators.
The table below lists the operators from highest precedence
(first executed) to lowest.  The Associativity column show how parentheses
would be inserted into an expression without parentheses.
The types of the arguments are: r:Rat b:Bool a:Any

| Operators| Function | Associativity |
| --------- | --------- | ------------- |
| `r^i` | `op_pow` | `x^(y^z)` |
| `+r` `-r` `!b` | (none) `op_neg` `op_not` | `-(-x)` `!(!x)` |
| `r*r` `r/r` | `op_mul` `op_div` | `(x*y)*z` `(x/y)/z` |
| `r+r` `r-r` | `op_add` `op_sub` | `(x+y)+z` `(x-y)-z` |
| `a==a` `a<>a` `a<a` `a<=a` `a>a` `a>=a` | `op_eq` `op_ne` `op_lt` `op_le` `op_gt` `op_ge` | non-associative |
| `b&&b` | `op_and` | `(x&&y)&&z` |
| `b`\|\|`b` | `op_or` | `(x`\|\|`y)`\|\|`z` |

Non-associative means that expressions like `a==b==c` are simply forbidden.

Operators are equivalent to calls to the associated function.  For example,
`a+b*c` is equivalent to `op_add(a, op_mul(b, c))`.

`rat` uses `<>` to test for inequality (not equals), rather than `!=`.

The operators have their usual meanings on numbers.  The right argument to
`^` (the second argument to `op_pow`) must be an `Int`.

The binary `+` operator (`op_add` function) is overloaded to `L + e`.  It
returns a list with element `e` appended to the end of list `L`.

The comparison operators ( `==`, `<`, etc.) work on any types.  Lists
sort orthographically.  Functions sort like their `path/name`.
And any `Function` is less than any `Rat` which is less than any `List`.

#### Parsing (lexing) shortcomings

In `+1` or `-2`, the sign is not an operator; it is just part of the `Rat`
literal.  That occasinally leads to bad parses.  To work around that, you
can add spaces, an explicit sign, or parentheses.

| Bad expression | Unintended result | Workaround |
| --- | ------ | ---- |
| `x-1` | error: `x -1` | use: `x - 1` or `x- 1` or `x-(1)` or `x-+1` |
| `x+1` | error: `x 1` | use: `x + 1` or `x+ 1` or `x+(1)` or `x++1` |
| `-3^2` | wrong: `(-3)^2` | use: `- 3^2` or `-(3^2)` or `-+3^2` |

### `use`
If function A wants to call function B, function A must have a `use` statement
at the top level (i.e., not within a `loop` or `if`, etc.) referencing A.
For instance, if you want to call the function in the file `sys/is_finite.rat`,
you need to have
```rat
use sys is_finite;
```
in your function.  Then you can call
`is_finite(inf)`.  Without the `use` statement, you would have an error
saying something like `is_finite: not a variable`.  Aside from the requirement
that it be at the top level, it can occur anywhere in the function; the
`use` does not have to precede the call in the function text.

If you want to use `sys/is_finite.rat`, but not refer to it as `is_finite`,
you can have
```rat
use sys is_finite as is_fine;
```
and then call `is_fine(inf)`.  Note that a `use` is somewhat like a `let`;
you cannot `use` two functions as the same name; nor can you create a
constant or variable with the same name as a function.

Every function in `auto` is automatically loaded; no `use` is necessary,
or even allowed.  All of the operators are `auto` functions.

### `function` declarations and `return`
#### `function` with a fixed number of strict arguments
Functions can declare their arguments with a function statement.
```rat
function (first, second, third);
let all = first + second + third; # refer to the arguments
say('Total is', as_text(all));
```
The arguments are constants within the function.
Like `use`, `function` must occur anywhere at the top level, not
necessarily before the arguments are used.
#### `function` with a variable number of strict arguments
The above `function` statement declares that a function must have the specified
number of variables.
```rat
function (); # no arguments
function (a); # one argument
function (x, y); # two arguments
function (one, two, three); # three arguments
```
But you can specify `...` as a final argument to allow a variable number
```rat
function (...); # any number
function (a, ...); # at least one argument
function (a, b, ...); # at least two arguments
```
If `...` is specified, the additional arguments end up in a `List` constant
called `args`.
#### `function` with lazy arguments
```rat
function (lazy ...); # any number of lazy arguments
function (a, lazy b, c, ...); # one strict, one lazy, one strict, and some strict arguments
function (lazy a, b, c, lazy ...);
```
When a function is called, the `lazy` arguments are not evaluated until
they are needed.
This is the `auto/op_and.rat`:
```rat
function (a, lazy b);
if a { b } else { 0 }
```
As in most languages, when you write `a && b`, if `b` is an expression, it
is not evaluated if `a` is `0` (false).
#### `function` for recursion
A recursive function can `use` itself by including a name:
```rat
function factorial(n);
if 0 == n { 1 } else { n * factorial(n - 1); };
```
As with `use as`, the name does not have to match the filename.
#### `return`
A function will return the last value evaluated if it reaches the end of
the file.  But it will return early with the specified value if it
evalues a `return` statements
```rat
return something; # can take an expression
return; # same as: return 0;
```

## Appendix

### Filesystem structure
`rat` assumes the current directory contains five sub-directories which
can contain `rat` functions, which are just text files with the extension
`'.rat'`.  Except for the extension, the name of the function is the name
of the file.
- `auto` contains functions which are automatically loaded into the environment.
- `sys` contains functions considered part of the basic `rat` system.
- `std` contains the `rat` standard library, beyond `sys`.
- `usr` is intended to hold third-party functions.
- `my` is intended to hold your functions.
This is not very sophisticated.  There are no lists of paths, or further
subdirectories.

### Built-in, `auto`, `sys`, and `std` functions.
`rat` includes some built-in functions, which are written in Rust.
Each built-in function behaves as if it is in either `auto` or `sys`.
There should be no distinction when writing `rat` code, so they are
just included in the lists below.

#### `auto` functions

- `as_text(a)` converts to text.  `as_text(3.14)` is `'3.14'`.  But note that
`as_text('ABC')` is `'list[65, 66, 67]'`.
- `catenate(L1, L2)` concatenates two lists, returning a longer list.
- `denominator(r)` returns the denomiator of `r`.  If `r` is `inf`, `-inf`,
or `nan`, it returns `0`.  Note that `Rat` is always fully reduced, so there
is no factor in common with the numerator.  The denominator is always
non-negative; the sign goes with the numerator.
- `element(L, i)` returns the `i`th element of list `L`.  The first
element is `element(L, 1)`.
- `eval(a)` evaluates its argument.  This can be used to use `rat` as a
command-line calculator.
- `inp()` returns a line of input from stdin.
- `input(t)` prints a prompt to stdout, then returns a line from stdin
- `length(L)` returns the length of a list
- `must(b, t)` if `!b`, prints `Text` `t` and exits
- `numerator(r)` returns the numerator (including the sign).
- `op_add(r1, r2)` adds two numbers
- `op_and(b1, lazy b2)` is `1` if both arguments are `1`
- `op_div(r1, r2)` divides
- `op_eq(a1, a2)` compares equal (all comparison operators work on all types)
- `op_ge(a1, a2)` compares greater or equal
- `op_gt(a1, a2)` compares greater
- `op_le(a1, a2)` compares less or equal
- `op_lt(a1, a2)` compares less
- `op_mul(r1, r2)` multiplies
- `op_ne(a1, a2)` compares not equal
- `op_neg(r)` negates
- `op_not(b)` toggles a `Bool`
- `op_or(b1, lazy b2)` is `1` if either argument is `1`
- `op_pow(r, i)` exponentiates
- `op_sub(r1, r2)` subtracts
- `out(t)` writes `Text` to stdout
- `reverse(L)` reverses a `List`
- `round(r)` rounds a `Rat` to the nearest integer.
- `say(...)` prints `Text` arguments, separated by spaces, terminated by a newline
- `sublist(L, i, n)` returns a sub-list of list `L`, starting with the
`i`th element and including `n` elements.

#### `sys` functions
- `abs(r)` absolute value
- `all(L)` - `1` (true) if all elements of `L` are `1`
- `any(L)` - `1` (true) if any elements of `L` are `1`
- `as_decimal(r, n)` display in decimal to `n` places
- `as_text_sci(r)` a rational variant of scientific notation
- `every(L, f)` returns a list with the function `f` applied to each element
of `L`
- `filter(L, f)` returns a list containing elements of `L` for which `f` is `1` (true).
- `gbye(t)` prints `Text` `t` and exits
- `inkey(b)` returns a single character from stdin.  If `b` is `1`, that
character is echoed to stdout; if `b` is `0`, it is not.  Implementation
note: I think this will not work under Windows, and probably will not
even build.
- `inputkey(t)` prompts to stdout; then returns a single character
from stdin.  See inkey().
- `insert_sublist(L1, i, L2)` returns a list with `L2` inserted into `L1` so that
the first element of `L2` has index `i`.
- `is_bool(r)` return `1` if the value is a `Bool`
- `is_char(r)` returns `1` if the value is a `Char`.
- `is_evald(lazy a)` returns `1` if the argument can be evaluated.
- `is_finite(r)` returns `1` for any `Rat` besides `inf`, `-inf` and `nan`
- `is_function(a)` returns `1` if the argument is a `Function`.
- `is_int(a)` returns `1` if the denominator is `1`
- `is_list(a)` returns `1` if the argument is a `List`
- `is_mutable(lazy a)` returns `1` if the argument is a variable.
- `is_rat(a)` returns `1` if the argument is a `Rat`
- `is_text(t)` returns `1` if the argument is `Text`
- `is_var(a)` returns `1` if the argument is a constant or variable.
- `modulo(r, q)` remainder after integer division
- `nyi(t)` exits, printing "NYI" (not yet implemented) and the text
- `outm(...)` prints multiple `Text` arguments.
- `outn(...)` prints multiple `Text` arguments, and a newline.
- `outs(...)` prints multiple `Text` arguments, separated by spaces.
- `parse(t)` converts `Text` to an expression
- `rat(t)` converts `Text` to a `Rat`
- `reduce(L, f, e0)` accumulates (starting with `e0`) the result of applying
binary function `f` to the accumulator and each element in turn.  That is,
it is a generalization of `sum`.
- `replace_element(L, i, e2)` replaces the element of `L` at index `i` with `e2`
- `replace_sublist(L1, i, n, L2)` replaces the sublist of `L` starting at
index `i` of length `n` with the new sublist `L2`
- `sign(r)` return `+1`, `0`, `-1` or `nan` to represent the sign of `r`
- `tree_text(lazy a)` converts an expression to `Text`
- `var_name(lazy v)` converts a constant, variable, or function name to `Text`
- `variable(t)` converts `Text` to a name.
- `without_sublist(L, i, n)` returns a list like `L` without the sublist starting
at index `i` of length `n`.

#### `std` functions
- `prod(L)` - product of all elements of `L`
- `root(r, i, e)` returns a value within `e` of the `i`th root of `r`.
- `sum(L)` - sum of all elements of `L`

#### Keywords and cheat sheet
The `rat` keywords are
- `as` - from `use` ... `as`
- `break` - sudden exit from `loop` or `each`
- `case` - in the `if` family
- `each` - loop over list elements
- `else` - optional after `if` family members
- `function` - argument declaration
- `if`
- `in` - from `each` ... `in`
- `inf` - an infinite number
- `lazy` - declaring lazy arguments in `function`, and keeping the `let`
family from evaluating the assigned value
- `let` - declare a constant
- `list`
- `loop` - loop forever
- `mutable` - declare a variable
- `my` - path
- `nan` - not a number
- `return` - sudden exit from a function
- `std` - path
- `switch` - in the `if` family
- `sys` - path
- `update` - modify a variable
- `use` - reference another function
- `usr` - path
- `when` - in the `if` family

### Examples in `usr` and `my`
#### `my` functions
- `liaber` - perhaps the simplest text adventure possible
- `pi_approx` - prints the sequence of best approximations to pi for
increasing denominators
#### `usr` functions
- `fact_rec`, `fact_loop` - two factorial implementations
- `fib_rec`, `fib_loop` - two fibonacci implementations

### Automated testing
`rat` has a primitive automated testing facility.  It has many limitations,
include a lack of mocks, which means it cannot be used for functions that
do input from the keyboard.  It also does not track coverage.

It is invoked by running `make rtest`, which runs `rtest.py` 10 times, once
for `*.ratu` files in `auto`; then for `*.ratu` files in the other paths
(`sys` `std` `usr` `my`), and then for `*.rati` files in each path.

See the existing `*.ratu` and `*.rati` files for examples.

#### `*.ratu` and `*.rati` file structure
Each file can have multiple tests, ordered sequentially.

##### "`--par`", "`--arg`", and "`--cmd`"
A test starts with a line starting "`--arg`", "`--par`", or "`--cmd`".
The rest of the line is the tail of the command line for the test.

In hopes of running many tests quickly, `make rtest` builds and uses the
release executable, so the start of the command is `./target/release/rat`.

`make rtest` then adds the path flag: `-sys`, `-std`, `-usr` or `-my`,
based on which path it is considering.  For the `auto` path, no flag is
added.

For a `*.ratu` file, the name of the file is then added.  For instance,
for commands in `auto/op_le.ratu`, the command starts
`./target/release/rat op_le`.

After that, the part of the `--cmd` line after `--cmd` itself is added.

If `--arg` is used, the rest of the line is quoted for the shell.
For `*.ratu` files, the filename is included in the quotes.

If `--par` is used, the rest of the line is enclosed in parenthesis;
and then quoted.  For `*.ratu` files, the filename is included in the
quotes, but not the parentheses.

For example, the first line of `auto/op_le.ratu` is `--par 3, 5`, so the
full command run is `./target/release/rat "op_le(3, 5)"`.

The first line of `auto/op_le.ratu` could equivalently be `--arg (3, 5)`.

In a `*.rati` file, the equivalent line would be `--cmd "op_le(3, 5)"` or
`--arg op_le(3, 5)` .

Usually in a `*.ratu` file, `--par` is appropriate, while in a `*.rati`
file, `--arg` is appropriate.

##### "`--rv`"
The next section of a test defines the expected return value from running
the command.  The return value is a small integer returned to the shell,
which you do not usually see.  A successful command has a return value of 0,
while an unsuccessful command has a non-zero return value.  "`--rv 1`" means
you expect the command to have a return value of 1; that implies you expect
the command to fail.

The "`--rv`" section is optional.  If omitted, it treats it like "`--rv 0`".

If you are not sure what return value to expect, omit the line, and if the
actual return value is not zero, `rtest.py` will kindly inform you of the
return value.

##### "`--out`"
The next sections tests what is written to stdout.  If a function called
from the command line completes normally, `rat` will print out its result.
Any output from the function (from `say`, for example) will be before the
result.

###### "`--out exact`" or "`--out`"
If nothing is specified on the "`--out`" line, or it says "`--out exact`",
then the lines that follow should exactly match the actual output from
the command.

If there is a line not starting "`--`" right after either "`--cmd`" or
"`--rv`", it implicitly starts a "`--out exact`" section

###### "`--out has`"
If `has` is specified, each line of the "`--out has`" section must occur
somewhere in the actual output.  They do not have to match full lines.
They do not have to occur in order.  They do not have to occur on different
lines of the actual input.

The test fails if any of the lines in the section does not match.

###### "`--out re`"
If `re` is expected, each line of the section is treated as a regular
expression which must match against some line of the actual output.
Multi-line matches are not supported.  As with `has`, the different
lines are independent.

##### "`--err`"
The final section is "`--err`".  It is much like "`--out`", but matches
against what is written to stderr.

Unlike "`--out`", "`--err`" defaults to checking `has`.  "`--err exact`"
can still be specified explicitly.

"`--err re`" is also supported.

### Operating systems

I believe that `rat` will not compile under Windows.
This limitation comes from the implementation of one built-in
function (`inkey`) which is not particularly important (it captures a keypress
from the keyboard).  It should be easy to make that an optional feature,
or even find a way to implement it on Windows; but I have not done so.
