# Project Research Summary

**Project:** Designer Dashboard
**Domain:** macOS menu-bar productivity app for Korean product-development designers in fashion/lifestyle manufacturing — multi-channel ingest (Slack/Gmail/GCal/Notion/KakaoTalk-via-.txt) + AI daily brief + AI-drafted send cards (MD/factory) with approval gate + sanitized read-only share URLs
**Researched:** 2026-05-03
**Confidence:** HIGH overall (HIGH on stack/architecture/pitfalls; MEDIUM on niche slices — KakaoTalk parser locale variants, Adobe AI/INDD thumbnails, Korean fashion-PLM patterns)

---

## Executive Summary

Designer Dashboard is the intersection of three mature product categories — daily-brief tools (Sunsama/Akiflow), fashion PLM (Backbone/Bamboo Rose), and always-on AI menu-bar UX (Raycast/ChatGPT Mac) — sitting in a wedge none currently occupy: **PLM-aware milestone tracking + Sunsama-style daily ritual + AI-drafted send-to-MD/factory cards + Korean-tool ingest (KakaoTalk .txt watch folder) + Korean honorific-aware AI tone**. PROJECT.md's stack and feature picks are largely correct; research found ~85% of table-stakes already enumerated, with 3 small but important table-stakes gaps to surface (global hotkey, snooze/roll-over, search across channels) and 2 sharpenings (12-stage milestone template baked in, honorific-aware drafting as a first-class field).

The recommended approach is a **Tauri 2.x (Rust + React) menu-bar app** with **Notion as the single Source of Truth** (LWW reconciliation by `last_edited_time`), **hybrid per-channel transport** (Slack Socket Mode + Cloudflare Worker bridge fallback for offline; Gmail/GCal polling; KakaoTalk FSEvents on a watch folder; Notion polling now + webhook bridge later), **single-writer SQLite WAL** for local persistence, **Anthropic Sonnet 4.6 + Haiku 4.5 with prompt caching** behind a centralized AI module, and a **Cloudflare Workers + KV + R2** backend on the free `*.workers.dev` subdomain for token-gated read-only share pages. The privacy boundary is a typed `SharedView` Rust struct that **structurally cannot contain message bodies** — message bodies live in local SQLite + Keychain only, never in cloud KV.

The top three risks are: (1) **AI hallucination in send-drafts** picking the wrong factory/sample/recipient — mitigated by structured-output ID validation against SQLite + a deterministic recipient-resolution layer + an approval gate that surfaces cited facts; (2) **OAuth refresh race conditions** (Slack 12h rotation, Notion single-use refresh tokens, Google 7-day testing-mode expiry) silently bricking integrations — mitigated by single-flight mutex per provider + CAS on Keychain writes + moving the Google app to Production status before launch; (3) **Notion API 2025-09-03 `data_source_id` migration trap** that breaks code copied from older tutorials and LLM training data — mitigated by baking discovery into the schema wizard from day one and forcing all callers through a typed `DataSourceRef`.

---

## Key Findings

### Recommended Stack

PROJECT.md's planned stack survives research scrutiny essentially unchanged. Four sharpenings: **(a) Auto-update** — use the official `tauri-plugin-updater` (ed25519 manifest) over the community Sparkle plugin (single-maintainer, +5 MB, no UX win). **(b) Keychain** — use `tauri-plugin-keyring` (community, wraps the well-trusted Rust `keyring` crate over macOS Login Keychain), explicitly NOT `tauri-plugin-stronghold` (deprecated in Tauri v3 + master-password UX is wrong for OAuth tokens). **(c) PSD/AI/INDD thumbnails** — primary path is `objc2-quick-look-thumbnailing` (FFI to native QLThumbnailGenerator); fallback chain is `qlmanage` shell-out → `psd` Rust crate for embedded JPEG → placeholder. Do NOT use `sharp` (does not support these formats). **(d) 8 AM scheduler** — see "Conflict Resolution" below; the resolved answer is launchd `RunAtLoad=true` + in-app tokio task, NOT launchd `StartCalendarInterval`.

**Top 5 stack decisions locked in:**

1. **Tauri 2.11.0 (Rust + React 18 + TypeScript 5.6 + Vite 6 + Tailwind v4 + shadcn/ui v2)** — ~30–50 MB bundle, ~1/4 the RAM of Electron; satisfies the "메모리 민감" constraint; native `tauri::tray` + `tauri::menu` for the menu-bar UI.
2. **Notion as SOTR with `@notionhq/client` v5.19 + API version `2026-03-11`** — `data_source_id` discovery mandatory on every database reference (see Pitfall 1); LWW by `last_edited_time` with optimistic-lock emulation (read-then-write within one logical operation).
3. **`tauri-plugin-keyring` (HuakunShen) + `tauri-plugin-sql` (sqlx-backed SQLite WAL) + `tauri-plugin-store` (non-secret prefs) + `tauri-plugin-fs` with `watch` feature (KakaoTalk folder via FSEvents) + `tauri-plugin-autostart` + `tauri-plugin-updater` (NOT Stronghold, NOT Sparkle plugin)** — official-first plugin set with one well-reasoned community addition.
4. **Anthropic SDK 0.92 with prompt caching (Sonnet 4.6 for drafts/brief, Haiku 4.5 for classification)** — centralized in a single `ai/` module so the cache prefix (system prompt + project list + schemas) is reused across channels; ~$10/month target requires >70% cache-hit rate.
5. **Cloudflare Workers + KV + R2 on the free `*.workers.dev` subdomain, with Hono 4.12 as the Worker framework** — two separate Workers (`share-worker` for public reads, `bridge-worker` for authenticated writes + Slack/Notion webhook reception); keeps privacy postures separate; zero infra cost in v1.

Detailed versions, install commands, and "what NOT to use" in [STACK.md](./STACK.md).

### Expected Features

PROJECT.md's `Active` list covers ~85% of table-stakes. Research surfaced 3 missing table-stakes that should be added before requirements lock-in, plus 2 sharpenings of items already mentioned implicitly.

