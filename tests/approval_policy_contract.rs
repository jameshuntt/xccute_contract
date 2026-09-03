use std::ffi::{OsStr, OsString};

use xccute_contract::{
    CommandApproval, CommandApprovalError, CommandPolicyError, CommandPolicyResult,
    CommandPreview, CommandPreviewPolicy, CommandValidationError, CommandValidationResult,
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

struct ApprovingPolicy;

impl CommandPreviewPolicy for ApprovingPolicy {
    fn approve(&self, preview: &CommandPreview) -> CommandPolicyResult {
        assert_eq!(preview.program(), OsStr::new("git"));
        Ok(CommandApproval::new("operator_ack", "james").with_reason("reviewed dry-run preview"))
    }
}

struct RejectingPolicy;

impl CommandPreviewPolicy for RejectingPolicy {
    fn approve(&self, preview: &CommandPreview) -> CommandPolicyResult {
        Err(CommandPolicyError::new(
            "operator_ack",
            format!("refusing preview `{}`", preview.display()),
        ))
    }
}

#[test]
fn approved_preview_runs_validation_and_policy_before_receipt() {
    let cmd = PreviewCommand {
        args: vec![OsString::from("status"), OsString::from("--short")],
    };

    let receipt = cmd
        .approved_preview(&ApprovingPolicy)
        .expect("valid command should receive approval receipt");

    assert_eq!(receipt.preview().display(), "git status --short");
    assert_eq!(receipt.approval().policy(), "operator_ack");
    assert_eq!(receipt.approval().actor(), "james");
    assert_eq!(receipt.approval().reason(), Some("reviewed dry-run preview"));

    let std_command = receipt.to_std_command();
    assert_eq!(std_command.get_program(), OsStr::new("git"));
    assert_eq!(
        std_command.get_args().collect::<Vec<_>>(),
        vec![OsStr::new("status"), OsStr::new("--short")]
    );
}

#[test]
fn approved_preview_returns_validation_error_before_policy_runs() {
    let error = InvalidPreviewCommand
        .approved_preview(&ApprovingPolicy)
        .expect_err("invalid command should fail before policy approval");

    match error {
        CommandApprovalError::Validation(error) => {
            assert_eq!(error.field(), Some("delete"));
            assert_eq!(error.rule(), Some("invalid_without"));
        }
        CommandApprovalError::Policy(_) => panic!("validation should run before policy approval"),
    }
}

#[test]
fn approved_preview_returns_policy_error_for_rejected_preview() {
    let cmd = PreviewCommand {
        args: vec![OsString::from("status"), OsString::from("--short")],
    };

    let error = cmd
        .approved_preview(&RejectingPolicy)
        .expect_err("policy should reject the otherwise valid preview");

    match error {
        CommandApprovalError::Policy(error) => {
            assert_eq!(error.policy(), "operator_ack");
            assert!(error.message().contains("git status --short"));
        }
        CommandApprovalError::Validation(_) => panic!("valid command should reach policy approval"),
    }
}
