# yafsh

Yet another Forth-based shell, written in Rust. A fun weekend project.

## What works

Stack-based shell using RPN -- push arguments, then execute commands.

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
