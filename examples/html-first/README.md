# HTML-First Example

A comprehensive HTML-first example with components, layouts, and multiple pages.

## Project Structure

```
app/
└── ui/
    ├── pages/
    │   ├── index.html        → GET /
    │   ├── dashboard.html    → GET /dashboard
    │   └── blog/
    │       ├── index.html    → GET /blog
    │       └── [slug].html   → GET /blog/:slug
    ├── components/
    │   ├── Header.cln            → <app-header>
    │   ├── Footer.cln            → <app-footer>
    │   ├── UserBadge.cln         → <user-badge>
    │   ├── StatCard.cln          → <stat-card>
    │   ├── NewsletterForm.cln    → <newsletter-form>
    │   ├── CommentForm.cln       → <comment-form>
    │   └── ShareButton.cln       → <share-button>
    ├── layouts/
    │   └── main.html
    └── public/
        └── css/
            └── main.css
```

## Features

- **Multiple pages**: Home, dashboard, blog listing, blog post
- **Custom components**: Header, footer, forms, buttons
- **Layouts**: Shared page layout
- **Static assets**: CSS styling
- **Dynamic routes**: Blog posts with slug parameter

## Running

```bash
# Scan
frame scan

# Build
frame build

# Run
frame serve
```
