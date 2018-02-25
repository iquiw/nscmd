# nscmd

execute command with Non Standard CoMmanD aliasing.

## Overview

On NetBSD, for example, `tar` command is NetBSD tar. GNU tar is installed as `gtar` via pkgsrc.
Some projects use GNU tar specific option as `tar` command option, and it fails.

In such case, execute the command like following;

``` console
$ nscmd tar=/usr/pkg/bin/gtar make
```

## Install

``` console
$ cargo install --git https://github.com/iquiw/nscmd
```

