# Roadmap: Designer Dashboard

**Created:** 2026-05-03
**Granularity:** standard
**Phases:** 9 (P0–P8)
**Coverage:** 100/100 v1 requirements mapped
**Mode:** yolo · parallelization enabled

## Goal

Ship a macOS menu-bar app that compresses a Korean product-development designer's morning routine from 30–60 minutes to 5 minutes by aggregating Slack/Gmail/GCal/Notion/KakaoTalk into a Notion-as-SOTR core, generating an AI daily brief, drafting outbound MD/factory messages behind an approval gate, and exposing sanitized read-only share URLs.

## Phases

- [ ] **Phase 0: Foundation Shell** — Tray, autostart, Keychain, SQLite WAL, OAuth single-flight, AI budget primitive, Notion rate limiter, Hardened Runtime baseline, wake observer
- [ ] **Phase 1: Notion SOTR + Schema Wizard + LWW** — `data_source_id` discovery, 12-stage milestone template, polling, LWW reconciler, 4-state notion_state, schema-by-id
- [ ] **Phase 2: Cloudflare Bridge + Share Skeleton + SharedView Privacy Boundary** — `share-worker`, `bridge-worker`, sanitizer-first SharedView Rust type, token issue/revoke, confidentiality tiers, sensitive-term lint
- [ ] **Phase 3: Slack + AI Classifier (FIRST AI integration)** — Socket Mode + bridge fallback, dedup, subtype handling, Haiku 4.5 classifier, idempotency cache, prompt caching exercised
- [ ] **Phase 4: Gmail + Google Calendar (paired)** — Shared Google OAuth (Production status), historyId/syncToken polling, quota-aware limiter, thread-level classification, milestone↔event sync
- [ ] **Phase 5: KakaoTalk Watch Folder + Adobe Asset Recognition** — NSOpenPanel-picked watch folder, 4-locale parser, size-stable debounce, idempotency hash, native QL thumbnails with fallback chain
- [ ] **Phase 6: AI Brief + Today-View + Snooze/Roll-over + Search** — In-app tokio scheduler + wake re-anchor, FTS5 search, snooze/roll-over UX, AI consent gate, blocked-state detection
- [ ] **Phase 7: Send Cards + Approval Gate + Recipient Verification** — Structured-output ID validation, deterministic recipient resolution, honorific-aware drafting, recipient/time-zone guards, Communication Log writeback
- [ ] **Phase 8: Polish + Distribution + Notion Webhooks (additive)** — 5-step onboarding wizard, code-signed/notarized DMG, `tauri-plugin-updater` with R2-hosted ed25519 manifest, Notion webhook bridge enabled, settings UI

## Parallelization

- **P3 unlocks P4 ∥ P5.** After Phase 3 establishes AI integration patterns (classifier, dedup, budget exercised), Phase 4 (Gmail + GCal) and Phase 5 (KakaoTalk + Adobe) can proceed concurrently — they share no upstream dependencies and write through the same Notion/sanitizer layer already built in P1/P2.
- **All other phases are sequential** because each builds primitives the next consumes (P0 → P1 → P2 → P3, then P6 → P7 → P8).

## Phase Details

### Phase 0: Foundation Shell

**Goal**: A signed, notarizable Tauri menu-bar app that auto-launches at login, stores OAuth tokens safely in Keychain (single-flight protected), persists state in SQLite WAL, recovers from sleep/wake, and provides the rate-limiter + AI-budget primitives every later phase will consume.
**Depends on**: Nothing (first phase)
**Requirements**: FOUND-01, FOUND-02, FOUND-03, FOUND-04, FOUND-05, FOUND-06, NOTION-09, COST-01, COST-02, DIST-02
**Success Criteria** (what must be TRUE):
  1. App launches on a fresh macOS install at user login, the tray icon is visible, and clicking it opens the main window in <200ms.
  2. A synthetic OAuth refresh storm (5 concurrent calls) results in exactly one token-endpoint request and a single Keychain write — and the global ⌥+Space hotkey opens the window from any app.
  3. After sleeping the Mac and waking it, all sync cursors are intact and a forced re-sync round runs automatically.
  4. `xcrun notarytool submit --wait` exits 0 in CI on a Hardened-Runtime-enabled bundle, and a `Db::write` test under load shows zero `SQLITE_BUSY` errors.
  5. AI budget primitive: setting daily ceiling to $0.01 in SQLite causes the next call to be refused with a user-visible notification; Notion limiter holds steady at ≤2.5 req/s under a 100-call burst.
