use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::error::{IdentityError, Result};

/// A permission in the system.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    /// Resource being accessed (e.g., "quantum", "energy", "identity").
    pub resource: String,
    /// Action being performed (e.g., "read", "write", "execute", "admin").
    pub action: String,
}

impl Permission {
    pub fn new(resource: &str, action: &str) -> Self {
        Self {
            resource: resource.to_string(),
            action: action.to_string(),
        }
    }

    /// Parse a permission from a string like "quantum:read".
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(IdentityError::RbacError(format!(
                "invalid permission format: {}",
                s
            )));
        }
        Ok(Self {
            resource: parts[0].to_string(),
            action: parts[1].to_string(),
        })
    }
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.resource, self.action)
    }
}

/// A role with associated permissions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub description: String,
    pub permissions: HashSet<Permission>,
    /// Parent roles (inherit permissions).
    pub inherits: Vec<String>,
}

impl Role {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: String::new(),
            permissions: HashSet::new(),
            inherits: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    pub fn remove_permission(&mut self, permission: &Permission) -> bool {
        self.permissions.remove(permission)
    }

    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }

    pub fn inherit_from(mut self, role_name: &str) -> Self {
        self.inherits.push(role_name.to_string());
        self
    }
}

/// Role assignment to a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleAssignment {
    pub user_id: String,
    pub roles: Vec<String>,
}

impl RoleAssignment {
    pub fn new(user_id: &str, roles: Vec<String>) -> Self {
        Self {
            user_id: user_id.to_string(),
            roles,
        }
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }

    pub fn add_role(&mut self, role: &str) {
        if !self.roles.contains(&role.to_string()) {
            self.roles.push(role.to_string());
        }
    }

    pub fn remove_role(&mut self, role: &str) -> bool {
        let len_before = self.roles.len();
        self.roles.retain(|r| r != role);
        self.roles.len() < len_before
    }
}

/// Manages roles and permissions.
pub struct RbacManager {
    roles: HashMap<String, Role>,
    assignments: HashMap<String, RoleAssignment>,
}

impl RbacManager {
    pub fn new() -> Self {
        Self {
            roles: HashMap::new(),
            assignments: HashMap::new(),
        }
    }

    /// Register a role.
    pub fn register_role(&mut self, role: Role) {
        self.roles.insert(role.name.clone(), role);
    }

    /// Get a role by name.
    pub fn get_role(&self, name: &str) -> Option<&Role> {
        self.roles.get(name)
    }

    /// Remove a role.
    pub fn remove_role(&mut self, name: &str) -> bool {
        self.roles.remove(name).is_some()
    }

    /// Assign roles to a user.
    pub fn assign_roles(&mut self, user_id: &str, roles: Vec<String>) {
        let assignment = RoleAssignment::new(user_id, roles);
        self.assignments.insert(user_id.to_string(), assignment);
    }

    /// Get a user's role assignment.
    pub fn get_assignment(&self, user_id: &str) -> Option<&RoleAssignment> {
        self.assignments.get(user_id)
    }

    /// Check if a user has a specific role.
    pub fn user_has_role(&self, user_id: &str, role: &str) -> bool {
        self.assignments
            .get(user_id)
            .map(|a| a.has_role(role))
            .unwrap_or(false)
    }

