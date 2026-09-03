//! The traits a generated argv command implements.
//!
//! This crate is small on purpose: it is the stable target
//! `simple_impl_derive` generates into, and a program that executes
//! commands depends on it alone.
//!
//! What the derives implement:
//!
//! - `CompositeShell` implements [`CompositeShellRoot`] for a root program.
//! - `SimpleSubCommand` / `CompositeSubCommand` implement [`CompositeArgvPart`].
//! - The rooted wrapper types implement [`CompositeShellCommand`] and
//!   [`ValidatedCommand`]; [`ValidatedCompositeShellCommand`] adds the
//!   validated preview, the policy approval receipt, and execution.

use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::process::{Command, ExitStatus, Output};

/// Root executable namespace.
///
/// Examples:
/// - `Git` -> `git`
/// - `Docker` -> `docker`
/// - `Cargo` -> `cargo`
///
/// This is the trait that `#[derive(CompositeShell)]` should generate.
pub trait CompositeShellRoot {
    fn program() -> &'static OsStr;
}

/// A typed argv-producing piece of a composite command.
///
/// Examples:
/// - `GitCommit` pushes: `commit -m ...`
/// - `GitRemote` pushes: `remote`
/// - `GitRemoteAdd` pushes: `add origin ...`
///
/// This is the trait that `#[derive(SimpleSubCommand)]` and
/// `#[derive(CompositeSubCommand)]` should generate.
pub trait CompositeArgvPart {
    fn push_argv_part(&self, out: &mut Vec<OsString>);
}

/// Final executable composite command.
///
/// This is the safe execution target. It executes with:
///
/// `Command::new(program).args(argv)`
///
/// not:
///
/// `sh -c "<string>"`
pub trait CompositeShellCommand {
    fn program(&self) -> &OsStr;

    fn push_args(&self, out: &mut Vec<OsString>);

    fn argv(&self) -> Vec<OsString> {
        let mut out = Vec::new();
        self.push_args(&mut out);
        out
    }

    fn to_std_command(&self) -> Command {
        let mut cmd = Command::new(self.program());
        cmd.args(self.argv());
        cmd
    }

    /// Build a no-side-effect preview of the command contract.
    ///
    /// This is useful for dry-run receipts, UI review, logging, and operator
    /// acknowledgment flows before execution is allowed.
    fn preview(&self) -> CommandPreview {
        CommandPreview::new(self.program().to_os_string(), self.argv())
    }

    fn xccute(&self) -> std::io::Result<Output> {
        self.to_std_command().output()
    }

    /// Human/debug display only.
    ///
    /// This is not the trusted execution path.
    fn build_display(&self) -> String {
        let mut parts = Vec::new();
        parts.push(self.program().to_string_lossy().into_owned());

        for arg in self.argv() {
            parts.push(arg.to_string_lossy().into_owned());
        }

        parts.join(" ")
    }

    /// Acceptable exit codes for this command.
    fn ok_codes(&self) -> &'static [i32] {
        &[0]
    }

    fn accept_status(&self, status: &ExitStatus) -> bool {
        match status.code() {
            Some(code) => self.ok_codes().contains(&code),
            None => status.success(),
        }
    }
}

/// No-side-effect command preview.
///
/// A preview is the argv-safe contract materialized after builder lowering but
/// before process execution. It is intentionally separate from shell display:
/// `program` and `argv` are the trusted values; `display` is only human/debug
/// text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPreview {
    program: OsString,
    argv: Vec<OsString>,
    display: String,
}

impl CommandPreview {
    pub fn new(program: OsString, argv: Vec<OsString>) -> Self {
        let mut parts = Vec::new();
        parts.push(program.to_string_lossy().into_owned());

        for arg in &argv {
            parts.push(arg.to_string_lossy().into_owned());
        }

        let display = parts.join(" ");

        Self {
            program,
            argv,
            display,
        }
    }

    pub fn program(&self) -> &OsStr {
        &self.program
    }

    pub fn argv(&self) -> &[OsString] {
        &self.argv
    }

    pub fn display(&self) -> &str {
        &self.display
    }

    pub fn into_parts(self) -> (OsString, Vec<OsString>, String) {
        (self.program, self.argv, self.display)
    }

    pub fn to_std_command(&self) -> Command {
        let mut cmd = Command::new(&self.program);
        cmd.args(&self.argv);
        cmd
    }
}

/// Explicit no-side-effect arrangement produced from a composed command.
///
/// This is the bridge target for XCCUTE planning layers. It deliberately wraps
/// the argv-safe [`CommandPreview`] instead of the human/display-oriented
/// `ShellCommand::build()` string, so root commands, subcommands, and fragments
/// can be planned without reparsing shell text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XccuteCommandArrangement {
    preview: CommandPreview,
}

