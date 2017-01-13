Special Forms
============

This document describes the implemented special forms.

The following syntax is used:

- `(proc a b)` means `proc` takes 2 arguments (here named `a` and `b`).
- `(proc opt?)` means `proc` takes 0 or 1 argument (here called `opt`).
- `(proc args*)` means `proc` takes 0 or more arguments, collectively called `args`.
- `(proc args+)` means `proc` takes 1 or more arguments, collectively called `args`.
- `(proc a rest*)` means `proc` takes 1 or more arguments. Here the first one is called `a`, the rest (if any) `rest`.

##Content

- [define](#define)
- [quote](#quote)
- [if](#if)
- [lambda](#lambda)
- [let](#let)
- [loop](#loop)
- [recur](#recur)
- [begin](#begin)

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

    (define mul (lambda (x y) (* x y)))
    mul
      => [PROC: (lambda (x y) (* x y))
    (mul 2 3)
      => 6

    (define f (x) (define bla x))
    (f 12)
    bla
      => error: bla not defined

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

##loop

Same as [let](#let), but defines a recursion point, see [recur](#recur).

##recur

`(recur args*)`

- args: some values

This is used to write recursive functions and loops. `recur` returns a special
recur-value, that carries any `args` you supply. This value triggers an error if
you attempt to evaluate/use it anyhow. The only way it can be used is with recurion
points.

There are 2 ways to establish a recursion point: [lambda](#lambda) and [loop](#loop).
If you return a recur-value in the bodies of either, instead of returning the value,
control flow jumps to the start of the lambda/loop. The number of `args` must match
the number of args (lambda) or bindings (loop). The args/bindings are rebound with the
`args`, then the body is evaluated again with the new args.

`recur` makes it possible to write recursive functions and loops, which get tail-call-optimized
deterministically.

###Examples

```clojure
    (define count-down (lambda (n)
      (if (= 0 n)
        'done
        (recur (- n 1))
      )
    ))
    (count-down 10000)
      => done

    (define recursive-count-down (lambda (n)
      (if (= 0 n)
        'done
        (recursive-count-down (- n 1))
      )
    ))
    (recursive-count-down 10000)
      => STACK_OVERFLOW

    (define fac (lambda (n)
      (loop (n n res 1)
        (if (< n 2)
          res
          (recur (- n 1) (* res n))
        )
      )
    ))
    (fac 6)
      => 720
```

## begin

`(begin expr* last)`

- expr: some s-expressions
- last: some s-expression

Evaluates all `expr`essions and `last`, returns the value of `last`.

###Examples

    (begin 1 2 3)
      => 3

    (begin (define a 1) 2 3)
      => 3
    a
      => 1
