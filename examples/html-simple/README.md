# HTML Simple Example

A simple example demonstrating HTML page serving with Clean Framework.

## Project Structure

```
app/
└── ui/
    ├── pages/
    │   ├── index.html    → GET /
    │   └── about.html    → GET /about
    └── public/
        └── css/
            └── style.css
```

## Running

```bash
# Scan the project
frame scan

# Build
frame build

# Run
frame serve
```

## Routes

| Route | Description |
|-------|-------------|
| `/` | Home page |
| `/about` | About page |
