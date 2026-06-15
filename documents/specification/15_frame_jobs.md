# Frame Jobs Specification (15)

**Project:** Frame – Full-Stack Framework for Clean Language
**Version:** 1.0
**Location:** `/documents/specification/15_frame_jobs.md`

---

> **See also:** [Architecture Boundaries](../../../foundation/management/ARCHITECTURE_BOUNDARIES.md) — component responsibilities and cross-component work policy.

## 1. Purpose

`frame.jobs` provides background job processing and cron scheduling for Clean Language server applications. Jobs are declared as named handlers inside `jobs:` blocks. A server-side worker loop picks up pending jobs from the `__clean_jobs` database table, calls the handler function, and handles retries with configurable backoff.

**Goals:**
- Reliable asynchronous task execution outside the HTTP request/response cycle.
- Configurable retry strategies with fixed or exponential backoff.
- Cron scheduling for periodic tasks.
- Inspect and cancel jobs by ID.

---

## 2. File Layout

```
app/jobs/           # Owned by frame.jobs plugin
  welcome.cln       # Welcome email job
  digest.cln        # Daily digest scheduled job
  reports.cln       # Report generation jobs
```

Files in `app/jobs/` are auto-processed by `frame.jobs` once the plugin is declared in `main.cln`. Each file may contain one `jobs:` block.

---

## 3. The `jobs:` Block

The `jobs:` block groups job handler declarations and scheduled job declarations.

```clean
jobs:
    sendWelcomeEmail:
        retry:
            maxAttempts: 3
            backoff: exponential
            delay: 5000
        timeout: 30000
        queue: "email"
        string email = job.args
        email.send(email, "Welcome!", "<p>Welcome to our service.</p>", "Welcome to our service.")

    generateReport:
        retry:
            maxAttempts: 2
            backoff: fixed
            delay: 10000
        timeout: 120000
        string reportId = job.args
        buildReport(reportId)
        job.succeed("{\"reportId\": \"" + reportId + "\"}")
```

---

## 4. Job Handler Format

Each named entry inside `jobs:` is a job handler:

```clean
jobs:
    handlerName:
        retry:
            maxAttempts: <integer>
            backoff: fixed | exponential
            delay: <milliseconds>
        timeout: <milliseconds>
        queue: "<queue-name>"
        // handler body — any valid Clean statements
        // job.args, job.id, job.attempt are in scope
```

All configuration fields are optional:

| Field | Default | Description |
|-------|---------|-------------|
| `retry: maxAttempts` | 1 | Number of attempts before marking the job failed |
| `retry: backoff` | `fixed` | Delay strategy between retries |
| `retry: delay` | 1000 | Base delay in milliseconds |
| `timeout` | 0 (unlimited) | Maximum handler run time in milliseconds |
| `queue` | `"default"` | Named queue for prioritized or isolated processing |

### 4.1 Retry Configuration

```clean
retry:
    maxAttempts: 5
    backoff: exponential
    delay: 2000
```

With `backoff: exponential` and `delay: 2000`, the delays between attempts are: 2 s, 4 s, 8 s, 16 s.

With `backoff: fixed` and `delay: 5000`, every retry waits 5 s.

---

## 5. Handler Context

Inside a job handler body, the following identifiers are in scope:

| Identifier | Type | Description |
|------------|------|-------------|
| `job.id` | `string` | UUID of the currently executing job record |
| `job.args` | `string` | JSON string passed to `queue.enqueue()` |
| `job.attempt` | `integer` | 1-based attempt number (1 = first try) |

### 5.1 Controlling Execution

| Function | Signature | Description |
|----------|-----------|-------------|
| `job.retryAfter(seconds)` | `(integer)` | Override backoff; wait exactly `seconds` before the next retry |
| `job.fail(reason)` | `(string)` | Permanently fail with `reason`; skips remaining retries |
| `job.succeed(resultJson)` | `(string)` | Complete successfully; persist `resultJson` as the result |

Calling `job.fail()` or `job.succeed()` immediately stops handler execution. `job.retryAfter()` must be called before the handler returns normally to override the default backoff.

---

## 6. Enqueuing Jobs

Jobs are enqueued from anywhere in the server code using `queue.enqueue()`:

