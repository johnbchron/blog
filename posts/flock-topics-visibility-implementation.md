+++
title = "Topics visibility migration — implementation plan"
date = "2026-07-24"
hidden = true
+++

Implementation plan for the model described in
`docs/plan/flock-topics-visibility-migration.md`. Executed as an
**expand → migrate → contract** movement across stacked branches (jj), each
built on top of the previous so the rollout can be measured and paused between
stages. Branch B is split into **B1 (new writes + dual-aware readers)** and
**B2 (backfill + flip)** so the cutover is a single controlled trigger rather
than a big-bang deploy.

## Locked decisions

1. **`friends` is net-new.** No existing data migrates into it. The Phase 2
   migration only ever produces `public` and `private` topics. `friends` is a
   go-forward option users pick manually.
2. **Share links kept for public only.** Phone invites (`pendingViewers`),
   passwords, and share-link *joining of non-public topics* are dropped.
   Invitations are friends-only (userId). A share link to a public topic is just
   a deep link — no gating.
3. **Hosts move onto the topic row** as `hostIds: v.array(v.id('users'))`.
   `topicMembers` *viewer* rows migrate into `topicInvitations`; the host role is
   read from `hostIds` going forward. (`topicMembers` itself is left in place for
   now — we'll rework co-hosting later.)
4. **Maybe-cells resolved:** an invited *private* topic shows as a card on the
   host's profile for the invitee; we build the "Invited to" private-topics list
   UI. A user's own prayers stay **invisible everywhere** (design doc line 27),
   which moots the "self content" column.
5. **No auto-subscribe on migration.** Previously-invited (`shared`) users become
   invited but are *not* auto-subscribed. They will only receive update/answered
   notifications after they explicitly subscribe.
6. **Keep the field name `privacy`.** We widen its union in place rather than
   introduce a `visibility` field. (Values change: add `friends`, drop `shared`.)
7. **`prayerPointsVisibility` is left untouched** — prayer points are being
   removed later, so we do not invest in their visibility here.
8. **Friend check for invitations = friend of the inviting host.** A host may
   only invite their own accepted friends. For passive `friends`-tier viewing,
   access = accepted friend of any host.

## Target schema (end state, after contract)

**`prayerTopics`**
- `privacy: v.union('public','friends','private')` — `shared` removed.
- `hostIds: v.array(v.id('users'))` — required after contract.
- Removed: `allowedViewers`, `password`. (`prayerPointsVisibility`,
  `sharingStatus`, `recipient` left as-is / out of scope.)

**`topicInvitations`** (new, homogeneous per design doc)
```
topicId:    v.id('prayerTopics')
userId:     v.id('users')        // invitee
invited:    v.boolean()          // false = rescinded (private only)
invitedBy:  v.id('users')        // the host who invited; drives friend scoping
createdAt:  v.number()
updatedAt:  v.number()
```
Indexes: `by_topic`, `by_user`, `by_topic_and_user`.
- public/friends: row created once on invite, never mutated. Purely a
  notification + "subscribe" CTA.
- private: `invited=true` also grants view/interact access; flipping to `false`
  rescinds it.

**`users`**
- `subscribedTopicIds: v.optional(v.array(v.id('prayerTopics')))` — present entry
  = subscribed. User-local, co-located for cheap reads.

**Dropped after contract:** `pendingViewers` table.

## Visibility rules (`convex/lib/privacy.ts` rewrite)

`canViewTopic(topic, userId, { isFriendOfHost, invitation })`:
- author or `topic.hostIds.includes(userId)` → **true**
- `public` → **true**
- `friends` → true iff `userId` is an accepted friend of any host
- `private` → true iff an active invitation (`invited === true`) exists

**Content matrix (implementation):**
- Topic page / interact: `canViewTopic`.
- Card on host's profile: public → always; friends → if friend; private → if
  invited.
- "Created" feed event: friends → to friends of host; private → to invitees;
  public → not broadcast.
- "Update" / "Answered" feed event + push: **only to subscribers** (any tier).
- Prayers: invisible everywhere.

**Invitation notifications** (`TopicNotificationType.INVITED`, new):
- public/friends copy: "[host] invited you to be part of [topic]. Subscribe to
  receive updates." CTA → subscribe.
- private copy: "[host] invited you to their private topic [topic]. Tap to view."
  CTA → view (access already granted).

---

## Branch A — expand + migrate (Phases 0–2)

Additive only. Safe to deploy and sit; no read paths change behavior.

**Phase 0 — types/validators**
- `FlockRN/types/PrayerSubtypes.ts`: add `FRIENDS = 'friends'` to `Privacy`.
- `convex/validators.ts`: widen `topicPrivacyValidator` to include `friends`
  (keep `shared` for now); add `INVITED` to `topicNotificationTypeValidator`.
- `shared/types/notificationTypes.ts`: add `INVITED = 'topic_invited'` (keep
  `SHARED` for legacy inbox rows).

**Phase 1 — schema expand** (`convex/schema.ts`, all optional/additive)
- `prayerTopics`: add `hostIds: v.optional(v.array(v.id('users')))`; widen
  `privacy` union to `public | private | shared | friends`.
- `users`: add `subscribedTopicIds: v.optional(...)`.
- Add `topicInvitations` table + indexes.
- `convex:typecheck`, deploy to dev.

**Phase 2 — backfill migration** (`convex/migrations/`, `@convex-dev/migrations`)

> **The `shared → private` value flip is deliberately NOT here.** In Branch A the
> live read paths are still the old logic, and old `canViewTopic`
> (`lib/privacy.ts:22`) treats `private` as author-only and never consults
> `topicInvitations`. Flipping the value now would strip access from every
> existing shared-topic viewer. So Phase 2 is purely additive/mirroring and
> leaves `privacy = 'shared'` intact; the flip happens in **B2**, after the
> dual-aware reader is live.

Write the backfill as **one idempotent, derive-from-current pass** that is safe
to run more than once (it is re-run at the start of B2 to catch anything written
during the A→B1 window):

- `backfillTopicNewShapes` (single migration, per topic):
  - `hostIds` = distinct(author + `topicMembers` host rows), only if changed.
  - For `shared` topics: upsert `topicInvitations` rows (`invited=true`,
    `invitedBy = author`) from the topic's **current** `allowedViewers` userId
    entries. Phone entries drop (no account existed). Upsert = create-if-absent,
    never downgrade an existing row.
  - Leaves `privacy` unchanged. **Never** writes `subscribedTopicIds`.
- Dry-run (`yarn migrate:dry`) then run on dev; verify invitation-row counts
  match `allowedViewers` userId totals and every topic has `hostIds`.
- Because it derives from current state and upserts, correctness does **not**
  depend on this Phase-2 run — it is a pre-warm. The authoritative run is in B2.
- **Branch A is user-invisible: additive data only, no privacy values change, no
  read path changes. Safe to deploy and pause on.**

---

## Branch B1 — new writes + dual-aware readers (Phase 3a), on top of A

Deploys with **zero observable change**: all topics are still `shared`, but every
write now also produces the new shapes and every reader understands both. This is
what makes the B2 flip safe. Nothing here flips a `privacy` value.

**Readers — port ALL of them to dual-read (both shapes) before any flip.** A
`private` topic is viewable with an active invitation; a still-`shared` topic via
`allowedViewers`. Correct before *and* after the flip. Full checklist:
- `convex/lib/privacy.ts`: `canViewTopic`, `canMatchTopicWhenPrayingFor`
  (accept friendship + invitation inputs; `friends`-tier matchable when praying
  for a friend host).
- `convex/prayerTopics.ts`: `isHostOfTopic` + `getAuthorizedTopicForViewer` read
  `hostIds`; `canUserViewTopic`, `searchTopics`, `getEligibleOnboardingFriendTopics`.
- `convex/ai/similarity.ts`, `convex/lib/recipientTopics.ts`.
- `convex/topicFeed.ts` / `convex/homepage.ts`: created events → friends-of-host
  + invitees; update/answered events + push → **subscribers only**; stop sourcing
  from `allowedViewers` / `topicMembers` viewer rows.
- **Guard:** by end of B1, a grep for `=== 'shared'` or `.allowedViewers` used
  for *authorization* returns nothing except the transitional dual-read helper.

**Writes — produce new shapes and stop writing dead old ones (same commits):**
- `hostIds` is the sole source of truth for host auth. `setTopicMemberRole` /
  `removeTopicMember` (topicMembers.ts) patch `hostIds`; keep writing the
  `topicMembers` host row too (harmless; co-host rework deferred) but never *read*
  it for auth.
- New mutations: `inviteToTopic` (host-only; **assert inviter–invitee
  friendship**; upsert `topicInvitations` + emit `INVITED` notification; set
  `participationStatus = GROUP` on first invite — replaces the deleted
  auto-upgrade's side effect), `rescindInvitation` (private; `invited=false`).
- `updatePrayerTopic`: when downgrading a topic **to** `private`, take
  `keepInvitedUserIds` (or `'all'`/`'none'`), **defaulting to keep-all** so nobody
  is silently locked out; kept → `invited=true`, dropped → `invited=false`.
- `createPrayerUpdate`: **remove the PRIVATE→SHARED auto-upgrade**; notify
  **subscribers only** for update/answered.
- `joinTopicFromShareLink`: narrow to **public topics only** — return the topic
  and offer a subscribe CTA (no auto-subscribe); non-public → "invite required".
  Delete the `allowedViewers`/phone-append (prayerTopics.ts:2425).
- Replace the `sendTopicSharedNotifications` path with the invitation flow.
- **Stop writing** `allowedViewers` / `password` / `pendingViewers` in every
  writer: `createPrayerTopicMutation:421`, `joinTopicFromShareLink:2425`,
  `setTopicMemberRole`/`removeTopicMember` (topicMembers.ts:244/298),
  `users.ts:800`. (Fields stay in the schema until Branch D.)
- New `convex/subscriptions.ts` (or in `users.ts`): `subscribeToTopic`,
  `unsubscribeFromTopic` (no-op if topic not visible), `getSubscribedTopics`,
  and **`getInvitedTopics`** (invitations `by_user`, `invited=true`, private,
  joined to topic + inviter) — needed by Branch C's "Invited to" surface.
- Tests under `__tests__/convex/`. Deploy and verify no behavior change.

## Branch B2 — backfill + flip (Phase 3b), on top of B1

The controlled cutover. Only run once B1 is confirmed in prod.

1. **Re-run** the idempotent `backfillTopicNewShapes` from Phase 2 — catches any
   topic created/shared during the A→B1 window (derives from current
   `allowedViewers`/topicMembers, upserts).
2. **`migrateTopicVisibility`**: flip `shared → private`; `public`/`private`
   unchanged. Invitations now exist for every affected topic, so access carries
   over via the dual-aware reader from B1.
3. Gate step 2 to run only after step 1 completes. Dry-run on dev, verify no
   topic ends up `private` without either being author-only or having invitation
   rows.

---

## Branch C — frontend swap (Phase 4), on top of B2

- `components/Topic/PrivacyPicker.tsx`: three options — Public / Friends /
  Private — with new copy. Invite picker is friends-only.
- `app/topic/shareTopic.tsx`: surface the share link **for public topics only**;
  friends-only invite for friends/private; remove password UI.
- **Downgrade-to-private confirmation sheet:** when a user switches a
  friends/public topic to private, show current invitees with keep/remove toggles
  (pre-checked = keep), wired to `updatePrayerTopic`'s `keepInvitedUserIds`.
- Private invited-list management (rescind) in topic edit.
- Subscribe / unsubscribe control on `app/topic/[id].tsx` (calls
  `subscribeToTopic`/`unsubscribeFromTopic`); a "Subscribed topics" list backed
  by `getSubscribedTopics`; a new **"Invited to"** private-topics list backed by
  `getInvitedTopics`.
- Badges/pills: `friends` variants in `TopicBadges.tsx`, `TopicPrivacyPill.tsx`.
- Feed rendering: gate update/answered cards on subscription; show invited
  private topic as a card on host profile.
- `TopicNotificationCard.tsx`: `INVITED` copy + tier CTA (subscribe vs. view).
- Point `useTopicMembers` / host reads at `hostIds`.

---

## Branch D — contract (Phase 5), on top of C

Only after A–C are validated in production and nothing writes the old fields
anymore (writes ceased in B1, frontend stopped sending them in C).

- `convex/schema.ts`: narrow `privacy` to `public | friends | private`; remove
  `allowedViewers`, `password`; drop `pendingViewers` table; make `hostIds`
  required.
- `convex/validators.ts`: narrow `topicPrivacyValidator`.
- Delete dead paths: `shared` branches, `pendingViewers` logic, `topicMembers`
  viewer-role usage, `sendTopicSharedNotifications` remnants.
- Keep `TopicNotificationType.SHARED` for legacy inbox rows (mirrors how
  `friend_added` is retained).
- Final typecheck + tests + deploy.

---

## jj branch structure

Stacked, each on top of the previous:
```
main
 └─ topics-visibility/expand-migrate     (Branch A:  Phases 0–2, additive)
     └─ topics-visibility/backend-writes (Branch B1: Phase 3a, dual-aware, no flip)
         └─ topics-visibility/cutover    (Branch B2: Phase 3b, backfill + flip)
             └─ topics-visibility/frontend (Branch C: Phase 4)
                 └─ topics-visibility/contract (Branch D: Phase 5)
```
Create with jj bookmarks on stacked changes so each stage can land and be
verified independently before the next is rebased forward. B1 and B2 are separate
bookmarks specifically so the flip (B2) can be triggered and watched on its own,
after B1 has soaked in prod.

## Risks / watch-items

- **Soak B1 before triggering B2.** The whole point of the split is that B1
  changes nothing observable; let it bake in prod, confirm the reader guard
  (no `=== 'shared'`/`.allowedViewers` auth reads remain), then fire B2.
- `friends`-tier passive visibility (friend of any host) vs. invitation scoping
  (inviter's own friends) is a deliberate asymmetry — verify it reads correctly.
- Phone `allowedViewers` are silently dropped in the backfill (no account
  existed); confirm none are unexpectedly resolvable.
- Removing the auto-upgrade changes who gets update notifications — with
  no-auto-subscribe, migrated invitees go quiet until they subscribe. Intended,
  but user-visible.
- Legacy inbox rows: keep old notification-type literals valid.
- `topicMembers` host rows and `hostIds` are dual-written through B–C; they must
  not drift. Reads use `hostIds` only.
