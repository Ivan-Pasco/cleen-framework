use std::collections::HashMap;

pub struct Session {
	pub id: String,
	pub user_id: i64,
	pub expires_at: i64,
}

pub struct SessionStore {
	sessions: HashMap<String, Session>,
}

impl SessionStore {
	pub fn new() -> Self {
		Self {
			sessions: HashMap::new(),
		}
	}

	pub fn create(&mut self, user_id: i64) -> String {
		let id = uuid::Uuid::new_v4().to_string();
		let expires_at = chrono::Utc::now().timestamp() + 3600;

		self.sessions.insert(id.clone(), Session {
			id: id.clone(),
			user_id,
			expires_at,
		});

		id
	}

	pub fn get(&self, id: &str) -> Option<&Session> {
		self.sessions.get(id)
	}

	pub fn remove(&mut self, id: &str) {
		self.sessions.remove(id);
	}
}

impl Default for SessionStore {
	fn default() -> Self {
		Self::new()
	}
}
