# treegrep

Prompted by @siraben, I wrote a little tool that allows one to do semantic find and replace on a file. It's nothing more than a little demo for now (and the code is a bit of a mess as I wrote it in under an hour), but I hope you'll find it interesting.

Make sure to clone recursively!

```bash
git clone https://github.com/slightknack/treegrep --recursive
```

Here's the output for the default example:

```
DONE!

given the following source code:
x = 1 + 2
y = True

I searched for the following tree-sitter query:
(binary_operator (integer) @a (integer) @b) @sub

This returned the following branch of the AST:
1 + 2

Using the following replacement template:
at_b + at_a

I spliced in the captured AST patterns to produce:
x = 2 + 1
y = True
```

This find-and-replace pattern swaps around the arguments to a binary addition operation.

Everything is in `src/main.rs`. Happy hacking!