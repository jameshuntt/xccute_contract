use std::ffi::{OsStr, OsString};

use xccute_contract::{CompositeShellCommand, ExitStatusWord};

/// `systemctl is-active UNIT`, its statuses named by hand.
struct IsActive;

const IS_ACTIVE: &[ExitStatusWord] = &[ExitStatusWord::accepted(0, "active"), ExitStatusWord::rejected(3, "inactive")];

impl CompositeShellCommand for IsActive {
    fn program(&self) -> &OsStr {
        OsStr::new("systemctl")
    }

    fn push_args(&self, out: &mut Vec<OsString>) {
        out.push("is-active".into());
    }

    fn exit_statuses(&self) -> &'static [ExitStatusWord] {
        IS_ACTIVE
    }
}

/// A command that names nothing.
struct True;

impl CompositeShellCommand for True {
    fn program(&self) -> &OsStr {
        OsStr::new("true")
    }

    fn push_args(&self, _out: &mut Vec<OsString>) {}
}

#[test]
fn a_command_names_its_statuses_and_finds_the_one_it_ended_with() {
    assert_eq!(IsActive.exit_statuses(), IS_ACTIVE);
    assert_eq!(IsActive.exit_status_of(Some(3)), Some(ExitStatusWord { code: 3, word: "inactive", accepted: false }));
    assert_eq!(IsActive.exit_status_of(Some(0)).map(|s| s.word), Some("active"));
    assert_eq!(IsActive.exit_status_of(Some(1)), None);
    assert_eq!(IsActive.exit_status_of(None), None, "a signal is no status");
}

#[test]
fn a_command_that_names_nothing_names_nothing() {
    assert!(True.exit_statuses().is_empty());
    assert_eq!(True.exit_status_of(Some(0)), None);
    assert_eq!(True.ok_codes(), &[0]);
}