**Plans**: TBD
**UI hint**: yes

### Phase 1: Notion SOTR + Schema Wizard + LWW

**Goal**: Notion is the durable spine. The schema wizard discovers `data_source_id`, auto-creates required properties, seeds the 12-stage fashion milestone template, and the LWW reconciler keeps local SQLite and Notion in sync without ghost projects, silent overwrites, or schema drift.
**Depends on**: Phase 0 (rate limiter, Keychain, SQLite, OAuth single-flight)
**Requirements**: NOTION-01, NOTION-02, NOTION-03, NOTION-04, NOTION-05, NOTION-06, NOTION-07, NOTION-08, TODAY-04, COST-05
**Success Criteria** (what must be TRUE):
  1. User pastes a Notion integration token in the wizard; within 30s the app discovers the project DB via `data_source_id`, adds any missing required property, and seeds the 12-stage milestone template (기획→…→출고).
  2. A Notion change appears in the app within 5 minutes; an in-app milestone/task check appears in Notion immediately (and `last_edited_by: <our-integration>` lets the reconciler skip its own writes — no AI loop).
  3. Concurrent edit of the same project (browser + app) shows a 3-way conflict UX, the loser's diff is appended to the Communication Log, and progress = 60%·milestones + 40%·tasks renders correctly.
  4. Archive→restore round-trip in Notion produces zero duplicate projects locally; trashed projects disappear from the dashboard within one reconciliation pass (4-state `notion_state` enum).
  5. User renames a Notion property; the schema-by-id mapping survives, and a "이름 변경 감지" prompt asks the user to confirm the rename rather than silently re-binding.
**Plans**: TBD
**UI hint**: yes

### Phase 2: Cloudflare Bridge + Share Skeleton + SharedView Privacy Boundary

**Goal**: Establish the privacy chokepoint and the webhook/share infrastructure before any channel can leak. The `SharedView` Rust type structurally cannot contain message bodies, the `share-worker` serves token-gated read-only HTML on `*.workers.dev`, and the `bridge-worker` is ready to receive Slack/Notion webhooks.
**Depends on**: Phase 1 (Notion data to publish)
**Requirements**: SHARE-01, SHARE-02, SHARE-03, SHARE-04, SHARE-05, SHARE-06, SHARE-07, SHARE-08, PRIV-03, PRIV-05
**Success Criteria** (what must be TRUE):
  1. User creates a share URL on a `shared`-tier project; the page loads on external mobile in <5s and contains zero strings matching message-body or email-subject sentinels (CI grep test passes).
  2. `SharedView` Rust type compiles only because it has no `body`/`preview`/`snippet`/`email_subject` fields — adding any forbidden field name fails the CI assertion.
  3. Revoking a token causes `*.workers.dev/p/{slug}/{token}` to return 403 within 5s on the next request, and the same project marked `internal` cannot have a share URL created at all.
  4. Sharing a project with the word `단가`/`원가`/`마진`/competitor name in any user-written field surfaces a pre-publish warning with the offending term highlighted; user can override or edit.
  5. "Delete all data" in settings wipes local SQLite and revokes every active KV token within 5s; subsequent `*.workers.dev` requests return 403 for all previously-issued tokens.
**Plans**: TBD
**UI hint**: yes

### Phase 3: Slack + AI Classifier (FIRST AI integration)

