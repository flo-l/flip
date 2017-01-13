Special Forms
============

This document describes the implemented special forms.

The following syntax is used:

- (proc a b) means proc takes 2 arguments (here named a and b).
- (proc opt?) means proc takes 0 or 1 argument (here called opt)
- (proc args*) means proc takes 0 or more arguments, collectively called args.
- (proc args+) means proc takes 1 or more arguments, collectively called args.
- (proc a rest*) means proc takes 1 or more arguments. Here the first one is called a, the rest (if any) rest

## define

`(define name expr)`

- name: a symbol
- expr: some s-expression

This binds `name` to the value of `expr` in the current scope. If `name` is already bound,
overwrites the binding.

###Examples

    (define a 1)
    a
      => 1
    (define a 42)
    a
      => 42


## quote

`(quote expr)`
`'expr`

- expr: some s-expression

This returns `expr` without evaluating it.

###Examples

    (define a 42)
    a
      => 42
    'a
      => a
    b
      => error: b not defined
    'b
      => b

## if

`(if expr then else)`

- expr: some s-expression evaluating to a bool
- then: some s-expression
- else: some s-expression

If `expr` returns true evaluates `then` and returns its result, else it evaluates `else`
and returns its result.

###Examples

  (if true 1 2)
    => 1
  (if false 1 2)
    => 2
  (if true "no error" undefined_symbol)
    => "no error"
  (if false "no error" undefined_symbol)
    => error: undefined_symbol not defined

## lambda

`(lambda name? args body)`

- name: a symbol representing the lambdas name
- args: a list of symbols `(symbol*)`
- body: some s-expression

This creates a new procedure (function, if you want). You can optionally add a
`name` for debugability. `args` defines the arguments your procedure takes.

When the procedure is called, a new scope is created.
In this scope, the supplied arguments are bound to the names in `args`. Then `body`
is evaluated in the new scope. The return value of `body` is returned and the new scope destroyed.

`lambda` also defines a recursion point, see [recur](#recur) for more info.

###Examples

    (let (x 1) x)
      => 1

    (let (x 1 y 2 z (+ x y)) (list x y z))
      => (1 2 3)

## let

`(let bindings body)`

- bindings: a list of bindings of the form: `(binding*)`
- binding: `name expr` where name is a symbol and expr is some s-expression
- body: some s-expression

This creates a new local scope. It binds the `name`s in `bindings` to the value of their
respective `expr`essions. The expressions are evaluated in the new scope.
This happens in order, so you can refer to already bound variables
in expressions of following variables. `body` is then executed in the new scope.

The value of `body` is returned. The new scope is destroyed afterwards.

###Examples

    (let (x 1) x)
      => 1

    (let (x 1 y 2 z (+ x y)) (list x y z))
      => (1 2 3)

##recur
