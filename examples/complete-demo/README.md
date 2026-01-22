# Complete Demo

A full-stack Clean Framework application demonstrating all core plugins working together.

## Features

- **Database Access** - User model with SQLite using frame.data
- **Registration Form** - Form validation and submission using frame.ui
- **Authentication** - Session-based auth with password hashing using frame.auth
- **API Endpoints** - RESTful API using frame.httpserver
- **Canvas Animation** - Interactive particle system using frame.canvas

## Project Structure

```
complete-demo/
├── app.cln                    # Main application config
├── app/
│   ├── pages/                 # SSR HTML pages (frame.ui)
│   │   ├── index.html         # Landing page with animation
│   │   ├── register.html      # Registration form
│   │   └── dashboard.html     # User dashboard
│   │
│   ├── api/                   # API endpoints (frame.httpserver)
│   │   └── users.cln          # User registration/login/list
│   │
│   ├── data/                  # Data models (frame.data)
│   │   └── User.cln           # User model with CRUD
│   │
│   ├── auth/                  # Auth config (frame.auth)
│   │   └── config.cln         # Sessions, password hashing
│   │
│   ├── canvas/                # Canvas apps (frame.canvas)
│   │   └── particles.cln      # Particle animation
│   │
│   └── components/            # Reusable components
│
└── public/
    └── css/
        └── style.css          # Application styles
```

## Running the Demo

```bash
# Navigate to the example
cd examples/complete-demo

# Start the development server
cleen serve

# Open in browser
open http://localhost:3000
```

## Pages

### Home (/)
Landing page with:
- Interactive particle animation background
- Feature highlights
- Call-to-action buttons

### Register (/register)
Registration form with:
- Client-side validation
- Password confirmation
- Server-side validation
- Auto-login after registration

### Dashboard (/dashboard)
User dashboard with:
- Stats cards (total users, active users, new today)
- User table from database
- Embedded canvas animation

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | /api/users/register | Register new user |
| POST | /api/users/login | Login user |
| GET | /api/users | List all users |
| GET | /api/users/:id | Get user by ID |

## Plugins Used

1. **frame.data** - ORM for User model
2. **frame.httpserver** - API endpoints
3. **frame.auth** - Authentication & sessions
4. **frame.ui** - SSR pages & forms
5. **frame.canvas** - Particle animation

## Key Concepts Demonstrated

### Folder Conventions
Each folder maps to a specific plugin:
- `pages/` → frame.ui (HTML pages)
- `api/` → frame.httpserver (endpoints)
- `data/` → frame.data (models)
- `auth/` → frame.auth (config)
- `canvas/` → frame.canvas (animations)

### File Extensions
- `.html` for SSR pages (full editor support)
- `.css` for styles (full editor support)
- `.cln` for Clean Language logic

### Clean Templating in HTML
```html
<h1>Welcome, {{ user.name }}</h1>
<ul cl-each="item in items">
    <li>{{ item.title }}</li>
</ul>
<div cl-if="user.isAdmin">Admin content</div>
```

### Type-Safe Data Models
```clean
data User:
    fields:
        id: integer primary autoincrement
        email: string unique
        username: string
```

### Declarative Endpoints
```clean
endpoints:
    POST "/api/users/register":
        body:
            email: string
            password: string
        handle:
            // Type-safe request handling
```

### Canvas Animations
```clean
canvas particles:
    state:
        particles: list<Particle>
    update:
        // Physics simulation
    render:
        // Draw particles
```