**Must-have (table stakes — already in PROJECT.md):** menu-bar tray + autostart + Keychain, Notion 2-way sync (SOTR), GCal 2-way sync, Gmail label-based mapping, Slack channel-based + AI-classified mapping, Adobe file thumbnail recognition from Slack uploads, KakaoTalk .txt watch-folder ingest with dedup, AI daily brief + macOS notification, AI-drafted send cards with edit/send/ignore gate + Communication Log, sanitized read-only share URL with token revoke, progress calculation (60% milestone + 40% task), local-first SQLite for message bodies, onboarding wizard, code-signed DMG + auto-update.

**Must-have gaps to add to PROJECT.md `Active`:**
- **Foundation: 글로벌 단축키로 대시보드 호출** (e.g., ⌥+Space) — universal in Raycast/ChatGPT Mac/Akiflow.
- **Today-view: 오늘의 액션을 스누즈/연기/내일로 이월할 수 있다** — without roll-over, the daily brief decays into a stale graveyard within a week.
- **Search: 흡수된 모든 채널 메시지를 로컬에서 검색할 수 있다** — Klu/Akiflow/Notion all have it; users will absolutely expect "그 공장 메시지 어디 있더라" search.

**Sharpenings to make explicit:**
- **Notion: 패션 12단계 마일스톤 템플릿이 마법사 실행 시 자동으로 생성된다** — the 12-stage template is in PROJECT.md `Context`, but should be an explicit `Active` requirement (don't make user build it).
- **Send: AI 초안이 수신자 역할(MD/공장/내부)에 따라 존댓말/반말/영문을 자동 선택한다** — Korean honorific-awareness is the cultural-fit differentiator; bake into the draft schema as a first-class field.

**Differentiators (already correctly in PROJECT.md, keep as headlines):** AI-drafted send cards with approval gate (no other tool in the category drafts outbound messages), KakaoTalk .txt watch-folder (no Western tool ingests it; Korean factories live there), local-first message-body storage (Sunsama/Motion/Akiflow all cloud-store), read-only share URL that hides bodies (only progress/dates), AI insights as annotations not scoring inputs (Motion-trust antidote).

**Defer to v1.x:**
- Shutdown ritual + tomorrow preview (Sunsama-pattern; high-value but adds time-on-task).
- Cmd+K command bar (depends on volume of jump-targets to be worth building).
- AI insights as page annotations (project-page comments) — defer if AI Brief itself ships first.
- Recurring task templates ("매주 월요일 공장 진행 보고").
- CSV/PDF export (Notion already exports).
- Multi-language send drafting (한/中/英 auto-detect).
- Per-project milestone customization (start with one fixed template, customize once users ask).

**Defer to v2+:** Multi-workspace Slack/Google, Adobe Creative Cloud Libraries direct integration, Windows/Linux desktop, mobile companion app, multi-user/team mode with MD-side write access, AI auto-categorization of new Slack channels, voice-to-task.

Detailed competitor matrix and feature dependencies in [FEATURES.md](./FEATURES.md).

### Architecture Approach

A single Tauri 2 app process hosts a Rust backend running one tokio runtime; a React/Tailwind frontend communicates via typed Tauri IPC. The Rust backend is organized around six modules: **`channels/<name>/`** (one folder per channel, each implementing its own native transport — Slack Socket Mode WebSocket, Gmail/GCal polling, Notion polling+webhook-bridge, KakaoTalk FSEvents); **`orchestrator/`** (scheduler + sleep/wake observer + mpsc EventBus that fans events from channels to the reconciler); **`sync/`** (Notion-as-SOTR reconciler + LWW resolver + Communication Log writer); **`ai/`** (centralized Anthropic client with prompt caching + classifier + drafter + briefer); **`data/`** (single-writer SQLite WAL with N reader pool — eliminates "database is locked" errors); **`share/`** (typed `SharedView` sanitizer + HTTPS publisher to CF Worker — the privacy chokepoint); plus **`secrets/`** (Keychain wrapper). Cloudflare-side: two separate Workers — `share-worker` (public, read-only, KV+R2) and `bridge-worker` (HMAC-authenticated, receives Slack/Notion webhooks, buffers to KV inbox, app long-polls `/inbox/drain`).

**Six architectural patterns** (full code in ARCHITECTURE.md):
1. **Webhook-to-Queue Bridge** — CF Worker accepts Slack/Notion webhooks, signs/verifies, buffers to KV; app long-polls and acks. Solves "desktop app can't expose a public HTTPS endpoint" without ngrok.
2. **Hybrid transport per channel** — Socket Mode for Slack online, bridge for Slack offline; polling for Gmail/GCal (Google explicitly recommends polling for desktop); FSEvents for KakaoTalk; webhook-bridge + polling fallback for Notion.
3. **Single-writer SQLite with async mutex** — one writer connection behind `tokio::sync::Mutex`, separate read pool of 4. Eliminates SQLITE_BUSY storms.
4. **LWW reconciliation by `last_edited_time`** — Notion always wins on conflict; loser's diff appended to Communication Log for audit.
5. **Sanitize-then-publish share boundary** — `SharedView` Rust type structurally has no `body`/`preview`/`excerpt`/`snippet`/`email_subject` fields; a single `sanitize()` function is the only constructor.
6. **macOS sleep/wake-aware scheduler** — `NSWorkspaceDidWakeNotification` triggers re-anchor + full sync round on wake.

### Critical Pitfalls

Top 5 risks (ranked by combined likelihood × blast radius), drawn from PITFALLS.md:

1. **AI hallucination in send-drafts → wrong factory/sample/recipient sent (Pitfall 11 + 16).** A mis-sent honorific or a confidential pricing detail leaked to a competing-line vendor permanently damages Korean factory/MD relationships. Avoid: structured-output JSON with required `recipient_id`/`project_id`/`milestone_id`/`sample_id` referencing existing SQLite IDs (validation rejects fabricated ones); deterministic recipient resolution (AI never populates `to:`); recipient verification block in UI showing project-association count + recipient local time; per-project AI prompt isolation (no "include all projects for context" — that's how cross-contamination leaks happen); confidentiality-tier per project (`internal | shared | public`).
2. **OAuth refresh races silently bricking integrations (Pitfall 6).** Slack 12h rotation, Notion single-use refresh tokens, Google 7-day testing-mode expiry. Two concurrent refreshes → one wins, the other gets `invalid_grant` and clobbers the valid token in Keychain → integration dead, user has to re-auth from scratch. Avoid: single-flight mutex per provider; compare-and-swap on Keychain writes (only overwrite if in-Keychain refresh_token matches the one we used); move Google app to Production status before launch; opt into Slack token rotation explicitly.
3. **Notion API 2025-09-03 `data_source_id` migration trap (Pitfall 1) + soft-delete state confusion (Pitfall 2) + 429 storm during initial bulk ingest (Pitfall 3).** Code copied from old tutorials/LLM training uses `database_id` + the legacy `/v1/databases/{id}/query` endpoint, breaks on multi-source DBs and silently writes to wrong tables. Trash/archive/hard-delete are three states that look the same to naive parsers, causing ghost-projects and duplicate-on-restore. Initial ingest of 50 projects × 20 blocks = 1000+ requests blows past Notion's 3 req/s. Avoid: bake `data_source_id` discovery into the schema wizard + force all callers through `DataSourceRef`; track 4-state `notion_state` enum (`active|archived|trashed|gone`); single global token-bucket at 2.5 req/s with `Retry-After` honor + exponential backoff; ETag-style cache by `(page_id, last_edited_time)` so we don't re-fetch unchanged blocks.
4. **AI cost runaway via feedback loops + retry storms (Pitfall 12).** Slack edit → re-classify → write to Notion → Notion webhook fires → reconciler reads it back → flagged for re-classification → AI runs every 30s. Within a week, $50 of Anthropic spend on what should be a $10/month app. Compounded by Anthropic's Apr 2026 nerf of prompt-cache TTL from 1h to 5min. Avoid: hard daily budget read from SQLite before every AI call (default $1/day, user can raise); idempotency by hash of `(model, system_prompt, user_message)` cached for 7 days; per-item `last_ai_run_at` + `ai_run_count` with freeze at >3 in 24h; tag every Notion update with `last_edited_by: <our integration>` and skip those in reconciliation (the loop-breaker); enforce in code that classification path can never call Sonnet (Haiku-only).
5. **macOS notarization rejection + Sparkle/Updater XPC signing failures (Pitfall 14).** Hardened Runtime not enabled by default → notarization fails → `codesign --deep --force` "fix" → Sparkle XPC services break → users on Sequoia get "App is damaged." Or notarization passes but Autoupdate.app helper not signed → silent update failures forever. Avoid: enable Hardened Runtime in `tauri.conf.json` from day one; sign helpers → frameworks → main app in correct order, never `--deep`; use `-spks`/`-spki` suffixes for Sparkle XPC bundles if Sparkle path chosen; CI smoke test runs `xcrun notarytool submit --wait` and blocks merge on failure; build number (`CFBundleVersion`) increments on every release; watch folder uses NSOpenPanel-picked path only (never `~/Library`) so Full Disk Access is not required.

Next-tier risks worth flagging (P1/P2 ranked but not top-5): **Slack 3s ACK violation → triple-delivery + 4× AI cost (Pitfall 4)**; **Slack `message_changed`/`message_deleted` subtypes ignored → stale Notion entries (Pitfall 5)**; **Gmail 250 quota-units/sec/user exceeded by parallel attachment fetches (Pitfall 7)**; **Gmail label-vs-thread classification splitting one thread across two projects (Pitfall 8)**; **KakaoTalk parser breaking on locale/BOM/encoding variants (Pitfall 9)**; **KakaoTalk file-still-being-written triggering premature parse (Pitfall 10)**; **Privacy leak via AI exfiltration without consent OR sanitization function bug (Pitfall 13)**; **Two-way sync race overwriting local edits silently (Pitfall 15)**; **Sample-lifecycle drift breaking schema-by-name when user renames Notion properties (Pitfall 17)**.

---

## Conflict Resolution

### Conflict 1: 8 AM daily brief scheduler — launchd vs in-app tokio task

**STACK.md says:** Use `launchd` user agent with `StartCalendarInterval` (`Hour=8 Minute=0`) — argues launchd "runs missed jobs the next time the Mac wakes," exactly what we want for a laptop that's closed at night.

**ARCHITECTURE.md says (Pattern 6, line 390):** Do NOT use `StartCalendarInterval`. Use `RunAtLoad=true` only (start app at login); the running app schedules its own 8am tokio timer; `NSWorkspaceDidWakeNotification` re-anchors the schedule on wake.

**PITFALLS.md confirms ARCHITECTURE.md (Anti-Pattern 6, line 614):** `StartCalendarInterval` causes launchd to launch a *new* process even if the app is already running, leading to duplicate notifications. Also, the brief needs the running app's in-memory state (today's events from all channels already aggregated locally) — a launchd-spawned process would have to cold-start the entire Rust backend just to send one notification.

**Resolved decision (ARCHITECTURE.md + PITFALLS.md win, with the wake-handling concern from STACK.md preserved):**

> **launchd LaunchAgent at `~/Library/LaunchAgents/com.user.designer-dashboard.plist` with `RunAtLoad=true` ONLY (no `StartCalendarInterval`)**. This guarantees the app is running at user login. The 8 AM brief itself is fired by an **in-app tokio task** that maintains its own next-tick deadline. A native **`NSWorkspaceDidWakeNotification`** observer (registered via `objc2`) sends a `WakeUp` message on the orchestrator's mpsc; on wake, the scheduler **re-anchors** by recomputing whether 8 AM was missed during sleep and either fires immediately (if missed today) or sets the next tick to today's or tomorrow's 8 AM. This preserves both concerns: (a) **no missed briefs after wake-from-sleep** (the wake observer is the catch-up mechanism, replacing launchd's calendar-interval semantics with in-process logic that has access to actual app state), and (b) **no duplicate processes** (launchd never spawns a competing process at 8 AM because it has no calendar interval; only `RunAtLoad` keeps the singleton alive).

