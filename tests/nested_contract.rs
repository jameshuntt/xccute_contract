//! A leaf two surfaces down, written by hand the way the derives write it:
//! `orb project member add NAME --role ROLE`, the leaf keeping its setter and
//! its exit statuses under both surfaces, and every level's help lines.

use std::ffi::{OsStr, OsString};
use std::marker::PhantomData;

use xccute_contract::{
    CommandHelp, CompositeArgvPart, CompositeShellCommand, CompositeShellRoot, ExitStatusWord, HelpLine,
    RootableNestedCommand, RootableNestedSurface,
};

struct Orb;

impl CompositeShellRoot for Orb {
    fn program() -> &'static OsStr {
        OsStr::new("orb")
    }
}

/// `project`, the first surface.
#[derive(Default)]
struct Project;

impl CompositeArgvPart for Project {
    fn push_argv_part(&self, out: &mut Vec<OsString>) {
        out.push("project".into());
    }
}

/// `member`, a surface under `project`.
#[derive(Default)]
struct Member;

impl CompositeArgvPart for Member {
    fn push_argv_part(&self, out: &mut Vec<OsString>) {
        out.push("member".into());
    }
}

struct MemberNested<R, P> {
    prefix: P,
    inner: Member,
    _root: PhantomData<R>,
}

impl<R: CompositeShellRoot, P: CompositeArgvPart> CompositeArgvPart for MemberNested<R, P> {
    fn push_argv_part(&self, out: &mut Vec<OsString>) {
        self.prefix.push_argv_part(out);
        self.inner.push_argv_part(out);
    }
}

impl<R: CompositeShellRoot, P: CompositeArgvPart> RootableNestedSurface<R, P> for Member {
    type NestedSurface = MemberNested<R, P>;

    fn nested_surface(self, prefix: P) -> Self::NestedSurface {
        MemberNested { prefix, inner: self, _root: PhantomData }
    }
}

/// `add NAME [--role ROLE]`, the leaf.
struct Add {
    name: OsString,
    role: Option<OsString>,
}

impl CompositeArgvPart for Add {
    fn push_argv_part(&self, out: &mut Vec<OsString>) {
        out.push("add".into());
        out.push(self.name.clone());
        if let Some(role) = &self.role {
            out.push("--role".into());
            out.push(role.clone());
        }
    }
}

const ADD_STATUSES: &[ExitStatusWord] = &[ExitStatusWord::accepted(0, "added"), ExitStatusWord::rejected(4, "exists")];

struct AddNested<R, P> {
    prefix: P,
    inner: Add,
    _root: PhantomData<R>,
}

impl<R: CompositeShellRoot, P: CompositeArgvPart> AddNested<R, P> {
    fn role(mut self, value: impl Into<OsString>) -> Self {
        self.inner.role = Some(value.into());
        self
    }
}

impl<R: CompositeShellRoot, P: CompositeArgvPart> CompositeShellCommand for AddNested<R, P> {
    fn program(&self) -> &OsStr {
        R::program()
    }

    fn push_args(&self, out: &mut Vec<OsString>) {
        self.prefix.push_argv_part(out);
        self.inner.push_argv_part(out);
    }

    fn ok_codes(&self) -> &'static [i32] {
        &[0]
    }

    fn exit_statuses(&self) -> &'static [ExitStatusWord] {
        ADD_STATUSES
    }
}

impl<R: CompositeShellRoot, P: CompositeArgvPart> RootableNestedCommand<R, P> for Add {
    type Nested = AddNested<R, P>;

    fn nested(self, prefix: P) -> Self::Nested {
        AddNested { prefix, inner: self, _root: PhantomData }
    }
}

impl CommandHelp for Add {
    fn help_lines() -> Vec<HelpLine> {
        vec![HelpLine::new("add NAME [--role ROLE]", "Adds a member.")]
    }
}

impl CommandHelp for Member {
    fn help_lines() -> Vec<HelpLine> {
        Add::help_lines().into_iter().map(|line| line.under("member")).collect()
    }
}

impl CommandHelp for Project {
    fn help_lines() -> Vec<HelpLine> {
        Member::help_lines().into_iter().map(|line| line.under("project")).collect()
    }
}

impl CommandHelp for Orb {
    fn help_lines() -> Vec<HelpLine> {
        Project::help_lines().into_iter().map(|line| line.under("orb")).collect()
    }
}

#[test]
fn a_leaf_two_surfaces_down_keeps_its_setter_and_statuses() {
    let member = <Member as RootableNestedSurface<Orb, Project>>::nested_surface(Member, Project);
    let add = Add { name: "cp".into(), role: None }.nested(member).role("owner");
    let add: AddNested<Orb, MemberNested<Orb, Project>> = add;

    assert_eq!(add.build_display(), "orb project member add cp --role owner");
    assert_eq!(add.exit_status_of(Some(4)).map(|s| s.word), Some("exists"));
    assert_eq!(add.ok_codes(), &[0]);
}

#[test]
fn help_lines_are_led_by_each_parent() {
    assert_eq!(Orb::help_lines(), [HelpLine::new("orb project member add NAME [--role ROLE]", "Adds a member.")]);
    assert_eq!(HelpLine::new("", "").under("serve").words, "serve");
    assert_eq!(HelpLine::new("serve", "").under("").words, "serve");
}