```clean
// Immediate execution
string jobId = queue.enqueue("sendWelcomeEmail", user.email)

// Deferred execution (Unix timestamp in seconds)
integer tomorrow = _time_now() + 86400
string jobId = queue.enqueueAt("sendDigest", user.id.toString(), tomorrow)
```

Both functions return the UUID job ID string, which can be used to inspect or cancel the job later.

---

## 7. The `schedule` Declaration

Recurring tasks are declared with `schedule` inside a `jobs:` block:

```clean
jobs:
    schedule dailyDigest "0 8 * * *":
        list<User> users = User.find:
            where:
                active == true
        integer idx = 0
        while idx < users.length()
            queue.enqueue("sendDigest", users[idx].id.toString())
            idx = idx + 1

    schedule cleanupTokens "0 */4 * * *":
        Token.delete:
            where:
                expiresAt < _time_now()
```

The cron expression uses standard 5-field syntax: `min hour dom mon dow`.

| Expression | Meaning |
|------------|---------|
| `"0 8 * * *"` | Daily at 08:00 UTC |
| `"*/15 * * * *"` | Every 15 minutes |
| `"0 0 * * 1"` | Every Monday at midnight UTC |
| `"0 */4 * * *"` | Every 4 hours |

### Cancelling a Schedule

```clean
boolean removed = schedule.cancel("dailyDigest")
```

---

## 8. Inspecting Jobs

```clean
// Get current status
string status = queue.status(jobId)
// Returns: "pending" | "running" | "completed" | "failed" | "cancelled"

// Get the result of a completed job
string result = queue.result(jobId)

// Cancel a pending job
boolean cancelled = queue.cancel(jobId)
```

---

## 9. Error Handling

If a handler throws an error that is not caught by an `onError:` block, the worker increments the attempt count and schedules a retry according to the configured backoff. When all attempts are exhausted, the job is marked `failed` with the error message.

```clean
jobs:
    processPayment:
        retry:
            maxAttempts: 3
            backoff: exponential
            delay: 10000
        string paymentId = job.args
        Payment payment = Payment.first:
            where: id == paymentId
        onError:
            job.fail("Payment record not found: " + paymentId)
        boolean ok = chargeCard(payment.token, payment.amount)
        if not ok
            if job.attempt < 3
                job.retryAfter(30)
            else
                job.fail("Card charge failed after 3 attempts")
        job.succeed("{\"paymentId\": \"" + paymentId + "\", \"charged\": true}")
```

---

## 10. Complete Example

### Welcome Email Job + Daily Digest Cron

```clean
// app/jobs/email-jobs.cln

jobs:
    sendWelcomeEmail:
        retry:
            maxAttempts: 3
            backoff: exponential
            delay: 3000
        timeout: 15000
        queue: "email"
        string to = job.args
        boolean sent = email.send(
            to,
            "Welcome to MyApp!",
            "<h1>Welcome!</h1><p>Your account is ready.</p>",
            "Welcome! Your account is ready."
        )
        if not sent
            job.fail("SMTP error: " + email.lastError())
        job.succeed("{\"sent\": true}")

    schedule dailyDigest "0 8 * * *":
        list<User> users = User.find:
            where:
                active == true
                AND emailDigest == true
        integer idx = 0
        while idx < users.length()
            queue.enqueue("sendWelcomeEmail", users[idx].email)
            idx = idx + 1
```

### Enqueueing from an Endpoint

```clean
// app/server/api/auth.cln
endpoints:
    POST "/api/auth/register" :
        CreateUser body = req.json(CreateUser)
        User u = User.insert:
            email = body.email
            name  = body.name
        queue.enqueue("sendWelcomeEmail", u.email)
        return json({ ok: true, userId: u.id })
```

---

## 11. `__clean_jobs` Schema

The worker auto-creates the following table on first use:

