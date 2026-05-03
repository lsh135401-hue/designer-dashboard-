# Feature Research

**Domain:** Morning brief / unified inbox / project pulse for product-development designers in manufacturing (fashion/lifestyle/accessories)
**Researched:** 2026-05-03
**Confidence:** MEDIUM-HIGH (HIGH on productivity-tool category — many active competitors and reviews; MEDIUM on PLM/manufacturing slice — gated PLM tools, less public detail; MEDIUM on Korean-context patterns — Korean docs sparse in English search)

## Executive Summary

Three product categories converge on this user:

1. **Daily-brief / unified-inbox tools** (Sunsama, Motion, Akiflow, Reclaim) own the "morning ritual + cross-channel ingest" pattern. They have **trained users to expect**: keyboard-first command bar, drag-from-Slack-or-email-to-task, snooze, end-of-day shutdown ritual, time-blocking. They almost universally do NOT do AI message *drafting* — they do AI *scheduling*. That gap is where Designer Dashboard differentiates.

2. **Fashion PLM tools** (Backbone, Bamboo Rose TotalPLM, Lifecycle PLM, sample.flow, WFX) own the "manufacturing milestones + sample tracking" pattern. They are **enterprise-priced, browser-heavy, and built for teams of 20+**. A solo designer drowns in their UX. Backbone is the most "designer-friendly" but still assumes a multi-seat workflow. None have a menu-bar daily-brief layer.

3. **Always-on AI menu bar tools** (ChatGPT Mac, Raycast AI, Claude Desktop) have set the bar for "summon AI from anywhere with one hotkey." Users now expect ⌥+Space-style invocation, persistent menu-bar icon, and instant-render windows.

**Designer Dashboard's wedge:** the intersection none of them sit in — *PLM-aware milestone tracking + Sunsama-style daily ritual + AI-drafted send-to-MD/factory cards + Korean-tool ingest (KakaoTalk .txt)*. The PROJECT.md Active list covers ~85% of table-stakes; gaps are noted in the "Missing from PROJECT.md" section below.

## Feature Landscape

### Table Stakes (Users Expect These — Must-Have for v1)

These exist across 3+ category competitors. Missing them makes the product feel broken or unfinished to a user coming from Sunsama/Notion/Slack.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Menu-bar tray icon with one-click expand | Raycast, ChatGPT Mac, Claude Desktop all do this | S | Already in PROJECT.md (Foundation) |
| OS-boot autostart + Keychain token persistence | Any always-on Mac app | S | Already in PROJECT.md (Foundation) |
| Global hotkey to summon dashboard | Raycast (⌥+Space), ChatGPT (⌥+Space), Akiflow (Cmd+K) | S | **Missing in PROJECT.md** — add a global hotkey requirement |
| Two-way Notion sync | Notion-as-SOTR is a category pattern (Sunsama, Akiflow, Cron all sync to Notion) | L | Already in PROJECT.md (Notion SOTR) |
| Two-way Google Calendar sync | Sunsama, Motion, Reclaim, Akiflow, Cron all do this | M | Already in PROJECT.md (Calendar) |
| Gmail label → project mapping | Sunsama "drag Gmail to task," Akiflow Gmail integration | M | Already in PROJECT.md (Gmail) |
| Slack message → task / project mapping | Sunsama Slack integration, Akiflow, Notion's /notion task slash command | M | Already in PROJECT.md (Slack) |
| Today-view dashboard with today's actions | Universal in Sunsama, Motion, Akiflow, Reclaim | S | Already in PROJECT.md (AI Brief — "오늘의 액션") |
| macOS notification at scheduled time | Sunsama daily-plan reminder, Motion daily reschedule, all calendar apps | S | Already in PROJECT.md (AI Brief) |
| Snooze / defer task to later | Akiflow, Sunsama, Things 3 — universal | S | **Missing in PROJECT.md** — needed because daily brief items will get deferred |
| Roll over unfinished tasks to next day | Sunsama core ritual, Akiflow inbox, Things 3 | S | **Missing in PROJECT.md** — without this, brief becomes graveyard of stale items |
| Search across ingested data | Klu, Notion AI, Akiflow Cmd+F, Raycast — universal | M | **Missing in PROJECT.md** — user will absolutely expect to search "그 공장 메시지 어디 있더라" |
| Project list / project detail page | Every PLM tool, every PM tool | S | Implicit in PROJECT.md but not explicit |
| File/image attachments visible per project | Slack, Notion, every PLM tool, sample-tracking tools | M | Partially in PROJECT.md (Slack/Adobe gallery) — extend to Gmail attachments |
| Communication log (who-said-what to whom, when) | Bamboo Rose, Backbone, Notion comments, Slack threads | M | Already in PROJECT.md (Send → Communication Log) |
| Code-signed installer + auto-update | Sparkle is Mac standard; users abandon Gatekeeper-blocked apps | M | Already in PROJECT.md (Distribution) |
| OAuth flow for each connected service | Every productivity app on the market | M | Already in PROJECT.md (Foundation) |
| Onboarding wizard | Sunsama 5-step setup, Akiflow setup wizard, Motion onboarding | M | Already in PROJECT.md (Onboarding) |
| Manufacturing milestone template (proto/sample/PP/TOP/bulk) | Backbone, Bamboo Rose, Lifecycle PLM, sample.flow all expose these stages | M | Implicit in PROJECT.md (12-stage template in Context) — should be **explicit Active requirement** so v1 ships with the template baked in, not requiring user setup |
| Progress indicator per project | Every PM tool — % complete is universal | S | Already in PROJECT.md (Progress) |
| Read-only share link with expiry | Notion's "Anyone with link + Link expires," Loom share, Figma | M | Already in PROJECT.md (Share) |

