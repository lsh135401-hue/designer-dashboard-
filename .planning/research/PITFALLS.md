# Pitfalls Research

**Domain:** macOS menu bar productivity app integrating Slack/Gmail/Google Calendar/Notion/KakaoTalk(.txt) with Anthropic AI for a Korean manufacturing-design workflow
**Researched:** 2026-05-03
**Confidence:** HIGH for OAuth/Slack/Notion/Gmail/Tauri/Anthropic (verified via official docs + community post-mortems); MEDIUM for KakaoTalk (community projects only, no official spec); MEDIUM for manufacturing-domain (industry articles, no domain-specific app post-mortems)

This document maps to the 8 phases in `PROJECT.md`:
- **P0** Foundation (menu bar shell, Keychain, SQLite, OAuth scaffolding)
- **P1** Notion SOTR (schema wizard, two-way sync)
- **P2** Calendar + Gmail (read paths)
- **P3** Slack + Adobe asset ingest
- **P4** KakaoTalk watch folder
- **P5** AI Brief + dashboard UX
- **P6** Send (drafts, approval gate, Communication Log)
- **P7** Share (Cloudflare Workers + KV/R2, sanitized snapshots)
- **P8** Distribution (notarization, Sparkle)

---

## Critical Pitfalls

### Pitfall 1: Notion API 2025-09-03 — `database_id` vs `data_source_id` confusion

**What goes wrong:**
Code that looks correct against Notion docs fails with "Could not find database with ID" or silently writes to the wrong table. The 2025-09-03 API release introduced multi-source databases: a "database" is now a *container* of one or more "data sources." Querying with `database_id` against a multi-source DB or against `POST /v1/databases/{id}/query` (the old endpoint) breaks. New code path is `POST /v1/data_sources/{data_source_id}/query`, and page creation requires `parent: { type: "data_source_id", data_source_id: "..." }`.

**Why it happens:**
- Tutorials, Stack Overflow answers, and most LLM training data still use `database_id`.
- Notion URLs expose database_id in the path — devs grab that, never realize they need a discovery step to fetch the data_source_id.
- The legacy endpoint still works for *single-source* DBs, masking the bug until a user adds a second source or until Notion fully sunsets the old path.

**How to avoid:**
1. On every database reference, do a discovery step: `GET /v1/databases/{database_id}` → read `data_sources[].id` → cache the data_source_id in our SQLite alongside the database_id.
2. Pin `Notion-Version: 2025-09-03` (or newer) header on all requests; never let it auto-default.
3. Wrap the Notion client so callers cannot pass a raw "id" — force them to pass a typed `DataSourceRef` resolved via discovery.
4. Schema wizard (Phase 1) MUST refresh data_source_id on every run, not cache it forever.

**Warning signs:**
- 404 with "Could not find database with ID" when the URL clearly contains that ID.
- Pages created successfully but appearing in a different view than expected.
- Properties being silently dropped on create (data source schema mismatch).

**Phase to address:** **P1 (Notion SOTR)** — bake discovery into the schema wizard from day one.

---

### Pitfall 2: Notion as SOTR — soft-deleted, archived, and trashed pages return inconsistently

**What goes wrong:**
A user moves a project page to Notion's trash. Our local SQLite still has it. The next sync either (a) recreates it on Notion (we treat it as a new local insert) or (b) sees `archived: true` and treats it as a normal page, showing it in our dashboard alongside live projects. Worse: the user "restores" it from trash but our cache still flags it as deleted, so we hide a live project.

**Why it happens:**
- Notion has *three* states that look similar in API responses: `archived: true` (page archived), `in_trash: true` (database soft-deleted, 30-day window), and hard-deleted (404). Older code treats only `archived` as the deletion signal.
- The `archived` flag is settable via API; `in_trash` requires the new Update Data Source endpoint.
- Tombstones in our local SQLite need to know *which* deletion happened to reverse correctly.

**How to avoid:**
1. Treat both `archived: true` AND `in_trash: true` as "hidden" but keep the row in SQLite with a `notion_state` enum (`active|archived|trashed|gone`).
2. Never create-on-write if a Notion page-id exists in SQLite with state ≠ `gone`. If user edits a soft-deleted project locally, surface a "this project is in Notion trash, restore?" prompt.
3. Hard-delete only after a 404 *and* a confirmed full-resync that doesn't return the page.
4. Run a daily reconciliation pass: for each local project, GET the page; update `notion_state` from response.

**Warning signs:**
- "Ghost projects" appearing on dashboard that the user already deleted.
- Duplicate projects after a user restores a trashed page.
- Notion shows a project but our dashboard doesn't (state cache stale).

**Phase to address:** **P1 (Notion SOTR)** — define `notion_state` schema and reconciliation worker before two-way sync ships.

---

### Pitfall 3: Notion API rate limits → 429 storm during initial bulk ingest

**What goes wrong:**
First-run ingest of N existing projects (each with milestones, tasks, file blocks) hammers Notion at >3 req/s. Notion returns 429s, our naive retry loop spins, the ingest never finishes, the onboarding wizard hangs at "Importing your projects…" and the user force-quits.

**Why it happens:**
- Notion's published rate limit is "an average of 3 requests per second" with bursts allowed.
- Page-block trees require 1 request per page + 1 per block-children fetch (paginated 100 at a time). 50 projects with 20 blocks each = 1000+ requests.
- Without `Retry-After` honoring + exponential backoff + concurrency cap, you can cascade.

**How to avoid:**
1. Single global Notion request limiter at 2.5 req/s (leave headroom). Use a token-bucket in Rust; no per-task rate limiting (because every task adds up).
2. Always honor `Retry-After` header on 429; default to exponential backoff (500ms, 1s, 2s, 4s, max 30s) when header missing.
3. For initial import, queue requests and show a real progress bar with ETA. Never spin a tight retry loop without backoff.
4. Use prompt caching's logical equivalent here: cache page-block fetches in SQLite keyed by page-id + `last_edited_time`; only re-fetch if `last_edited_time` changed.

**Warning signs:**
- Onboarding wizard appearing to hang for >30s.
- Logs showing repeated 429s with no Retry-After honored.
- Notion `last_edited_by` showing our integration spamming pages with no-op updates.

**Phase to address:** **P0 (Foundation)** for the rate-limiter primitive; **P1 (Notion SOTR)** for the ingest UX.

---

### Pitfall 4: Slack Events API — sub-3s ACK requirement violated → retry storms duplicate messages 3×

**What goes wrong:**
Our Slack event handler does the AI classification + Notion write + SQLite insert *before* returning HTTP 200. It takes 4 seconds. Slack flags it as failed, retries at ~immediately, +1min, +5min — all three retries happen because each retry also takes >3s. Now we have 3-4 copies of the same Slack message in Notion's Communication Log, AI classification ran 3-4 times (cost), and our logs are flooded.

**Why it happens:**
- Slack's contract is **strict**: HTTP 200 within 3 seconds or retry. Most devs find this only after seeing duplicates in production.
- "Make it async" sounds easy but requires a real queue (in-process channel + worker, or persisted queue if we want survival across restarts).
- The retry includes `x-slack-retry-num` and `x-slack-retry-reason` headers — cheap to detect — but easy to forget to check.

**How to avoid:**
1. Slack webhook handler returns 200 IMMEDIATELY after enqueueing the raw event payload to a Tokio channel + persisting the event_id to SQLite.
2. Worker thread pulls from the channel, runs AI/Notion/SQLite work.
3. Deduplication: before processing, check `event_id` (or `event.client_msg_id` for messages, or `event.ts` as fallback) against a `processed_events` table with a unique constraint. Skip if seen.
4. On `x-slack-retry-num >= 1` header, log a warning so we know our handler is too slow.
5. Use Slack Socket Mode (no HTTP webhook) since this is a local app — avoids requiring a public URL entirely. Socket mode still has duplicate delivery issues per slackapi/python-slack-events-api#93, so dedup is still required.

