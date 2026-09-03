# xccute_contract

The traits a generated command implements. This is the runtime half of the
subcommand derives in
[`simple_impl_derive`](https://crates.io/crates/simple_impl_derive): the
derive writes the impls, this crate owns the vocabulary they implement, and a
program that executes commands depends on this crate alone.

A command here is data, never a string handed to a shell:

- `CompositeShellRoot`: a root program (`git`, `orb`) named once.
- `CompositeArgvPart`: something that pushes its segment and arguments onto
  an argv of `OsString`s.
- `CompositeShellCommand`: a whole command under a root: `argv()`,
  `build_display()`, `preview()`, `to_std_command()`, `xccute()`.
- `ValidatedCommand`: `validate()` runs the structural rules (`requires`,
  `invalid_without`, `only_pair_with`, `conflicts_with`, `one_of`,
  `at_least_one_of`) and preflight hooks, returning a
  `CommandValidationError` that names the field and the rule.
- `ValidatedCompositeShellCommand`: `validated_preview()`,
  `approved_preview(&policy)` and `xccute_validated()`, in that order of
  trust: nothing runs before validation, and a `CommandPreviewPolicy` hands
  back a `CommandApprovalReceipt` before execution.
- `XccutePolicyMetadata`: `sensitive`, `requires_sudo`, `iam_scope`,
  `path_role`, `dry_run_default` attached to a command type.
- `RootableSubCommand` / `RootableCompositeSurface`: how a leaf command or a
  nested surface is placed under a root.

No dependencies.

## License

MIT OR Apache-2.0.
