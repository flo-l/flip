My take on implementing a lisp-like language in Rust
======================

This is just for fun. The code is opensource, because I was looking for similar projects
before I started and couldn't find many. So feel free to inspire yourself if you want to write
your own programming language or whatever.

I initially followed this marvelous guide: [Scheme from Scratch](http://peter.michaux.ca/articles/scheme-from-scratch-introduction).

Afterwards I've taken the liberty of diverging from the official scheme spec wherever I felt like it.
I didn't implement automatic tail-call-optimization for example. Instead I did it the
clojure way with `recur`.

I'd love to receive feedback on the code, so if you want to chat about it, ask a question
or propose an improvement feel free to open an issue on GitHub!

###Documentation
Can be found in the docs folder of the repo.

###Screenshots

![REPL Screenshot](https://cloud.githubusercontent.com/assets/5130545/21939319/84b57bdc-d9bf-11e6-95c8-a769d90e25b3.png)

###Features
In no particular order and incomplete:

- REPL with history and autocomplete

- Primitive types:
  - Bool (true & false, not #t & #f)
  - Char
  - Integer (i64, no floats)
  - List
  - String (UTF8)
  - Symbol (ASCII, interned strings)

- All types are immutable, you can only change bindings

- Parsing with good error messages (inspired by rustc)

- Evaluate S-expressions

- Define items in current scope with (define *name* *whatever*)

- Define rust fns and make them callable in scheme (see src/native for examples)
  - I implemented some stuff, like basic math, list operations, etc. all in src/native/primitive_forms.rs with a ton of macros to reduce boilerplate
  - Type conversions also in src/native/primitive_forms.rs

- Create scheme procedures with (lambda *optional_name* (args*) code)
  - They have their own scope

- GC: No, just Rc for all values

- Dynamic scopes
  - (let (x 1 y 2 z (+ x y)) (list x y z)) gives you (1 2 3)
  - let behaves like let* in clojure
  - there's also loop, which works like let, but establishes a recursion point, see clojure docs

- Tail calls
  - (recur arg1 arg2 ..) will make a tail call
  - works the same as in clojure
  - recur should only be used in tail position
  - when recur is evaluated, code execution jumps to the next recursion point
  - lambda and loop both create a recursion point
  - example: (loop (x 1) (if (< x 10) (recur (+ x 1)) x )) this is a tail-call-optimized loop that counts from 1 to 10

###Planned Features

- Conditions
- Refactor Value so that it doesn't use Rc for everything, just Strings and Pairs maybe
- Float support
- Maybe a better tokenizer with nom
- Concurrency
- stdlib