**Rationale for this resolution:** ARCHITECTURE.md and PITFALLS.md are mutually consistent and were written with deeper awareness of the duplicate-process anti-pattern; STACK.md captured the right *concern* (missed-on-wake) but recommended the wrong mechanism. The synthesis above is strictly better than either single approach because it gives us OS-managed app lifecycle (launchd `RunAtLoad`) AND in-process state-aware scheduling (tokio + wake observer) AND wake-recovery (the same wake observer that re-anchors polling cursors).

**STACK.md should be updated** at the next research-pass to reflect this resolution; for now, the resolution lives in this SUMMARY.md and in ARCHITECTURE.md Pattern 6.

### Conflict 2 (softer): Sparkle plugin vs official `tauri-plugin-updater`

**STACK.md** recommends the **official `tauri-plugin-updater`** (ed25519 manifest) as primary, with `tauri-plugin-sparkle-updater` as alternative only if real Sparkle's appcast.xml is required.

**PROJECT.md (Active list, "Distribution")** says "Sparkle을 통해 자동 업데이트" — the user's stated intent is Sparkle.

**ARCHITECTURE.md `packaging/sparkle/`** assumes Sparkle (appcast.xml + signing).

**Resolved decision:** Default to **`tauri-plugin-updater` (official)** unless the user has a specific reason for real Sparkle (e.g., enterprise IT requirement for the appcast format). Rationale: first-party + cross-platform-future-proof + same UX + smaller bundle (no Sparkle.framework dependency) + lower single-maintainer risk. Surface this to the user during requirements definition for explicit confirmation before locking it into the roadmap. Treat PROJECT.md's "Sparkle" wording as shorthand for "auto-update mechanism" rather than a hard commitment to the Sparkle framework specifically.

