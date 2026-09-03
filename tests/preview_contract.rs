use std::ffi::{OsStr, OsString};

use xccute_contract::{
    CommandPreview, CommandValidationError, CommandValidationErrorKind, CommandValidationResult,
    CompositeShellCommand, ValidatedCommand, ValidatedCompositeShellCommand,
};

#[derive(Default)]
struct PreviewCommand {
    args: Vec<OsString>,
}

impl CompositeShellCommand for PreviewCommand {
    fn program(&self) -> &OsStr {
        OsStr::new("git")
    }

    fn push_args(&self, out: &mut Vec<OsString>) {
        out.extend(self.args.iter().cloned());
    }
}

impl ValidatedCommand for PreviewCommand {}

struct InvalidPreviewCommand;

impl CompositeShellCommand for InvalidPreviewCommand {
    fn program(&self) -> &OsStr {
        OsStr::new("danger")
    }

    fn push_args(&self, out: &mut Vec<OsString>) {
        out.push(OsString::from("delete"));
        out.push(OsString::from("target"));
    }
}

impl ValidatedCommand for InvalidPreviewCommand {
    fn validate(&self) -> CommandValidationResult {
        Err(CommandValidationError::structural("delete requires explicit force")
            .with_field("delete")
            .with_rule("invalid_without"))
    }
}

#[test]
fn command_preview_preserves_trusted_program_argv_and_debug_display() {
    let preview = CommandPreview::new(
        OsString::from("git"),
        vec![OsString::from("status"), OsString::from("--short")],
    );

    assert_eq!(preview.program(), OsStr::new("git"));
    assert_eq!(
        preview.argv(),
        &[OsString::from("status"), OsString::from("--short")]
    );
    assert_eq!(preview.display(), "git status --short");
}

#[test]
fn composite_command_can_build_no_side_effect_preview() {
    let cmd = PreviewCommand {
        args: vec![OsString::from("remote"), OsString::from("-v")],
    };

    let preview = cmd.preview();

    assert_eq!(preview.program(), OsStr::new("git"));
    assert_eq!(
        preview.argv(),
        &[OsString::from("remote"), OsString::from("-v")]
    );
    assert_eq!(preview.display(), "git remote -v");
}

#[test]
fn validated_preview_runs_validation_before_materializing_receipt() {
    let error = InvalidPreviewCommand
        .validated_preview()
        .expect_err("invalid command should not preview as acknowledged");

    assert_eq!(error.kind(), &CommandValidationErrorKind::Structural);
    assert_eq!(error.field(), Some("delete"));
    assert_eq!(error.rule(), Some("invalid_without"));
}

#[test]
fn command_preview_can_materialize_std_command_without_shell_string() {
    let preview = CommandPreview::new(
        OsString::from("git"),
        vec![OsString::from("commit"), OsString::from("-a")],
    );
    let command = preview.to_std_command();

    assert_eq!(command.get_program(), OsStr::new("git"));
    assert_eq!(
        command.get_args().collect::<Vec<_>>(),
        vec![OsStr::new("commit"), OsStr::new("-a")]
    );
}
