# Clean Framework Starter Template

A minimal starter template for Clean Framework projects.

## Project Structure

```
app/
├── ui/
│   ├── pages/           # HTML routes
│   │   └── index.html
│   ├── components/      # Reusable components
│   │   └── Header.cln
│   └── public/          # Static assets
│       └── css/
│           └── style.css
└── server/
    └── api/             # API endpoints
        └── hello.cln
```

## Getting Started

```bash
# Scan to verify structure
frame scan

# Build the project
frame build

# Run the server
frame serve
```

## Routes

| File | Route | Type |
|------|-------|------|
| `app/web/pages/index.html` | `/` | HTML |
| `app/server/api/hello.cln` | `/api/hello` | JSON |

## Customization

1. **Add pages**: Create `.html` files in `app/web/pages/`
2. **Add components**: Create `.cln` files in `app/web/components/`
3. **Add API routes**: Create `.cln` files in `app/server/api/`
4. **Add styles**: Put CSS in `assets/css/`
