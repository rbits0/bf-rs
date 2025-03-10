# bf-rs
A [brainfuck](https://en.m.wikipedia.org/wiki/Brainfuck) interpreter written in Rust.

## Features
- Display memory and instructions
- Step-by-step debugging
- Breakpoints
- Macros (maybe I need a better name)

## Usage
`bf-rs [OPTIONS] <FILEPATH>`

### Breakpoints
Enable with `-b`

Mark breakpoints with `@`

## Macros
Enable with `-m`

Define macros like this:
```
macro_name {
    [+>-<]
}
```
and call it like this: `@macro_name@`

Recursion is not allowed, so make sure no macros can call themselves

## Debug Modes
### Verbose
Enable with `-d verbose`
Prints memory and the current character for each step

### Step
Enable with `-d step`
Same as verbose, but pauses at each step until you press enter