impl XccuteCommandArrangement {
    pub fn new(program: OsString, argv: Vec<OsString>) -> Self {
        Self {
            preview: CommandPreview::new(program, argv),
        }
    }

    pub fn from_preview(preview: CommandPreview) -> Self {
        Self { preview }
    }

    pub fn program(&self) -> &OsStr {
        self.preview.program()
    }

    pub fn argv(&self) -> &[OsString] {
        self.preview.argv()
    }

    pub fn display(&self) -> &str {
        self.preview.display()
    }

    pub fn preview(&self) -> &CommandPreview {
        &self.preview
    }

    pub fn into_preview(self) -> CommandPreview {
        self.preview
    }

    pub fn to_std_command(&self) -> Command {
        self.preview.to_std_command()
    }
}

/// Extension trait for final composite commands that can expose an arrangement.
///
/// The blanket impl keeps this as a sidecar over [`CompositeShellCommand`]. It
/// does not change `ShellCommand` compatibility behavior and does not introduce
/// execution.
pub trait XccuteArrangedCommand: CompositeShellCommand {
    fn arrange(&self) -> XccuteCommandArrangement {
        XccuteCommandArrangement::from_preview(self.preview())
    }
}

impl<T> XccuteArrangedCommand for T where T: CompositeShellCommand {}



/// Sensitivity marker attached beside an argv-safe XCCUTE arrangement.
///
/// This is policy metadata only. It does not execute, wrap, or authorize the
/// command by itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum XccuteCommandSensitivity {
    #[default]
    Normal,
    Sensitive,
}

/// Sidecar policy metadata generated beside composite command arrangements.
///
/// The trusted command material remains [`CommandPreview`] / `program + argv`.
/// This metadata tells the planning bridge which behind-the-scenes checks are
/// required before any privileged arrangement can be considered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XccuteCommandPolicyMetadata {
    sensitivity: XccuteCommandSensitivity,
    requires_sudo: bool,
    iam_scope: Option<String>,
    path_roles: Vec<String>,
    dry_run_default: bool,
}

impl Default for XccuteCommandPolicyMetadata {
    fn default() -> Self {
        Self {
            sensitivity: XccuteCommandSensitivity::Normal,
            requires_sudo: false,
            iam_scope: None,
            path_roles: Vec::new(),
            dry_run_default: true,
        }
    }
}

impl XccuteCommandPolicyMetadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sensitive(mut self) -> Self {
        self.sensitivity = XccuteCommandSensitivity::Sensitive;
        self
    }

    pub fn with_sensitive(mut self, sensitive: bool) -> Self {
        if sensitive {
            self.sensitivity = XccuteCommandSensitivity::Sensitive;
        }
        self
    }

    pub fn with_requires_sudo(mut self, requires_sudo: bool) -> Self {
        self.requires_sudo = requires_sudo;
        self
    }

    pub fn with_iam_scope(mut self, iam_scope: impl Into<String>) -> Self {
        self.iam_scope = Some(iam_scope.into());
        self
    }

    pub fn with_optional_iam_scope<S: Into<String>>(mut self, iam_scope: Option<S>) -> Self {
        self.iam_scope = iam_scope.map(Into::into);
        self
    }

    pub fn with_path_role(mut self, path_role: impl Into<String>) -> Self {
        let path_role = path_role.into();
        if !self.path_roles.iter().any(|existing| existing == &path_role) {
            self.path_roles.push(path_role);
        }
        self
    }

    pub fn with_dry_run_default(mut self, dry_run_default: bool) -> Self {
        self.dry_run_default = dry_run_default;
        self
    }

    pub fn merge(mut self, other: Self) -> Self {
        if other.sensitivity == XccuteCommandSensitivity::Sensitive {
            self.sensitivity = XccuteCommandSensitivity::Sensitive;
        }
        self.requires_sudo |= other.requires_sudo;
        if other.iam_scope.is_some() {
            self.iam_scope = other.iam_scope;
        }
        for path_role in other.path_roles {
            if !self.path_roles.iter().any(|existing| existing == &path_role) {
                self.path_roles.push(path_role);
            }
        }
        self.dry_run_default &= other.dry_run_default;
        self
    }

    pub fn sensitivity(&self) -> XccuteCommandSensitivity {
        self.sensitivity
    }

    pub fn is_sensitive(&self) -> bool {
        self.sensitivity == XccuteCommandSensitivity::Sensitive
    }

    pub fn requires_sudo(&self) -> bool {
        self.requires_sudo
    }

    pub fn iam_scope(&self) -> Option<&str> {
        self.iam_scope.as_deref()
    }

    pub fn path_roles(&self) -> &[String] {
        &self.path_roles
    }

    pub fn dry_run_default(&self) -> bool {
        self.dry_run_default
    }
}