**Warning signs:**
- Same Slack message classified into Notion 2-4 times.
- Cost spike on Anthropic API correlating with active Slack channels.
- `x-slack-retry-num` headers showing up in logs.

**Phase to address:** **P3 (Slack)** — design for async handling from the first commit; never let a synchronous AI call block ack.

---

### Pitfall 5: Slack `message_changed` / `message_deleted` events ignored → stale data in Notion

**What goes wrong:**
A user edits or deletes a Slack message after we've already classified it. Our Communication Log in Notion keeps showing the original wrong text. Worse for `message_deleted`: a confidential message the user wanted to scrub still lives in our local SQLite + Notion.

**Why it happens:**
- These events have `subtype: "message_changed"` or `"message_deleted"` and look syntactically different from regular messages — many parsers filter on `event.type === "message"` and ignore subtypes.
- `message_changed` includes `previous_message` — easy to miss that the *new* state is in `event.message`, not the top-level event.
- `message_deleted` includes only `deleted_ts` and `channel`, so you must look up your stored record by ts.

**How to avoid:**
1. Handle subtypes explicitly: `message`, `message_changed`, `message_deleted`, `bot_message` (which we likely want to filter out — our own future Send messages echo back).
2. Store every Slack message in SQLite keyed by `(channel_id, ts)` so edits/deletes can find the local row.
3. On edit: update local row, update Notion Communication Log entry, optionally re-run classification if text changed substantially.
4. On delete: delete local row, delete Notion entry (or mark `redacted: true` to preserve audit trail per user preference). Default to redact, not delete, since this is an audit log.

**Warning signs:**
- User reports "I deleted that message but it's still in Notion."
- Communication Log entries that don't match the live Slack thread.

**Phase to address:** **P3 (Slack)** — wire all message subtypes in the same PR as initial Slack ingest.

---

### Pitfall 6: OAuth refresh token rotation race conditions — Slack 12h expiry, Notion single-use refresh tokens

**What goes wrong:**
Two things refresh the same token concurrently — e.g., a background sync worker and a foreground "refresh now" button. Both POST to the token endpoint with the current refresh_token. Provider returns a *new* refresh_token to whichever request lands first; the second request gets `invalid_grant` because its refresh_token was just rotated. We store the second response's error result, overwriting the valid new token in Keychain. Now the integration is bricked — no refresh_token works, user has to re-authorize from scratch.

**Why it happens:**
- Slack tokens with rotation expire every 12 hours. Notion rotates refresh tokens on every refresh. Google Cloud apps in "Testing" mode invalidate refresh tokens after 7 days — easy to forget when shipping.
- Many OAuth client libraries lack mutex around the refresh call.
- "Last write wins" on Keychain storage means a failed refresh response can clobber a successful one if ordering goes wrong.

**How to avoid:**
1. Single-flight pattern: a global mutex per provider. Only one refresh in flight at a time; other callers await the same future and receive the same result.
2. Compare-and-swap on Keychain write: only overwrite if the in-Keychain refresh_token matches the one we used. If it changed, our refresh attempt is stale — discard our result.
3. **Move Google OAuth app to "Production" status before launch** to avoid the 7-day testing-mode expiry trap. Document this as a launch checklist item.
4. For Slack User Tokens (`xoxp-`), opt into token rotation explicitly; don't rely on legacy non-expiring tokens (Slack is deprecating these).
5. On `invalid_grant`, do NOT retry — surface a re-auth prompt immediately. Retrying with an invalid token can lock account-level rate limits.

**Warning signs:**
- Sporadic `invalid_grant` errors with no clear pattern.
- Users reporting "I have to re-login to [Slack/Notion/Google] every few days."
- Multiple concurrent token-endpoint calls in logs within milliseconds of each other.

**Phase to address:** **P0 (Foundation)** — the Keychain + OAuth refresh primitive must be single-flighted from day one. Adding mutex later requires changing every integration call site.

---

### Pitfall 7: Gmail API quota — 250 quota-units/user/sec exceeded by parallel attachment downloads

**What goes wrong:**
We do an initial email backfill: list inbox, then for each message fetch full payload + each attachment in parallel. Within 30 seconds we exceed 250 quota units/user/sec (a `messages.get` is 5 units; a `messages.attachments.get` is 5 units; we run 60+ in parallel). 429s start; our retry doesn't honor `Retry-After`; backfill grinds to a halt. Worse: the project-level quota (1,000,000,000 units/day) seems infinite, so devs assume there's no limit.

**Why it happens:**
- Per-user limit is 250 units/sec/user — it's the *real* limit for a personal app like ours. The generous project quota lulls devs into ignoring it.
- Gmail API quota costs vary wildly per method: `messages.list` is 5, `messages.get` is 5, `messages.send` is 100, `users.history.list` is 2.
- Batch HTTP requests don't reduce quota cost — they only reduce HTTP overhead. Each sub-request still counts.

**How to avoid:**
1. Token-bucket rate limiter at 200 units/sec (headroom). Each method's cost is encoded in our wrapper; the limiter consumes the right number of tokens before issuing the call.
2. Use `users.history.list` (2 units) for incremental sync after the initial backfill — far cheaper than re-listing.
3. Use Gmail Push notifications (Pub/Sub `users.watch`) for near-real-time updates without polling. Watch must be re-armed every 7 days or it stops silently.
4. For attachment downloads: only fetch when the user actually opens the message in our UI (lazy). Don't bulk-prefetch.
5. Honor `Retry-After` header always.

**Warning signs:**
- Backfill stuck at "Importing emails… 1,234 of N" for minutes with no progress.
- 429 with "Quota exceeded for quota metric 'Queries'" in error response.
- After 7 days of running, Gmail push notifications silently stop arriving.

**Phase to address:** **P2 (Gmail)** — quota-aware client + watch re-arming cron from the start.

---

### Pitfall 8: Gmail label vs thread classification — same email categorized 3 ways

**What goes wrong:**
A Gmail thread has 5 messages; user labeled the *thread* "Project Acme" via Gmail UI. We pull message-level labels and find: messages 1, 3, 5 have the label (incoming), messages 2, 4 don't (outgoing replies). Our classifier sees inconsistent labels and either splits the thread across projects or skips it.

A second case: Gmail's `users.history.list` returns `messagesAdded` for new messages — but per the gmailpush issue, sometimes new messages don't appear in `messagesAdded`, only in the broader `messages` array.

**Why it happens:**
- Gmail labels are message-scoped, not thread-scoped, even though the UI shows them at thread level (it shows "the union of all message labels").
- The History API has well-documented inconsistencies between `messagesAdded`, `labelsAdded`, and the `messages` ID list.
- Threads can span multiple labels (project A and project B) when forwarded.

**How to avoid:**
1. Classify at the **thread** level, not message level. Aggregate all message labels in a thread; if all messages share a unique project label, classify into that project; otherwise fall back to AI.
2. When processing history records, dedupe by `threadId`, take the first message, then `messages.get` the full thread to see current state.
3. Document the assumption "1 thread = 1 project" with the user in onboarding; surface ambiguous threads in a "needs review" inbox rather than auto-misclassifying.
4. Re-fetch via `messages.get(format=metadata)` on every history event for ground truth — don't trust history payload alone.

**Warning signs:**
- Same email thread appearing under two different projects in our dashboard.
- Email replies missing from a thread we ingested.
- User complaints about "wrong project" tags.