### Conflict 3 (softer): Notion sync — webhooks Phase 1 vs Phase 8

**STACK.md** is silent on phase ordering for Notion webhooks.

**ARCHITECTURE.md Suggested Build Order** has webhooks deferred to **Phase 8 (Polish + Distribution)**; the Notion-SOTR phase ships polling-only (every 2 min via `last_edited_time` filter).

**PITFALLS.md** does not take a position on phase order, only on correctness.

**Resolved decision:** Follow **ARCHITECTURE.md** — ship polling-only in the Notion-SOTR phase; layer webhooks late as a defense-in-depth additive upgrade. Polling alone is sufficient for the 5-min sync requirement (PROJECT.md), and webhook plumbing requires the CF Worker bridge (which is already in an earlier phase for other reasons) plus HMAC verification to be production-ready, which is best left to the polish pass. This avoids gating the entire Notion phase on a real-time mechanism we don't strictly need.

---

## Implications for Roadmap

Based on combined research, **suggested 9-phase structure** that respects the Notion-SOTR build dependency, the privacy-boundary-first pattern (sanitizer before any channel), the "AI budget primitive before any AI integration" pattern, and the parallelization opportunity for read-only channels after the share boundary is established.

### Phase 0: Foundation Shell

**Rationale:** Every other phase needs persistence, scheduling, OAuth scaffolding, and signing baseline. Establishing OAuth single-flight + Hardened Runtime + Keychain wrapper now avoids retrofitting them later (which Pitfalls 6 and 14 both warn would require touching every call site).
**Delivers:** Tauri 2 menu-bar tray app that auto-starts at login (launchd `RunAtLoad=true` only), single-writer SQLite WAL with sqlx + migrations, `tauri-plugin-keyring` wrapper, `NSWorkspaceDidWakeNotification` observer + tokio scheduler skeleton, **OAuth refresh primitive with single-flight mutex per provider**, **global Notion token-bucket rate limiter at 2.5 req/s** (used in Phase 1 onward), **AI budget primitive in SQLite** (used in Phase 3 onward), Hardened Runtime + entitlements + signing config baseline.
**Uses:** Tauri 2.11, `tauri-plugin-sql`, `tauri-plugin-keyring`, `tauri-plugin-store`, `tauri-plugin-autostart`, `tauri-plugin-notification`, `objc2-foundation`, `keyring` crate.
**Avoids:** Pitfalls 6 (OAuth refresh race), 14 (notarization rejection), partial mitigation of 3 (rate limit primitive ready for Phase 1) and 12 (budget primitive ready for Phase 3).
**Verification gate:** App launches at login on a fresh macOS install; survives sleep/wake without losing state; Keychain stores+retrieves a synthetic OAuth token; concurrent-refresh test (5 parallel calls) results in exactly 1 token endpoint request; `xcrun notarytool submit --wait` exits 0 in CI.

### Phase 1: Notion SOTR + Schema Wizard + Multi-Track Progress

**Rationale:** Notion is the spine — every channel writes through this layer. ARCHITECTURE.md confirms: this phase before any channel. Pitfalls 1, 2, 15, and 17 all converge on this phase, so building the abstractions correctly here saves rewrites.
**Delivers:** Notion OAuth flow + token storage; **`data_source_id` discovery cached per database** (Pitfall 1); schema wizard that detects missing properties + the **12-stage fashion milestone template baked in** (Feature gap from FEATURES.md); polling at 2-min intervals with `last_edited_time` cursor (webhooks deferred to Phase 8); **LWW reconciler with optimistic-lock emulation** (read-then-write in one logical operation — Pitfall 15); 4-state `notion_state` enum (`active|archived|trashed|gone`) + daily reconciliation pass (Pitfall 2); **schema-by-property-id** (not display name) + schema diff UX on every wizard run (Pitfall 17); Communication Log writer; conflict log table for audit + 3-way merge UI for conflicts.
**Uses:** `@notionhq/client` v5.19 with `notionVersion: '2026-03-11'`, the rate-limiter from P0.
**Avoids:** Pitfalls 1 (data_source_id), 2 (soft-delete states), 3 (429 storm — UX side; primitive came from P0), 15 (two-way sync race), 17 (sample-lifecycle drift + schema rename).
**Verification gate:** Synthetic 100-project ingest stays under 3 req/s; archive→restore round-trip produces no duplicates; concurrent edit (browser + app) shows conflict UX; user-renames-property-in-Notion → app stays connected and surfaces a "we detected a rename, confirm mapping" prompt.

### Phase 2: Cloudflare Bridge + Share Skeleton + `SharedView` Privacy Boundary

