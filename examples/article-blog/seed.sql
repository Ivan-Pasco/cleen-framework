-- Article Blog Database Schema and Seed Data
-- Run with: sqlite3 /tmp/blog.db < seed.sql

-- Drop existing tables
DROP TABLE IF EXISTS article_tags;
DROP TABLE IF EXISTS tags;
DROP TABLE IF EXISTS articles;
DROP TABLE IF EXISTS authors;

-- Create authors table
CREATE TABLE authors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    bio TEXT,
    avatar TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Create articles table
CREATE TABLE articles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    excerpt TEXT,
    content TEXT NOT NULL,
    cover_image TEXT,
    category TEXT,
    read_time INTEGER DEFAULT 5,
    published INTEGER DEFAULT 0,
    author_id INTEGER REFERENCES authors(id),
    published_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Create tags table
CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

-- Create article_tags junction table
CREATE TABLE article_tags (
    article_id INTEGER REFERENCES articles(id) ON DELETE CASCADE,
    tag_id INTEGER REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (article_id, tag_id)
);

-- Create indexes
CREATE INDEX idx_articles_slug ON articles(slug);
CREATE INDEX idx_articles_published ON articles(published);
CREATE INDEX idx_articles_category ON articles(category);

-- Insert authors
INSERT INTO authors (name, bio, avatar) VALUES
    ('Elena Marchetti', 'Senior Editor, Culture & Design', 'https://images.unsplash.com/photo-1494790108377-be9c29b29330?w=200&h=200&fit=crop'),
    ('James Chen', 'Tech Correspondent', 'https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?w=200&h=200&fit=crop'),
    ('Sofia Andersson', 'Creative Director', 'https://images.unsplash.com/photo-1438761681033-6461ffad8d80?w=200&h=200&fit=crop');

