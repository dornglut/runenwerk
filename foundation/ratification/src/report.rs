use alloc::vec::Vec;

use crate::{RatificationIssue, RatificationSeverity, RatificationStatus};

/// Ordered ratification report.
///
/// The report preserves issue order and derives its status from the highest
/// issue severity. It does not execute commands, mutate state, commit history,
/// undo, redo, share, reconcile, or own domain-specific validation rules.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RatificationReport<Code, Subject> {
    status: RatificationStatus,
    issues: Vec<RatificationIssue<Code, Subject>>,
}

impl<Code, Subject> RatificationReport<Code, Subject> {
    pub fn new() -> Self {
        Self {
            status: RatificationStatus::Accepted,
            issues: Vec::new(),
        }
    }

    pub fn accepted() -> Self {
        Self::new()
    }

    pub fn from_issue(issue: RatificationIssue<Code, Subject>) -> Self {
        Self::from_issues([issue])
    }

    pub fn from_issues(issues: impl IntoIterator<Item = RatificationIssue<Code, Subject>>) -> Self {
        let issues = issues.into_iter().collect::<Vec<_>>();
        let status = Self::status_for_issues(&issues);

        Self { status, issues }
    }

    pub fn status(&self) -> RatificationStatus {
        self.status
    }

    pub fn is_accepted(&self) -> bool {
        self.status.is_accepted()
    }

    pub fn is_clean(&self) -> bool {
        self.status.is_clean()
    }

    pub fn has_warnings(&self) -> bool {
        self.status.has_warnings()
            || self
                .issues
                .iter()
                .any(|issue| issue.severity() == RatificationSeverity::Warning)
    }

    pub fn is_rejected(&self) -> bool {
        self.status.is_rejected()
    }

    pub fn is_fatal(&self) -> bool {
        self.status.is_fatal()
    }

    pub fn has_blocking_issues(&self) -> bool {
        self.issues.iter().any(RatificationIssue::is_blocking)
    }

    pub fn is_empty(&self) -> bool {
        self.issues.is_empty()
    }

    pub fn len(&self) -> usize {
        self.issues.len()
    }

    pub fn issues(&self) -> &[RatificationIssue<Code, Subject>] {
        &self.issues
    }

    pub fn iter(&self) -> core::slice::Iter<'_, RatificationIssue<Code, Subject>> {
        self.issues.iter()
    }

    pub fn highest_severity(&self) -> Option<RatificationSeverity> {
        self.issues.iter().map(RatificationIssue::severity).max()
    }

    pub fn push(&mut self, issue: RatificationIssue<Code, Subject>) {
        self.issues.push(issue);
        self.recompute_status();
    }

    pub fn with_issue(mut self, issue: RatificationIssue<Code, Subject>) -> Self {
        self.push(issue);
        self
    }

    pub fn extend(&mut self, issues: impl IntoIterator<Item = RatificationIssue<Code, Subject>>) {
        self.issues.extend(issues);
        self.recompute_status();
    }

    pub fn merge(&mut self, other: Self) {
        self.issues.extend(other.issues);
        self.recompute_status();
    }

    pub fn merged(mut self, other: Self) -> Self {
        self.merge(other);
        self
    }

    fn recompute_status(&mut self) {
        self.status = Self::status_for_issues(&self.issues);
    }

    fn status_for_issues(issues: &[RatificationIssue<Code, Subject>]) -> RatificationStatus {
        RatificationStatus::from_highest_severity(
            issues.iter().map(RatificationIssue::severity).max(),
        )
    }
}

impl<Code, Subject> Default for RatificationReport<Code, Subject> {
    fn default() -> Self {
        Self::new()
    }
}
