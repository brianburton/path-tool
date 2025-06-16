## Overview

Have you ever wanted to view a long PATH or CLASSPATH in a readable form
to diagnose a problem?  Or wanted to optimize the PATH by removing duplicates
and non-existent directories?  If so you might find this program useful.

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

To add to the back of the PATH use the `append` command.

```shell
$ path-tool append /usr/local/bin
/bin:/sbin:/usr/bin:/Users/brian/.cargo/bin:/usr/local/bin
```

To print a summary of problem areas associated with the current path use the `analyze` command.
Reported problems include:

* `Invalid Directories`: Directories in the path that either do not exist or are not directories.
* `Duplicate Directories`: Directories that appear more than once in the path.
* `Shadowed Files`: Files that will not be reachable because a directory earlier in the path contains files with the same name.

```shell
$ path-tool analyze
Invalid Directories:
    None

Duplicate Directories:
    /Users/myname/.cargo/bin

Shadowed Files:
    /usr/local/bin
        dvipdf  =>  /opt/homebrew/bin
        eps2eps  =>  /opt/homebrew/bin
        gs  =>  /opt/homebrew/bin
        gsbj  =>  /opt/homebrew/bin
        gsdj  =>  /opt/homebrew/bin
        gsdj500  =>  /opt/homebrew/bin
        gslj  =>  /opt/homebrew/bin
        gslp  =>  /opt/homebrew/bin
        gsnd  =>  /opt/homebrew/bin
        npm  =>  /opt/homebrew/bin
        npx  =>  /opt/homebrew/bin

    /sbin
        md5sum  =>  /opt/homebrew/bin
        sha1sum  =>  /opt/homebrew/bin
        sha224sum  =>  /opt/homebrew/bin
        sha256sum  =>  /opt/homebrew/bin
        sha384sum  =>  /opt/homebrew/bin
        sha512sum  =>  /opt/homebrew/bin

    /bin
        bash  =>  /opt/homebrew/bin

    /usr/bin
        awk  =>  /opt/homebrew/bin
        cmp  =>  /opt/homebrew/bin
        cpp  =>  /usr/local/bin
        diff  =>  /opt/homebrew/bin
        diff3  =>  /opt/homebrew/bin
```

## Filtering Options

The `--filter` option removes any non-existent directories from your PATH.
The `--normalize` option does the same as `--filter` but also replaces any 
symbolic links with the directory they point to.

## Installation

Clone this repo and install from source.
If you have cargo installed the easy way is with the install command.

```shell
cargo install --path .
```

## Usage

```
Usage: path-tool [OPTIONS] <COMMAND>

Commands:
  print    Print the current PATH one directory per line
  new      Build a new PATH from directories
  add      Add directories to front of PATH
  append   Add directories to back of PATH
  analyze  Analyze the current PATH
  help     Print this message or the help of the given subcommand(s)

Options:
  -e, --env <ENV>  Name of path environment variable [default: PATH]
  -f, --filter     Filter non-directories from path
  -p, --pretty     Print path one directory per line
  -n, --normalize  Normalize directory names in path
  -h, --help       Print help
  -V, --version    Print version
```