### Differentiators (Competitive Advantage — Strong v1 Candidates)

These don't exist (or are weak) in the competitive set and align with the Core Value of "5분 안에 오늘의 액션과 줄 지시 초안."

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **AI-drafted send cards (MD/factory)** with edit→send→ignore gate | Sunsama/Motion/Akiflow surface tasks but **never draft outbound messages**. Notion AI summarizes but doesn't propose sends. PLM tools have approval workflows but no AI drafting. This is the unique wedge. | L | Already in PROJECT.md (Send) — keep as headline differentiator |
| **KakaoTalk .txt watch-folder ingest** | No Western tool ingests KakaoTalk. Critical for Korean designers — KakaoTalk is where 50%+ of factory chatter lives. The watch-folder pattern (Hazel-style) is clever workaround for absent API. | M | Already in PROJECT.md (KakaoTalk) — uniquely Korean differentiator |
| **Korean-first UI + AI tone** (Sonnet 4.6 한국어) | Sunsama/Motion/Akiflow are English-only or shallow i18n. JANDI/Swit cover messaging but not unified-brief. AI drafting in Korean honorifics (존댓말) for MD/factory is non-trivial cultural fit. | M | Implicit in PROJECT.md Constraints — make **explicit Active requirement** that AI drafts respect 존댓말/반말 distinction per recipient |
| **Read-only share URL that hides message bodies** (only progress/dates exposed) | Notion's public share exposes everything; Backbone tech-pack share exposes specs. None expose *just* sanitized status. This is the sharing pattern MD/factory actually need without leaking designer's private notes. | M | Already in PROJECT.md (Share) |
| **Adobe file recognition from Slack attachments (.psd/.ai/.indd thumbnails)** | Slack shows these as generic file icons. Backbone has an Illustrator plugin but only for the Backbone workflow. Auto-thumbnailing inside the project gallery is a small feature with outsized recognition value for designers. | M | Already in PROJECT.md (Slack/Adobe) |
| **AI insights as annotations, NOT scoring inputs** | Motion auto-prioritizes (and is wrong); Reclaim auto-schedules (and is wrong). Designer Dashboard's "AI suggests, human owns score" is a trust-positioning differentiator users will value once they're burned by Motion. | S | Already in PROJECT.md (AI Brief) |
| **Local-first message-body storage** (SQLite, never to cloud) | Sunsama/Motion/Akiflow all store everything in their cloud. For a designer handling unreleased product designs, this is a real fear and a real selling point. | M | Already in PROJECT.md (Privacy) |
| **Manufacturing-aware milestone template baked in** (12-stage 기획→출고) | Sunsama/Motion/Akiflow are domain-agnostic. PLM tools have the template but aren't menu-bar daily-brief tools. Shipping with the 12-stage template pre-built saves 2 hours of user setup. | S | Implicit in PROJECT.md — make **explicit** |
| **Shutdown ritual + tomorrow preview** | Sunsama owns this in productivity world. Adding it gives designers the closure pattern Sunsama users love, plus AI can draft tomorrow's send-cards overnight. | M | **Missing in PROJECT.md** — strong addition |
| **Global Cmd+K command bar** (jump to project, create task, send draft) | Raycast/Akiflow/Linear set this expectation. Without it, app feels slow. | M | **Missing in PROJECT.md** — should be added |