**Phase to address:** **P2 (Gmail)** — settle on thread-level classification before AI is wired in.

---

### Pitfall 9: KakaoTalk .txt parsing — three locale variants + encoding edge cases

**What goes wrong:**
We write a parser against the Korean KakaoTalk PC export format (`2020년 8월 10일 월요일`, message lines like `[홍길동] [오후 3:21] 안녕하세요`). It works in dev. A user with English KakaoTalk Desktop exports a file: dates are `August 10, 2020 Monday`, message lines are `[John] [3:21 PM] Hello`. Parser silently produces zero messages. A second user has a UTF-8 BOM on their export from an older KakaoTalk version: first line fails regex match because of the invisible `U+FEFF` prefix. A third user pastes a Mac KakaoTalk export that uses `오전/오후` plus 24-hour times mixed.

**Why it happens:**
- KakaoTalk has *no published* export-format spec. Format varies by:
  - KakaoTalk client locale (한국어/English/中文/日本語)
  - Platform (PC Mac/PC Windows/Mobile email-export)
  - Version (BOM presence varies in older exports per kakaotalk-analyzer findings)
- "Saved to .txt as UTF-8" sounds safe but BOM, CRLF vs LF line endings, and date-divider format all vary.
- Attachment references appear as `<사진>`, `<Photo>`, `<File: filename.pdf>` — no canonical format; the file itself is *not* in the .txt.

