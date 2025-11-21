/// Complete roundtrip example demonstrating Frame Data ORM
///
/// This example shows:
/// - Database connection
/// - Model definitions
/// - CRUD operations (Create, Read, Update, Delete)
/// - Transactions
/// - Complex queries with WHERE conditions
/// - Relationships (manual, since auto-relationships are deferred)

use anyhow::Result;
use frame_data::{Connection, DbConfig, Model, QueryBuilder, Transaction};
use serde::{Deserialize, Serialize};

/// User model - represents a blog user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
	pub id: Option<i64>,
	pub name: String,
	pub email: String,
	pub active: bool,
}

impl Model for User {
	fn table_name() -> &'static str {
		"users"
	}
}

/// Post model - represents a blog post
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
	pub id: Option<i64>,
	pub user_id: i64,
	pub title: String,
	pub content: String,
	pub published: bool,
}

impl Model for Post {
	fn table_name() -> &'static str {
		"posts"
	}
}

/// Comment model - represents a comment on a post
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
	pub id: Option<i64>,
	pub post_id: i64,
	pub author_name: String,
	pub content: String,
	pub approved: bool,
}

impl Model for Comment {
	fn table_name() -> &'static str {
		"comments"
	}
}

#[tokio::main]
async fn main() -> Result<()> {
	println!("🚀 Frame Data ORM - Complete Roundtrip Example\n");

	// Enable SQLite driver
	install_driver();

	// Step 1: Connect to database
	println!("📊 Step 1: Connecting to database...");
	let mut conn = connect_to_database().await?;
	println!("✅ Connected!\n");

	// Step 2: Create schema
	println!("🏗️  Step 2: Creating database schema...");
	create_schema(&mut conn).await?;
	println!("✅ Schema created!\n");

	// Step 3: Create records
	println!("➕ Step 3: Creating records...");
	let user = create_user(&mut conn).await?;
	println!("✅ Created user: {:?}\n", user);

	let post = create_post(&mut conn, user.id.unwrap()).await?;
	println!("✅ Created post: {:?}\n", post);

	// Step 4: Read records
	println!("📖 Step 4: Reading records...");
	read_records(&mut conn).await?;
	println!();

	// Step 5: Update records
	println!("✏️  Step 5: Updating records...");
	update_post(&mut conn, post.id.unwrap()).await?;
	println!();

	// Step 6: Complex queries
	println!("🔍 Step 6: Running complex queries...");
	complex_queries(&mut conn).await?;
	println!();

	// Step 7: Transactions
	println!("💰 Step 7: Demonstrating transactions...");
	demonstrate_transactions(&mut conn).await?;
	println!();

	// Step 8: Relationships (manual)
	println!("🔗 Step 8: Working with relationships...");
	demonstrate_relationships(&mut conn, user.id.unwrap(), post.id.unwrap()).await?;
	println!();

	// Step 9: Delete records
	println!("🗑️  Step 9: Deleting records...");
	delete_records(&mut conn).await?;
	println!();

	println!("🎉 Roundtrip complete! All operations successful.");

	Ok(())
}

fn install_driver() {
	use sqlx::any::install_default_drivers;
	static INIT: std::sync::Once = std::sync::Once::new();
	INIT.call_once(|| {
		install_default_drivers();
	});
}

async fn connect_to_database() -> Result<Connection> {
	let mut conn = Connection::new("sqlite");

	let config = DbConfig {
		database_url: "sqlite:file::memory:?cache=shared".to_string(),
		max_connections: 5,
		min_connections: 1,
		connection_timeout: 5000,
		query_timeout: 10000,
	};

	conn.connect(config).await?;
	Ok(conn)
}

async fn create_schema(conn: &mut Connection) -> Result<()> {
	// Drop existing tables
	let _ = conn.execute("DROP TABLE IF EXISTS comments", vec![]).await;
	let _ = conn.execute("DROP TABLE IF EXISTS posts", vec![]).await;
	let _ = conn.execute("DROP TABLE IF EXISTS users", vec![]).await;

	// Create users table
	conn.execute(
		"CREATE TABLE users (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			name TEXT NOT NULL,
			email TEXT NOT NULL UNIQUE,
			active INTEGER NOT NULL DEFAULT 1
		)",
		vec![],
	).await?;

	// Create posts table
	conn.execute(
		"CREATE TABLE posts (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			user_id INTEGER NOT NULL,
			title TEXT NOT NULL,
			content TEXT NOT NULL,
			published INTEGER NOT NULL DEFAULT 0,
			FOREIGN KEY (user_id) REFERENCES users(id)
		)",
		vec![],
	).await?;

	// Create comments table
	conn.execute(
		"CREATE TABLE comments (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			post_id INTEGER NOT NULL,
			author_name TEXT NOT NULL,
			content TEXT NOT NULL,
			approved INTEGER NOT NULL DEFAULT 0,
			FOREIGN KEY (post_id) REFERENCES posts(id)
		)",
		vec![],
	).await?;

	Ok(())
}

async fn create_user(conn: &mut Connection) -> Result<User> {
	let user = User {
		id: None,
		name: "Alice Johnson".to_string(),
		email: "alice@example.com".to_string(),
		active: true,
	};

	let created_user = User::create(conn, user).await?;

	// Since create() doesn't return the generated ID yet, let's fetch it
	let users = User::where_clause(conn, "email", serde_json::json!("alice@example.com")).await?;
	Ok(users[0].clone())
}

async fn create_post(conn: &mut Connection, user_id: i64) -> Result<Post> {
	let post = Post {
		id: None,
		user_id,
		title: "Getting Started with Frame Data ORM".to_string(),
		content: "Frame Data is a powerful ORM that provides type-safe database access with ACID transactions...".to_string(),
		published: false,
	};

	let created_post = Post::create(conn, post).await?;

	// Fetch the created post
	let posts = Post::where_clause(conn, "title", serde_json::json!("Getting Started with Frame Data ORM")).await?;
	Ok(posts[0].clone())
}