**Goal**: Slack messages flow into the system in real-time (Socket Mode) with an offline-catch-up path through the bridge worker; AI classifies via Haiku 4.5 only, with prompt caching, idempotency, and a manual-review queue for low-confidence routes. This is where the AI-budget primitive from P0 finally gets exercised at production volume.
**Depends on**: Phase 0 (budget, Keychain), Phase 1 (Notion writer), Phase 2 (bridge worker, sanitizer)
**Requirements**: SLACK-01, SLACK-02, SLACK-03, SLACK-04, SLACK-05, SLACK-06, SLACK-07, COST-03, COST-04, COST-06, COST-07, PRIV-01, PRIV-02, PRIV-04
**Success Criteria** (what must be TRUE):
  1. Slack handler returns ACK in <100ms under load; the same `(channel_id, event_ts)` arriving via both Socket Mode and the bridge produces exactly one Notion Communication Log entry.
  2. Editing a Slack message in the workspace updates the corresponding Notion entry within 1 minute; deleting a message redacts the local + Notion entry in the same window (subtype handling for `message_changed`/`message_deleted`/`bot_message`).
  3. Channel→project mapping accuracy ≥90% on a 200-message synthetic test set; messages with AI confidence <0.7 land in a "어디에 속해요?" review queue rather than auto-routing.
  4. First-run AI consent screen requires explicit opt-in before any Anthropic egress; declining produces zero outbound bytes (verified by network capture). Per-channel AI-off toggle stops classifier for that channel only.
  5. Prompt cache hit rate ≥70% after 200 classifications; identical `(model, system_prompt, user_message)` calls within 7 days return cached responses; classifier code path can never construct a Sonnet request (compile-time guard).
**Plans**: TBD
**UI hint**: yes

### Phase 4: Gmail + Google Calendar (paired)

**Goal**: A single Google OAuth flow (Production status) connects both Gmail and GCal; both sync via incremental polling with a quota-aware limiter; Gmail classifies at thread level (never message level); GCal events stay in two-way sync with milestone due dates.
**Depends on**: Phase 3 (AI patterns, classifier infrastructure)
**Requirements**: GMAIL-01, GMAIL-02, GMAIL-03, GMAIL-04, GMAIL-05, GMAIL-06, GCAL-01, GCAL-02, GCAL-03, GCAL-04
**Success Criteria** (what must be TRUE):
  1. User completes one Google OAuth flow; both Gmail and GCal scopes are authorized; the app is in Google's Production state (no 7-day testing-mode expiry).
  2. Labeled emails reach 100% project-mapping accuracy; unlabeled emails attempt sender/subject matching, then fall into the "어디에 속해요?" queue — and a single thread never appears under two projects (thread-level classification).
  3. Synthetic 50-call burst at high-cost methods (`messages.get` × 50) is throttled by the token-bucket limiter to <250 quota-units/sec/user with zero 429s; `historyId`-based incremental sync requires no full inbox rescan.
  4. Editing a milestone's `due_date` in the app creates or updates the corresponding GCal event within 90s; `syncToken`-based GCal polling pulls only changed events on each tick.
  5. Email bodies appear in local SQLite only; network capture during a 1-hour active session shows zero email-body bytes egressing to Cloudflare KV.
**Plans**: TBD

### Phase 5: KakaoTalk Watch Folder + Adobe Asset Recognition

**Goal**: KakaoTalk `.txt` exports dropped into a user-picked folder are parsed (KR PC / KR Mobile / EN PC / ZH PC), de-duplicated, and AI-extracted into Notion within 1 minute; Adobe-format files (`.psd/.ai/.indd/.sketch/.fig/.png/.jpg/.pdf`) uploaded to Slack appear as thumbnails in the project gallery within 30s, with graceful degradation for formats native QL can't handle.
**Depends on**: Phase 3 (AI patterns shared with KakaoTalk extraction); parallelizable with Phase 4
**Requirements**: KAKAO-01, KAKAO-02, KAKAO-03, KAKAO-04, KAKAO-05, KAKAO-06, KAKAO-07, KAKAO-08, ADOBE-01, ADOBE-02, ADOBE-03, ADOBE-04
**Success Criteria** (what must be TRUE):
  1. User picks a watch folder via NSOpenPanel (no Full Disk Access required); a mid-write `.txt` (size still changing) does not trigger a parse — only files stable for ≥2s are processed.
  2. CI fixtures for all 4 locale formats (KR PC / KR Mobile / EN PC / ZH PC) parse with correct dates including 오전 12:30 → 00:30 and 오후 3:21 → 15:21; UTF-8 BOM and CRLF/LF variants succeed; unsupported locales surface a clear error message.
  3. Re-importing the same `.txt` produces zero new rows (SHA256 idempotency on `(timestamp, sender, body)`); a brand-new chat-room name triggers a "어떤 프로젝트?" mapping wizard whose answer is remembered.
  4. AI extracts decisions/dates/requests from the `.txt` and writes them to the matching Notion project within 60s of file stability.
  5. A `.psd` uploaded to Slack appears as a thumbnail in the project gallery in <30s; `.ai`/`.indd` for which native QL fails fall through `qlmanage` → embedded JPEG → placeholder + "외부에서 보기" without blocking the workflow; filename pattern matches (시안 v\d, sample, _v\d, seq\d) auto-tag the version.
