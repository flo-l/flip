Special Forms
============

This document describes the implemented special forms.

The following syntax is used:

- (proc a b) means proc takes 2 arguments (here named a and b).
- (proc args*) means proc takes 0 or more arguments, collectively called args.
- (proc args+) means proc takes 1 or more arguments, collectively called args.
- (proc a rest*) means proc takes 1 or more arguments. Here the first one is called a, the rest (if any) rest

## let

   (let bindings body)

- bindings: a list of bindings of the form: `(binding*)`
- binding: `name expr` where name is a symbol and expr is some s-expression
- body: some s-expression

This creates a new local scope. It binds the names in bindings to the value of their
respective expressions. The expressions are evaluated in the new scope.
This happens in order, so you can refer to already bound variables
in expressions of following variables. Body is then executed in the new scope.

The value of body is returned. The new scope is destroyed afterwards.

###Examples

    (let (x 1) x)
      => 1

    (let (x 1 y 2 z (+ x y)) (list x y z))
      => (1 2 3)