| Column | Type | Description |
|--------|------|-------------|
| `id` | TEXT PK | UUID job identifier |
| `name` | TEXT | Job handler name |
| `args` | TEXT | Serialised arguments (JSON string) |
| `status` | TEXT | `pending` / `running` / `completed` / `failed` / `cancelled` |
| `attempt` | INTEGER | Current attempt number (1-based) |
| `max_attempts` | INTEGER | Maximum attempts before permanent failure |
| `backoff` | TEXT | `fixed` or `exponential` |
| `delay` | INTEGER | Base retry delay in milliseconds |
| `timeout` | INTEGER | Handler timeout in milliseconds (0 = unlimited) |
| `queue` | TEXT | Named queue |
| `scheduled_at` | INTEGER | Unix timestamp for deferred jobs |
| `created_at` | INTEGER | Unix timestamp when job was enqueued |
| `updated_at` | INTEGER | Unix timestamp of last status change |
| `result` | TEXT | JSON result from `job.succeed()` |
| `error` | TEXT | Error message from `job.fail()` or uncaught exception |

---

## 12. Scheduling Semantics

### 12.1 Cron expression format

Standard 5-field cron syntax: `minute hour day-of-month month day-of-week`.

| Field | Range | Special |
|---|---|---|
| minute | 0–59 | `*`, `*/n`, `a,b,c`, `a-b` |
| hour | 0–23 | same |
| day-of-month | 1–31 | same, `?` (any) |
| month | 1–12 or `jan`–`dec` | same |
| day-of-week | 0–6 or `sun`–`sat` (0 = Sunday) | same |

Examples (timezone is **UTC** unless overridden):

| Expression | Meaning |
|---|---|
| `"0 8 * * *"` | Every day at 08:00 UTC |
| `"*/15 * * * *"` | Every 15 minutes |
| `"0 0 * * 1"` | Mondays at midnight UTC |
| `"30 9 1 * *"` | First of every month at 09:30 UTC |
| `"0 12 * * mon-fri"` | Weekdays at noon UTC |

### 12.2 Timezone

Cron expressions are evaluated in **UTC** by default. To run in a local timezone, embed the offset in the calculation when generating the expression — frame.jobs does not currently parse timezone names. (TZ-aware cron is on the roadmap; see [13_frame_future_evolution.md](13_frame_future_evolution.md).)

### 12.3 Execution guarantees

| Property | Guarantee |
|---|---|
| Cron firing | **At-least-once**. If a worker is offline at the scheduled minute, the run is dispatched as soon as a worker comes online (up to a one-hour catch-up window). |
| Enqueued job | **At-least-once**. Handler may run twice if a worker crashes after starting but before recording completion. Handlers must be **idempotent**. |
| Concurrent runs | A single scheduled name has at most one in-flight run at a time. The next firing is skipped if the previous run is still active and `coalesce = false` (default). |
| Ordering | Within a queue, jobs are dispatched in `scheduled_at` order, then `created_at`. Cross-queue ordering is not guaranteed. |

### 12.4 Backoff strategies

`fixed` — every retry waits exactly `delay` ms:

| Attempt | Wait |
|---|---|
| 1 → 2 | `delay` |
| 2 → 3 | `delay` |
| 3 → 4 | `delay` |

`exponential` — each retry doubles the previous wait:

| Attempt | Wait | Example (`delay=2000`) |
|---|---|---|
| 1 → 2 | `delay` | 2 s |
| 2 → 3 | `delay * 2` | 4 s |
| 3 → 4 | `delay * 4` | 8 s |
| 4 → 5 | `delay * 8` | 16 s |

The runtime caps individual waits at 1 hour to prevent pathological backoff. `job.retryAfter(seconds)` inside a handler overrides the computed delay for the next attempt only.

### 12.5 Choosing a strategy

| Use case | Strategy |
|---|---|
| Transient network errors | `exponential`, `delay: 1000`, `maxAttempts: 5` |
| Rate-limited APIs | `fixed`, `delay: 60000`, `maxAttempts: 10` (one retry per minute) |
| Heavy work that may need manual intervention | `fixed`, `delay: 300000`, `maxAttempts: 2` (give an operator 5 min to react) |

---

## 13. Plugin Contracts v2 Integration

frame.jobs **1.0.x** declares every bridge with `hosts = ["server"]` — the queue, scheduler, retry state, and cron timers all live in the long-running server process ([bridge-host-classes.md §2](../../../foundation/spec/plugins/contracts/bridge-host-classes.md#2-the-hosts-field)). A browser-mode build receives no-op stubs; enqueuing from the client must always go through an `endpoints:` route on the server.

---

**End of Document 15 — Frame Jobs Specification**
