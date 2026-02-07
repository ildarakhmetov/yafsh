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
yafsh> "hello" echo       # "hello" is Str → becomes arg → runs: echo hello
hello                      # Output auto-prints
yafsh[:1]> wc              # Output from echo → pipes as stdin to wc
```

The prompt tells you what's on the stack:

| Prompt | Meaning |
|--------|---------|
| `yafsh>` | stack empty |
| `yafsh[3]>` | 3 inputs (Str/Int), no outputs |
| `yafsh[:2]>` | no inputs, 2 outputs (Output) |
| `yafsh[2:1]>` | 2 inputs + 1 output |

The `.s` command shows the full stack with type markers:
`"hello"` for Str, `42` for Int, `«data»` for Output.

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
<3> "a" "b" «c»
```

### Word definitions

```
yafsh> : hi "hello, world!" . ;
yafsh> hi
hello, world!
yafsh> : twice dup . . ;
yafsh> "yo" twice
yo
yo
```

### Control flow

```
yafsh> 1 if "yes" else "no" then .
yes
yafsh> 0 if "yes" else "no" then .
no
```

### Type conversions and exit codes

```
yafsh> "data" >output         # Str -> Output (makes it pipeable)
yafsh> 42 >string             # Int -> Str
yafsh> /bin/false             # run a failing command
yafsh> ? .                    # print last exit code
1
```

### Feature list

- **Values**: strings (`"hello"`), integers (`42`), captured output
- **Stack ops**: `dup`, `swap`, `drop`, `clear`, `over`, `rot`
- **I/O**: `.` (print), `.s` (show stack), `type` (print, no newline), `>output`, `>string`
- **Shell**: auto PATH lookup, auto-piping of output as stdin, depth control, `cd`, `?` (exit code)
- **Word definitions**: `: greet "hello" . ;`
- **Control flow**: `if` / `else` / `then`
- **Globs**: `*.rs` expands to matching files

## Acknowledgements

Inspired by and based on [fsh](https://github.com/AlexanderBrevig/fsh) by Alexander Brevig -- an elegant Forth-based shell written in OCaml. Thank you for the great reference implementation!

## License

MIT