**Plans**: TBD
**UI hint**: yes

### Phase 6: AI Brief + Today-View + Snooze/Roll-over + Search

**Goal**: All channels feeding the system, the daily 8 AM brief fires reliably (in-app tokio task with wake re-anchor), the Today-View dashboard supports snooze/roll-over so the brief never decays into a graveyard, and SQLite FTS5 powers cross-channel search across all ingested data.
**Depends on**: Phase 3, 4, 5 (all data sources feeding briefer)
**Requirements**: BRIEF-01, BRIEF-02, BRIEF-03, BRIEF-04, BRIEF-05, BRIEF-06, BRIEF-07, TODAY-01, TODAY-02, TODAY-03, SEARCH-01, SEARCH-02, SEARCH-03, SEARCH-04
**Success Criteria** (what must be TRUE):
  1. A macOS notification fires at the user-configured time (default 8:00); clicking it opens the dashboard in <5s with "오늘 액션 N건, 가장 급한 일: X" prominently shown — and the launchd plist contains `RunAtLoad=true` only (no `StartCalendarInterval`).
  2. After the Mac sleeps through 8 AM and wakes at 11 AM, the wake observer detects the missed brief and fires it within seconds — verified by a sleep-fixture test.
  3. Each Today-View action supports [완료/스누즈 N분/내일로 이월/주말로 이월/취소]; items not handled within 7 days are auto-rolled per the daily roll-over policy rather than piling up untouched.
  4. SQLite FTS5 returns search results across Slack/Gmail/KakaoTalk/Notion in <500ms over a 10k-message synthetic corpus; results include channel, project, date, sender; clicking a Slack/Notion/Gmail result opens a deep link to the original.
  5. AI insights ("샘플 사이클이 평균보다 늦음") appear as project-page annotations only and never affect the 60/40 progress score; over 4 weeks of self-measurement ≥70% of brief actions are actually handled the same day.
**Plans**: TBD
**UI hint**: yes

### Phase 7: Send Cards + Approval Gate + Recipient Verification

**Goal**: AI drafts outbound messages with structured-output JSON whose entity IDs must reference real SQLite rows; recipient resolution is deterministic code (AI never populates `to:`); the verification UI shows recipient name/channel/cross-project association/local time; honorifics adapt automatically to recipient role; every send writes to the Communication Log and logs an AI-vs-final diff.
**Depends on**: Phase 6 (search for grounding, dashboard for UI), Phase 1 (Communication Log writer), Phase 3/4 (Slack and Gmail send paths)
**Requirements**: SEND-01, SEND-02, SEND-03, SEND-04, SEND-05, SEND-06, SEND-07, SEND-08, SEND-09, SEND-10, SEND-11, SEND-12
**Success Criteria** (what must be TRUE):
  1. AI draft cards appear with [수정/발송/무시/다른 채널로] buttons; a draft referencing a fabricated `recipient_id`/`project_id`/`milestone_id`/`sample_id` is rejected by the validation layer (AI is asked to retry with a constrained prompt).
  2. Recipient verification block above the draft body shows name, channel (Slack DM/Gmail), cross-project association count, and recipient local time; sending into recipient's 22:00–06:00 requires an extra confirm.
  3. AI prompt for Project A receives only Project A context; an adversarial test set proves no Project B fact appears in a Project A draft (cross-contamination prevention); honorific selection (존댓말/반말/영문) matches recipient role across MD / 공장 / 내부 fixtures.
  4. Sending via Slack `chat.postMessage` or Gmail `drafts.send` appends an entry to the Notion project's Communication Log immediately and logs the AI body vs final-sent diff for the weekly "AI changed N facts" metric.
  5. Over 4 weeks of dogfooding, AI candidate adoption rate (sent or sent-after-edit) ≥50%, and zero confirmed wrong-factory or wrong-project sends.