async fn read_records(conn: &mut Connection) -> Result<()> {
	// Find user by ID
	if let Some(user) = User::find(conn, 1).await? {
		println!("Found user by ID: {} ({})", user.name, user.email);
	}

	// Get all users
	let all_users = User::all(conn).await?;
	println!("Total users: {}", all_users.len());

	// Find posts by user
	let user_posts = Post::where_clause(conn, "user_id", serde_json::json!(1)).await?;
	println!("User has {} posts", user_posts.len());

	Ok(())
}

async fn update_post(conn: &mut Connection, post_id: i64) -> Result<()> {
	// Fetch the post
	if let Some(mut post) = Post::find(conn, post_id).await? {
		println!("Before update - Published: {}", post.published);

		// Update the post
		post.published = true;
		post.title = "Getting Started with Frame Data ORM (Updated)".to_string();

		Post::update(conn, post_id, post).await?;

		// Verify the update
		if let Some(updated_post) = Post::find(conn, post_id).await? {
			println!("After update - Published: {}", updated_post.published);
			println!("New title: {}", updated_post.title);
		}
	}

	Ok(())
}

async fn complex_queries(conn: &mut Connection) -> Result<()> {
	// Use QueryBuilder for complex queries
	let query = QueryBuilder::new("posts")
		.select(vec!["id".to_string(), "title".to_string(), "published".to_string()])
		.where_eq("user_id", serde_json::json!(1))
		.where_eq("published", serde_json::json!(1))
		.order_by("id", frame_data::OrderDirection::Desc)
		.limit(10);

	let (sql, params) = query.to_sql();
	println!("Generated SQL: {}", sql);
	println!("Parameters: {:?}", params);

	let rows = conn.query(&sql, params).await?;
	println!("Found {} published posts", rows.len());

	for row in rows {
		let title: String = row.get("title")?;
		println!("  - {}", title);
	}

	Ok(())
}

async fn demonstrate_transactions(conn: &mut Connection) -> Result<()> {
	println!("Starting atomic transaction...");

	// Create a scope for the transaction
	{
		let mut tx = Transaction::new(conn);
		tx.begin().await?;

		// Operation 1: Create a new user
		tx.execute(
			"INSERT INTO users (name, email, active) VALUES (?, ?, ?)",
			vec![
				serde_json::json!("Bob Smith"),
				serde_json::json!("bob@example.com"),
				serde_json::json!(1),
			],
		).await?;

		// Operation 2: Create a post for the new user
		// Note: We'd need to get the last insert ID in a real scenario
		tx.execute(
			"INSERT INTO posts (user_id, title, content, published) VALUES (?, ?, ?, ?)",
			vec![
				serde_json::json!(2), // Assuming Bob is ID 2
				serde_json::json!("Bob's First Post"),
				serde_json::json!("Hello from Bob!"),
				serde_json::json!(1),
			],
		).await?;

		// Commit both operations atomically
		tx.commit().await?;
		println!("✅ Transaction committed - both user and post created atomically");
	}

	// Demonstrate rollback
	{
		let mut tx = Transaction::new(conn);
		tx.begin().await?;

		tx.execute(
			"INSERT INTO users (name, email, active) VALUES (?, ?, ?)",
			vec![
				serde_json::json!("Charlie"),
				serde_json::json!("charlie@example.com"),
				serde_json::json!(1),
			],
		).await?;

		// Rollback instead of commit
		tx.rollback().await?;
		println!("✅ Transaction rolled back - Charlie was not created");
	}

	// Verify Charlie doesn't exist
	let charlie = User::where_clause(conn, "name", serde_json::json!("Charlie")).await?;
	println!("Charlie exists: {}", !charlie.is_empty());

	Ok(())
}

async fn demonstrate_relationships(conn: &mut Connection, user_id: i64, post_id: i64) -> Result<()> {
	// Manual relationship queries (since auto-relationships are deferred)

	// Get user's posts
	println!("Fetching user's posts...");
	let user_posts = Post::where_clause(conn, "user_id", serde_json::json!(user_id)).await?;
	println!("User {} has {} posts", user_id, user_posts.len());

	// Create comments
	println!("\nCreating comments...");
	for i in 1..=3 {
		let comment = Comment {
			id: None,
			post_id,
			author_name: format!("Commenter {}", i),
			content: format!("This is comment number {}", i),
			approved: i % 2 == 0, // Approve every other comment
		};
		Comment::create(conn, comment).await?;
	}

	// Get post's comments
	let post_comments = Comment::where_clause(conn, "post_id", serde_json::json!(post_id)).await?;
	println!("Post {} has {} comments", post_id, post_comments.len());

	// Get only approved comments
	let approved_comments = conn.query(
		"SELECT * FROM comments WHERE post_id = ? AND approved = ?",
		vec![serde_json::json!(post_id), serde_json::json!(1)],
	).await?;
	println!("Post {} has {} approved comments", post_id, approved_comments.len());

	Ok(())
}

async fn delete_records(conn: &mut Connection) -> Result<()> {
	// Delete a comment
	let deleted = Comment::delete(conn, 1).await?;
	println!("Deleted comment 1: {}", deleted);

	// Try to delete non-existent record
	let not_deleted = Comment::delete(conn, 999).await?;
	println!("Deleted non-existent comment 999: {}", not_deleted);

	// Count remaining comments
	let remaining_comments = Comment::all(conn).await?;
	println!("Remaining comments: {}", remaining_comments.len());

	Ok(())
}
