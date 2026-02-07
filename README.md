# yafsh

Yet another Forth-based shell, written in Rust. A fun weekend project.

## What works

Stack-based shell using RPN -- push arguments, then execute commands.

```
yafsh> "hello" "world" echo    # push args, run command
hello world
yafsh> ls                      # output auto-prints and stays on stack
yafsh[:1]> "-l" wc             # output auto-pipes as stdin
```

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