**Plans**: TBD
**UI hint**: yes

### Phase 8: Polish + Distribution + Notion Webhooks (additive)

**Goal**: A non-technical Korean designer can install, onboard in 5 steps (each integration skippable), receive auto-updates safely from a code-signed/notarized DMG via `tauri-plugin-updater` (ed25519 manifest on R2), and benefit from real-time Notion webhooks layered over the polling baseline as additive defense in depth.
**Depends on**: All prior phases
**Requirements**: ONBOARD-01, ONBOARD-02, ONBOARD-03, ONBOARD-04, DIST-01, DIST-03, DIST-04, DIST-05
**Success Criteria** (what must be TRUE):
  1. First-run wizard walks Notion → Google → Slack → KakaoTalk folder → 알림 시간 in 5 steps; each step has a "나중에 연결" option that doesn't block; settings allows re-entry to any step.
  2. Beta cohort of 5 users completes onboarding at ≥90% completion rate; every step's done/skip state is visible in settings.
  3. CI runs `xcrun notarytool submit --wait` on every PR and blocks merge on failure; `CFBundleVersion` increments automatically each release; the resulting DMG installs on a fresh macOS without Gatekeeper warnings.
  4. `tauri-plugin-updater` pulls an ed25519-signed manifest from R2 hosted at the project's `*.workers.dev` URL and successfully upgrades a 0.x build to 0.x+1 on a clean install.
  5. Notion webhooks are wired via the `bridge-worker` from P2 and arrive within seconds; if webhooks fail, the existing 5-min polling baseline still satisfies the NOTION-04 sync requirement (additive, never primary).
**Plans**: TBD
**UI hint**: yes

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 0. Foundation Shell | 0/0 | Not started | - |
| 1. Notion SOTR + Schema Wizard + LWW | 0/0 | Not started | - |
| 2. Cloudflare Bridge + Share Skeleton + SharedView | 0/0 | Not started | - |
| 3. Slack + AI Classifier | 0/0 | Not started | - |
| 4. Gmail + Google Calendar | 0/0 | Not started | - |
| 5. KakaoTalk Watch Folder + Adobe Asset Recognition | 0/0 | Not started | - |
| 6. AI Brief + Today-View + Snooze/Roll-over + Search | 0/0 | Not started | - |
| 7. Send Cards + Approval Gate + Recipient Verification | 0/0 | Not started | - |
| 8. Polish + Distribution + Notion Webhooks | 0/0 | Not started | - |

## Notes

- **Phase numbering uses P0–P8** (matches research SUMMARY.md and ARCHITECTURE.md). STATE.md `current_phase=0`.
- **Privacy-boundary-first**: P2 (`SharedView` sanitizer) lands before any channel that ingests message bodies (P3+). The Rust type system enforces "no body fields" at compile time.
- **AI-budget-primitive in P0, exercised first in P3**: avoids the cost-runaway pitfall (Pitfall 12). Classifier code path is compile-time-restricted to Haiku 4.5.
- **OAuth single-flight in P0**: avoids retrofitting refresh-race protection into every channel later (Pitfall 6).
- **Conflict-resolved scheduler**: P6 uses launchd `RunAtLoad=true` only + in-app tokio task + `NSWorkspaceDidWakeNotification` re-anchor (NOT `StartCalendarInterval`).
- **Updater choice**: `tauri-plugin-updater` (official, ed25519) over `tauri-plugin-sparkle-updater` per Conflict 2 resolution. PROJECT.md "Sparkle" wording treated as shorthand for "auto-update mechanism."
- **Notion webhooks are additive in P8**, never primary; the 2-min polling baseline from P1 satisfies NOTION-04 alone.
- **Coverage**: 100/100 v1 REQ-IDs mapped, no orphans, no duplicates. See REQUIREMENTS.md Traceability section.

---
*Last updated: 2026-05-03 after roadmap creation*
