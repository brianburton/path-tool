## Overview

Have you ever wanted to view a long PATH or CLASSPATH in a readable form
to diagnose a problem?  Or wanted to optimize the PATH by removing duplicates
non-existent directories?  If so you might find this program useful.

To view your PATH in a readable form use the `print` command.

```shell
$ echo $PATH
/bin:/sbin:/usr/bin:/Users/brian/.cargo/bin
$ path-tool print
/bin
/sbin
/usr/bin
/Users/brian/.cargo/bin
```

To add one or more directories to the front of your PATH use the `add` command.
The new PATH will be written to stdout so that you can use the value to replace
the environment variable value.

```shell
$ path-tool add /usr/local/bin
/usr/local/bin:/bin:/sbin:/usr/bin:/Users/brian/.cargo/bin
```

To add to the end of the PATH use the `append` command.

```shell
$ path-tool append /usr/local/bin
/bin:/sbin:/usr/bin:/Users/brian/.cargo/bin:/usr/local/bin
```

The `--filter` option removes any non-existent directories from your PATH.
The `--normalize` option does the same as `--filter` but also replaces any symbolic links
with the directory they point to.

## Installation

Sorry, but you'll need to clone this repo and install from source.

## Usage

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
