-- Article Blog Database Schema
-- Run with: sqlite3 blog.db < schema.sql

-- Articles table
CREATE TABLE IF NOT EXISTS articles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL,
    lead TEXT NOT NULL,
    category TEXT NOT NULL,
    author TEXT NOT NULL,
    author_title TEXT DEFAULT '',
    author_image TEXT DEFAULT '',
    cover_image TEXT NOT NULL,
    thumbnail_image TEXT NOT NULL,
    read_time INTEGER DEFAULT 5,
    content TEXT NOT NULL,
    featured INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Insert sample articles
INSERT INTO articles (slug, title, lead, category, author, author_title, author_image, cover_image, thumbnail_image, read_time, content, featured) VALUES
(
    'renaissance-of-analog',
    'The Renaissance of Analog',
    'In an age of infinite digital streams, a growing movement rediscovers the tactile joy of vinyl records, film photography, and handwritten letters.',
    'Culture',
    'Elena Marchetti',
    'Senior Editor',
    'https://images.unsplash.com/photo-1494790108377-be9c29b29330?w=120&q=80',
    'https://images.unsplash.com/photo-1493225457124-a3eb161ffa5f?w=1200&q=80',
    'https://images.unsplash.com/photo-1493225457124-a3eb161ffa5f?w=900&q=80',
    8,
    '<p>There is something happening in the cultural zeitgeist that feels both nostalgic and revolutionary. As our lives become increasingly mediated by screens and algorithms, a quiet rebellion is taking shape in living rooms, studios, and coffee shops around the world.</p><h2>The Vinyl Revival</h2><p>Record sales have surged for the sixteenth consecutive year, with vinyl now outselling CDs for the first time since 1987. But this is not mere nostalgia. It is a conscious choice to engage with music differently.</p><blockquote>When I put on a record, I am committing to an experience. There is no skipping, no shuffle. Just presence.</blockquote><p>The ritual matters. The careful removal from the sleeve, the gentle placement on the turntable, the satisfying click of the needle finding its groove. These physical interactions create a relationship with music that playlists simply cannot replicate.</p><h2>Film Photography Returns</h2><p>Similarly, film photography has experienced a renaissance among both hobbyists and professionals. The limitations of film have become features rather than bugs. Every click of the shutter becomes a decision, not just a reflex.</p>',
    1
),
(
    'designing-next-billion',
    'Designing for the Next Billion',
    'How inclusive design principles are reshaping technology for emerging markets and underserved communities worldwide.',
    'Design',
    'Sofia Andersson',
    'Design Lead',
    'https://images.unsplash.com/photo-1580489944761-15a19d654956?w=120&q=80',
    'https://images.unsplash.com/photo-1531297484001-80022131f5a1?w=1200&q=80',
    'https://images.unsplash.com/photo-1531297484001-80022131f5a1?w=600&q=80',
    6,
    '<p>The next billion internet users will come from places and circumstances very different from Silicon Valley. Designing for them requires a fundamental rethinking of our assumptions about connectivity, devices, and digital literacy.</p><h2>Beyond High-Speed Connectivity</h2><p>In many regions, 2G networks remain the norm. Data is expensive, often paid by the megabyte. Apps must be designed to work offline, sync intelligently, and minimize bandwidth usage without sacrificing functionality.</p><blockquote>Good design is not about the latest features. It is about solving real problems for real people in their real contexts.</blockquote><h2>Device Constraints as Design Opportunities</h2><p>Entry-level smartphones often have limited storage, processing power, and screen resolution. Rather than seeing these as limitations, forward-thinking designers view them as opportunities to create more focused, efficient experiences.</p>',
    0
),
(
    'battery-revolution',
    'The Quiet Revolution in Batteries',
    'Solid-state batteries promise to transform everything from electric vehicles to grid-scale energy storage.',
    'Technology',
    'James Chen',
    'Tech Editor',
    'https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?w=120&q=80',
    'https://images.unsplash.com/photo-1558618666-fcd25c85cd64?w=1200&q=80',
    'https://images.unsplash.com/photo-1558618666-fcd25c85cd64?w=600&q=80',
    7,
    '<p>While headlines focus on AI and social media, a quieter revolution is unfolding in materials science labs around the world. Solid-state batteries could be the technology that finally makes sustainable energy truly practical.</p><h2>Beyond Lithium-Ion</h2><p>Lithium-ion batteries have served us well, but they are approaching their theoretical limits. Solid-state technology promises higher energy density, faster charging, longer lifespan, and dramatically improved safety.</p><blockquote>The transition from liquid to solid electrolytes is not just an incremental improvement. It is a paradigm shift in how we store energy.</blockquote><h2>From Lab to Production</h2><p>The challenge now is scaling production. Several companies have announced breakthrough results, and major automotive manufacturers are racing to bring solid-state vehicles to market by 2027.</p>',
    0
),
(
    'architecture-solitude',
    'The Architecture of Solitude',
    'A meditation on spaces designed for one, and what they reveal about our relationship with ourselves.',
    'Architecture',
    'Elena Marchetti',
    'Senior Editor',
    'https://images.unsplash.com/photo-1494790108377-be9c29b29330?w=120&q=80',
    'https://images.unsplash.com/photo-1600585154340-be6161a56a0c?w=1200&q=80',
    'https://images.unsplash.com/photo-1600585154340-be6161a56a0c?w=600&q=80',
    5,
    '<p>In an age of constant connection, architects are increasingly designing spaces meant for solitude. These are not merely small spaces, but thoughtful environments that encourage introspection and presence.</p><h2>The Cabin Movement</h2><p>From Scandinavia to the Pacific Northwest, minimalist cabins have become symbols of intentional living. These structures strip away the unnecessary, leaving only what serves contemplation and rest.</p><blockquote>Architecture can create the conditions for solitude, but it cannot force us to be present. That remains our choice.</blockquote><h2>Urban Retreats</h2><p>Not everyone can escape to a forest cabin. Urban architects are responding with micro-retreats, rooftop sanctuaries, and thoughtfully designed corners that offer moments of quiet within the city.</p>',
    0
);

-- Index for slug lookups
CREATE INDEX IF NOT EXISTS idx_articles_slug ON articles(slug);
-- Index for featured articles
CREATE INDEX IF NOT EXISTS idx_articles_featured ON articles(featured);
