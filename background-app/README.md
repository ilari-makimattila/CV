CV Background
=============

This is a silly code snippet that I hacked together in an hour for my CV
background. It doesn't do anything useful at all.

Running
-------

```bash
cargo run
```

[Rust Playground](https://play.rust-lang.org/?version=stable&mode=release&edition=2018&gist=99dbd48c888c9ff3c2cd96ed1ad72a66)

Explanation
-----------

This is a multi-threading app with one thread producing applications
and the other consuming them. There's also more explicit communication
between the threads. This is because I wanted the produced log to look
nice.
