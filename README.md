[![Crates.io](https://img.shields.io/crates/v/mar)](https://crates.io/crates/mar)
[![docs.rs](https://img.shields.io/docsrs/mar)](https://docs.rs/mar)
[![License](https://img.shields.io/github/license/Mossop/mar)](https://www.mozilla.org/en-US/MPL/)

This is a Rust implementation of the [Mozilla Archive (MAR) file format][1]
used to deliver automatic updates to Firefox.  It includes both a library and
a command-line tool for reading and writing MAR files.

Currently supports:

* Reading the list of files in a MAR archive
* Extracting file content from a MAR archive

Not yet supported:

* Creating MAR archives
* Signing MAR archives
* Verifying signed MAR archives

This code is subject to the terms of the Mozilla Public License, v. 2.0.

[1]: https://wiki.mozilla.org/Software_Update:MAR
