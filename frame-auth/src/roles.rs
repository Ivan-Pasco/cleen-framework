use crate::User;
use std::collections::HashMap;

pub struct Role {
	pub name: String,
	pub permissions: Vec<Permission>,
}

pub struct Permission {
	pub name: String,
}

pub fn can(user: &User, permission: &str) -> bool {
	// Check if user has permission based on their roles
	// This would load role definitions from config
	true
}
