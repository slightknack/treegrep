Prompted by @siraben, I wrote a little tool that allows one to do semantic find and replace on a file. It's nothing more than a little demo for now (and the code is a bit of a mess as I wrote it in under an hour), but I hope you'll find it interesting.

Make sure to clone recursively!

```bash
git clone https://github.com/slightknack/tree-sitter-sub --recursive
```

Here's the output for the default example:

```
DONE!

given the following source code:
x = 1 + 0

I matched the following query:
(binary_operator (integer) @a (integer) @b) @sub

and then used the following substitution:
7 + at_a + 1 - at_b + 7

to produce the following spliced fragment:
7 + 1 + 1 - 0 + 7

which (via @sub) corresponds to the following location in the source code:
1 + 0

using this information, I was able to reconstruct the source with the edit applied:
x = 7 + 1 + 1 - 0 + 7
```