use std::ffi::{OsStr, OsString};

use xccute_contract::{
    CommandExecutionError, CommandValidationError, CommandValidationErrorKind,
    CommandValidationResult, CompositeShellCommand, ValidatedCommand,
    ValidatedCompositeShellCommand,
};

#[derive(Default)]
struct DummyCommand {
    args: Vec<OsString>,
}

impl CompositeShellCommand for DummyCommand {
    fn program(&self) -> &OsStr {
        OsStr::new("dummy")
    }

    fn push_args(&self, out: &mut Vec<OsString>) {
        out.extend(self.args.iter().cloned());
    }
}

impl ValidatedCommand for DummyCommand {}

struct FailingCommand;

impl CompositeShellCommand for FailingCommand {
    fn program(&self) -> &OsStr {
        OsStr::new("danger")
    }

    fn push_args(&self, out: &mut Vec<OsString>) {
        out.push(OsString::from("delete"));
        out.push(OsString::from("target"));
    }
}

impl ValidatedCommand for FailingCommand {
    fn validate(&self) -> CommandValidationResult {
        Err(CommandValidationError::structural("delete is invalid without force")
            .with_field("delete")
            .with_rule("invalid_without"))
    }
}

#[test]
fn validation_error_carries_kind_field_rule_and_message() {
    let error = CommandValidationError::runtime_preflight("source path does not exist")
        .with_field("source")
        .with_rule("path_exists");

    assert_eq!(error.kind(), &CommandValidationErrorKind::RuntimePreflight);
    assert_eq!(error.message(), "source path does not exist");
    assert_eq!(error.field(), Some("source"));
    assert_eq!(error.rule(), Some("path_exists"));
    assert_eq!(
        error.to_string(),
        "runtime_preflight validation failed: source path does not exist [field=source] [rule=path_exists]"
    );
}

#[test]
fn default_validated_command_is_noop_and_preserves_argv_safe_command_path() {
    let cmd = DummyCommand {
        args: vec![OsString::from("status"), OsString::from("--short")],
    };

    assert!(cmd.validate().is_ok());
    assert!(cmd.to_validated_std_command().is_ok());
    assert_eq!(cmd.argv(), vec![OsString::from("status"), OsString::from("--short")]);
    assert_eq!(cmd.build_display(), "dummy status --short");
}

#[test]
fn validated_composite_command_blocks_invalid_command_before_execution() {
    let error = FailingCommand
        .to_validated_std_command()
        .expect_err("structural validation should fail");

    assert_eq!(error.kind(), &CommandValidationErrorKind::Structural);
    assert_eq!(error.field(), Some("delete"));
    assert_eq!(error.rule(), Some("invalid_without"));
}

#[test]
fn validation_errors_can_flow_into_validated_execution_error() {
    let error = CommandExecutionError::from(
        CommandValidationError::custom("custom hook rejected command").with_rule("validate_with"),
    );

    match error {
        CommandExecutionError::Validation(validation) => {
            assert_eq!(validation.kind(), &CommandValidationErrorKind::Custom);
            assert_eq!(validation.rule(), Some("validate_with"));
        }
        CommandExecutionError::Io(_) => panic!("expected validation error"),
    }
}
