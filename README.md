# Wolff
---------
![example workflow](https://github.com/Flu/wolff-lang/actions/workflows/build.yml/badge.svg)
![example workflow](https://github.com/Flu/wolff-lang/actions/workflows/tests.yml/badge.svg)
## Description
This is the main source repository for the Wolff compiler.

Wolff is both the compiler and the name of the language. It is a LISP-like language that is strongly-typed and has immutability in mind. The compiler is written in Rust as a self-learning exercise, both for compiler theory and implementation language. Once the compiler evolves, maybe we will bootstrap it. Or maybe it will gain sentience.

## Quick start
I will asume you are using GNU/Linux (otherwise, why are you here?).
First, you will need to install rustup with these instructions:
https://www.rust-lang.org/tools/install

Then:
1. `git clone https://github.com/Flu/wolff-lang.git`
2. `cd wolff-lang`
3. `rustup update && rustup install nightly`
4. `rustup override set nightly`
5. `cargo build`

### Opening a prompt

`cargo run`

At this point, the prompt should open and you will have a REPL of the current state of the language, which at the moment doesn't include much. It will look something like this:

```
Wolff interpreter v0.1.0
No previous history
λ
```

If you see anything else it means I fucked up.
Upon exiting the prompt, a file called "history.txt" will be created in the root folder of the project. This will remember the history of your prompt 

### Compiling from a file

Starting the compiler and giving it a file means just giving the path of the file to wolff. If you do this through cargo and not the release executable, we need to make sure cargo doesn't believe the arguments are meant for him:

`cargo run -- ~/file.wolff`

Of course, you are not limited to using the .wolff extension, it can be anything as long as it contains valid Wolff code.

## About the language
Work very much in progress, this changes from day to day. I will not update it in a while.
