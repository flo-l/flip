My take on implementing a scheme-like language in Rust
======================

This is just for fun. The code is opensource, because I was looking for similar projects
before I started and couldn't find many. So feel free to inspire yourself if you want to write
your own programming language or something similar.

I've taken the liberty of diverging from the official scheme spec wherever I felt like it.

I'd love to receive feedback on the code, so if you want to chat about it, ask a question
or propose an improvement feel free to open an issue on GitHub!

I more or less followed this marvelous guide: [Scheme from Scratch](http://peter.michaux.ca/articles/scheme-from-scratch-introduction).

What works so far, in no particular order and incomplete:

- REPL with history and autocomplete

- Primitive types:
  - Bool (true & false, not #t & #f)
  - Char
  - Integer (i64, no floats)
  - Pair
  - List
  - String (UTF8)
  - Symbol (ASCII, interned strings)

- All types are immutable, you can only change bindings

- Parsing with good error messages (inspired by rustc)

- Evaluate S-expressions

- Define items in current scope with (define *name* *whatever*)

- Define rust fns and make them callable in scheme (see src/native.rs for examples)
  - I implemented some stuff, like basic math, list operations, etc. all in native.rs with a ton of macros to reduce boilerplate
  - Also all special forms like if, lambda etc. are implemented there
  - Type conversions also in native.rs

- Create scheme procedures with (lambda *optional_name* (args*) code)
  - They have their own scope

- GC: No, just Rc for all values

- Dynamic scopes
  - (let (x 1 y 2 z (+ x y)) (list x y z)) gives you (1 2 3)
  - let behaves like let* in clojure

- Tail calls
  - (recur arg1 arg2 ..) will make a tail call
  - works the same as in clojure

Planned:

- Conditions
- Refactor Value so that it doesn't use Rc for everything, just Strings and Pairs maybe
- Float support
- Maybe a better tokenizer with nom
- Concurrency
- stdlib
