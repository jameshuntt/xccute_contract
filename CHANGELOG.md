# Changelog

## 0.1.3 (2026-10-19)

a command could sit under a root, or under one surface under a root, and the derive's wrapper for the second only carried the leaf's argv, validation and policy. a leaf two surfaces down had nowhere to go, and a leaf one surface down lost its setters and its exit statuses, so custodian flattened `job start|close`, `count report` and `mcp serve` into one leaf with an action word. nothing here could say a command's help line either; custodian kept a usage table beside its declarations. these are the traits the derive writes into for both, added beside the old ones, so every impl that exists keeps building.

### what's new

- the bridges stopped at a root: `RootableSubCommand` places a leaf under a root and `RootableCompositeSurface` a surface under a root: [src/lib.rs L736-L762](https://github.com/jameshuntt/xccute_contract/blob/ad41aacd689226667398e1cf401d48b57611c889/src/lib.rs#L736-L762). `RootableNestedCommand<R, P>` places a leaf after the words of `P`, the surface it sits under, keeping the leaf's own wrapper with its setters and statuses; `RootableNestedSurface<R, P>` places a surface under a surface, and its wrapper is itself a `CompositeArgvPart`, the prefix for what is under it, so the depth has no limit: [src/lib.rs L764-L794](https://github.com/jameshuntt/xccute_contract/blob/34f29777afa0206038e82fd7a40914a8dca8deda/src/lib.rs#L764-L794).
- `HelpLine` is one line a declaration answers to: its words (`start [NAME] [--kind KIND]`) and its doc. `under(word)` leads it with a surface's segment or a root's program: [src/lib.rs L796-L824](https://github.com/jameshuntt/xccute_contract/blob/34f29777afa0206038e82fd7a40914a8dca8deda/src/lib.rs#L796-L824).
- `CommandHelp::help_lines()`: a leaf has one line, a surface every line under it led by its segment, a root every line under it led by its program: [src/lib.rs L826-L833](https://github.com/jameshuntt/xccute_contract/blob/34f29777afa0206038e82fd7a40914a8dca8deda/src/lib.rs#L826-L833).

### proof

- a leaf two surfaces down, written by hand the way the derive writes it, builds `orb project member add cp --role owner` through its own setter and names its statuses; the help lines come back led by each parent, and an empty word or line leaves no stray space: [tests/nested_contract.rs L145-L161](https://github.com/jameshuntt/xccute_contract/blob/34f29777afa0206038e82fd7a40914a8dca8deda/tests/nested_contract.rs#L145-L161).
- the derive's three-level golden and its built-and-run test are in simple_impl_derive 0.2.7.
- public API against 0.1.2 (rustdoc JSON): 13 items added, none removed or changed.
- `cargo test`: 15 passed (stable and 1.70).

### README and Cargo.toml

- the two bridges and `CommandHelp` in the list of traits: [README.md L31-L35](https://github.com/jameshuntt/xccute_contract/blob/34f29777afa0206038e82fd7a40914a8dca8deda/README.md#L31-L35).
- version 0.1.2 to 0.1.3: [Cargo.toml L3-L3](https://github.com/jameshuntt/xccute_contract/blob/34f29777afa0206038e82fd7a40914a8dca8deda/Cargo.toml#L3-L3).

## 0.1.2 (2026-10-18)

a command could say which exit codes it accepts, but not what any code means. a program that reads a status by name (systemctl is-active's 3, inactive; ssh's 255, the login failed) had to keep its own table beside the command and turn it into rules by hand. now the command names its statuses itself, and the derive can declare them.

### what's new

- `CompositeShellCommand` only had `ok_codes` and `accept_status`: [src/lib.rs L97-L108](https://github.com/jameshuntt/xccute_contract/blob/b774ab76a5746021d404c108f821bb1653f847a7/src/lib.rs#L97-L108). it gains two methods with defaults, so every impl that exists keeps building: `exit_statuses()`, the statuses the command names in the order they were declared (none unless it declares them), and `exit_status_of(code)`, the named status a process ended with: [src/lib.rs L109-L121](https://github.com/jameshuntt/xccute_contract/blob/b2de328f6d389ca59a8a8b9830d5c1631af43ab4/src/lib.rs#L109-L121).
- `ExitStatusWord` is one named status: the code, the program's word for it, and whether the line goes on after it. `accepted` and `rejected` are const constructors, so a table of them can be a const, and `of(code)` matches a process's code, a signal never matching: [src/lib.rs L123-L150](https://github.com/jameshuntt/xccute_contract/blob/b2de328f6d389ca59a8a8b9830d5c1631af43ab4/src/lib.rs#L123-L150).

### proof

- a command naming is-active's 0 and 3 finds each, finds none for a code it does not name or for a signal; a command naming nothing names nothing and still accepts 0: [tests/exit_status_contract.rs L35-L49](https://github.com/jameshuntt/xccute_contract/blob/b2de328f6d389ca59a8a8b9830d5c1631af43ab4/tests/exit_status_contract.rs#L35-L49).
- `cargo test`: 13 passed (stable and 1.78).

### Cargo

- version 0.1.1 to 0.1.2, and CHANGELOG.md is in the package: [Cargo.toml L3](https://github.com/jameshuntt/xccute_contract/blob/b2de328f6d389ca59a8a8b9830d5c1631af43ab4/Cargo.toml#L3-L3), [Cargo.toml L13](https://github.com/jameshuntt/xccute_contract/blob/b2de328f6d389ca59a8a8b9830d5c1631af43ab4/Cargo.toml#L13-L13).