/// Sidecar trait for commands that expose XCCUTE planning metadata.
///
/// This intentionally stays separate from [`CompositeShellCommand`] and from
/// older `ShellCommand` surfaces. Privileged planning can require
/// `CompositeShellCommand + XccutePolicyMetadata` without making the display or
/// compatibility traits security-critical.
pub trait XccutePolicyMetadata {
    fn xccute_policy(&self) -> XccuteCommandPolicyMetadata;
}

/// A positive policy/operator approval attached to a command preview.
///
/// Approval is intentionally metadata around the trusted [`CommandPreview`].
/// The executable contract remains `program + argv`; approval records who or
/// what policy allowed that preview to proceed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandApproval {
    policy: String,
    actor: String,
    reason: Option<String>,
}

impl CommandApproval {
    pub fn new(policy: impl Into<String>, actor: impl Into<String>) -> Self {
        Self {
            policy: policy.into(),
            actor: actor.into(),
            reason: None,
        }
    }

    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    pub fn policy(&self) -> &str {
        &self.policy
    }

    pub fn actor(&self) -> &str {
        &self.actor
    }

    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }
}

/// A preview that has passed validation and an external approval policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandApprovalReceipt {
    preview: CommandPreview,
    approval: CommandApproval,
}

impl CommandApprovalReceipt {
    pub fn new(preview: CommandPreview, approval: CommandApproval) -> Self {
        Self { preview, approval }
    }

    pub fn preview(&self) -> &CommandPreview {
        &self.preview
    }

    pub fn approval(&self) -> &CommandApproval {
        &self.approval
    }

    pub fn into_parts(self) -> (CommandPreview, CommandApproval) {
        (self.preview, self.approval)
    }

    pub fn to_std_command(&self) -> Command {
        self.preview.to_std_command()
    }
}

/// Error returned when a preview is rejected by a policy/operator gate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPolicyError {
    policy: String,
    message: String,
}

impl CommandPolicyError {
    pub fn new(policy: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            policy: policy.into(),
            message: message.into(),
        }
    }

    pub fn policy(&self) -> &str {
        &self.policy
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for CommandPolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "policy `{}` rejected command: {}", self.policy, self.message)
    }
}

impl Error for CommandPolicyError {}

/// Result returned by a policy/operator gate.
pub type CommandPolicyResult = Result<CommandApproval, CommandPolicyError>;

/// Policy hook that can approve or reject a validated command preview.
///
/// Implementations can represent local dry-run policy, operator approval,
/// environment policy, or a higher-level decision system. They receive the
/// no-side-effect preview, not a shell string.
pub trait CommandPreviewPolicy {
    fn approve(&self, preview: &CommandPreview) -> CommandPolicyResult;
}

/// Error returned while trying to produce an approved preview receipt.
#[derive(Debug)]
pub enum CommandApprovalError {
    Validation(CommandValidationError),
    Policy(CommandPolicyError),
}

impl fmt::Display for CommandApprovalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandApprovalError::Validation(error) => error.fmt(f),
            CommandApprovalError::Policy(error) => error.fmt(f),
        }
    }
}

impl Error for CommandApprovalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CommandApprovalError::Validation(error) => Some(error),
            CommandApprovalError::Policy(error) => Some(error),
        }
    }
}

impl From<CommandValidationError> for CommandApprovalError {
    fn from(error: CommandValidationError) -> Self {
        CommandApprovalError::Validation(error)
    }
}

impl From<CommandPolicyError> for CommandApprovalError {
    fn from(error: CommandPolicyError) -> Self {
        CommandApprovalError::Policy(error)
    }
}

/// High-level category for command validation failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandValidationErrorKind {
    /// A field relationship failed before runtime environment checks.
    ///
    /// Examples: `invalid_without`, `only_pair_with`, `conflicts_with`.
    Structural,

    /// A runtime environment/preflight check failed.
    ///
    /// Examples: path exists, source is directory, executable exists, permission check.
    RuntimePreflight,

    /// A user-provided custom validation function failed.
    Custom,
}

impl fmt::Display for CommandValidationErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandValidationErrorKind::Structural => f.write_str("structural"),
            CommandValidationErrorKind::RuntimePreflight => f.write_str("runtime_preflight"),
            CommandValidationErrorKind::Custom => f.write_str("custom"),
        }
    }
}

