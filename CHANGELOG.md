# Changelog

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
