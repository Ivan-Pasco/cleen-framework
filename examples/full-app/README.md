# Full App Example

A complete full-stack task management application with multiple pages, components, and database integration.

## Project Structure

```
app/
├── ui/
│   ├── pages/
│   │   ├── index.html        → GET /
│   │   ├── dashboard.html    → GET /dashboard
│   │   ├── tasks.html        → GET /tasks
│   │   ├── projects.html     → GET /projects
│   │   ├── settings.html     → GET /settings
│   │   └── login.html        → GET /login
│   ├── components/
│   │   ├── header.cln            → <app-header>
│   │   ├── sidebar.cln           → <app-sidebar>
│   │   ├── stats-card.cln        → <stats-card>
│   │   ├── task-card.cln         → <task-card>
│   │   ├── task-form.cln         → <task-form>
│   │   └── kanban-board.cln      → <kanban-board>
│   └── public/
│       ├── index.html
│       └── styles.css
└── server/
    ├── api/
    │   └── main.cln
    └── models/
        └── Schema.cln
```

## Features

- Task management with Kanban board
- Project organization
- User settings
- Dashboard with statistics
- Authentication flow

## Running

```bash
# Scan
frame scan

# Build
frame build

# Run
frame serve
```