### Anti-Features (Deliberately Do NOT Build)

Features that look good in user requests but create real harm or scope explosion. Some are already correctly in PROJECT.md's Out of Scope; new ones surfaced below.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Auto-send (no approval gate) | "Save me even more time" | Designer tone errors damage MD/factory relationships permanently. One mis-sent honorific can sour a 6-month vendor relationship in Korea. | Always-approval gate. Opt-in only for structured reminders ("D-3 샘플 도착") | 
| KakaoTalk official API integration | "Just talk to the API" | No personal-chat API exists. Unofficial automation gets accounts banned. PROJECT.md correctly handles this. | .txt watch-folder (already chosen) |
| Mobile companion app | Designers want to glance on phone | App-store distribution + push infra + auth replication doubles scope. Read-only share URL covers 80% of glance-on-phone need. | Mobile-friendly share URL (already chosen) |
| Multi-workspace Slack/Google support | "I have personal and work Google" | Token storage, account-switcher UI, scope-of-permissions all multiply. | v2 — single-account v1 (already chosen) |
| Real-time chat between designer and MD/factory inside the app | "Why leave the app?" | Re-implementing Slack/KakaoTalk inside is a death spiral. Network effects are zero (MD/factory won't install your app). | Read-only share + draft-and-send through user's existing channels |
| Auto-prioritize tasks by AI (Motion-style) | "Let AI decide what's important" | Designers have intricate tone/relationship/strategic priorities AI cannot infer. Wrong prioritization erodes trust fast. | AI suggests, human owns priority (already PROJECT.md philosophy) |
| Generic project templates | "More templates = more useful" | Bloat. Manufacturing designers need ONE good 12-stage template, not 20 mediocre ones. | Ship one excellent fashion-product template; let user customize per project |
| Time-tracking / timesheet | PM tools all have it | Solo designer doesn't bill hours; adding it is feature creep into a different category | Skip entirely |
| Built-in video calls / screen sharing | "Talk to factory in-app" | Already covered by Zoom/Slack huddles; impossible to compete | Skip entirely |
| AI auto-translate factory messages to Korean | "Chinese factory sends in Mandarin" | Translation errors in technical specs cause production defects. High blast radius. | Show original + AI-suggested *gloss*, with explicit "consult before acting" warning |
| Public roadmap / kanban board | Trello/Linear-style | Different category. SOTR is Notion — re-implementing kanban inside violates Notion-as-SOTR constraint | Use Notion's board view; deep-link to it |
| Unlimited file storage / asset library | Backbone has it | Cloud cost + infra burden + duplication of Notion/Drive | Reference files where they live; thumbnails only |
| Custom workflow automation (Zapier-style) | Power users always ask | Maintaining a workflow engine is a separate product | Use Notion automations or Make/n8n externally |

## Feature Dependencies

```
[OAuth + Keychain] ─── required by ───> [All channel ingests]

[Notion 2-way sync (SOTR)]
    ├──required by──> [Project list / detail]
    ├──required by──> [AI Brief generation]
    ├──required by──> [Send cards → Communication Log]
    └──required by──> [Read-only share URL] (snapshots from Notion state)

[Calendar sync] ──enhances──> [AI Brief] (today's meetings inform brief)

[Gmail/Slack/KakaoTalk ingest] ──required by──> [AI Brief inputs]
                                ──required by──> [Send draft generation]
                                ──required by──> [Search across channels]

[Adobe file recognition] ──depends on──> [Slack ingest]

[Send card flow]
    ├──requires──> [Notion sync] (for project context)
    ├──requires──> [AI draft generation]
    └──requires──> [Slack send + Gmail send capability]

[Read-only share URL]
    ├──requires──> [Notion sync] (snapshot source)
    ├──requires──> [Cloudflare Workers + KV + R2]
    └──requires──> [Token revocation flow] (immediate 403)

[Shutdown ritual] ──depends on──> [Today-view dashboard]
                  ──enhances────> [AI Brief next-day generation]

[Search across channels] ──depends on──> [Local SQLite index of all ingested messages]
                         ──enhances────> [Send card grounding]

[Cmd+K command bar] ──depends on──> [Search index] (for jump-to-project)

[Manufacturing milestone template] ──depends on──> [Notion DB schema wizard]
                                    ──enhances───> [Progress calculation]
                                    ──enhances───> [AI Brief] (knows what stage to nudge)
```

### Dependency Notes

- **Notion sync is the spine.** Almost everything else either feeds it or reads from it. Delays here block 80% of v1.
- **Search across channels is bigger than it looks.** Requires indexing all ingested messages locally and a search UI. But once it exists, Cmd+K, AI grounding, and "find that conversation" all light up.
- **Schema wizard + milestone template are tightly coupled.** Ship together — the wizard creates the properties the template needs.
- **Send flow has the longest dependency chain.** It needs Notion + AI + Slack/Gmail send capability + Communication Log writeback. Plan as a vertical slice.

## MVP Definition

### Launch With (v1)

The minimum to validate "designer can compress 30-60min morning routine into 5 minutes."

- [ ] **Foundation** — Menu-bar tray, autostart, Keychain, OAuth (PROJECT.md)
- [ ] **Global hotkey** to summon dashboard (NEW — table stakes)
- [ ] **Notion 2-way sync + schema wizard** with **fashion 12-stage milestone template baked in** (PROJECT.md, but make template explicit)
- [ ] **Calendar 2-way sync** (PROJECT.md)
- [ ] **Gmail label-based project mapping** (PROJECT.md)
- [ ] **Slack message-to-project mapping + Adobe file thumbnail recognition** (PROJECT.md)
- [ ] **KakaoTalk .txt watch-folder ingest** with dedup (PROJECT.md) — Korean differentiator
- [ ] **AI Daily Brief** at scheduled time, macOS notification, click → 5s dashboard open (PROJECT.md)
- [ ] **Today-view dashboard** with today's actions, with **snooze + roll-over to tomorrow** (PROJECT.md + NEW table stakes)
- [ ] **Search across all ingested channels** (NEW — table stakes; users will leave without this)
- [ ] **Send card flow**: AI draft → edit/send/ignore → Slack or Gmail send → Communication Log writeback (PROJECT.md). **Honorific-aware drafting (존댓말 per recipient role).**
- [ ] **Read-only share URL** with token issue/expiry/revoke, sanitized payload (no message bodies) (PROJECT.md)
- [ ] **Progress calculation** (60% milestone + 40% task) (PROJECT.md)
- [ ] **Local SQLite for message bodies, never to cloud** (PROJECT.md)
- [ ] **Onboarding wizard** (PROJECT.md)
- [ ] **Code-signed DMG + Sparkle auto-update** (PROJECT.md)

### Add After Validation (v1.x)

Add once core ritual is validated and users are returning daily.

- [ ] **Shutdown ritual** with tomorrow preview (Sunsama-pattern; high-value but adds time-on-task)
- [ ] **Cmd+K global command bar** (jump-to-project, create-task, draft-send) — only worth building once you have enough things to jump between
- [ ] **AI insights as project-page annotations** (PROJECT.md AI Brief sub-item) — defer if AI Brief itself ships first
- [ ] **Recurring task templates** (e.g., "every Monday: send weekly factory progress") — useful but not blocker
- [ ] **CSV/PDF export of project status report** — MD/factory will eventually request a PDF for offline review; not blocker because Notion can export
- [ ] **Multi-language send drafting** (auto-detect 한/中/英 recipient and draft accordingly) — PROJECT.md has stub; expand once Korean works
- [ ] **Per-project custom milestone overrides** (e.g., this jewelry project skips QC stage) — start with one fixed template, add customization once users ask

### Future Consideration (v2+)

- [ ] **Multi-workspace Slack/Google** (PROJECT.md Out of Scope) — only if power users demand
- [ ] **Adobe Creative Cloud Libraries direct integration** (PROJECT.md Out of Scope) — heavy OAuth, low marginal value
- [ ] **Windows/Linux desktop** (PROJECT.md Out of Scope) — Tauri makes feasible; gate on demand
- [ ] **Mobile companion app** — share URL covers 80%; only build if read-only-share friction proves real
- [ ] **Multi-user / team mode** with MD-side write access — requires real auth/permissions infra; v2+ category jump
- [ ] **Custom milestone templates per industry** (lifestyle vs accessories vs apparel each have different stages) — only after fashion template is proven
- [ ] **AI auto-categorization of new Slack channels into projects** without manual mapping — magic but requires significant accuracy work
- [ ] **Voice-to-task** ("Hey, remind me to chase the factory tomorrow") — Mac Dictation / Whisper integration, nice-to-have

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Notion 2-way sync (SOTR) | HIGH | HIGH | P1 |
| AI Daily Brief | HIGH | MEDIUM | P1 |
| Send card flow with approval gate | HIGH | HIGH | P1 |
| KakaoTalk .txt watch-folder | HIGH | MEDIUM | P1 |
| Slack/Gmail/Calendar ingest | HIGH | MEDIUM | P1 |
| Read-only share URL | HIGH | MEDIUM | P1 |
| Menu-bar + global hotkey + autostart | HIGH | LOW | P1 |
| Onboarding wizard | HIGH | MEDIUM | P1 |
| Search across channels | HIGH | MEDIUM | P1 (table stakes) |
| Snooze + roll-over to tomorrow | HIGH | LOW | P1 (table stakes) |
| 12-stage milestone template baked in | HIGH | LOW | P1 (differentiator) |
| Honorific-aware AI drafting | HIGH | MEDIUM | P1 (Korean differentiator) |
| Adobe file thumbnail recognition | MEDIUM | MEDIUM | P1 (already in PROJECT.md) |
| Local-first SQLite (no cloud msg bodies) | HIGH | LOW | P1 (privacy stance) |
| Code-signed DMG + Sparkle | HIGH | MEDIUM | P1 (distribution) |
| Shutdown ritual + tomorrow preview | MEDIUM | MEDIUM | P2 |
| Cmd+K command bar | MEDIUM | MEDIUM | P2 |
| AI insights as page annotations | MEDIUM | LOW | P2 |
| Recurring task templates | MEDIUM | MEDIUM | P2 |
| CSV/PDF export | LOW | LOW | P2 (Notion already exports) |
| Per-project milestone customization | MEDIUM | MEDIUM | P2 |
| Multi-language send | MEDIUM | MEDIUM | P2 |
| Voice-to-task | LOW | MEDIUM | P3 |
| Multi-workspace | LOW | HIGH | P3 |
| Mobile app | LOW | HIGH | P3 |

**Priority key:**
- P1 — Must ship in v1 (table stakes or core differentiator)
- P2 — Add after v1 validates (v1.x — value confirmed, scope expansion)
- P3 — Defer until clear demand (v2+)

## Competitor Feature Analysis

| Feature | Sunsama | Motion | Akiflow | Backbone PLM | Notion+Slack | **Designer Dashboard** |
|---------|---------|--------|---------|--------------|--------------|------------------------|
| Daily morning brief | Yes (manual ritual) | Yes (auto-schedule) | Yes (inbox triage) | No | No | **AI-drafted brief, designer reviews** |
| Cross-channel ingest | Slack/Gmail/Asana/Linear etc. | Slack, Gmail, Calendar | Slack, Gmail, Teams, Notion | Limited (Illustrator plugin) | Slack ↔ Notion only | **Slack + Gmail + Calendar + Notion + KakaoTalk (.txt)** |
| KakaoTalk ingest | No | No | No | No | No | **Yes (watch-folder)** — uniquely Korean |
| Notion as SOTR | Optional integration | No | Sync option | No | Yes | **Yes (mandatory)** |
| AI message drafting (outbound) | No | No | No | No | No (Notion AI summarizes only) | **Yes (review-and-send gate)** |
| Manufacturing milestone template | No | No | No | Yes (gated, enterprise) | No | **Yes (12-stage, baked in)** |
| Read-only share with sanitization | No (Notion has public share but exposes all) | No | No | Tech-pack live links (full content) | Notion public page (exposes all) | **Yes (no message bodies, only progress)** |
| Korean UI + Korean AI tone | No | No | No | No | Notion has Korean UI; Slack does too | **Yes — first-class** |
| Menu-bar always-on | Web app (Mac wrapper) | Web app | Yes (native Mac) | Web app | No | **Yes (Tauri native)** |
| Global hotkey | Cmd+K within app | No global | Cmd+K global | No | No | **Yes (planned — add to PROJECT.md)** |
| Local-first message storage | No (cloud) | No (cloud) | No (cloud) | No (cloud) | No (cloud) | **Yes (SQLite + Keychain)** |
| Approval gate before send | N/A (doesn't send) | N/A | N/A | Workflow approvals exist | Manual | **Yes (every send is gated)** |
| Snooze / defer | Yes | Auto-reschedule | Yes | No | No | **Add to v1** |
| Roll-over unfinished tasks | Yes (Sunsama signature) | Auto | Yes | No | No | **Add to v1** |
| Shutdown ritual | Yes (Sunsama signature) | No | No | No | No | **v1.x candidate** |
| Search across all sources | Within Sunsama only | Within Motion only | Cmd+F across inbox | Within Backbone | Notion AI search across connections | **Add to v1 — table stakes gap** |

## Missing from PROJECT.md "Active" — Recommendations to Add

Based on the gap analysis above, the following should be added to PROJECT.md `### Active` before requirements definition:

1. **Foundation: 글로벌 단축키로 대시보드 호출** (e.g., ⌥+Space) — table stakes for menu-bar app category
2. **Today-view: 오늘 액션을 스누즈/연기/내일로 이월할 수 있다** — without this, daily brief becomes stale list
3. **Search: 흡수된 모든 채널 메시지를 로컬 검색할 수 있다** — users will absolutely expect this; Klu/Akiflow/Notion all have it
4. **Notion (SOTR): 패션 12단계 마일스톤 템플릿이 마법사 실행 시 자동으로 생성된다** — make the 12-stage template explicit Active requirement, not just Context
5. **Send: AI 초안이 수신자 역할(MD/공장/내부)에 따라 존댓말/반말/영문을 자동 선택한다** — Korean-honorific awareness is a Korean-context differentiator
6. **(v1.x candidate, not Active yet) Shutdown ritual: 저녁 종료 의식으로 오늘 회고 + 내일 브리프 사전 생성** — Sunsama-pattern, defer to v1.x
7. **(v1.x candidate, not Active yet) Cmd+K 명령 바: 프로젝트 점프, 태스크 생성, 발송 초안 호출** — defer until volume justifies
8. **(v1.x candidate, not Active yet) 노션 → CSV/PDF 리포트 내보내기** — Notion already exports, only needed if MD requests offline reports

## Korean-Context Insights

1. **존댓말 / 반말 distinction in AI drafts is non-trivial.** Western tools draft in flat tone. A Korean designer addressing an MD (formal) vs an internal junior (casual) vs a long-time factory contact (friendly-formal) needs different registers. Sonnet 4.6 handles this well *if prompted explicitly per recipient*. This must be a first-class send-card field, not an afterthought.

2. **KakaoTalk is the de-facto factory channel in Korea.** Even the most "professional" Korean fashion factories operate on personal KakaoTalk groups, not Slack. The .txt watch-folder approach is the only safe ingest path (KakaoTalk's official Business API does not cover personal/group chats, and unofficial automation results in account suspension within days). This is genuinely defensible.

3. **JANDI / Swit are common Korean Slack alternatives.** Some MDs in Korean fashion conglomerates (Samsung C&T Fashion, LF, E-Land) use JANDI or Swit instead of Slack. PROJECT.md targets Slack only — note this as a potential v2 expansion if user pool spans larger conglomerates.

4. **Notion is rapidly mainstream for Korean designers**, but adoption is uneven across MDs and factories. The read-only share URL pattern is critical — MDs/factories should NOT need a Notion account.

5. **macOS adoption is high among Korean creative professionals**, validating the Mac-only v1 choice. Korean fashion design schools (Hongik, Kookmin) standardize on Mac.

## Sources

### Daily-brief / unified-inbox tools
- [Sunsama Daily Planning Guide](https://www.sunsama.com/blog/the-official-daily-planning-guide)
- [Sunsama Daily Planning](https://www.sunsama.com/daily-planning)
- [Sunsama Daily Planning and Shutdown](https://www.sunsama.com/features/daily-planning-and-shutdown)
- [Motion AI Calendar features](https://www.usemotion.com/features/ai-calendar)
- [Motion AI Task Manager](https://www.usemotion.com/features/ai-task-manager)
- [Akiflow Keyboard Shortcuts](https://product.akiflow.com/help/articles/7262522-keyboard-shortcuts)
- [Akiflow Inbox & Calendar](https://akiflow.com/features/inbox-calendar/)
- [Reclaim AI features](https://reclaim.ai/)
- [Reclaim Tasks vs Habits](https://help.reclaim.ai/en/articles/11325700-habits-vs-tasks-vs-focus-time-when-to-use-each-in-reclaim)
- [Sunsama vs Motion — Morgen comparison](https://www.morgen.so/blog-posts/sunsama-vs-motion)
- [Sunsama vs Motion — alfred comparison](https://get-alfred.ai/blog/sunsama-vs-motion)

### Notion / Slack / Cron / sharing
- [Notion Slack integrations](https://www.notion.com/integrations/slack)
- [Notion Slack integration guide](https://www.notion.com/help/guides/unleashing-productivity-with-notions-slack-integration)
- [Introducing Notion Calendar (formerly Cron)](https://www.notion.com/blog/introducing-notion-calendar)
- [Notion Calendar product page](https://www.notion.com/product/calendar)
- [Notion sharing & permissions](https://www.notion.com/help/sharing-and-permissions)
- [Notion link expiration rules — Metomic](https://www.metomic.io/resource-centre/how-to-use-notions-new-expiry-rules)
- [Notion enterprise search](https://www.notion.com/product/enterprise-search)

### Manufacturing / fashion PLM
- [Backbone PLM (now Bamboo Rose)](https://bamboorose.com/backbone/)
- [Bamboo Rose TotalPLM](https://bamboorose.com/)
- [Bamboo Rose PLM & Sourcing](https://bamboorose.com/product-lifecycle-management/)
- [PLM as backbone of fashion industry — Share PLM](https://shareplm.com/product-lifestyle-management-plm-as-a-backbone-of-the-fashion-industry/)
- [17 PLM tools for fashion — Creative Force](https://www.creativeforce.io/blog/17-product-lifecycle-management-solutions-for-smart-fashion-apparel-cf)
- [sample.flow fashion sample tracking](https://www.sampleflow.io/)
- [Lifecycle PLM Fashion Sample Management](https://www.lifecycleplm.com/platform/fashion-sample-management)
- [Apparel Samples Management — WFX](https://www.worldfashionexchange.com/fashion-plm-software/apparel-samples-and-feedback-and-approval.html)

### Korean tools
- [JANDI features](https://www.jandi.com/landing/en/features)
- [JANDI collaboration](https://www.jandi.com/landing/en/features/collaboration)
- [Swit](https://swit.io/)
- [KakaoTalk overview — Wikipedia](https://en.wikipedia.org/wiki/KakaoTalk)
- [KakaoTalk on PC — Windows Forum](https://windowsforum.com/threads/kakaotalk-on-pc-desktop-native-vs-web-vs-emulator-trade-offs.394039/)
- [Top Korean Productivity Apps — Analytics Insight](https://www.analyticsinsight.net/apps/top-korean-apps-for-productivity-and-time-management)

### Always-on AI menu bar UX
- [ChatGPT on Mac menu bar — MacRumors](https://www.macrumors.com/how-to/access-chatgpt-mac-menu-bar/)
- [ChatGPT chat bar launch — OpenAI](https://help.openai.com/en/articles/9295241-how-to-launch-the-chat-bar)
- [Raycast main site](https://www.raycast.com/)
- [Raycast Menu Bar Commands API](https://developers.raycast.com/api-reference/menu-bar-commands)
- [Raycast Manual](https://manual.raycast.com/)

### Unified inbox / search / digest
- [Best Unified Inbox Apps 2026 — Toolfinder](https://toolfinder.com/best/unified-inbox-apps)
- [Klu Unified Search across Notion, Gmail, Slack](https://klu.so/blog/enterprise-unified-search-notion-gmail-slack)
- [AI Daily Brief overview — alfred](https://get-alfred.ai/glossary/daily-brief)
- [How I built an AI Daily Brief — Mishaev/Medium](https://mark-mishaev.medium.com/how-i-built-an-ai-powered-daily-brief-that-saves-me-2-hours-every-day-2504a015f79f)

### Approval workflow / design feedback
- [Design annotation & feedback tools — StreamWork](https://www.streamwork.com/post/10-best-design-annotation-and-feedback-tools)
- [Approval workflow software — StreamWork](https://www.streamwork.com/post/top-creative-approval-software)

### Manufacturing milestones
- [30 Project Milestone Examples — ProjectManager](https://www.projectmanager.com/blog/project-milestone-examples)
- [Manufacturing production planning template — Sage](https://www.sage.com/en-us/blog/manufacturing-production-planning-template/)

### Export
- [Notion export your content](https://www.notion.com/help/export-your-content)

---
*Feature research for: morning brief / unified inbox / project pulse for Korean product-development designers in fashion-manufacturing*
*Researched: 2026-05-03*
