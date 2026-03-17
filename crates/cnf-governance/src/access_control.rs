//! # Access control policy enforcement
//!
//! Manages user-based access control to resources.
//! Provides simple whitelist-based permission checking.

use crate::error::CnfGovernanceError;
use std::collections::HashSet;

pub struct AccessControl {
    pub allowed_users: HashSet<String>,
}

impl AccessControl {
    pub fn new(users: &[&str]) -> Self {
        AccessControl {
            allowed_users: users.iter().map(|u| u.to_string()).collect(),
        }
    }

    pub fn check(&self, user: &str, resource: &str) -> Result<bool, CnfGovernanceError> {
        if self.allowed_users.contains(user) {
            Ok(true)
        } else {
            Err(CnfGovernanceError::AccessDenied(format!("{} denied {}", user, resource)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_allowed() {
        let ac = AccessControl::new(&["admin", "default"]);
        assert!(ac.check("admin", "res").unwrap());
    }

    #[test]
    fn user_denied() {
        let ac = AccessControl::new(&["admin", "default"]);
        assert!(ac.check("bob", "res").is_err());
    }

    #[test]
    fn multiple_checks() {
        let ac = AccessControl::new(&["admin", "default"]);
        let _ = ac.check("admin", "x");
        let err = ac.check("user", "y");
        assert!(err.is_err());
    }

    #[test]
    fn check_nonexistent_user() {
        let ac = AccessControl::new(&["admin", "default"]);
        let res = ac.check("", "res");
        assert!(res.is_err());
    }
}
