# yafsh

Yet another Forth-based shell, written in Rust. A fun weekend project.

## What works

Stack-based shell using RPN -- push arguments, then execute commands.

### How it works

There are three value types on the stack:

- **Str** -- strings and command arguments (`"hello"`, unquoted words)
- **Int** -- integers (`42`, `-1`)
- **Output** -- captured command output (result of running a shell command)

The key distinction is between **Str** and **Output**. When a command runs, it
consumes **Str/Int** values as command-line arguments and **Output** values as
stdin. The command's stdout is captured and pushed back as a new **Output**.

```
yafsh> "hello" echo       # "hello" is Str -> becomes arg -> runs: echo hello
hello                      # Output auto-prints
yafsh[:1]> wc              # Output from echo -> pipes as stdin to wc
```

The prompt tells you what's on the stack:

| Prompt | Meaning |
|--------|---------|
| `yafsh>` | stack empty |
| `yafsh[3]>` | 3 inputs (Str/Int), no outputs |
| `yafsh[:2]>` | no inputs, 2 outputs (Output) |
| `yafsh[2:1]>` | 2 inputs + 1 output |

The `.s` command shows the full stack with type markers:
`"hello"` for Str, `42` for Int, and `<<data>>` for Output.

### Basics

```
yafsh> "hello" .              # push a string, print it
hello
yafsh> 1 2 3 .s              # push integers, show stack
<3> 1 2 3
yafsh> clear                  # wipe the stack
```

### Running commands

```
yafsh> "hello" "world" echo   # push args, then command
hello world
yafsh> ls                     # output auto-prints, stays on stack
yafsh[:1]> "-l" wc            # output auto-pipes as stdin
8
```

### Piping and depth control

```
yafsh> ls                     # capture directory listing
yafsh[:1]> "-i" grep src      # pipe through grep
yafsh[:1]> "-c" wc            # count matching lines
```

```
yafsh> "a" "b" "c" 1 echo    # depth limit: only "c" goes to echo
yafsh[2:1]> .s               # "a" and "b" remain on stack
<3> "a" "b" <<c>>
```

### Arithmetic

Standard Forth-style RPN arithmetic. Push operands, then the operator:

```
yafsh> 3 7 + .                # 3 + 7 = 10
10
yafsh> 10 3 - .               # 10 - 3 = 7
7
yafsh> 6 7 * .                # 6 * 7 = 42
42
yafsh> 15 3 / .               # 15 / 3 = 5
5
yafsh> 10 3 mod .             # 10 mod 3 = 1
1
yafsh> 10 3 /mod .s           # quotient and remainder
<2> 3 1
```

Chain operations naturally:

```
yafsh> 2 3 + 4 * .            # (2 + 3) * 4 = 20
20
```

### Comparisons and boolean logic

Comparisons return `1` (true) or `0` (false). `=` and `<>` work on both
integers and strings; `>`, `<`, `>=`, `<=` are integer-only:

```
yafsh> 5 3 > .                # 5 > 3? yes
1
yafsh> "hello" "hello" = .    # string equality
1
yafsh> 5 3 <> .               # not equal
1
```

Boolean operators treat `0` as false and any non-zero integer as true:

```
yafsh> 1 1 and .              # true AND true
1
yafsh> 1 0 or .               # true OR false
1
yafsh> 0 not .                # NOT false
1
yafsh> 1 0 xor .              # exclusive or
1
```

### Control flow

```
yafsh> 1 if "yes" else "no" then .
yes
yafsh> 0 if "yes" else "no" then .
no
```

Combine with comparisons for conditional logic:

```
yafsh> 5 3 > if "big" else "small" then .
big
```

### Loops

Forth-style loops for iteration:

```
yafsh> 0 begin 1 + dup 5 = until .       # begin...until (runs at least once)
5
yafsh> 5 begin dup 0 > while dup . 1 - repeat drop   # begin...while...repeat
5
4
3
2
1
yafsh> 0 5 do i . loop                   # counted loop with index
0
1
2
3
4
yafsh> 0 10 do i . 2 +loop               # counted loop with step
0
2
4
6
8
```

Iterate over output lines with `each ... then`:

```
yafsh> ls each "file: " swap concat . then
file: src
file: Cargo.toml
...
```

Nested loops use `i` for the inner index and `j` for the outer:

```
yafsh> 0 2 do 0 2 do j 10 * i + . loop loop
0
1
10
11
```

### Word definitions

```
yafsh> : hi "hello, world!" . ;
yafsh> hi
hello, world!
yafsh> : square dup * ;
yafsh> 5 square .
25
yafsh> : positive? 0 > if "yes" else "no" then . ;
yafsh> 5 positive?
yes
```

### String operations

```
yafsh> "hello " "world" concat .
hello world
```

### Type conversions and exit codes

```
yafsh> "data" >output         # Str -> Output (makes it pipeable)
yafsh> 42 >string             # Int -> Str
yafsh> /bin/false             # run a failing command
yafsh> ? .                    # print last exit code
1
```

### Environment variables

```
yafsh> "/usr/local/bin" "PATH" env-prepend   # prepend to PATH
yafsh> "HOME" getenv .                       # read an env var
/home/user
yafsh> "myval" "MY_VAR" setenv               # set a variable
yafsh> "MY_VAR" unsetenv                     # remove it
```

### Directory navigation

```
yafsh> "/tmp" pushd           # save current dir, change to /tmp
yafsh> popd                   # return to saved directory
yafsh> "~" cd                 # cd supports ~ expansion
```

### File I/O

Write command output to files:

```
yafsh> ls >output "listing.txt" >file       # write to file (truncate)
yafsh> "more data" >output "log.txt" >>file # append to file
```

### Introspection

```
yafsh> words                  # list all available words
yafsh> help                   # show built-in help
yafsh> "dup" see              # show documentation for a word
dup: ( a -- a a ) Duplicate top item
```

### Feature list

- **Values**: strings (`"hello"`), integers (`42`), captured output
- **Stack ops**: `dup`, `swap`, `drop`, `clear`, `over`, `rot`
- **I/O**: `.` (print), `.s` (show stack), `type` (no newline), `>output`, `>string`
- **File I/O**: `>file` (write), `>>file` (append)
- **Arithmetic**: `+`, `-`, `*`, `/`, `mod`, `/mod`, `*/`
- **Comparisons**: `=`, `>`, `<`, `>=`, `<=`, `<>`
- **Boolean**: `and`, `or`, `not`, `xor`
- **String**: `concat`
- **Shell**: auto PATH lookup, auto-piping, depth control, `cd`, `?` (exit code)
- **Environment**: `getenv`, `setenv`, `unsetenv`, `env-append`, `env-prepend`, `env`
- **Directory**: `cd`, `pushd`, `popd`
- **Word definitions**: `: square dup * ;`
- **Control flow**: `if` / `else` / `then`
- **Loops**: `begin`/`until`, `begin`/`while`/`repeat`, `do`/`loop`, `do`/`+loop`, `each`/`then`
- **Loop indices**: `i` (inner), `j` (outer)
- **Globs**: `*.rs` expands to matching files
- **Introspection**: `words`, `help`, `see`

## Building

```
cargo build --release
```

## Running

```
cargo run
```

Or after building:

```
./target/release/yafsh
```

## Testing

```
cargo test
```

Tests cover all features including loops, nesting, and error handling.

## Acknowledgements

Inspired by and based on [fsh](https://github.com/AlexanderBrevig/fsh) by Alexander Brevig -- an elegant Forth-based shell written in OCaml. Thank you for the great reference implementation!

## License

MIT