    /// Check if a user has a specific permission.
    pub fn user_has_permission(&self, user_id: &str, permission: &Permission) -> bool {
        let assignment = match self.assignments.get(user_id) {
            Some(a) => a,
            None => return false,
        };

        for role_name in &assignment.roles {
            if let Some(role) = self.roles.get(role_name) {
                if role.has_permission(permission) {
                    return true;
                }

                // Check inherited roles
                for inherited in &role.inherits {
                    if let Some(parent) = self.roles.get(inherited) {
                        if parent.has_permission(permission) {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// Authorize a user action.
    pub fn authorize(&self, user_id: &str, resource: &str, action: &str) -> Result<()> {
        let permission = Permission::new(resource, action);

        if self.user_has_permission(user_id, &permission) {
            Ok(())
        } else {
            Err(IdentityError::PermissionDenied(format!(
                "user {} lacks permission {}",
                user_id, permission
            )))
        }
    }

    /// Get all permissions for a user (including inherited).
    pub fn get_user_permissions(&self, user_id: &str) -> HashSet<Permission> {
        let mut permissions = HashSet::new();

        if let Some(assignment) = self.assignments.get(user_id) {
            for role_name in &assignment.roles {
                if let Some(role) = self.roles.get(role_name) {
                    permissions.extend(role.permissions.clone());

                    for inherited in &role.inherits {
                        if let Some(parent) = self.roles.get(inherited) {
                            permissions.extend(parent.permissions.clone());
                        }
                    }
                }
            }
        }

        permissions
    }

    /// Remove a user's role assignment.
    pub fn remove_user(&mut self, user_id: &str) -> bool {
        self.assignments.remove(user_id).is_some()
    }

    /// Get the number of registered roles.
    pub fn role_count(&self) -> usize {
        self.roles.len()
    }

    /// Get the number of user assignments.
    pub fn user_count(&self) -> usize {
        self.assignments.len()
    }
}

impl Default for RbacManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_creation() {
        let perm = Permission::new("quantum", "read");
        assert_eq!(perm.resource, "quantum");
        assert_eq!(perm.action, "read");
    }

    #[test]
    fn test_permission_from_str() {
        let perm = Permission::from_str("energy:write").unwrap();
        assert_eq!(perm.resource, "energy");
        assert_eq!(perm.action, "write");
    }

    #[test]
    fn test_permission_from_str_invalid() {
        assert!(Permission::from_str("invalid").is_err());
    }

    #[test]
    fn test_role_creation() {
        let mut role = Role::new("admin");
        role.add_permission(Permission::new("quantum", "read"));
        role.add_permission(Permission::new("quantum", "write"));

        assert!(role.has_permission(&Permission::new("quantum", "read")));
        assert!(!role.has_permission(&Permission::new("energy", "read")));
    }

    #[test]
    fn test_role_inheritance() {
        let mut admin = Role::new("admin");
        admin.add_permission(Permission::new("quantum", "admin"));

        let mut user = Role::new("user");
        user.add_permission(Permission::new("quantum", "read"));
        user = user.inherit_from("admin");

        assert_eq!(user.inherits, vec!["admin"]);
    }

    #[test]
    fn test_rbac_manager() {
        let mut manager = RbacManager::new();

        let mut admin_role = Role::new("admin");
        admin_role.add_permission(Permission::new("quantum", "admin"));
        manager.register_role(admin_role);

        manager.assign_roles("user1", vec!["admin".to_string()]);

        assert!(manager.user_has_role("user1", "admin"));
        assert!(manager.user_has_permission("user1", &Permission::new("quantum", "admin")));
    }

    #[test]
    fn test_authorize() {
        let mut manager = RbacManager::new();

        let mut role = Role::new("developer");
        role.add_permission(Permission::new("quantum", "read"));
        manager.register_role(role);

        manager.assign_roles("user1", vec!["developer".to_string()]);

        assert!(manager.authorize("user1", "quantum", "read").is_ok());
        assert!(manager.authorize("user1", "quantum", "write").is_err());
    }

    #[test]
    fn test_inherited_permissions() {
        let mut manager = RbacManager::new();

        let mut admin = Role::new("admin");
        admin.add_permission(Permission::new("system", "admin"));
        manager.register_role(admin);

        let mut user = Role::new("user");
        user.add_permission(Permission::new("quantum", "read"));
        user = user.inherit_from("admin");
        manager.register_role(user);

        manager.assign_roles("user1", vec!["user".to_string()]);

        assert!(manager.user_has_permission("user1", &Permission::new("quantum", "read")));
        assert!(manager.user_has_permission("user1", &Permission::new("system", "admin")));
    }

    #[test]
    fn test_get_user_permissions() {
        let mut manager = RbacManager::new();

        let mut role = Role::new("dev");
        role.add_permission(Permission::new("quantum", "read"));
        role.add_permission(Permission::new("energy", "read"));
        manager.register_role(role);

        manager.assign_roles("user1", vec!["dev".to_string()]);

        let perms = manager.get_user_permissions("user1");
        assert_eq!(perms.len(), 2);
    }

    #[test]
    fn test_role_assignment() {
        let mut assignment = RoleAssignment::new("user1", vec!["admin".to_string()]);

        assert!(assignment.has_role("admin"));
        assert!(!assignment.has_role("user"));

        assignment.add_role("user");
        assert!(assignment.has_role("user"));

        assignment.remove_role("admin");
        assert!(!assignment.has_role("admin"));
    }

    #[test]
    fn test_remove_user() {
        let mut manager = RbacManager::new();
        manager.assign_roles("user1", vec!["admin".to_string()]);

        assert!(manager.remove_user("user1"));
        assert!(!manager.remove_user("user1"));
    }
}
