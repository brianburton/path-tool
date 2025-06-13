```
Usage: path-tool [OPTIONS] <COMMAND>

Commands:
  print   Print the current PATH one directory per line
  new     Build a new PATH from directories
  add     Add directories to front of PATH
  append  Add directories to back of PATH
  help    Print this message or the help of the given subcommand(s)

Options:
  -e, --env <ENV>  Name of path environment variable [default: PATH]
  -f, --filter     Filter non-directories from path
  -p, --pretty     Print path one directory per line
  -n, --normalize  Normalize directory names in path
  -h, --help       Print help
  -V, --version    Print version
```