/// Error returned when a command builder is invalid before execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandValidationError {
    kind: CommandValidationErrorKind,
    message: String,
    field: Option<String>,
    rule: Option<String>,
}

impl CommandValidationError {
    pub fn new(kind: CommandValidationErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            field: None,
            rule: None,
        }
    }

    pub fn structural(message: impl Into<String>) -> Self {
        Self::new(CommandValidationErrorKind::Structural, message)
    }

    pub fn runtime_preflight(message: impl Into<String>) -> Self {
        Self::new(CommandValidationErrorKind::RuntimePreflight, message)
    }

    pub fn custom(message: impl Into<String>) -> Self {
        Self::new(CommandValidationErrorKind::Custom, message)
    }

    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    pub fn with_rule(mut self, rule: impl Into<String>) -> Self {
        self.rule = Some(rule.into());
        self
    }

    pub fn kind(&self) -> &CommandValidationErrorKind {
        &self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn field(&self) -> Option<&str> {
        self.field.as_deref()
    }

    pub fn rule(&self) -> Option<&str> {
        self.rule.as_deref()
    }
}

impl fmt::Display for CommandValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} validation failed: {}", self.kind, self.message)?;

        if let Some(field) = &self.field {
            write!(f, " [field={field}]")?;
        }

        if let Some(rule) = &self.rule {
            write!(f, " [rule={rule}]")?;
        }

        Ok(())
    }
}

impl Error for CommandValidationError {}

/// Validation result for generated command builders.
pub type CommandValidationResult = Result<(), CommandValidationError>;

/// Trait implemented by generated commands that can validate before execution.
///
/// The default implementation is intentionally no-op so simple generated
/// commands can opt in without gaining behavior until validation rules exist.
pub trait ValidatedCommand {
    fn validate(&self) -> CommandValidationResult {
        Ok(())
    }
}

/// Error returned by validated execution helpers.
#[derive(Debug)]
pub enum CommandExecutionError {
    Validation(CommandValidationError),
    Io(std::io::Error),
}

impl fmt::Display for CommandExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandExecutionError::Validation(error) => error.fmt(f),
            CommandExecutionError::Io(error) => error.fmt(f),
        }
    }
}

impl Error for CommandExecutionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CommandExecutionError::Validation(error) => Some(error),
            CommandExecutionError::Io(error) => Some(error),
        }
    }
}

impl From<CommandValidationError> for CommandExecutionError {
    fn from(error: CommandValidationError) -> Self {
        CommandExecutionError::Validation(error)
    }
}

impl From<std::io::Error> for CommandExecutionError {
    fn from(error: std::io::Error) -> Self {
        CommandExecutionError::Io(error)
    }
}

/// Extension trait for commands that are both argv-safe and validatable.
///
/// Future generated commands can implement [`ValidatedCommand`] and receive this
/// validated execution path automatically.
pub trait ValidatedCompositeShellCommand: CompositeShellCommand + ValidatedCommand {
    fn to_validated_std_command(&self) -> Result<Command, CommandValidationError> {
        self.validate()?;
        Ok(self.to_std_command())
    }

    fn validated_preview(&self) -> Result<CommandPreview, CommandValidationError> {
        self.validate()?;
        Ok(self.preview())
    }

    fn approved_preview<P>(&self, policy: &P) -> Result<CommandApprovalReceipt, CommandApprovalError>
    where
        P: CommandPreviewPolicy,
    {
        let preview = self.validated_preview()?;
        let approval = policy.approve(&preview)?;
        Ok(CommandApprovalReceipt::new(preview, approval))
    }

    fn xccute_validated(&self) -> Result<Output, CommandExecutionError> {
        self.validate()?;
        Ok(self.to_std_command().output()?)
    }
}

impl<T> ValidatedCompositeShellCommand for T where T: CompositeShellCommand + ValidatedCommand {}

/// Bridge from a final subcommand fragment into a rooted executable command.
///
/// Future generated shape:
///
/// `Git::commit()` -> `<GitCommit as RootableSubCommand<Git>>::Rooted`
pub trait RootableSubCommand<R>
where
    R: CompositeShellRoot,
{
    type Rooted: CompositeShellCommand;

    fn rooted(self) -> Self::Rooted;
}

/// Bridge from a composite subcommand surface into a rooted surface.
///
/// Future generated shape:
///
/// `Git::remote()` -> `<GitRemote as RootableCompositeSurface<Git>>::RootedSurface`
pub trait RootableCompositeSurface<R>
where
    R: CompositeShellRoot,
{
    type RootedSurface;

    fn rooted_surface(self) -> Self::RootedSurface;
}