**How to avoid:**
1. **Locale-detection first**: scan the first 50 lines for date-divider patterns (`\d+년 \d+월 \d+일`, `\w+ \d+, \d+`, `\d+年\d+月\d+日`). Tag the file with a detected locale before parsing.
2. Maintain a `Parser` per locale; reject files we can't classify with a clear "Unsupported KakaoTalk export format. Please send us this file (anonymized) so we can add support."
3. Always strip UTF-8 BOM (`U+FEFF`) on read. Always normalize CRLF → LF.
4. Treat attachments as opaque markers: extract the bracketed token, store `attachment_placeholder: true`, do NOT try to follow the reference (the file isn't there).
5. **Idempotency**: hash each parsed message `(timestamp, sender, body)` → SHA256 → unique constraint in SQLite. Re-importing the same .txt or an overlapping export window must produce zero duplicates.
6. Build a corpus of test fixtures from at least: KR PC, KR Mobile, EN PC, ZH PC. Add fixtures from beta users.
7. Time ambiguity: `오후 3:21` → 15:21; `오전 12:30` → 00:30 (Korean uses 12-hour with 오전/오후, but the convention for midnight differs from English AM/PM). Hard-code these correctly with explicit unit tests.

**Warning signs:**
- A user drops a .txt and zero messages get ingested with no error shown.
- All messages from one user import as the same date (date-divider regex failure).
- Messages dated 12:00 AM/PM appearing at noon vs midnight randomly.
- Same conversation appears twice after a re-export.

**Phase to address:** **P4 (KakaoTalk)** — locale detection + idempotency hash are non-negotiable in the first cut.

---

### Pitfall 10: KakaoTalk watch folder — file-still-being-written triggers premature parse

**What goes wrong:**
User exports a 50MB chat history. macOS FSEvents fires `Created` the moment the file appears, while KakaoTalk Desktop is still writing. Our watcher reads the file → sees a half-written file ending mid-line → parser errors or worse, parses partial content as final and ingests truncated messages. On the user's next watch trigger (file modified when write completes), we re-parse the full file → we now have partial messages from the first read PLUS full messages from the second.

**Why it happens:**
- FSEvents (and notify-rs / Tauri's plugin-fs-watch) fire on file creation, not on file-write-complete. There's no atomic "file is ready" event on macOS.
- KakaoTalk Desktop writes large exports as a single open-write-close, no .tmp rename.
- Naive watchers debounce by milliseconds; large files take seconds.

**How to avoid:**
1. Debounce file events by stable-size: on event, poll file size every 500ms; only process when size is unchanged for ≥2 seconds.
2. Compute a content hash *and* a size on first read; refuse to ingest if size changes between hash and ingest.
3. Always idempotency-hash messages (per Pitfall 9 #5) so a re-parse is a no-op.
4. Keep a `processed_files` table keyed by `(path, sha256, size)`; skip re-processing identical files.
5. Show the user a "Processing… (X of Y messages)" toast so a delay is explained, not perceived as a hang.

**Warning signs:**
- User reports "some messages from my export are missing."
- Parse errors in logs immediately followed by a successful parse seconds later.
- The same chat window's messages appearing in fragments across two ingest events.

**Phase to address:** **P4 (KakaoTalk)** — debounce-by-size before any parsing logic ships.

---

### Pitfall 11: AI hallucination in instruction drafts — wrong factory, wrong sample, wrong project

**What goes wrong:**
The AI draft says "Hi 김 사장님 (Acme Factory), please confirm sample #2 of Project A is shipped by Friday." But the actual sample-2 of Project A was assigned to *Beta Factory*, and 김 사장님 is the Acme contact for *Project C*. The user, scanning quickly at 8 AM, sends it. Acme Factory now thinks they owe a sample they don't, and confidential info about Project A leaked to the wrong vendor.

**Why it happens:**
- LLMs confabulate plausible-sounding entity associations, especially when context is large and contains multiple similarly-structured projects.
- Korean honorifics + factory naming (the same `사장님` title is used for many people) make wrong-name errors invisible to a fast read.
- Without strong grounding, the model "fills in the blank" rather than refusing.
- Stanford-cited research shows RAG + guardrails can cut hallucination ~96% vs baseline — but only if both are wired correctly.

**How to avoid:**
1. **Strict structured output**: AI returns JSON with `recipient_id`, `project_id`, `milestone_id`, `sample_id` — all referencing IDs that MUST exist in our SQLite. Validation layer rejects any draft referencing a non-existent ID and asks AI to retry with a constrained prompt.
2. **Source attribution on every claim**: each draft includes a `cited_facts` array listing the SQLite rows the draft is based on. UI renders these as expandable chips next to the draft so user can verify before send.
3. **Recipient confirmation gate**: before send, UI shows "Sending to: 김 사장님 <kim@acme.kr> via Slack DM. ✓ This is Acme Factory's contact for Project A." User must click confirm. The recipient resolution is computed by deterministic code, not by AI.
4. **Anti-injection**: refuse to send messages where AI-generated body contains email addresses, phone numbers, or URLs not present in our SQLite contact list.
5. **Tone & content review**: AI generates body; deterministic code attaches recipient + subject. Never let AI populate the `to:` field.
6. Log every draft + final-sent text + diff in Communication Log; surface "AI changed N facts you sent" as a weekly review metric.

**Warning signs:**
- Drafts mentioning factory names that don't appear in our contacts list.
- Drafts referencing sample numbers that don't exist on the project.
- User makes the same correction repeatedly (recipient swaps, name fixes).

**Phase to address:** **P6 (Send)** — structured output + ID-validation layer is the cornerstone of the entire send flow. Do NOT ship send without it.

---

### Pitfall 12: Cost runaway — AI re-classifies the same message in a loop

**What goes wrong:**
A Slack message arrives, gets AI-classified, ingested. A `message_changed` event fires (user added a reaction — Slack delivers this as `message_changed` with `subtype: "reaction_added"` in some flows). We re-classify, write back to Notion. Notion's webhook (or our polling) sees the update, our reconciliation loop reads it back, somehow flags it as "needs re-classification," AI runs again. Now it's running every 30 seconds in a feedback loop. Within a week, $50 of Anthropic spend on what should be a $10/month app.

A second mode: a user adds 10 projects in Notion at once, our sync worker spawns 10 concurrent AI calls without rate limiting, hits Anthropic's RPM limit, gets 429, retries naively, hits limit again, the retry storm uses tokens on every failed attempt. (Anthropic billing counts errored requests for some account types per Anthropic billing guide.)

**Why it happens:**
- Loops between "data changes → AI runs → data changes" are the classic distributed-system pitfall.
- No spend ceiling enforced locally — bills arrive monthly, by which point damage is done.
- Prompt caching default TTL was nerfed from 1h to 5min in early April 2026 — costs that looked cheap in dev no longer match production.

**How to avoid:**
1. **Hard daily budget in code**: read `ai_spend_today_usd` from SQLite before every AI call. If > user-configured ceiling (default $1.00/day), refuse to call and surface a notification. User can raise the ceiling explicitly.
2. **Idempotency keys for AI calls**: hash `(model, system_prompt, user_message)` → if seen in last 7 days, return cached result. Never re-run identical prompts.
3. **Loop detection**: each item (message, page, draft) has a `last_ai_run_at` timestamp + `ai_run_count` in SQLite. If run_count > 3 in 24h on the same item, alert + freeze AI for that item.
4. **Concurrency cap**: max 2 concurrent Anthropic calls globally. Token-bucket at 50% of our actual rate limit.
5. **Use Haiku 4.5 for classification, Sonnet 4.6 only for drafts**: per PROJECT.md decision. Enforce in code — classification path can never call Sonnet.
6. **Prompt caching**: structure system prompts so the static portion (project list, schemas, style guide) is the cacheable prefix. Verify cache hit rate >70% in production telemetry; alert if it drops.
7. **Don't auto-trigger AI on Notion writes WE made**: tag every Notion update with `last_edited_by: <our integration>` and skip those in our reconciliation loop. (This is the loop-breaker.)

**Warning signs:**
- Daily AI spend > 2× expected.
- `ai_run_count` for any single item climbing past 5.
- Anthropic 429s appearing in logs.
- Cache hit rate <50%.

**Phase to address:** **P5 (AI Brief)** for the budget primitive; **P3/P4** must NOT call AI without going through this primitive. Add it before any AI integration.

---

### Pitfall 13: Privacy leak — message bodies sent to Anthropic without consent OR included in share-URL snapshots

**What goes wrong:**
Two failure modes:
- **AI exfiltration**: AI classification needs the message body to work. By default we send the whole body. User assumed "data stays local" because PROJECT.md says so. They notice the network egress, lose trust, uninstall.
- **Share-URL leak**: sanitization function meant to strip message bodies has a bug — it strips `body` field but leaves `preview` (a 100-char snippet we cached for the dashboard). Confidential design specs leak to a public URL.

**Why it happens:**
- "Privacy" promises in marketing copy are easy; enforcement in code requires deliberate gates.
- Sanitization for outbound data should be a single chokepoint, not sprinkled across N call sites.
- Anthropic's no-training default is real but doesn't change the fact that bodies leave the device.

**How to avoid:**
1. **Explicit consent UI on first run**: "AI features need to send message text to Anthropic for classification & drafts. Anthropic does not train on this data. [Allow / Disable AI features]." Persist choice; never call AI without consent flag = true.
2. **Per-message redaction option**: user can mark a Slack channel or Notion DB as "AI-off" — those bodies never leave the device.
3. **Sanitization chokepoint**: a single Rust function `sanitize_for_share(project) -> SharedSnapshot` with a typed return that contains ONLY allowed fields (progress %, milestone names, dates, next-step text written by user). NO message bodies, NO email subjects, NO file contents. Unit tests assert that fields like `body`, `preview`, `excerpt`, `snippet`, `email_subject` don't exist on `SharedSnapshot`.
4. **Outbound HTTPS chokepoint**: Tauri `allowlist` configured to permit only Anthropic + Notion + Slack + Google + Cloudflare + Sparkle update URL. Block everything else. Surface a request-log UI "View what's been sent."
5. **Network test in CI**: spin up a mock HTTPS server, run a synthetic share-snapshot, assert no message-body strings appear in the request body.

**Warning signs:**
- Network egress to non-allowlisted hosts.
- Share-URL snapshot containing strings that look like message content (test: search for known sentinel strings).
- Anthropic API calls happening when AI features are disabled.

**Phase to address:** **P7 (Share)** for the sanitization chokepoint; **P5 (AI Brief)** for consent UI; both before public beta.

---

### Pitfall 14: macOS notarization rejection — Sparkle helpers, missing entitlements, deep signing

**What goes wrong:**
Build #1 ships. Notarization fails with "The signature of the binary is invalid" or "The executable does not have the hardened runtime enabled." Devs scramble; first attempted fix is `codesign --deep --force` which "works" — until users on macOS Sequoia get "App is damaged" because deep-signed XPC services have wrong nesting. Or: notarization passes, app launches, but auto-update via Sparkle fails silently because Autoupdate.app sub-bundle wasn't signed.

**Why it happens:**
- Hardened Runtime is required for notarization since macOS 10.14.5 — but it's not enabled by default in many build pipelines. Apps work in dev (no notarization) but fail at distribution.
- Sparkle's `--deep` codesign flag is explicitly documented as breaking XPC service signatures — but every Stack Overflow answer recommends `--deep`.
- Sparkle XPC services need exact bundle ID suffixes (`-spks`, `-spki`); other suffixes cause cryptic XPC errors.
- Notification Center / FSEvents permissions on macOS Sequoia require user prompts; without proper Info.plist usage descriptions, prompts don't even appear and silently fail.

**How to avoid:**
1. **From day one**: Hardened Runtime enabled in Tauri config (`bundle.macOS.hardenedRuntime = true`). Add explicit entitlements file with required exceptions only (no `com.apple.security.cs.allow-unsigned-executable-memory` unless absolutely needed — it's a notarization red flag).
2. **Sign in correct order**: helper bundles → frameworks → main app. Never use `--deep`. Use Tauri's signing pipeline, but verify with `codesign --verify --deep --strict --verbose=2` after build.
3. **For Sparkle**: use `-spks` and `-spki` suffixes for XPC bundles. Sign Autoupdate.app helper explicitly.
4. **Info.plist usage descriptions**: 
   - `NSUserNotificationsUsageDescription`: "매일 아침 브리프와 주요 알림을 보내기 위해 필요합니다." (currently no longer required for UNUserNotifications, but still good practice)
   - `NSAppleEventsUsageDescription` if we ever script other apps
   - For FSEvents on watched folders: no permission required for paths the user picked themselves via NSOpenPanel (sandbox or not), but Full Disk Access required if we watch `~/Library` or system paths. Make watch folder selection explicit so we never need FDA.
5. **Set up a notarization smoke test in CI**: every build runs `xcrun notarytool submit --wait`. PR cannot merge if notarization fails.
6. **Sparkle EdDSA**: generate the EdDSA key BEFORE first release. Once shipped, you can rotate but never remove. Store the private key in a CI secret store; the public key is bundled in the app.
7. **Build number monotonicity**: Sparkle compares `CFBundleVersion`, not `CFBundleShortVersionString`. CI must increment build number on every release.

**Warning signs:**
- Notarization fails with cryptic error codes.
- Sparkle update downloads but installation hangs.
- Users report "App is damaged" after install (unsigned helper).
- Notifications silently don't appear; permissions dialog never shown.

**Phase to address:** **P0 (Foundation)** for Hardened Runtime + entitlements baseline; **P8 (Distribution)** for Sparkle wiring. Notarization smoke test in CI before P8.

---

### Pitfall 15: Two-way sync race — same project edited locally and in Notion within the same minute

**What goes wrong:**
At 9:00:00, user edits the project's "next step" in our app. Our local SQLite updates immediately. Sync worker fires, POSTs to Notion, gets back updated `last_edited_time = 9:00:01`.

At 9:00:00 (concurrently), user is also editing the same project in Notion's web UI. They hit save at 9:00:02. Notion stores `last_edited_time = 9:00:02`.

Our sync worker's outbound write (9:00:01) succeeded BEFORE Notion's web edit (9:00:02). When we next poll, we see `last_edited_time = 9:00:02`, assume Notion is newer, overwrite our local SQLite with Notion's content. The user's local edit is lost — but they saw it succeed locally and have no idea it was reverted.

The reverse failure: clock skew between user's machine and Notion's server can cause our local timestamp to look "newer" than a Notion edit that actually happened later.

**Why it happens:**
- Last-Write-Wins with wall-clock timestamps is fundamentally racy. Even NTP-synced clocks drift by seconds.
- Notion's offline solution uses `lastDownloadedTimestamp` per page on the client + comparison to server's `lastUpdatedTime` — this works because Notion controls both client and server. We don't control Notion's server.
- Two-way sync is genuinely hard; "two one-way pipelines" is the wrong mental model.

**How to avoid:**
1. **Commit to Notion-as-SOTR semantics**: Notion always wins on conflict. Our local SQLite is a cache, not a source. Any local edit goes to Notion FIRST and only commits locally on Notion's 200 response (with the new `last_edited_time` from Notion).
2. **Optimistic locking via `last_edited_time`**: when we POST a local edit, include `If-Match: <last_edited_time we read>`. If Notion's current value differs, abort with conflict. (Notion API doesn't natively support If-Match; emulate by re-reading + comparing within the same logical operation, fail fast on mismatch.)
3. **Conflict UX**: when conflict detected, show a 3-way merge UI: "Your version | Notion's version | Pick one or merge." Never silently overwrite either side.
4. **Per-field granularity**: track `last_edited_time` per *property*, not per page, where Notion exposes it. Allows merging non-overlapping edits automatically.
5. **No background sync of dirty local changes — flush local edits to Notion synchronously** before reading anything else from Notion. Avoids the race entirely.
6. **Conflict log**: every conflict resolution decision is logged to SQLite; user can review weekly to see if our resolution is matching their intent.

**Warning signs:**
- User reports "I edited X in the app and then later it changed back."
- Conflict log entries climbing.
- Our `last_edited_time` cache showing values in the future (clock skew on user's machine).

**Phase to address:** **P1 (Notion SOTR)** — write-then-read pattern + optimistic-lock emulation must be the default sync primitive. Don't add two-way sync without conflict UX.

---

### Pitfall 16: Manufacturing-domain — confidential design exposed via share URL OR sent to wrong factory

**What goes wrong:**
Two scenarios specific to this domain:

- **Wrong-factory leak via Send**: User has Project A (premium leather bag, samples at Factory X) and Project B (similar leather bag prototype, samples at Factory Y). AI draft for Project A's "sample request" template gets generated with the Factory X email but accidentally references Project B's confidential pricing in the body (because both projects' contexts were in the prompt window). The leak is to a *legitimate vendor* of ours — but they now know our pricing strategy for a competing product line.

- **Share-URL leak**: User shares a project's read-only URL with MD (merchandiser) thinking only progress is exposed. The "next step" field, written by the user, includes "negotiating with Factory Z to undercut Factory X by 15%." MD forwards URL to Factory X via accident. Strategic damage.

**Why it happens:**
- Manufacturing relationships are dense and overlapping: same factories work on multiple projects, MDs work across multiple lines, vendors compete for the same business.
- The "user-written" content (next steps, notes) is rarely scrubbed — but it's the most sensitive.
- Time zone confusion (Korean designer, factories in Vietnam/China/Bangladesh) means messages get sent at odd hours when nobody catches an error.

**How to avoid:**
1. **Per-project "confidentiality tier"**: user marks each project as `internal | shared | public`. `internal` projects cannot be added to a share URL at all (UI removes the option). `shared` projects' next-step field is excluded from share unless explicitly opted in.
2. **AI prompt isolation per project**: when drafting for Project A, ONLY Project A's data goes into the AI context. No "include all projects so AI has more context" — that's how cross-contamination happens.
3. **Recipient verification UI** (per Pitfall 11): every send shows a recipient block with name, email/Slack, AND project-association count. "Sending to 김 사장님 (Factory X). They are associated with 3 of your projects. This message is about: Project A." If user sends Project A info to a Factory X contact who's also on Project B, no leak — but if recipient is Factory Y's contact, surface a warning.
4. **Time zone display**: every send draft shows recipient's local time at moment of send. Block sends with a confirm prompt if recipient time is between 22:00–06:00 local.
5. **Share URL audit log**: every URL access logged with IP, user agent, timestamp. User reviews log weekly. Token revocation on any suspicious access.
6. **Default share-URL expiry of 7 days**: extending requires explicit action. Old URLs auto-expire; reduces blast radius of any leak.
7. **"Sensitive language" lint on share content**: regex check for terms like "단가" (unit price), "원가" (cost), "마진" (margin), competitor factory names. Surface a warning before the share URL is created.

**Warning signs:**
- Send drafts mentioning data from a project not selected.
- User regularly correcting recipient before sending.
- Share URL access logs showing access from unexpected geographies/IPs.
- Sends going out at recipient's 3 AM.

**Phase to address:** **P6 (Send)** for recipient verification + AI isolation + TZ guards; **P7 (Share)** for tier system + sensitive-language lint + audit log.

---

### Pitfall 17: Manufacturing-domain — sample lifecycle mismatch with Notion schema causes status drift

**What goes wrong:**
The 12-stage milestone template in PROJECT.md (기획→1차 시안→…→출고/입고) assumes a linear progression. Reality: a sample arrives, fails QC, goes back to 샘플 수정, but production for *another* color of the same product is already mid-양산. Our progress percentage (milestone × 60% + task × 40%) starts producing nonsensical values like 73% on a project that's actually blocked. User loses trust in the dashboard's accuracy.

A related failure: Notion property naming drift. User renames "샘플 도착" to "샘플 입고" in Notion. Our schema expects exact match. Sync silently breaks for this property; status shows blank; AI brief omits this project from the daily list.

**Why it happens:**
- Linear lifecycle templates rarely match real manufacturing where sub-products, re-iterations, and parallel work are normal.
- Korean designers commonly rename Notion properties to fit their team's vocabulary.
- Schema-as-contract requires a contract that handles drift.

**How to avoid:**
1. **Schema by ID + display name**, not by display name alone: the schema wizard stores Notion's stable property `id` (a hash). User can rename freely; sync follows the id.
2. **Stage as data, not enum**: store stages as named records in a Notion "Stages" DB. Project links to its current stage. User can add/remove/reorder stages; our code never hard-codes the 12 stages.
3. **Progress calculation handles multi-track**: progress % is calculated per track (e.g., "Color: Red" track at 80%, "Color: Blue" track at 40%). UI shows aggregated + drill-down. No misleading single number for multi-track projects.
4. **"Blocked" as a first-class state**: when AI detects "샘플 수정" cycle (return-to-stage), the project shows a "Blocked" badge regardless of % completion.
5. **Schema diff on every wizard run**: surface to user "We detected a property rename: 샘플 도착 → 샘플 입고. Confirm we should map our internal `sample_arrived` field to this?" Never silently re-bind.
6. **Validation in onboarding**: walk user through 1-2 real projects in their Notion to confirm our parsing matches their model before committing.

**Warning signs:**
- Progress percentages > 100% or going down without explanation.
- Projects disappearing from dashboard after a Notion property edit.
- AI brief omitting projects the user knows are active.

**Phase to address:** **P1 (Notion SOTR)** for schema-by-id + multi-track support; **P5 (AI Brief)** for blocked-state detection.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Synchronous Slack event handling (no queue) | Fewer moving parts in P3 | Duplicates, retry storms, correlated AI cost spikes | **Never** — Slack's 3s ACK contract is non-negotiable |
| Skip Notion data_source_id discovery (use database_id) | One less API call per request | Breaks at any user with multi-source DB; Notion will sunset old endpoints | **Never** — bake discovery in from P1 |
| Single AI prompt with all projects' context | Better cross-project insights | Cross-contamination leaks (Pitfall 16) | Only for the daily brief view, never for outbound drafts |
| Wall-clock LWW conflict resolution (no UX) | Ship two-way sync in 1 day | Silent data loss, user trust collapse | Acceptable in P1 IF conflict log is shipped + weekly review UI by P5 |
| Hard-code KakaoTalk parser to Korean PC format | Ship P4 in days | Silent zero-message imports for any other locale | **Never** — locale detection is part of MVP |
| `codesign --deep` to "just make it work" | Notarization passes today | Sparkle XPC breaks; future macOS rejects bundle | **Never** — sign in correct order |
| Skip optimistic locking on Notion writes | Less code in sync worker | Lost edits whenever user has Notion open | **Never** — single-user app or not, the user IS the concurrent editor |
| In-memory rate limiter (no persistence) | Simple | After app restart, can burst into 429 hell | OK for v1; persist if we add background-process workers in v2 |
| AI for recipient resolution | "Smart" UX | Wrong-factory leaks (Pitfall 16) | **Never** — recipient resolution is deterministic code only |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Notion API 2025-09-03 | Using `database_id` everywhere | Discovery → cache `data_source_id`; use new endpoints |
| Notion API | Treating `archived` and `in_trash` as the same | Three-state enum (`active|archived|trashed|gone`) |
| Slack Events | Doing work before HTTP 200 | Enqueue first, ACK in <100ms, process async |
| Slack subtypes | Filtering only `type: "message"` | Handle `message_changed`, `message_deleted`, `bot_message` explicitly |
| Slack OAuth | Caching the long-lived legacy token forever | Opt into rotation; refresh every <12h; mutex around refresh |
| Gmail | Treating per-user 250 quota/sec as project-wide | Token-bucket per user; encode each method's cost |
| Gmail | `messages.list` polling for changes | Use `users.history.list` + `users.watch` Pub/Sub (re-arm every 7d) |
| Gmail | Classifying at message level | Classify at thread level; aggregate labels |
| Google OAuth | Leaving app in "Testing" mode | Move to "Production" before launch (kills 7-day refresh expiry) |
| Notion OAuth | Concurrent refreshes from multiple workers | Single-flight mutex; CAS on Keychain write |
| KakaoTalk | One regex for one locale | Locale detection → dispatch to per-locale parser |
| KakaoTalk | Trust file-watcher on Created event | Debounce by stable size + size match between hash and ingest |
| Anthropic | Naive retry on 429 | Honor `retry-after`; cap total daily spend in code |
| Anthropic | Rebuilding system prompt every call | Static prefix → cache marker → dynamic suffix; verify >70% cache hit |
| Tauri IPC | Passing large payloads as JSON args | Use `tauri::ipc::Channel` for streams >few KB |
| Tauri multi-monitor | Calling `set_position` after restart | Check current monitor; `available_monitors()` before positioning |
| Sparkle | `codesign --deep` | Sign helpers explicitly with correct suffixes (`-spks`, `-spki`) |
| Sparkle | Bumping `CFBundleShortVersionString` only | Sparkle compares `CFBundleVersion` (build number) — increment that |
| FSEvents | Watching `~/Library` paths | Watch only user-picked paths via NSOpenPanel — avoids Full Disk Access requirement |
| macOS notification | Forgetting Info.plist usage description | Add description; request permission on first relevant action |

---

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Initial Notion ingest without rate limit | Onboarding hangs | Token bucket at 2.5 req/s + progress UI | First user with >30 projects |
| Re-fetch all Notion blocks on every sync | Slow daily sync, hits rate limit | Cache by `last_edited_time`; only re-fetch changed | After ~50 projects with deep nesting |
| Polling Slack `conversations.history` instead of Events | Daily quota burns; data lag | Socket Mode or Events API + dedup | Within first week |
| Polling Gmail every minute | Quota drain, lag | Push notifications via Pub/Sub + watch re-arm | After ~1k emails |
| Loading entire SQLite into memory for dashboard | Slow startup, memory ↑ | Indexed queries; load only visible projects + lazy lists | After ~100 projects |
| AI on every Notion change event | Bills + cost loop | Idempotency cache by content hash; loop detection | First feedback loop (week 1 of beta) |
| Synchronous Tauri IPC for large data | Frozen UI | `tauri::ipc::Channel` for streams; chunked responses | Sharing screenshot-like assets |
| Storing full attachment binaries in SQLite | DB bloat → slow queries | Reference filesystem path; SQLite stores metadata only | After ~100 attachments |
| Re-parsing entire KakaoTalk .txt on every change | CPU spike, duplicates | Idempotency hash + `processed_files` table | Re-export of >10MB chat |
| Computing progress % synchronously on every render | UI jank | Materialized view; recompute on data change | After ~30 projects |

---

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| OAuth tokens in plaintext config file | Credential theft from any process with disk read | macOS Keychain via `keyring` crate; never in JSON/SQLite |
| Allowlist `*` in Tauri CSP | Webview can fetch arbitrary URLs (data exfiltration vector) | Explicit allowlist: Anthropic, Notion, Slack, Google, Cloudflare, Sparkle |
| Share URL tokens with no expiry | Permanent leak if URL forwarded | Default 7-day expiry; rotation on demand; revocation = immediate 403 |
| Share URL stored as page-id (predictable) | URL guessing | UUIDv4 or 256-bit random token; not derived from page-id |
| Sanitization function distributed across N call sites | Easy to miss a field | Single `sanitize_for_share()` chokepoint; tests assert forbidden fields absent |
| Logging request bodies for debugging | Tokens + message content in log files | Redact `Authorization`, `body`, `text` at log layer; never log raw payloads |
| AI prompts contain raw API keys | Key leak in Anthropic logs | Strip credentials from any text before AI; use placeholders |
| Cloudflare KV accepts unauthenticated writes | Anyone can poison snapshots | Worker enforces JWT signed by app's private key on every write; KV writes only from Worker |
| FSEvents triggers process arbitrary commands from .txt content | Arbitrary code execution if parser flawed | Parser is pure Rust; no shell-out, no eval; bounded memory per file |
| Sparkle update appcast over HTTP | MITM swaps in malicious update | HTTPS-only appcast; EdDSA signature verification on every download |
| User opt-in to AI assumed by default | Privacy promise broken without consent | Explicit opt-in screen; disable AI features entirely if declined |
| AI generates `to:` field for outbound | Wrong-recipient leak | Recipient is deterministic code only; AI generates only body |

---

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| AI brief at 8 AM but app takes 30s to open from notification | Designer dismisses notification, never opens | Pre-warm app on schedule; click → 5s open per PROJECT.md requirement |
| Showing all 12 lifecycle stages to a project that's at stage 2 | Cognitive overload | Show current stage + next 2 + collapsed past |
| Send draft shown without recipient context | Designer hits send, leak | Recipient block ABOVE draft body, with project-association count |
| Conflict resolution as auto-merge | Silent data loss | 3-way diff modal; user picks |
| KakaoTalk import with no progress indicator | Designer thinks app crashed | Progress bar + ETA + "X messages parsed" counter |
| Onboarding wizard requires all 5 services up front | Designer abandons if Slack auth fails | Allow skip; mark integration as "not connected"; let user enable later |
| Ambiguous Slack-to-project classification (60% confidence) silently picked | Wrong project, wrong context to AI later | Show low-confidence items in "needs review" inbox; require user click |
| Share URL revocation has 5-min cache lag | User thinks they revoked but external can still see | Synchronous KV deletion; UI confirms only after delete responds |
| AI insight shown as if it were ground truth | User trusts hallucinated facts | Visual distinction (italic, "AI suggests") + click-to-cite-source |
| "Sync error" toast with no actionable info | User can't fix | Specific message + "Retry" + "What does this mean?" link |
| Korean UI but error messages in English | Trust loss | Localize all strings including errors; test with full Korean fixtures |
| Time-of-day-blind sends | Send at 3 AM Vietnam time, factory ignores | Show recipient local time; warn if outside business hours |

---

## "Looks Done But Isn't" Checklist

- [ ] **Slack ingest:** ACKs in <100ms with empty body BEFORE any work — verify with `time curl` against handler
- [ ] **Slack ingest:** `message_changed` and `message_deleted` events handled — verify by editing a message in Slack and checking Notion update
- [ ] **Slack ingest:** Idempotency via `event_id` unique constraint — verify by replaying same payload twice, second is no-op
- [ ] **Notion sync:** Uses `data_source_id` with API version 2025-09-03 — verify by inspecting outbound HTTP, not just response success
- [ ] **Notion sync:** Tombstones for archived/trashed pages — verify by archiving a page in Notion, restoring, ensuring no duplicate created locally
- [ ] **Notion sync:** Optimistic lock on writes — verify by editing same page in browser + app concurrently, conflict UX shown
- [ ] **OAuth refresh:** Single-flight under concurrent calls — verify by triggering 5 parallel refreshes, only 1 token endpoint call
- [ ] **OAuth refresh:** Google app moved to Production status — verify with `gcloud projects describe`
- [ ] **Gmail ingest:** Watch re-armed within 7 days — verify with cron + log assertion
- [ ] **Gmail ingest:** Token-bucket honors method-specific quota costs — verify by saturating with high-cost method, observing throttle
- [ ] **KakaoTalk parser:** All 4 locale fixtures (KR PC, KR Mobile, EN PC, ZH PC) parse — CI test
- [ ] **KakaoTalk parser:** UTF-8 BOM stripped — fixture has BOM, parser succeeds
- [ ] **KakaoTalk parser:** Re-import is idempotent — same file twice, zero new rows on second
- [ ] **Watch folder:** Debounced by stable size — fixture mid-write doesn't trigger parse
- [ ] **AI drafts:** All entities (recipient, project, sample) reference existing IDs — validation rejects fabricated IDs
- [ ] **AI drafts:** No PII/contact info inserted that wasn't in source data — regex test on 100 sample drafts
- [ ] **AI calls:** Daily spend ceiling enforced — set ceiling to $0.01, verify next call refused
- [ ] **AI calls:** Cache hit rate >70% in production — telemetry assertion
- [ ] **AI calls:** Loop detection alerts — verify by manually triggering 4 re-runs on same item
- [ ] **Send recipient:** Resolved by deterministic code, not AI — code review assertion
- [ ] **Send recipient:** Cross-project association warning shown when applicable
- [ ] **Share URL:** Snapshot contains zero message bodies — search snapshot JSON for sentinel strings from each integration
- [ ] **Share URL:** Revocation produces 403 within 5s — synchronous test against real Worker
- [ ] **Share URL:** Sensitive-term lint warns before share creation — fixture content with "원가" triggers warning
- [ ] **Share URL:** Default expiry 7 days — UI default + KV TTL match
- [ ] **macOS:** Hardened Runtime enabled — `codesign -d --entitlements - <app>` shows `cs.allow-jit` style flags
- [ ] **macOS:** Notarization passes in CI — `xcrun notarytool submit --wait` exit 0
- [ ] **macOS:** Sparkle Autoupdate.app helper signed — `codesign --verify --deep --strict`
- [ ] **macOS:** Build number monotonically increases on every release — CI check
- [ ] **macOS:** Watch folder uses NSOpenPanel-picked path only — no hard-coded `~/Library` paths
- [ ] **macOS:** Notifications appear on first launch (permission requested) — manual smoke test on clean macOS install
- [ ] **Tauri:** CSP allowlist excludes wildcards — `tauri.conf.json` review
- [ ] **Tauri:** Multi-monitor window position restored correctly — manual test on 2-monitor setup
- [ ] **Outbound network:** No requests to non-allowlisted hosts — Little Snitch / mitmproxy capture during 1h session
- [ ] **Logs:** No tokens or message bodies in log files — grep test after a beta session
- [ ] **Onboarding:** Works with each service skipped individually — manual test
- [ ] **i18n:** Every string has Korean — grep for English strings in src/

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Notion `data_source_id` not migrated | LOW | Run discovery on every existing database row; backfill `data_source_id` column; deploy hotfix |
| Slack duplicate messages in Notion | MEDIUM | Dedup script: group by `(channel_id, ts, body_hash)` in Communication Log, keep oldest, delete rest. Recompute project assignments. |
| OAuth token bricked (race) | LOW per user, HIGH if widespread | Surface re-auth prompt in app; user re-completes OAuth. Add single-flight mutex hotfix to prevent recurrence. |
| Gmail watch expired (7-day silent failure) | LOW | Re-arm watch; backfill missing time window via `messages.list` with `after:` query |
| KakaoTalk parser silent fail | MEDIUM | Add new locale parser; for affected users, prompt re-import of their .txt files |
| AI cost runaway | LOW (one-time bill) to HIGH (recurring) | Disable AI immediately; investigate loop with `ai_run_count`; deploy budget cap; refund users if billed |
| Wrong-factory message sent | HIGH (relationship damage) | Cannot un-send. Recovery: contact recipient, request deletion, log incident, postmortem AI prompt to prevent recurrence |
| Share URL leaked | HIGH (strategic damage) | Immediate token revocation; audit log review for access; notify user; postmortem on what content was visible |
| Notion conflict overwrote local edit | MEDIUM | Conflict log has both versions; offer restore-from-conflict-log UI to user |
| Sparkle update bricked install | HIGH | Push hotfix appcast pointing at known-good version; user reinstalls from DMG; lost auto-updates require manual notification |
| Notarization rejected last-minute | MEDIUM | Ship via direct DMG (signed but not notarized) with Gatekeeper override instructions; fix and re-notarize |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| 1. Notion data_source_id confusion | P1 | Outbound HTTP capture shows new endpoint format |
| 2. Notion soft-delete states | P1 | Archive→restore round-trip produces no duplicates |
| 3. Notion 429 storm during ingest | P0 (limiter) + P1 (UX) | Synthetic 100-project ingest stays under 3 req/s |
| 4. Slack 3s ACK violation | P3 | Handler returns 200 in <100ms (load test) |
| 5. Slack subtype handling | P3 | Edit + delete in Slack reflected in Notion within 1 min |
| 6. OAuth refresh race | P0 | Concurrent refresh test → single token endpoint call |
| 7. Gmail per-user quota | P2 | Synthetic high-rate test → throttled, no 429 |
| 8. Gmail label/thread classification | P2 | Multi-message thread → single project assignment |
| 9. KakaoTalk locale variants | P4 | All 4 locale fixtures parse in CI |
| 10. KakaoTalk watch debouncing | P4 | Mid-write fixture doesn't trigger parse |
| 11. AI hallucinated entities | P6 | Validation rejects drafts with fabricated IDs |
| 12. AI cost runaway | P5 (budget primitive) before any AI in P3/P4 | Spend ceiling enforced; loop detection alert |
| 13. Privacy leak (AI + share) | P5 (consent), P7 (sanitization) | CI assertion: forbidden fields absent from snapshot |
| 14. macOS notarization rejection | P0 (runtime) + P8 (Sparkle) | CI notarytool exit 0 |
| 15. Two-way sync race | P1 | Concurrent edit test → conflict UX shown |
| 16. Manufacturing-domain leaks | P6 (send guards) + P7 (share tiers) | Recipient/sensitive-term tests |
| 17. Sample lifecycle drift | P1 (schema by ID) + P5 (blocked state) | User rename in Notion → app stays connected |

---

## Sources

**Slack:**
- [The Events API | Slack Developer Docs](https://docs.slack.dev/apis/events-api/)
- [Slack API Integration: Handling Errors and Retries](https://www.questionbase.com/resources/blog/slack-api-integration-handling-errors-retries)
- [Debugging Slack Integration: From 6 Duplicate Responses to Instant Acknowledgment](https://dev.to/jeremy_longshore/debugging-slack-integration-from-6-duplicate-responses-to-instant-acknowledgment-36ij)
- [Socket mode is duplicating the messages · slackapi/python-slack-events-api#93](https://github.com/slackapi/python-slack-events-api/issues/93)
- [Slack Token Rotation](https://docs.slack.dev/authentication/using-token-rotation/)
- [message_changed event](https://api.slack.com/events/message/message_changed)

**Notion:**
- [Notion API 2025-09-03 Upgrade Guide](https://developers.notion.com/docs/upgrade-guide-2025-09-03)
- [Notion API Rate Limits — Real Fix](https://dev.to/kanta13jp1/notion-api-rate-limits-are-breaking-your-automation-heres-the-real-fix-o5p)
- [Notion API Database Limits & Workarounds](https://dev.to/kanta13jp1/notion-database-limits-workarounds-7-walls-every-power-user-hits-5n0)
- [Notion OAuth refresh token invalid_grant](https://nango.dev/blog/notion-oauth-refresh-token-invalid-grant/)
- [How we made Notion available offline](https://www.notion.com/blog/how-we-made-notion-available-offline)
- [Notion API Updates 2026](https://fazm.ai/blog/notion-api-updates-2026)

**Gmail:**
- [Gmail API Usage limits](https://developers.google.com/workspace/gmail/api/reference/quota)
- [Gmail API rate limits, and why they matter](https://www.mailsweeper.co/blog/gmail-api-rate-limits-why-they-matter)
- [Configure push notifications in Gmail API](https://developers.google.com/workspace/gmail/api/guides/push)
- [Adventures in the Gmail PubSub API | Mixmax](https://mixmax.com/blog/adventures-in-the-gmail-pubsub-api/)
- [Watch() sends message regardless of label · googleapis/google-api-nodejs-client#2301](https://github.com/googleapis/google-api-nodejs-client/issues/2301)

**KakaoTalk:**
- [KakaoTalk chat export format (Grokipedia)](https://grokipedia.com/page/KakaoTalk_chat_export_format)
- [graup/kakaotalk-analyzer](https://github.com/graup/kakaotalk-analyzer/blob/master/kakaotalk.py)
- [hkboo/kakaotalk_chat_analysis](https://github.com/hkboo/kakaotalk_chat_analysis/blob/master/01_read_txt_and_data_preprocessing.py)
- [jooncco/kakaotalk-chat-exporter](https://github.com/jooncco/kakaotalk-chat-exporter)

**Anthropic / AI:**
- [Anthropic Rate limits](https://platform.claude.com/docs/en/api/rate-limits)
- [Anthropic API Pricing 2026](https://www.finout.io/blog/anthropic-api-pricing)
- [Anthropic admits Claude Code quotas running out too fast](https://www.theregister.com/2026/03/31/anthropic_claude_code_limits/)
- [Anthropic quietly nerfed Claude Code's 1-hour cache](https://www.xda-developers.com/anthropic-quietly-nerfed-claude-code-hour-cache-token-budget/)
- [Anthropic Prompt Caching](https://www.mindstudio.ai/blog/anthropic-prompt-caching-claude-subscription-limits)
- [Reducing hallucinations in LLMs (Amazon Bedrock)](https://aws.amazon.com/blogs/machine-learning/reducing-hallucinations-in-large-language-models-with-custom-intervention-using-amazon-bedrock-agents/)
- [Grounding Reality – Cresta on LLM Hallucinations](https://cresta.com/blog/grounding-reality---how-cresta-tackles-llm-hallucinations-in-enterprise-ai)
- [Guardrails for Truth: Minimising LLM Hallucinations](https://medium.com/@shivamarora1/safeguard-and-reduce-llm-hallucinations-using-guardrails-77e2299528ff)

**Tauri / macOS:**
- [Tauri 2 Configuration Reference](https://v2.tauri.app/reference/config/)
- [Tauri 2.0 Stable Release](https://v2.tauri.app/blog/tauri-20/)
- [Tauri multi-monitor window bug #14019](https://github.com/tauri-apps/tauri/issues/14019)
- [Tauri macOS physical position bug #7890](https://github.com/tauri-apps/tauri/issues/7890)
- [Tauri Permissions](https://v2.tauri.app/security/permissions/)
- [How I Built a macOS Menu Bar HUD with Rust + Tauri 2.0](https://dev.to/hiyoyok/how-i-built-a-macos-menu-bar-hud-with-rust-tauri-20-pij)
- [tauri-macos-menubar-app-example](https://github.com/ahkohd/tauri-macos-menubar-app-example)
- [macOS Apps: From Sandboxing to Notarization](https://blog.xojo.com/2024/08/22/macos-apps-from-sandboxing-to-notarization-the-basics/)
- [Notarization: the hardened runtime](https://eclecticlight.co/2021/01/07/notarization-the-hardened-runtime/)
- [Hardened Runtime and Sandboxing](https://lapcatsoftware.com/articles/hardened-runtime-sandboxing.html)
- [Code Signing and Notarization: Sparkle and Tears (Steinberger)](https://steipete.me/posts/2025/code-signing-and-notarization-sparkle-and-tears)
- [Sparkle Documentation](https://sparkle-project.org/documentation/)
- [macOS Notarization, HSM Code Signing Keys, and Sparkle Issues (Duo Labs)](https://duo.com/labs/tech-notes/macos-notarization-hardware-backed-code-signing-keys-and-sparkle-code-signing-issues)

**Sync / Manufacturing:**
- [Engineering Challenges of Bi-Directional Sync (Stacksync)](https://www.stacksync.com/blog/the-engineering-challenges-of-bi-directional-sync-why-two-one-way-pipelines-fail)
- [Conflict Resolution: Last-Write-Wins vs CRDTs](https://dzone.com/articles/conflict-resolution-using-last-write-wins-vs-crdts)
- [Supplier miscommunication on designs causes downstream issues (Colab)](https://www.colabsoftware.com/research/supplier-miscommunication-on-designs-causes-major-downstream-production-issues)
- [Manufacturing Design Mistakes (LeelineBags)](https://www.leelinebags.com/manufacturing-design-mistakes/)
- [The Real Consequences of Outdated Manufacturing Documentation (Canvas GFX)](https://www.canvasgfx.com/blog/outdated-documentation-dangers)

---
*Pitfalls research for: macOS menu bar productivity app integrating Slack/Gmail/Google Calendar/Notion/KakaoTalk(.txt) with Anthropic AI for Korean manufacturing-design workflow*
*Researched: 2026-05-03*