-- Insert articles
INSERT INTO articles (title, slug, excerpt, content, cover_image, category, read_time, published, author_id, published_at) VALUES
(
    'The Renaissance of Analog: Why We''re Returning to Physical Media',
    'renaissance-of-analog',
    'In an age of infinite digital streams, a growing movement is rediscovering the tactile joy of vinyl records, film photography, and handwritten letters.',
    '<p>There''s something happening in the cultural zeitgeist that feels both nostalgic and revolutionary. As our lives become increasingly mediated by screens and algorithms, a quiet rebellion is taking shape in living rooms, studios, and coffee shops around the world.</p><h2>The Vinyl Revival</h2><p>Record sales have surged for the sixteenth consecutive year, with vinyl now outselling CDs for the first time since 1987. But this isn''t mere nostalgia—it''s a conscious choice to engage with music differently.</p><blockquote>"When I put on a record, I''m committing to an experience. There''s no skipping, no shuffle. Just presence."</blockquote><p>The ritual matters. The careful removal from the sleeve, the gentle placement on the turntable, the satisfying click of the needle finding its groove. These physical interactions create a relationship with music that playlists simply cannot replicate.</p><h2>Film Photography''s Quiet Return</h2><p>Similarly, film photography has experienced a renaissance among both hobbyists and professionals. The limitations of film—the finite number of exposures, the delay in seeing results, the cost of each frame—have become features rather than bugs.</p><p>Photographers describe a more intentional approach when shooting film. Every click of the shutter becomes a decision, not just a reflex. The resulting images often carry a quality that digital cameras struggle to replicate.</p>',
    'https://images.unsplash.com/photo-1493225457124-a3eb161ffa5f?w=1200&h=800&fit=crop',
    'Culture',
    8,
    1,
    1,
    datetime('now', '-1 day')
),
(
    'Designing for the Next Billion Users',
    'designing-next-billion',
    'How inclusive design principles are reshaping technology for emerging markets and creating better products for everyone.',
    '<p>The next wave of internet users won''t look like the early adopters who shaped today''s digital landscape. They''ll connect through affordable smartphones on intermittent networks, often in languages that read right-to-left or require complex character rendering.</p><h2>Constraints Breed Innovation</h2><p>When Google redesigned their apps for users in India, Indonesia, and Brazil, they discovered that constraints weren''t obstacles—they were opportunities. Smaller app sizes meant faster downloads. Offline-first thinking created more resilient experiences.</p><p>The lessons learned are now improving products for all users. Features designed for low bandwidth have become essential during crowded commutes. Interfaces simplified for new users have reduced cognitive load for everyone.</p><h2>Beyond Translation</h2><p>True localization goes far beyond language. It requires understanding cultural contexts, payment preferences, and social dynamics that vary dramatically across regions.</p>',
    'https://images.unsplash.com/photo-1531297484001-80022131f5a1?w=1200&h=800&fit=crop',
    'Design',
    6,
    1,
    3,
    datetime('now', '-2 days')
),
(
    'The Quiet Revolution in Battery Technology',
    'battery-revolution',
    'Solid-state batteries promise to transform everything from electric vehicles to grid storage. Here''s what''s actually happening in the labs.',
    '<p>Every major technological shift of the past decade—from smartphones to electric vehicles—has been constrained by the same bottleneck: battery technology. Now, after decades of incremental improvements, we may be on the verge of a genuine breakthrough.</p><h2>Beyond Lithium-Ion</h2><p>Solid-state batteries replace the liquid electrolyte in conventional batteries with a solid material. The potential benefits are transformative: higher energy density, faster charging, longer lifespan, and dramatically improved safety.</p><blockquote>"We''re not talking about 10% improvements. We''re talking about batteries that could double the range of electric vehicles while cutting charging time to minutes."</blockquote><p>Toyota, QuantumScape, and a constellation of startups are racing to solve the manufacturing challenges that have kept solid-state batteries in the lab.</p><h2>The Path Forward</h2><p>The biggest hurdle isn''t the science—it''s scaling production. Making solid-state batteries in a lab is one thing; manufacturing them by the millions at competitive costs is another challenge entirely.</p>',
    'https://images.unsplash.com/photo-1558618666-fcd25c85cd64?w=1200&h=800&fit=crop',
    'Technology',
    7,
    1,
    2,
    datetime('now', '-3 days')
),
(
    'The Architecture of Solitude',
    'architecture-solitude',
    'A meditation on spaces designed for one, and what they reveal about our relationship with privacy and contemplation.',
    '<p>In Japanese architecture, there''s a concept called ''ma''—the pregnant emptiness between objects that gives them meaning. It''s an idea that feels increasingly radical in an age of maximized square footage and open floor plans.</p><p>Yet across the world, architects are rediscovering the power of spaces designed not for gathering, but for solitude. Reading nooks that cradle a single body. Garden pavilions that frame one view. Meditation rooms stripped of everything except light.</p><h2>The Luxury of Less</h2><p>These spaces share a quality that''s difficult to describe but immediately recognizable: they feel like permission. Permission to be alone. Permission to think slowly. Permission to simply exist without performing for an audience.</p><p>In our hyperconnected world, such spaces have become rare luxuries. They represent a counter-movement to the dominant architectural trends of transparency and flexibility.</p>',
    'https://images.unsplash.com/photo-1600585154340-be6161a56a0c?w=1200&h=800&fit=crop',
    'Design',
    5,
    1,
    1,
    datetime('now', '-4 days')
),
(
    'When Algorithms Meet Artisans',
    'algorithms-meet-artisans',
    'How machine learning is being used to preserve traditional crafts and unlock new possibilities for human creativity.',
    '<p>In a workshop outside Kyoto, a fourth-generation ceramicist is collaborating with an unlikely partner: a neural network trained on thousands of historic glaze formulas. The algorithm doesn''t replace his expertise—it expands it, suggesting combinations that might take a lifetime to discover through experimentation alone.</p><h2>Augmented Craftsmanship</h2><p>This isn''t the automation narrative we''ve been told to fear. Instead, it''s a new kind of partnership between human intuition and computational power. The machine excels at pattern recognition across vast datasets. The human brings judgment, context, and the irreplaceable quality of care.</p><p>Similar collaborations are emerging in textile design, furniture making, and even perfumery. In each case, the technology serves as a creative amplifier rather than a replacement.</p><h2>Preserving Knowledge</h2><p>Perhaps most importantly, these projects are helping to document and preserve traditional techniques that might otherwise be lost. Machine learning models trained on historical artifacts become a form of cultural memory.</p>',
    'https://images.unsplash.com/photo-1565193566173-7a0ee3dbe261?w=1200&h=800&fit=crop',
    'Technology',
    6,
    1,
    2,
    datetime('now', '-5 days')
),
(
    'The Taste of Memory',
    'taste-of-memory',
    'Why certain flavors transport us through time, and what science is learning about the profound connection between food and nostalgia.',
    '<p>You bite into something—a particular cookie, a specific spice—and suddenly you''re seven years old again, standing in your grandmother''s kitchen. The experience is so vivid, so complete, that for a moment the present dissolves entirely.</p><p>This phenomenon, sometimes called the Proust effect, represents one of the most powerful and least understood aspects of human memory. And researchers are finally beginning to decode its mysteries.</p><h2>The Olfactory Shortcut</h2><p>Unlike other senses, smell and taste have a direct neural pathway to the hippocampus and amygdala—the brain regions responsible for memory and emotion. This anatomical quirk means that flavors can access memories that visual or auditory cues cannot reach.</p><blockquote>"Taste memories are often our oldest memories. They''re formed before we have language to describe them, which is why they feel so primal."</blockquote><p>Researchers have found that these early taste memories can influence our food preferences for life, shaping everything from comfort food choices to cultural identity.</p>',
    'https://images.unsplash.com/photo-1556909114-f6e7ad7d3136?w=1200&h=800&fit=crop',
    'Culture',
    7,
    1,
    3,
    datetime('now', '-6 days')
);

-- Insert some tags
INSERT INTO tags (name) VALUES
    ('analog'),
    ('vinyl'),
    ('design'),
    ('technology'),
    ('culture'),
    ('architecture'),
    ('ai'),
    ('sustainability');

-- Link articles to tags
INSERT INTO article_tags (article_id, tag_id) VALUES
    (1, 1), (1, 2), (1, 5),
    (2, 3), (2, 4),
    (3, 4), (3, 8),
    (4, 3), (4, 6),
    (5, 4), (5, 7),
    (6, 5);

-- Verify data
SELECT 'Authors:' as info, COUNT(*) as count FROM authors;
SELECT 'Articles:' as info, COUNT(*) as count FROM articles;
SELECT 'Tags:' as info, COUNT(*) as count FROM tags;

-- Show articles
SELECT id, title, category, author_id, published FROM articles;