**Rationale:** The `SharedView` sanitizer is the privacy chokepoint that every later channel must respect. ARCHITECTURE.md's reasoning: establishing the boundary type early makes leaking message bodies a compile error, not a code-review burden. CF Worker bridge skeleton is also a prerequisite for Slack/Notion webhook reception, so it's natural to land both together.
**Delivers:** CF Worker `share-worker` skeleton (token-based KV reader, plain HTML render); CF Worker `bridge-worker` skeleton with `/webhook/*` + `/inbox/drain` + `/inbox/ack` endpoints (no real webhooks wired yet); `SharedView` Rust type with structurally-absent body/preview/snippet fields + `sanitize_for_share()` chokepoint + CI assertion that forbidden field names don't appear; HMAC-signed publisher; token issue/revoke commands (default 7-day expiry per Pitfall 16); **per-project confidentiality tier (`internal|shared|public`)**; **sensitive-language lint** for terms like 단가/원가/마진/competitor names before share creation; Hono 4.12 on Workers with `wrangler.toml` bindings.
**Uses:** Hono, Cloudflare Workers + KV + R2, `wrangler` 3.x, `hono/jwt` for token issuance.
**Avoids:** Pitfall 13 (privacy leak via share — sanitization chokepoint enforced by Rust type system), Pitfall 16 (manufacturing leak via share URL — tier system + sensitive-term lint).
**Verification gate:** Synthetic share-snapshot serialized to JSON contains zero strings matching known sentinel values from any integration; revocation produces 403 within 5s against real Worker; sensitive-term fixture triggers warning UI; CI test: search snapshot for forbidden field names.

### Phase 3: Slack — Real-time + Webhook Fallback + AI Classifier (FIRST AI integration)

