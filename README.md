# zxcvbn-rs
[![Build Status](https://travis-ci.org/xd009642/zxcvbn-rs.svg?branch=master)](https://travis-ci.org/xd009642/zxcvbn-rs) [![Build status](https://ci.appveyor.com/api/projects/status/34f5i5qf5kip9gx8?svg=true)](https://ci.appveyor.com/project/xd009642/zxcvbn-rs) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

This is a Rust port of [`zxcvbn`](https://github.com/dropbox/zxcvbn) a password strength checker which is based on how password crackers work. This project largely works and the remaining efforts involve testing and improving the user interface. It is currently not actively maintained.

To improve the readability and cleanness of the spatial matching code I created a crate to generated directed keyboard adjacency graphs. It can be found here [`keygraph-rs`](https://crates.io/crates/keygraph-rs).

## Current usage.

To use just run zxcvbn-rs with the password supplied as an argument (quotes will be required for passwords with spaces in).

Here is some current sample output:

```text
===============================================
Password:		password
Guesses raw:		2
Guesses log10:		0.3010299956639812
Score:			VeryWeak
Guess times:
  Online throttled:	0s
  Online unthrottled:	0s
  Offline slow:		0s
  Offline fast:		0s

This is a top-10 common password
===============================================
[
    BaseMatch {
        pattern: "Dictionary",
        start: 0,
        end: 7,
        token: "password",
        data: Dictionary {
            matched_word: "password",
            rank: 2,
            dictionary_name: "Passwords",
            reversed: false,
            l33t: None
        }
    }
]
```

Future work will include improving the messages outputted and implementing the Display trait for the BaseMatch struct.

## Roadmap.

After the functionality in the original zxcvbn project is replicated there is no further features planned. This code was largely a learning exercise of Rust. Further work will likely be on improving facets of the code base and using it as a testing group for tools such as cargo-fuzz.

## Related projects

* [shssoichiro/zxcvbn-rs](https://github.com/shssoichiro/zxcvbn-rs)
