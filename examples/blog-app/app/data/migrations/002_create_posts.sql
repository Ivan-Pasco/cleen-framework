-- Migration: Create posts table
-- Created: 2025-01-01 00:00:01

-- Up migration: Create the posts table
CREATE TABLE posts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    excerpt TEXT,
    slug TEXT NOT NULL UNIQUE,
    author_id INTEGER NOT NULL,
    published BOOLEAN NOT NULL DEFAULT 0,
    tags TEXT,  -- JSON array stored as TEXT
    views INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP,
    FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create indexes for better query performance
CREATE INDEX idx_posts_author_id ON posts(author_id);
CREATE INDEX idx_posts_slug ON posts(slug);
CREATE INDEX idx_posts_published_created ON posts(published, created_at DESC);
CREATE INDEX idx_posts_views ON posts(views DESC);

-- Down migration: Drop the posts table
-- DROP TABLE posts;