**Rationale:** Slack is the highest-volume channel; needs Notion writer + sanitizer + AI budget primitive ready. This is where the first AI calls land, so the budget primitive from P0 finally gets exercised. Dual-path (Socket Mode WS + bridge drain) requires de-dup design from day one (Pitfall 4).
**Delivers:** Slack OAuth (Bot + User token, opt into rotation); Socket Mode WebSocket loop via `slack-morphism-rust` (async, ack <100ms, enqueue first); CF bridge `/webhook/slack` with HMAC signing verification + KV inbox; `webhook_drain.rs` to pull events when Socket Mode was offline; **dedup by `(channel_id, event_ts)` with unique constraint** (handles dual-path duplicates + Slack's own retry duplicates); **handle subtypes: `message`, `message_changed`, `message_deleted`, `bot_message`** (Pitfall 5); centralized `ai/classifier.rs` (Haiku 4.5 only) with confidence threshold + manual-review inbox for <0.7; `chat.postMessage` + `files.upload` for Send/Attachments; **prompt caching with verified >70% cache hit telemetry**; idempotency cache for AI calls keyed by `(model, system_prompt, user_message)`.
**Uses:** `@slack/web-api` v7, `slack-morphism-rust` (Rust Socket Mode), Anthropic SDK 0.92 (Haiku 4.5), the budget primitive + rate limiter + Notion writer + sanitizer from earlier phases.
**Avoids:** Pitfall 4 (3s ACK), Pitfall 5 (subtype handling), Pitfall 12 (cost runaway — first real exercise of budget + cache + concurrency cap).
**Verification gate:** Slack handler returns 200 in <100ms (load test); replay same event twice → second is no-op; edit message in Slack → Notion entry updates within 1 min; cache hit rate >70% in synthetic high-volume test; daily spend ceiling at $0.01 refuses subsequent AI calls.

### Phase 4: Gmail + Google Calendar (paired — shared OAuth, similar polling transport)

**Rationale:** Similar transport (polling), shared OAuth flow, can ship together. ARCHITECTURE.md explicitly recommends polling for both per Google's own docs (Pub/Sub is for servers).
**Delivers:** Google OAuth (single flow, both scopes); **app moved to Production status** (Pitfall 6); Gmail historyId-based polling at 60s with 2-units `users.history.list`; **token-bucket rate limiter encoding per-method quota costs** (Pitfall 7); **thread-level classification** with re-fetch via `messages.get(format=metadata)` for ground truth (Pitfall 8); ambiguous threads surface in "needs review" inbox; Gmail send via `drafts.send` for Send pipeline; GCal syncToken-based polling at 90s; GCal event creation/update for milestone due_date sync; Pub/Sub `users.watch` for near-real-time Gmail (with 7-day re-arm cron — Pitfall 7).
**Uses:** `googleapis` v144, `google-auth-library` v9.
**Avoids:** Pitfall 6 (Google testing-mode 7-day expiry — moved to Production), Pitfall 7 (Gmail quota), Pitfall 8 (label/thread classification).
**Verification gate:** Synthetic high-rate test → throttled with no 429s; multi-message thread → single project assignment; watch re-armed within 7 days verified by cron+log assertion; labeled emails 100% mapped to projects (PROJECT.md success criterion).

### Phase 5: KakaoTalk Watch Folder + Adobe File Recognition (parallelizable with P4)

**Rationale:** Independent of Slack/Gmail/Notion — can be parallelized after the share boundary (P2) lands. KakaoTalk is the Korean-context differentiator; Adobe file recognition is in PROJECT.md and architecturally lives in `channels/slack/` so it pairs naturally with this phase.
**Delivers:** `notify` crate watch folder on **NSOpenPanel-picked path only** (Pitfall 14 — no Full Disk Access required); **debounce by stable size** (poll every 500ms, process only when size unchanged ≥2s — Pitfall 10); **locale detection** for KakaoTalk .txt format (KR PC/KR Mobile/EN PC/ZH PC fixtures in CI — Pitfall 9); per-locale parser modules with explicit unit tests for 오전/오후 ↔ AM/PM conversions; UTF-8 BOM strip + CRLF→LF normalize; **idempotency by SHA256 of `(timestamp, sender, body)` with unique constraint**; `processed_files` table keyed by `(path, sha256, size)`; reject-with-message on unsupported locale; Slack-attached `.psd/.ai/.indd` thumbnail generation via `objc2-quick-look-thumbnailing` (primary) → `qlmanage` shell-out → `psd` Rust crate for embedded JPEG → placeholder; thumbnails uploaded to R2 with URL embedded in `SharedView`.
**Uses:** `notify` 6.x, `objc2-quick-look-thumbnailing`, `psd` Rust crate, R2.
**Avoids:** Pitfalls 9 (locale variants), 10 (file-still-being-written), 14 (Full Disk Access), and the AI/INDD thumbnail edge case (graceful degradation chain).
**Verification gate:** All 4 locale fixtures parse in CI; mid-write fixture doesn't trigger parse; same .txt re-imported twice → zero new rows; Slack-uploaded `.psd` produces thumbnail in gallery within 30s (PROJECT.md success criterion).

### Phase 6: AI Brief + Today-View Dashboard + Snooze + Roll-over + Search

**Rationale:** Needs all channels feeding data to be useful. This is also where the table-stakes gaps from FEATURES.md (snooze, roll-over, search) land — they're all today-view UX features that depend on the data being there. Includes the central scheduling pattern from Conflict 1 (in-app tokio task + wake-observer re-anchor).
**Delivers:** `briefer.rs` aggregating today's events from all channels (Sonnet 4.6); macOS notification at 8 AM via in-app tokio task with `NSWorkspaceDidWakeNotification` re-anchor (Conflict 1 resolution); click-handler routes to dashboard within 5s; today-view UI with **snooze + roll-over to tomorrow** (Feature gap); **local SQLite full-text search across all ingested messages** (Feature gap — table stakes); insight generation as project-page annotations (NOT scoring inputs); **blocked-state detection** when AI sees "샘플 수정" cycle (Pitfall 17); **explicit AI consent UI on first run** + per-channel/per-DB AI-off override (Pitfall 13); Korean honorific-aware drafting context primitives ready for P7.
**Uses:** Anthropic Sonnet 4.6 + Haiku 4.5 with prompt caching, SQLite FTS5 for search, `objc2` for wake observer.
**Avoids:** Pitfall 13 (AI consent), Pitfall 17 (blocked-state), Conflict 1 anti-pattern (no launchd `StartCalendarInterval`).
**Verification gate:** 8 AM brief fires correctly after sleep-through-8am; snoozed item reappears at chosen time; search returns results across Slack/Gmail/KakaoTalk in <500ms over 10k messages; AI consent declined → zero Anthropic egress confirmed by network capture.

### Phase 7: Send Cards + Approval Gate + Communication Log + Recipient Verification

**Rationale:** Highest-blast-radius feature (Pitfalls 11 + 16). Depends on every prior phase: Notion writer for Communication Log, Slack/Gmail send capability, AI for drafting, search for grounding, project schema for ID validation. Ships last among feature phases so all primitives are mature.
**Delivers:** AI drafter with **structured-output JSON validation** (`recipient_id`, `project_id`, `milestone_id`, `sample_id` all references to existing SQLite IDs — Pitfall 11); **deterministic recipient resolution** (AI never populates `to:` — Pitfall 11); recipient verification UI block above draft body with name + email/Slack + project-association count + recipient local time (block sends 22:00–06:00 recipient-local with confirm — Pitfall 16); cited-facts chips on every draft for verification; **per-project AI prompt isolation** (no cross-project context bleed — Pitfall 16); **honorific-aware drafting** (존댓말/반말/영문 selected per recipient role — Feature sharpening); send execution via Slack `chat.postMessage` or Gmail `drafts.send`; idempotency check on `draft.sent_at`; Communication Log writeback via Notion writer; AI body vs final-sent text diff logged for weekly "AI changed N facts" review.
**Uses:** Sonnet 4.6, all prior phases' integration APIs.
**Avoids:** Pitfalls 11 (AI hallucination), 16 (manufacturing leak via wrong-factory/wrong-time send).
**Verification gate:** AI draft referencing fabricated sample_id → validation rejects; 100-draft regex test → no PII not in source data; draft for Project A never references Project B in body (cross-contamination test); send to Acme contact for Project A shows "associated with 3 of your projects" warning; send at recipient 3 AM blocked with confirm.

### Phase 8: Polish + Distribution + Notion Webhooks (additive upgrade)

**Rationale:** Distribution polish + the additive upgrade from polling to webhooks. ARCHITECTURE.md notes webhooks here are "purely additive over the polling baseline (defense in depth)" — never the sole sync mechanism.
**Delivers:** 5-step onboarding wizard (each integration skippable individually — UX Pitfall); code signing + `xcrun notarytool` smoke test in CI (Pitfall 14); `tauri-plugin-updater` (official) wired with ed25519 signed manifest hosted on R2 + custom `*.workers.dev` URL (per Conflict 2 resolution); Notion webhook bridge ENABLED via the CF Worker `bridge-worker` from P2 (HMAC-SHA256 with `verification_token`); settings UI for sync cadence, AI budget ceiling, share token list with revoke buttons, watch-folder picker, notification time; share URL audit log UI with weekly review; conflict log review UI; "AI changed N facts" weekly metric; full Korean i18n (no English error messages — UX pitfall).
**Uses:** `tauri-plugin-updater`, Notion webhooks, all prior phases.
**Avoids:** Pitfall 14 (notarization + signing — full CI gate), late surface of UX pitfalls.
**Verification gate:** Notarization passes in CI; auto-update from old DMG to new DMG works on clean macOS install; onboarding skip-each-step path works; `grep` for English strings in `src/` returns zero matches.

### Phase Ordering Rationale

- **P0 → P1 → P2 → (P3 + [P4 ∥ P5]) → P6 → P7 → P8.** P3 must precede P4/P5 only because P3 establishes the AI integration patterns (classifier, budget primitive exercise, dedup) that P4/P5 reuse. P4 and P5 can be parallelized after P3.
- **Privacy-boundary-first (P2 before any read channel).** The `SharedView` sanitizer type must exist before any channel can `From<X>` its data into a snapshot — otherwise the privacy invariant becomes a code-review burden instead of a compile error.
- **Notion-SOTR-second (P1 before any read channel).** Every channel writes to Notion's Communication Log; the writer + reconciler must be hardened before being called from N call sites.
- **AI-budget-primitive in P0, exercised first in P3.** Pitfall 12 explicitly warns: budget primitive must exist before any AI integration. P0 builds it; P3 first calls AI.
- **OAuth single-flight in P0.** Pitfall 6 warns retrofitting requires touching every call site. Build it before any OAuth flow lands.
- **Send (P7) ships last among feature phases** because it depends on the maturity of every other primitive (writer, recipient deterministic resolution, AI structured output, search for grounding) and has the largest blast radius (Pitfalls 11 + 16).
- **Notion webhooks deferred to P8** as additive defense-in-depth, never as primary mechanism (Conflict 3 resolution + ARCHITECTURE.md guidance).

### Research Flags

**Phases likely needing deeper research during planning (`/gsd-research-phase` recommended):**
- **Phase 5 (KakaoTalk + Adobe):** Locale fixture coverage requires real-world `.txt` exports from at least KR PC, KR Mobile, EN PC, ZH PC users — research couldn't verify all variants exhaustively. Adobe AI/INDD thumbnail generator behavior on user's specific files needs empirical testing (STACK.md flags this as a known gap with a graceful-degradation fallback chain).
- **Phase 7 (Send):** Korean honorific drafting prompt-engineering needs validation against real recipients (MD vs factory vs internal) — research surfaced the requirement but didn't validate prompt patterns. Cross-project contamination prevention needs adversarial test case generation.
- **Phase 8 (Distribution):** macOS Sequoia signing/notarization edge cases (especially if Sparkle plugin is chosen instead of `tauri-plugin-updater`) — research surfaced the patterns but actual hardware/macOS-version variation may surface new issues at codesign-and-test time.

**Phases with standard patterns (likely skip `/gsd-research-phase`):**
- **Phase 0 (Foundation):** Tauri 2 + sqlx + Keychain are well-documented; OAuth single-flight is a textbook pattern.
- **Phase 1 (Notion SOTR):** All major risks already mapped in PITFALLS.md (1, 2, 3, 15, 17); pattern code in ARCHITECTURE.md.
- **Phase 2 (CF Bridge + Sanitizer):** Hono on Workers is well-trodden; sanitizer is internal type design.
- **Phase 3 (Slack):** Patterns + pitfalls all enumerated; Socket Mode + dedup is well-documented.
- **Phase 4 (Gmail + GCal):** Google's own docs are explicit; quota costs are public.
- **Phase 6 (AI Brief + Search + Today-view):** SQLite FTS5 + Anthropic SDK are standard; wake observer is in ARCHITECTURE.md Pattern 6.

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All versions verified against current docs (Tauri 2.11.0, Hono 4.12.16, Anthropic SDK 0.92.0, Notion API 2026-03-11) on research date 2026-05-03. MEDIUM caveat on `tauri-plugin-keyring` (community, well-architected, single-maintainer risk) and PSD/AI/INDD thumbnails (native QL gap for AI/INDD documented). |
| Features | MEDIUM-HIGH | HIGH on productivity-tool category (many active competitors with public reviews); MEDIUM on PLM/manufacturing slice (gated tools, less public detail); MEDIUM on Korean-context patterns (Korean docs sparse in English search — JANDI/Swit/KakaoTalk-PC behavior validated via community projects, not official specs). |
| Architecture | HIGH | Tauri/Rust patterns, SQLite single-writer, CF KV bridge, Notion webhooks, Slack Socket Mode all verified against official 2025-2026 docs. The 6 patterns + 6 anti-patterns are independently grounded. |
| Pitfalls | HIGH | OAuth/Slack/Notion/Gmail/Tauri/Anthropic verified via official docs + community post-mortems. MEDIUM on KakaoTalk (community projects only, no official spec) and on manufacturing-domain pitfalls (industry articles, no domain-specific app post-mortems). |

**Overall confidence:** **HIGH** with two MEDIUM pockets called out below.

### Gaps to Address

1. **KakaoTalk locale + format coverage.** No published export-format spec exists. We have community-project parser samples for KR PC; need real fixtures from KR Mobile, EN PC, and at least one ZH PC user before Phase 5 ships. **Action:** during Phase 5 planning, use `/gsd-research-phase` and recruit beta users for fixture donation; build a "send us your unsupported file" feedback loop in the app itself.
2. **Adobe `.ai`/`.indd` thumbnail fallback fitness.** Native macOS QLThumbnailGenerator is unreliable for `.ai` (often falls back to generic icon) and depends on Adobe Quick Look generators that Sequoia changed how it loads. **Action:** during Phase 5 implementation, test the fallback chain (qlmanage → psd crate → placeholder) against real designer files; consider shipping a small Adobe Quick Look generator helper if usage data shows >20% placeholder-only rate.
3. **Korean honorific drafting prompt validation.** Sonnet 4.6 handles 존댓말/반말/영문 well "if prompted explicitly per recipient" — but the actual prompt shape needs empirical validation against real MD/factory/internal sends. **Action:** during Phase 7 planning, dogfood with 20+ real drafts before opening to beta; collect "AI changed N facts" weekly metric.
4. **Sparkle vs `tauri-plugin-updater` user preference (Conflict 2).** PROJECT.md says "Sparkle"; STACK.md recommends the official plugin. **Action:** confirm with user during requirements definition whether "Sparkle" was a hard commitment or shorthand for "auto-update mechanism."
5. **Notion webhook payload schema for the 2026-03-11 API.** Notion's webhook docs lag the API release; payload field stability cannot be 100% verified at research time. **Action:** Phase 8 should treat webhooks as additive over polling (per Conflict 3 resolution); polling is the canonical sync mechanism even after webhooks land.
6. **Cloudflare KV write quota** (1000/day free tier). With 30s debounce per project × 10 projects × 100 share updates/day = 1000 — exactly at the ceiling. **Action:** monitor in production; raise debounce to 60s if approaching ceiling, or upgrade to paid KV ($0.50/M writes is trivial).
7. **macOS Sequoia QuickLook plugin loading changes** (referenced in STACK.md sources). Old QL plugins no longer load on Sequoia, which affects fallback strategy for Adobe files. **Action:** Phase 5 testing must happen on Sequoia specifically.

---

## Open Questions for User / Future Research

These items need explicit user confirmation or follow-up research before locking into the roadmap:

1. **Auto-update mechanism (Conflict 2):** Confirm Sparkle vs `tauri-plugin-updater`. Recommendation: official plugin unless enterprise-IT requires real Sparkle.
2. **Daily brief notification time:** PROJECT.md says 8 AM but should be user-configurable. Confirm default + range (e.g., 6–10 AM).
3. **Watch folder default path for KakaoTalk:** No system convention exists. Recommend prompting user via NSOpenPanel during onboarding (defaults to `~/Documents/KakaoTalk/`).
4. **Confidentiality tier defaults:** Should new projects default to `internal` (safe) or `shared` (convenient)? Recommend `internal` with one-click promotion to `shared`.
5. **AI daily budget default:** Pitfall 12 suggests $1/day. Confirm — too low risks blocking legitimate use; too high risks runaway. Recommend $1/day initially with telemetry to tune.
6. **Share URL default expiry:** Pitfall 16 suggests 7 days. Confirm — MDs/factories may need longer for slow project cycles. Recommend 7 days with one-click extend per URL.
7. **Multi-track progress UI:** Architecture supports per-track (e.g., Color: Red 80%, Color: Blue 40%), but PROJECT.md only mentions a single % per project. Confirm whether per-track is v1 or v1.x.
8. **JANDI/Swit support (Korean Slack alternatives):** FEATURES.md notes some Korean fashion conglomerates use these instead of Slack. Confirm: v1 Slack-only, or v2 expansion if user pool spans larger conglomerates.
9. **Onboarding skip-step UX:** Should integrations be deferrable to "later," or required up-front? Recommend deferrable with "Connect later" badge to avoid Slack-auth-failure abandonment (UX Pitfall).

---

## Sources

### Primary (HIGH confidence — official docs + version-pinned releases)

- Tauri 2 official docs + plugin overview — https://v2.tauri.app/
- Tauri 2.11.0 release — https://github.com/tauri-apps/tauri/releases
- Notion API 2025-09-03 Upgrade Guide — https://developers.notion.com/docs/upgrade-guide-2025-09-03
- Notion Webhooks Reference — https://developers.notion.com/reference/webhooks
- Slack Events API + Socket Mode — https://docs.slack.dev/apis/events-api/, https://api.slack.com/apis/socket-mode
- Slack Token Rotation — https://docs.slack.dev/authentication/using-token-rotation/
- Gmail API Usage limits + push notifications — https://developers.google.com/workspace/gmail/api/reference/quota, https://developers.google.com/workspace/gmail/api/guides/push
- Google Calendar push notifications — https://developers.google.com/workspace/calendar/api/guides/push
- Anthropic SDK 0.92.0 + prompt caching — https://github.com/anthropics/anthropic-sdk-typescript/releases, https://platform.claude.com/docs/en/build-with-claude/prompt-caching
- Anthropic Rate limits — https://platform.claude.com/docs/en/api/rate-limits
- Cloudflare Workers KV docs (eventual consistency, free tier) — https://developers.cloudflare.com/kv/
- Hono 4.12.16 — https://github.com/honojs/hono/releases
- macOS launchd man page + tutorial — https://keith.github.io/xcode-man-pages/launchd.plist.5.html, https://www.launchd.info/
- NSWorkspace didWakeNotification — https://developer.apple.com/documentation/appkit/nsworkspace/didwakenotification
- macOS QLThumbnailGenerator + Sequoia changes — https://developer.apple.com/documentation/quicklookthumbnailing/qlthumbnailgenerator, https://eclecticlight.co/2024/10/31/how-sequoia-has-changed-quicklook-and-its-thumbnails/
- Sparkle Documentation — https://sparkle-project.org/documentation/
- macOS notarization + Hardened Runtime — https://eclecticlight.co/2021/01/07/notarization-the-hardened-runtime/, https://lapcatsoftware.com/articles/hardened-runtime-sandboxing.html
- SQLite single-writer pattern — https://emschwartz.me/psa-your-sqlite-connection-pool-might-be-ruining-your-write-performance/

### Secondary (MEDIUM confidence — community consensus, multi-source agreement)

- `tauri-plugin-keyring` (community by HuakunShen) — https://github.com/HuakunShen/tauri-plugin-keyring
- `tauri-plugin-sparkle-updater` v0.2.4 (community by ahonn) — https://github.com/ahonn/tauri-plugin-sparkle-updater
- Sunsama/Motion/Akiflow comparison + daily-planning patterns — Morgen, alfred, official sites (Sunsama, Motion, Akiflow, Reclaim)
- Backbone PLM / Bamboo Rose / Lifecycle PLM / sample.flow — vendor sites + Share PLM industry survey
- Notion API rate limits + 2026 updates — https://fazm.ai/blog/notion-api-rate-limits-2026
- Notion OAuth refresh `invalid_grant` post-mortem — https://nango.dev/blog/notion-oauth-refresh-token-invalid-grant/
- Anthropic prompt caching TTL nerf (Mar 2026: 1h → 5min) — community report verified against official docs
- KakaoTalk export format (Grokipedia + community parsers) — graup/kakaotalk-analyzer, hkboo/kakaotalk_chat_analysis, jooncco/kakaotalk-chat-exporter
- Code Signing and Notarization: Sparkle and Tears (Steinberger) — https://steipete.me/posts/2025/code-signing-and-notarization-sparkle-and-tears
- Engineering Challenges of Bi-Directional Sync (Stacksync) — https://www.stacksync.com/blog/the-engineering-challenges-of-bi-directional-sync-why-two-one-way-pipelines-fail

### Tertiary (LOW confidence — single source or inference, validate during implementation)

- Korean honorific drafting prompt patterns — inferred from Anthropic Korean-language quality reputation; no published prompt-engineering paper specific to 존댓말/반말 register selection.
- KakaoTalk Mac vs Windows export format divergence — inferred from cross-version community parser variance; no official spec.
- Gmail History API `messagesAdded` inconsistency — gmailpush GitHub issue + one community blog; not in official docs.
- macOS Sequoia FSEvents permission model under sandboxed-vs-not — inferred from xojo.com sandboxing primer + lapcat hardened-runtime article.
- Anthropic billing for errored requests (some account types) — Anthropic billing guide note; needs production validation.
- Manufacturing-domain pitfalls (Pitfalls 16 + 17) — industry articles (Colab, LeelineBags, Canvas GFX) extrapolated to designer-app context; no domain-specific app post-mortem available.

Per-file detailed source lists in [STACK.md](./STACK.md), [FEATURES.md](./FEATURES.md), [ARCHITECTURE.md](./ARCHITECTURE.md), [PITFALLS.md](./PITFALLS.md).

---
*Research completed: 2026-05-03*
*Ready for roadmap: yes*
