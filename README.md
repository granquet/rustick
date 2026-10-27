Description
===

A very basic metronome written in Rust

By default, the metronome will run the pattern in an infinite loop.
If you want the metronome to stop after a certain number of patterns (bars), provide the --bars argument.

Unit defines the value of the note in the pattern, the default 4 is 1 click = a quarter note.
In effect `--unit 4` and a pattern containing 4 clicks is 4/4

The click pattern has 4 different sounds:
```
X -> Accent
x -> Normal
- -> Upbeat
. -> Silence
```

The default pattern "Xxxx" is the standard 4 notes bar with the first note accentuated.
A pattern with: "X-x-X-x-" and `--unit 8` will have 1 and 3 accented, 2 and 4 as normal notes and upbeats in between.

The default is to use your terminal to show you bars and notes, but a slint UI can be shown with the `--gui` selector.

Usage: rustick [OPTIONS]
===

```
Options:
  -b, --bars <BARS>
  -t, --tempo <TEMPO>      [default: 80]
  -u, --unit <UNIT>        [default: 4]
  -p, --pattern <PATTERN>  [default: Xxxx]
      --gui
  -h, --help               Print help
  -V, --version            Print version
```
