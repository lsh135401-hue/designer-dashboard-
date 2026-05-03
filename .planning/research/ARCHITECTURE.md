# Architecture Research

**Domain:** macOS menu-bar productivity app — multi-channel ingestion + Notion as Source of Truth + read-only public sharing
**Researched:** 2026-05-03
**Confidence:** HIGH (Tauri/Rust patterns, SQLite, Cloudflare KV, Notion webhooks, Slack Socket Mode all verified against official docs and current 2025-2026 sources)

---

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       macOS USER SPACE (Tauri 2.x app)                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌─────────────────────────────┐      ┌──────────────────────────────┐     │
│   │   FRONTEND (React/TS)        │◄────►│  TAURI IPC (invoke/emit)     │     │
│   │   - Menu bar tray UI         │      │  - Typed commands (TauRPC)   │     │
│   │   - Dashboard window         │      │  - Event channel (push)      │     │
│   │   - Draft cards / Send modal │      │                              │     │
│   └─────────────────────────────┘      └────────────┬─────────────────┘     │
│                                                     │                        │
│   ┌─────────────────────────────────────────────────▼─────────────────┐     │
│   │                  RUST BACKEND  (single tokio runtime)              │     │
│   │                                                                    │     │
│   │   ┌───────────────────┐   ┌──────────────────────────────┐         │     │
│   │   │ orchestrator/     │   │ ai/  (central, NOT per-chan) │         │     │
│   │   │ - Scheduler       │◄──►│ - Anthropic SDK + caching    │         │     │
│   │   │ - Wake/sleep obs. │   │ - Classifier / Drafter        │         │     │
│   │   │ - Job dispatcher  │   │ - Insight generator           │         │     │
│   │   └─────────┬─────────┘   └──────────────────────────────┘         │     │
│   │             │                                                       │     │
│   │   ┌─────────▼──────────┐  ┌────────────────────────────┐            │     │
│   │   │ channels/          │  │ sync/  (Notion = SOTR)     │            │     │
│   │   │ ┌─slack/  (WS)─┐   │  │ - Reconciler               │            │     │
│   │   │ ┌─gmail/  (poll)   │  │ - LWW by last_edited_time  │            │     │
│   │   │ ┌─gcal/   (poll)   │  │ - Communication Log writer │            │     │
│   │   │ ┌─notion/ (poll)   │  │ - Schema wizard            │            │     │
│   │   │ └─kakao/  (FSEv)─┘ │  └─────────┬──────────────────┘            │     │
│   │   └─────────┬──────────┘            │                               │     │
│   │             │  (mpsc EventBus)      │                               │     │
│   │             ▼                       ▼                               │     │
│   │   ┌──────────────────────────────────────────────┐                  │     │
│   │   │ data/  (SQLite WAL + 1 writer + N readers)   │                  │     │
│   │   │ - messages, attachments, projects, drafts    │                  │     │
│   │   │ - sync_cursors (per-channel position)        │                  │     │
│   │   └────────────────┬─────────────────────────────┘                  │     │
│   │                    │                                                 │     │
│   │   ┌────────────────▼─────────────────────────────┐                  │     │
│   │   │ share/  (snapshot generator → CF Worker push)│                  │     │
│   │   │ - sanitize: strip message bodies             │                  │     │
│   │   │ - publish: progress, dates, next steps only  │                  │     │
│   │   └────────────────┬─────────────────────────────┘                  │     │
│   └────────────────────┼─────────────────────────────────────────────── ┘     │
│                        │  HTTPS PUT (sanitized snapshot)                      │
│   ┌────────────────────┼─────────────────────────────────────────────┐       │
│   │ secrets/  (macOS Keychain via security-framework)                 │       │
│   │ - OAuth tokens, refresh tokens, signing keys, share secret        │       │
│   └───────────────────────────────────────────────────────────────────┘       │
└────────────────────────┼─────────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                  CLOUDFLARE EDGE  (read-only share + webhook bridge)         │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌──────────────────┐  ┌──────────────────┐  ┌────────────────────────┐     │
│  │ Worker: /share/* │  │ Worker: /admin/* │  │ Worker: /webhook/*     │     │
│  │ - Token check    │  │ - Auth: app HMAC │  │ - Slack signing verify │     │
│  │ - Read KV/R2     │  │ - PUT snapshot   │  │ - Notion HMAC verify   │     │
│  │ - Render HTML    │  │ - Revoke token   │  │ - Buffer to KV queue   │     │
│  └────────┬─────────┘  └────────┬─────────┘  └────────────┬───────────┘     │
│           │                     │                         │                  │
│  ┌────────▼─────────────────────▼─────────────────────────▼───────────┐     │
│  │ KV: snapshots/{token} | revoked/{token} | inbox/{channel}/{id}    │     │
│  │ R2: thumbnails/{project}/{file_id}.webp                            │     │
│  └────────────────────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────────────────────┘
                         ▲
                         │ Long-poll /webhook/inbox/drain
┌────────────────────────┴──────────────┐
│ External services                      │
│ Slack (Socket Mode WS) ───────► app    │  (no Cloudflare needed)
│ Slack Events (webhook) ──► CF Worker   │  (fallback for when app offline)
│ Gmail API (polling, 60s) ──── app      │
│ Google Cal (polling) ────────── app    │
│ Notion API (poll + webhook→CF) ► app   │
│ Filesystem ~/KakaoTalk/*.txt ─► app    │
└────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Implementation |
|-----------|----------------|----------------|
| `frontend/` | UI rendering, draft review, manual actions | React 18 + Tailwind + shadcn/ui |
| `tauri::commands` | Typed RPC surface for frontend → Rust | `#[tauri::command]` async fns + TauRPC for type generation |
| `orchestrator/` | Scheduler, sleep/wake observer, job dispatcher | tokio + `mac-notification-sys` for NSWorkspace observers |
| `channels/<name>/` | Per-channel ingestion (1 module each) | tokio task per channel, mpsc to EventBus |
| `sync/` | Reconciliation between local SQLite and Notion | LWW by `last_edited_time`; idempotent upserts |
| `ai/` | Single AI orchestrator (classifier + drafter + insights) | Anthropic SDK; centralized so prompt-caching works across channels |
| `data/` | SQLite WAL — 1 writer connection + N readers | sqlx with `acquire_writer()` mutex pattern |
| `share/` | Sanitize + publish snapshots, manage tokens | hyper/reqwest → CF Worker `/admin/snapshot` |
| `secrets/` | OS Keychain storage for OAuth + signing keys | `keyring` crate (security-framework on macOS) |
| `cf-worker-share/` | Render read-only HTML, check tokens | Workers + KV + R2 |
| `cf-worker-bridge/` | Receive Slack/Notion webhooks, buffer for app | Workers + KV (durable inbox queue) |

---

## Recommended Project Structure

```
designer-dashboard/
├── src/                            # React frontend
│   ├── routes/
│   │   ├── tray.tsx                # menu-bar popover
│   │   ├── dashboard.tsx           # main window
│   │   └── settings.tsx
│   ├── features/
│   │   ├── brief/                  # daily AI brief
│   │   ├── drafts/                 # send-draft cards
│   │   ├── projects/               # project list + detail
│   │   └── share/                  # share URL management UI
│   └── lib/
│       ├── tauri-bindings.ts       # generated by TauRPC
│       └── store.ts                # Zustand
│
├── src-tauri/                      # Rust backend
│   ├── src/
│   │   ├── main.rs                 # tauri::Builder, plugin init
│   │   ├── commands/               # IPC surface (#[tauri::command])
│   │   │   ├── projects.rs
│   │   │   ├── drafts.rs
│   │   │   ├── share.rs
│   │   │   └── auth.rs
│   │   ├── orchestrator/
│   │   │   ├── scheduler.rs        # tokio interval loops
│   │   │   ├── wake_observer.rs    # NSWorkspace didWakeNotification
│   │   │   └── event_bus.rs        # mpsc fan-in from channels
│   │   ├── channels/
│   │   │   ├── mod.rs              # trait Channel { fn run(); }
│   │   │   ├── slack/
│   │   │   │   ├── socket_mode.rs  # tokio_tungstenite WS loop
│   │   │   │   ├── webhook_drain.rs# pull from CF Worker queue
│   │   │   │   └── api.rs          # web API (chat.postMessage, files.upload)
│   │   │   ├── gmail/
│   │   │   │   ├── poller.rs       # historyId-based incremental sync
│   │   │   │   └── api.rs          # send draft
│   │   │   ├── gcal/
│   │   │   │   ├── poller.rs       # syncToken-based incremental
│   │   │   │   └── api.rs
│   │   │   ├── notion/
│   │   │   │   ├── webhook_drain.rs# pull from CF Worker queue
│   │   │   │   ├── poller.rs       # last_edited_time fallback (every 2-5min)
│   │   │   │   ├── writer.rs       # append Communication Log
│   │   │   │   └── schema.rs       # DB discovery + wizard
│   │   │   └── kakao/
│   │   │       └── fs_watcher.rs   # notify crate (FSEvents)
│   │   ├── sync/
│   │   │   ├── reconciler.rs       # SOTR application
│   │   │   ├── lww.rs              # Last-Write-Wins by last_edited_time
│   │   │   └── conflict_log.rs
│   │   ├── ai/
│   │   │   ├── client.rs           # Anthropic SDK with prompt caching
│   │   │   ├── classifier.rs       # message → project routing
│   │   │   ├── drafter.rs          # generate send-drafts
│   │   │   └── briefer.rs          # daily 8am brief
│   │   ├── data/
│   │   │   ├── pool.rs             # sqlx: 1 writer + N readers
│   │   │   ├── migrations/
│   │   │   └── repo/               # one file per aggregate
│   │   ├── share/
│   │   │   ├── sanitizer.rs        # STRIPS message bodies
│   │   │   ├── publisher.rs        # PUT snapshot to CF Worker
│   │   │   └── tokens.rs
│   │   ├── secrets/
│   │   │   └── keychain.rs         # `keyring` crate wrapper
│   │   └── error.rs
│   ├── tauri.conf.json
│   └── Cargo.toml
│
├── cloudflare/
│   ├── share-worker/               # /share/{token} read-only renderer
│   │   ├── src/index.ts
│   │   └── wrangler.toml
│   └── bridge-worker/              # webhook receiver + drain queue
│       ├── src/index.ts            # /webhook/slack, /webhook/notion, /inbox/drain
│       └── wrangler.toml
│
├── packaging/
│   ├── DesignerDashboard.app/
│   ├── com.user.designer-dashboard.plist  # launchd LaunchAgent
│   └── sparkle/                    # appcast.xml + signing
│
└── .planning/
```

### Structure Rationale

- **`channels/<name>/` one folder per channel:** each channel has wildly different transport (WebSocket vs polling vs filesystem); a `Channel` trait would force the lowest common denominator. Keep them sibling modules implementing a small `IngestSink` interface that pushes to the central EventBus.
- **`ai/` centralized, not per-channel:** Anthropic prompt caching keys on the prefix, so reusing the same system prompt across all classification calls cuts cost ~5x. Per-channel AI would defeat caching.
- **`sync/` separated from `channels/`:** channels emit raw events; reconciler decides what to write to Notion. This keeps the SOTR rule in exactly one place — easier to audit privacy invariants.
- **`share/` is its own module:** the sanitizer is the privacy boundary. Isolating it means message-body leakage requires importing `share::publisher` from outside `share/` — a grep-able invariant.
- **`cloudflare/` two separate workers:** `share-worker` is read-heavy and public; `bridge-worker` is write-heavy and authenticated. Different security postures, different KV namespaces, different rate-limit profiles. Keeping them separate avoids one mistake leaking everything.

---

## Architectural Patterns

### Pattern 1: Webhook-to-Queue Bridge (CF Worker as durable inbox)

**What:** Slack and Notion both want a public HTTPS endpoint for webhooks. A desktop app cannot provide one. Solution: a Cloudflare Worker accepts the webhook, verifies the signature, and writes the event to KV under `inbox/{channel}/{ulid}`. The desktop app long-polls a `/inbox/drain` endpoint authenticated with an HMAC the app shares with the Worker. After successful drain, app calls `/inbox/ack` with the ULID list.

**When to use:** Any third-party that requires a public webhook URL but your client is a desktop app.

**Trade-offs:**
- (+) Survives app being offline (events buffered up to 24h in KV)
- (+) Works behind NAT, no port forwarding, no ngrok
- (+) Same pattern reused for Slack and Notion (and future services)
- (-) Adds 50-200ms latency vs Socket Mode
- (-) Requires CF Worker uptime (acceptable: 99.99% SLA)
- (-) KV eventual consistency (60s globally) — fine for an inbox, not a cache

**Example (CF Worker bridge):**
```typescript
// cloudflare/bridge-worker/src/index.ts
export default {
  async fetch(req: Request, env: Env): Promise<Response> {
    const url = new URL(req.url);

    if (url.pathname === "/webhook/slack") {
      // Slack signing verification (X-Slack-Signature, X-Slack-Request-Timestamp)
      if (!await verifySlack(req, env.SLACK_SIGNING_SECRET)) return new Response("403", { status: 403 });
      const body = await req.json();
      // URL verification challenge
      if (body.type === "url_verification") return Response.json({ challenge: body.challenge });
      // Buffer event
      const id = ulid();
      await env.INBOX.put(`inbox/slack/${id}`, JSON.stringify(body), { expirationTtl: 86400 });
      return new Response("ok"); // Slack must get 200 within 3s
    }

    if (url.pathname === "/inbox/drain") {
      if (!await verifyAppHmac(req, env.APP_SHARED_SECRET)) return new Response("401", { status: 401 });
      const list = await env.INBOX.list({ prefix: "inbox/", limit: 100 });
      const events = await Promise.all(list.keys.map(k => env.INBOX.get(k.name)));
      return Response.json({ events: list.keys.map((k, i) => ({ key: k.name, body: events[i] })) });
    }
    // /inbox/ack { keys: [...] } — DELETEs after app commits to local DB
  }
}
```

### Pattern 2: Hybrid transport per channel (best tool for the job)

**What:** Don't force one delivery mechanism on all channels. Pick what each provider supports best:

| Channel | Primary | Fallback | Rationale |
|---------|---------|----------|-----------|
| **Slack** | Socket Mode (WebSocket) when app online | CF Worker bridge when offline | Socket Mode is real-time and needs no public URL; bridge fills the offline gap |
| **Gmail** | Polling via `users.history.list` (60s interval) | n/a | Google's official guidance: Pub/Sub is for servers, polling is recommended for desktop apps |
| **Google Calendar** | Polling via `events.list` with `syncToken` (90s interval) | n/a | Push channels require HTTPS + cert renewal every 1-7 days; polling is simpler |
| **Notion** | CF Worker webhook bridge (`page.created`, `page.content_updated`) | Polling `last_edited_time` every 2 min | Notion 2025-09-03 has webhooks; polling fills gaps when bridge is down |
| **KakaoTalk** | FSEvents on watch folder | n/a | Only available channel; `notify` crate uses FSEvents on macOS |

**When to use:** Any multi-source ingestion where providers differ in capability.

**Trade-offs:**
- (+) Lowest latency where possible, simplest where not
- (+) Each channel is independently testable
- (-) More code (no shared transport layer)
- (-) Per-channel cursor state (Slack ts, Gmail historyId, GCal syncToken, Notion last_edited_time)

### Pattern 3: Single-Writer SQLite with Async Mutex

**What:** SQLite WAL allows many concurrent readers but **exactly one writer**. With multiple ingestion tasks all wanting to write, a naive `Pool<Sqlite>` causes "database is locked" errors. Use an explicit single-writer connection with an async mutex, plus a separate read pool.

**When to use:** Any SQLite app with concurrent ingestion sources (this is the common case).

**Trade-offs:**
- (+) Eliminates SQLITE_BUSY errors
- (+) Faster than letting the pool serialize via locking
- (-) Slightly more boilerplate than a single pool

**Example:**
```rust
// src-tauri/src/data/pool.rs
pub struct Db {
    writer: Arc<tokio::sync::Mutex<SqliteConnection>>,
    readers: SqlitePool,  // size = min(num_cpus, 4)
}

impl Db {
    pub async fn new(path: &Path) -> Result<Self> {
        let opts = SqliteConnectOptions::new()
            .filename(path)
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_secs(5))
            .synchronous(SqliteSynchronous::Normal);

        let writer = SqliteConnection::connect_with(&opts).await?;
        let readers = SqlitePoolOptions::new()
            .max_connections(4)
            .connect_with(opts.read_only(true)).await?;
        Ok(Self { writer: Arc::new(Mutex::new(writer)), readers })
    }

    pub async fn write<F, T>(&self, f: F) -> Result<T>
    where F: for<'c> FnOnce(&'c mut SqliteConnection) -> BoxFuture<'c, Result<T>>
    {
        let mut conn = self.writer.lock().await;
        f(&mut *conn).await
    }
}
```

### Pattern 4: Notion as Source of Truth — LWW Reconciliation

**What:** Both local app and Notion can mutate a project. On every sync cycle, compare local `mirror.last_edited_time` (cached when we last pulled) vs Notion's current `last_edited_time`. Three cases:

```
Local UNCHANGED && Notion UNCHANGED → no-op
Local UNCHANGED && Notion CHANGED   → pull Notion → local
Local CHANGED   && Notion UNCHANGED → push local → Notion
Local CHANGED   && Notion CHANGED   → CONFLICT: Notion wins (SOTR), append local
                                       diff to Communication Log for audit
```

**When to use:** Bidirectional sync where one side is authoritative.

**Trade-offs:**
- (+) Simple, deterministic, auditable
- (+) Aligns with stated SOTR rule
- (-) Loses local edits in conflict cases (mitigated by Communication Log)
- (-) `last_edited_time` updates only every 60s in Notion — race window exists

**Example:**
```rust
// src-tauri/src/sync/lww.rs
pub enum Resolution { NoOp, PullFromNotion, PushToNotion, ConflictNotionWins }

pub fn resolve(local: &ProjectMirror, remote: &NotionPage) -> Resolution {
    let local_dirty = local.local_modified_at > local.last_synced_at;
    let remote_changed = remote.last_edited_time > local.last_synced_at;
    match (local_dirty, remote_changed) {
        (false, false) => Resolution::NoOp,
        (false, true)  => Resolution::PullFromNotion,
        (true,  false) => Resolution::PushToNotion,
        (true,  true)  => Resolution::ConflictNotionWins, // append loser to Communication Log
    }
}
```

### Pattern 5: Sanitize-Then-Publish Share Boundary

**What:** Share-URL snapshots are generated by `share/sanitizer.rs`, which builds a `SharedView` struct that **only contains** progress %, milestone names + dates, next-step text, project title. Message bodies, attachments, drafts, AI insights are structurally absent from the type. The publisher serializes `SharedView` directly — no `From<Project>` impl exists that would let the wrong type slip through.

**When to use:** Any feature where a privacy invariant must hold.

**Trade-offs:**
- (+) Compiler-enforced: leaking message bodies requires actively rewriting `SharedView`
- (+) Easy to audit: grep for `SharedView` definition
- (-) Schema duplication (acceptable — these views diverge intentionally)

**Example:**
```rust
// src-tauri/src/share/sanitizer.rs
#[derive(Serialize)]
pub struct SharedView {
    pub project_title: String,
    pub progress_percent: u8,
    pub current_milestone: String,
    pub upcoming_dates: Vec<MilestoneDate>,
    pub next_step_summary: String,    // 1 sentence, AI-generated, no quotes
    // NOTE: NO messages, NO attachments, NO drafts, NO message_count
}

// The ONLY way to build a SharedView:
pub fn sanitize(project: &Project, milestones: &[Milestone]) -> SharedView { ... }
```

### Pattern 6: macOS Sleep/Wake-Aware Scheduler

**What:** Polling cursors get stale during sleep. On `NSWorkspaceDidWakeNotification`, force a sync round across all channels and re-anchor the launchd interval timer.

**When to use:** Any scheduled work on a laptop that might sleep mid-cycle.

**Implementation:**
1. **launchd LaunchAgent** (`~/Library/LaunchAgents/com.user.designer-dashboard.plist`) with `RunAtLoad=true` ensures app starts at login. Do **not** use `StartCalendarInterval` for the daily 8am brief — it doesn't fire if the Mac was asleep and won't catch up reliably for a one-shot. Instead, the running app schedules its own 8am tokio timer and uses launchd only for the bootstrap.
2. **Wake observer** registers for `NSWorkspaceDidWakeNotification` via `objc2`/`mac-notification-sys` and sends a `WakeUp` message on the orchestrator's mpsc.
3. **On wake**: scheduler resets next-tick to "now + 5s", triggers a full sync round, re-checks the 8am-brief deadline.

**Trade-offs:**
- (+) Cursors stay fresh after wake
- (+) Brief still fires on the right calendar day even after long sleep
- (-) Requires unsafe Objective-C bindings or a plugin

---

## Data Flow

### Flow 1: Slack message arrives → AI classifies → Notion log → share KV

```
Slack server
   │  (1) WebSocket frame: { type: "message", channel: "C123", text: "샘플 도착" }
   ▼
channels/slack/socket_mode.rs
   │  (2) Acks frame, parses to RawEvent::Slack(...)
   │      (If app offline, this same payload arrived via /webhook/slack →
   │       cf-worker-bridge KV → drained later by webhook_drain.rs)
   ▼
orchestrator/event_bus.rs   (mpsc channel, buffered=1024)
   │  (3) RawEvent enqueued
   ▼
sync/reconciler.rs   (single consumer task)
   │  (4) Persist raw event to data/messages (SQLite write via writer mutex)
   │  (5) Look up project_id in cache:
   │       a. exact channel name match → projects.slack_channel_id
   │       b. miss → call ai/classifier.rs (with 2-line context + project list)
   │       c. confidence < 0.7 → flag for manual review, do NOT auto-route
   │  (6) Write project_links row (message_id → project_id, confidence)
   ▼
sync/notion_writer.rs
   │  (7) Append to Communication Log property of Notion page:
   │       PATCH /v1/blocks/{page_id}/children
   │       { children: [{ paragraph: { rich_text: [{ text: "[Slack #채널] 샘플 도착 — 14:32" }]}}]}
   │       (NO message body — only sender + summary + timestamp + link back)
   ▼
share/publisher.rs   (debounced 30s per project)
   │  (8) Recompute progress, milestones; build SharedView via sanitize()
   │  (9) HTTPS PUT https://share.designer-dash.workers.dev/admin/snapshot
   │       { project_id, view, hmac } → CF Worker writes KV
   ▼
Cloudflare KV: snapshots/{token} updated
   │  (10) MD/factory hits /share/{token} → Worker reads KV → renders HTML
   ▼
Frontend (React)
   │  (11) Tauri event "project_updated" → dashboard re-renders
   └────  (12) macOS notification (if from new project or matches user filter)
```

### Flow 2: User clicks "Send" on AI draft → Slack → Notion → local DB → UI

```
React (drafts/SendButton.tsx)
   │  (1) await invoke("send_draft", { draftId, channel, recipient })
   ▼
commands/drafts.rs::send_draft   (#[tauri::command])
   │  (2) Load Draft from data/repo/drafts.rs
   │  (3) Idempotency: if draft.sent_at.is_some() → return Ok(already_sent)
   ▼
channels/slack/api.rs::post_message
   │  (4) POST https://slack.com/api/chat.postMessage
   │       { channel: "C123", text: draft.body, thread_ts?: ... }
   │       Auth: Bearer token from secrets/keychain.rs
   │  (5) Slack returns { ok: true, ts: "1234567890.123" }
   ▼
data writer (single mutex)
   │  (6) UPDATE drafts SET sent_at=now, slack_ts='...' WHERE id=?
   │  (7) INSERT messages (channel='slack', ts, body, project_id, sent_by_app=true)
   ▼
sync/notion_writer.rs::append_communication_log
   │  (8) PATCH /v1/blocks/{project_page}/children
   │       Append: "[발송→#채널] " + draft.summary + " — 16:08"
   │  (9) On 409/conflict from Notion: retry once with refreshed parent
   │       On persistent failure: write to drafts.notion_sync_pending=true,
   │                              orchestrator retries on next sync cycle
   ▼
share/publisher.rs (debounced)
   │  (10) Snapshot updated (next_step_summary may change)
   ▼
tauri::AppHandle.emit("draft_sent", { draftId, slack_ts })
   │  (11) React updates card to "Sent ✓ 16:08", removes from inbox
   └────  Returns Ok(SendResult) to original invoke
```

---

## Suggested Build Order

The PROJECT.md milestones are not numbered, but the requirements imply a stack. Here is the dependency-validated order with rationale per step:

```
[Phase 1: Foundation Shell]
  1. Tauri 2 menu-bar tray + auto-start LaunchAgent
  2. SQLite WAL with single-writer pattern + migrations
  3. Keychain wrapper (`keyring` crate)
  4. Sleep/wake observer + tokio scheduler skeleton
  Why first: every other phase needs persistence + scheduling.
  Validation gate: app launches at login, survives sleep/wake, writes to SQLite.

[Phase 2: Notion SOTR]
  5. Notion OAuth flow + token storage
  6. Schema discovery + missing-field wizard
  7. Polling (last_edited_time every 2 min) — defer webhooks until Phase 7
  8. LWW reconciler + Communication Log writer
  9. Conflict log table for audit
  Why second: Notion is the SOTR. Every channel writes through this layer.
  Risk: schema wizard may surface unexpected user variations — budget extra time.

[Phase 3: Cloudflare Bridge + Share Skeleton]
  10. CF Worker bridge skeleton (no real webhooks yet, just /inbox/* endpoints)
  11. CF Worker share endpoint (token-based KV reader, plain HTML)
  12. Sanitizer module + SharedView type + publisher
  13. Token issue/revoke commands (frontend + admin endpoint)
  Why third (before channels): share/sanitizer is the privacy boundary that
    every later channel must respect. Establishing the boundary type early
    prevents later channels from accidentally bypassing it.

[Phase 4: Slack — Real-time + Webhook Fallback]
  14. Slack OAuth + Bot/User token storage
  15. Socket Mode WebSocket loop (slack-morphism-rust)
  16. CF bridge: /webhook/slack handler with signing verify + KV inbox
  17. webhook_drain.rs: pull events when Socket Mode was offline
  18. AI classifier (channel name → project, confidence threshold)
  19. chat.postMessage + files.upload for Send/Attachments
  Why fourth: Slack is the highest-volume channel; needs Notion writer ready.
  Critical: dual-path (WS + bridge drain) requires de-duplication by event ts.

[Phase 5: Gmail + Google Calendar]
  20. Google OAuth (single flow, both scopes)
  21. Gmail historyId polling (60s)
  22. Gmail send (drafts.send) for Send pipeline
  23. GCal syncToken polling (90s)
  24. GCal event creation/update for milestone due_date sync
  Why fifth: similar transport (polling), shared OAuth, can ship together.

[Phase 6: KakaoTalk + Adobe File Recognition]
  25. notify crate watch folder for ~/Documents/KakaoTalk/*.txt
  26. Parser for KakaoTalk export format (date, sender, body)
  27. Idempotency by content hash
  28. Slack attachment .psd/.ai/.indd recognition + thumbnail (R2 upload)
  Why sixth: independent of other channels, can be parallelized with Phase 5.

[Phase 7: AI Brief + Daily Push]
  29. Briefer module (collects today's events from all channels)
  30. macOS notification at 8am via tokio scheduled task
  31. Click-handler routes to dashboard
  32. Insight generation (project annotations, NOT scoring)
  Why seventh: needs all channels feeding data to be useful.

[Phase 8: Polish + Distribution]
  33. Onboarding wizard (5 steps)
  34. Code signing + notarization
  35. Sparkle auto-update + appcast
  36. Notion webhooks ENABLED (was polling-only) — adds real-time
  37. Settings UI for cadence, filters, share token management
  Why last: distribution polish + the Notion webhook upgrade is purely
    additive over the polling baseline (defense in depth).
```

**Critical dependencies:**
- Phase 2 before any channel — every channel writes to Notion.
- Phase 3 (sanitizer) before any channel — type safety prevents leaks.
- Phase 4-6 channels can be parallelized after Phase 3.
- AI module appears in Phase 4 first; centralized so caching works in Phase 5+.

---

## Scaling Considerations

This is a single-user desktop app. "Scale" means **resource consumption on the user's Mac** and **CF Worker free-tier limits**.

| Scale | Architecture Adjustments |
|-------|--------------------------|
| 1 user, 5 channels, 100 msg/day | Default everything. Free tier all the way. |
| 1 user, 10 projects, 1000 msg/day | Increase Slack/Gmail batch size; add SQLite indexes on (project_id, created_at). |
| Multi-user (v2, hypothetical) | Move CF KV → Workers D1 per-user; add tenancy to SharedView keys. |

### Scaling Priorities

1. **First bottleneck: Notion API rate limits** (3 req/sec average). Mitigation: batch Communication Log appends (collect for 30s, then one PATCH); always poll incremental via `last_edited_time` filter.
2. **Second bottleneck: Anthropic cost/latency.** Mitigation: prompt caching on system prompt + project list (set cache_control on the list); use Haiku 4.5 for classification, Sonnet 4.6 only for drafting and brief.
3. **Third bottleneck: SQLite WAL checkpoint pauses** if the DB gets large. Mitigation: nightly VACUUM; separate large blob storage (attachment metadata only in DB, files on disk).
4. **CF KV writes** are 1000/day free. With 30s debounce, even 100 share updates/day per project × 10 projects = 1000. Acceptable; monitor.

---

## Anti-Patterns

### Anti-Pattern 1: Channel-Local AI Calls

**What people do:** Each channel module imports the AI client and makes its own classification calls.
**Why it's wrong:** Anthropic prompt caching keys on prefix. Different channels with different system prompts = no cache hits = 3-5x cost.
**Do this instead:** All AI calls go through `ai/classifier.rs` and `ai/drafter.rs` with one canonical system prompt. Channels submit `ClassifyRequest { channel, raw_text, hints }` and receive `Classification`.

### Anti-Pattern 2: Tauri Command Doing the Work Synchronously

**What people do:** `#[tauri::command] async fn ingest_slack() { /* 30s of work */ }` blocks the IPC reply.
**Why it's wrong:** Frontend hangs; if user closes window mid-operation, work is dropped.
**Do this instead:** Commands enqueue jobs to the orchestrator and return immediately with a job_id. Use `app.emit("job_progress", ...)` to push updates to frontend.

### Anti-Pattern 3: Storing Message Bodies in CF KV "Just in Case"

**What people do:** Cache full message bodies in KV "to make share pages richer."
**Why it's wrong:** Violates the explicit privacy invariant. Once in KV, exfiltration risk forever.
**Do this instead:** `SharedView` type is the only thing that crosses the share boundary. Compiler enforces.

### Anti-Pattern 4: Polling Notion with `databases.query` Every Minute Without Filter

**What people do:** `POST /v1/databases/{id}/query` with no filter every 60s.
**Why it's wrong:** Burns rate limit, returns full dataset every time.
**Do this instead:** Filter by `last_edited_time > {cursor}`. Cursor is stored in `sync_cursors` table. Update cursor only after successful local commit.

### Anti-Pattern 5: One sqlx Pool, Many Writers

**What people do:** `Pool::connect(...)?.max_connections(10)` and let everything write through it.
**Why it's wrong:** SQLite WAL serializes writers anyway; pool just hides "database is locked" errors as long mysterious waits.
**Do this instead:** Single writer connection behind `Mutex`, separate read pool of 4. (See Pattern 3.)

### Anti-Pattern 6: Using launchd `StartCalendarInterval` for the 8am Brief

**What people do:** Configure plist with `<key>StartCalendarInterval</key>... Hour=8`.
**Why it's wrong:** If the Mac slept through 8am, launchd fires the job once on wake — but launchd will then launch a *new* process even if the app is already running, leading to duplicate notifications. Also, the brief needs the running app's in-memory state.
**Do this instead:** launchd `RunAtLoad=true` to start app at login; the running app schedules its own daily 8am tokio task and reschedules on wake.

---

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| **Slack Events** | Socket Mode WebSocket (primary) + CF Worker webhook (fallback when offline) | De-dupe by `event_ts`. Socket Mode requires app-level token (`xapp-`). |
| **Slack Web API** | HTTPS direct from app (chat.postMessage, files.upload) | Bot token; rate limit Tier 3 ~50/min. |
| **Gmail** | Polling `users.history.list?startHistoryId=...` every 60s | Pub/Sub explicitly NOT recommended for desktop apps per Google docs. |
| **Google Calendar** | Polling `events.list?syncToken=...` every 90s | Push channels need HTTPS+cert+renewal; not worth it for desktop. |
| **Notion API** | Webhooks via CF Worker bridge (Phase 8) + polling fallback (Phase 2+) | Webhook payloads HMAC-SHA256 signed with `verification_token`. 2025-09-03 API uses `data_source_id`. |
| **Anthropic** | HTTPS direct from app, prompt caching enabled | All system prompts identical across calls; only user content varies. |
| **macOS Keychain** | `keyring` crate (security-framework backend) | Service name `com.user.designer-dashboard`; key per channel. |
| **macOS FSEvents** | `notify` crate `RecommendedWatcher` | Watches `~/Documents/KakaoTalk/` (configurable); debounce 500ms. |
| **Cloudflare KV** | HTTPS PUT with HMAC auth from app to bridge worker | App writes snapshots; CF Worker writes inbox events. Two namespaces. |
| **Cloudflare R2** | HTTPS PUT for thumbnails (Adobe file previews) | One key per attachment; URL embedded in SharedView. |
| **Sparkle** | Appcast.xml hosted on CF Pages | EdDSA signature; user can disable in settings. |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| Frontend ↔ Rust | Tauri IPC (TauRPC for typed) + `emit`/`listen` events | Commands return immediately; long ops use job_id + events. |
| `channels/*` → `orchestrator/event_bus` | tokio mpsc, buffered=1024 | One sender per channel; one consumer (reconciler). |
| `sync/reconciler` → `data/` | Direct calls (same crate) via single-writer mutex | All writes go through here. |
| `sync/reconciler` → `ai/` | Direct calls; classifier blocks reconciler briefly | OK because reconciler is single-task. |
| `data/` → `share/publisher` | "Project changed" event via mpsc; publisher debounces 30s | Decouples ingest spikes from CF KV writes. |
| `share/publisher` → CF Worker | HTTPS POST with HMAC | Stateless retry on 5xx; backoff 1s/4s/16s. |
| `secrets/keychain` | Direct calls; never serialized to disk | Tokens never enter SQLite. |
| App ↔ CF Worker bridge | HTTPS with shared HMAC secret (in Keychain) | Long-poll `/inbox/drain`; ack with ULID list. |

---

## Sources

### Notion
- [Notion Webhooks Reference](https://developers.notion.com/reference/webhooks) — webhooks require public HTTPS endpoint (HIGH)
- [Notion API 2025-09-03 Upgrade Guide](https://developers.notion.com/docs/upgrade-guide-2025-09-03) — `data_source_id`, multi-source databases (HIGH)
- [Notion Webhooks Complete Guide 2025](https://dev.to/robbiecahill/notion-webhooks-a-complete-guide-for-developers-2025-hop) — HMAC-SHA256 with verification_token (MEDIUM)
- [Notion API Rate Limits 2026](https://fazm.ai/blog/notion-api-rate-limits-2026) — 3 req/sec average (MEDIUM)

### Slack
- [Comparing HTTP & Socket Mode](https://docs.slack.dev/apis/events-api/comparing-http-socket-mode/) — official guidance (HIGH)
- [Socket Mode Overview](https://api.slack.com/apis/socket-mode) — WebSocket, no public URL needed (HIGH)
- [Building a serverless Slack bot using Cloudflare Workers](https://blog.cloudflare.com/building-a-serverless-slack-bot-using-cloudflare-workers/) — webhook proxy pattern (HIGH)
- [slack-morphism-rust](https://github.com/abdolence/slack-morphism-rust) — Rust Socket Mode client (HIGH)
- [Running Slack App on Cloudflare Workers](https://dev.to/seratch/running-slack-app-on-cloudflare-workers-3hhn) — verifyRequestSignature pattern (MEDIUM)

### Google APIs
- [Configure push notifications in Gmail API](https://developers.google.com/workspace/gmail/api/guides/push) — Google explicitly recommends polling for desktop/mobile (HIGH)
- [Google Calendar Push Notifications](https://developers.google.com/workspace/calendar/api/guides/push) — HTTPS callback with valid SSL required (HIGH)

### Tauri / Rust
- [Tauri 2 IPC](https://v2.tauri.app/concept/inter-process-communication/) — JSON-RPC-like, async commands (HIGH)
- [Tauri + Async Rust Process](https://rfdonnelly.github.io/posts/tauri-async-rust-process/) — tokio mpsc + spawn pattern (HIGH)
- [TauRPC](https://github.com/MatsDK/TauRPC) — typesafe IPC (MEDIUM)
- [notify-rs](https://github.com/notify-rs/notify) — FSEvents on macOS (HIGH)
- [tokio-tungstenite](https://github.com/snapview/tokio-tungstenite) — async WebSocket (HIGH)

### SQLite
- [PSA: Your SQLite Connection Pool Might Be Ruining Your Write Performance](https://emschwartz.me/psa-your-sqlite-connection-pool-might-be-ruining-your-write-performance/) — single-writer pattern (HIGH)
- [SQLite WAL Mode and Connection Strategies](https://dev.to/software_mvp-factory/sqlite-wal-mode-and-connection-strategies-for-high-throughput-mobile-apps-beyond-the-basics-eh0) — busy_timeout, max_connections (MEDIUM)
- [SqliteConnectOptions in sqlx](https://docs.rs/sqlx/latest/sqlx/sqlite/struct.SqliteConnectOptions.html) — journal_mode(Wal), busy_timeout (HIGH)

### Cloudflare
- [Cloudflare Workers KV docs](https://developers.cloudflare.com/kv/) — eventual consistency 60s, free tier 1000 writes/day (HIGH)
- [Use WebSockets with Durable Objects](https://developers.cloudflare.com/durable-objects/best-practices/websockets/) — alternative if real-time bridge needed (HIGH)
- [Cloudflare Workers KV How it Works](https://developers.cloudflare.com/kv/concepts/how-kv-works/) — read-heavy optimization (HIGH)

### macOS launchd / Sleep
- [launchd.plist(5) man page](https://keith.github.io/xcode-man-pages/launchd.plist.5.html) — RunAtLoad, StartCalendarInterval semantics (HIGH)
- [A launchd Tutorial](https://www.launchd.info/) — sleep/wake behavior, fires-on-wake semantics (HIGH)
- [NSWorkspace didWakeNotification](https://developer.apple.com/documentation/appkit/nsworkspace/didwakenotification) — official wake event (HIGH)
- [Detect macOS sleep/wake events](https://medium.com/@clyapp/using-swift-to-detect-osx-system-events-sleep-wakeup-lock-unlock-screensaver-display-change-529cae9a3e23) — observer pattern (MEDIUM)

---

*Architecture research for: macOS menu-bar productivity app with multi-channel ingestion*
*Researched: 2026-05-03*
