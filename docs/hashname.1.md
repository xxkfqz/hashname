---
title: HASHNAME(1)
date: 2026-09-12
header: Hashname Manual
footer: hashname 1.0.1
---

# NAME
hashname — utility for renaming files to their SHA256 hash.


# SYNOPSIS
`hashname [OPTION...] [FILES...]`


# DESCRIPTION
hashname can be used to rename files based on the hash of their contents. This is primarily used when you have a directory containing many files without a naming scheme (for example, a directory of cat pictures downloaded from various sites).

The utility uses multithreading to process files. This is especially noticeable if files are stored in cache, on a RAM disk, or on fast NVMe drives.

hashname does not move files, only changes their names. If `--force-rename` (`-F`) is specified, the replacement is performed by renaming the file. More details: <https://doc.rust-lang.org/std/fs/fn.rename.html>.


# OPTIONS
`-d, --dry-run`
: Do not actually rename files

`-f, --force-rehash`
: Process the file even if its name looks like it has already been processed

`-F, --force-rename`
: Rename/replace a file even there if another file with the same name exists

`-i, --ignore-extension`
: Ignore file extensions when checking and renaming

`-v, --verbose`
: Explain what is being done

`-h, --help`
: Print help

`-V, --version`
: Print version


# EXAMPLES
Simple example:
```
hashname data.bin
```


Process all files in current directory, remove duplicates:
```
hashname -f ./*
```


# SEE ALSO
Project repository: <https://github.com/xxkfqz/hashname>