# Cluaiz Inbox Design Spec (Old Frontend Match)

## 1) Goal
Recreate the old inbox experience from:
- `cluaiz-old/Frontend/src/app/dashboard/communication/inbox/page.tsx`
- `cluaiz-old/Frontend/src/app/dashboard/communication/inbox/AgentChatWindow.tsx`

Target in new app:
- Same functional UX
- Same layout behavior
- Same conversation flow model
- Same operational controls (chat, filters, overlays, mission control)

## 2) Layout Blueprint (Must Match)
### Desktop
- Left: Inbox Sidebar (search, filters, groups, statuses, conversation list)
- Center: Chat Window (messages, composer, typing state, reply context, actions)
- Right: Mission Control / Dossier / Context panel (mode-driven)

### Mode States
- `chat`: standard list + chat + right control area
- `dossier`: profile/detail focused view with chat assist
- `marketplace`: talent/workforce marketplace mode

### Mobile
- Drawer-first navigation for side panels
- Center chat remains primary
- Right tools open as contextual drawer/panel

## 3) Core Feature Matrix (Old -> New)
### A. Conversation State
- Conversations list with stable sort:
  - pin priority
  - last message time priority
- Selected conversation tracking
- Unread counts and read timestamp logic
- Real-time updates without list jitter

### B. Sidebar Capabilities
- Search across conversations
- Filters: All, Unread, Online, Favorites, Groups, Calls, SMS, Assistants, Archive
- Date range filtering
- Group-aware filtering and bulk group operations
- Status pills/labels

### C. Chat Window Capabilities
- Message streaming and render pipeline
- Markdown + GFM support
- Typewriter animation for AI output
- Context menu per message (copy/reply/star/reaction/delete/audio)
- Reply thread context bar
- Message status indicators
- Notification/mute controls
- Voice playback hooks
- Typing indicator (user + agent)

### D. Overlays / Slideovers
- New Contact
- New Group
- Creator Selection
- Support
- Booking Settings
- Neural Logs
- Settings Overlay
- Profile Detail Page

### E. Right Panel / Mission Control
- Task controls and assistant actions
- Context-aware operations for selected conversation
- Dossier and AI workplace contextual utilities

## 4) Data + Realtime Behavior
### Required Services/Contracts
- `InboxService`: active chats, statuses, detail fetch, actions
- `GroupService`: groups, membership updates
- `Socket`: live chat updates, join/leave, typing events, status changes

### Realtime Rules
- Deduplicate incoming events by id/content+timestamp
- Update in-place then re-sort
- Avoid visual jumping during incoming events
- Respect selected conversation state while list updates

## 5) New App Package Mapping
### app-core
- state orchestration (conversations, view mode, selection, filters)
- sorting + dedupe utilities
- mode routing (chat/dossier/marketplace)

### ui-core
- reusable primitives:
  - sidebar container
  - conversation list item
  - message bubble
  - context menu
  - mission control panel
  - slideover shell

### protocol
- typed events:
  - `inbox.chat.updated.v1`
  - `inbox.chat.created.v1`
  - `inbox.typing.changed.v1`
  - `inbox.selection.changed.v1`

### shell-web
- page composition and wiring only
- no business logic duplication

## 6) UI Behavior Contract
- Header optional (user toggle)
- Left sidebar always resizable on desktop
- Right panel open/close toggle from left/nav command
- Center chat remains stable during panel toggles
- Panel widths persisted per mode

## 7) Old Frontend Dependencies to Reuse (Functionally)
From old frontend behavior, these categories are relevant:
- UI primitives (button/input/scroll/avatar/badge)
- Motion/animation (`framer-motion` style behavior)
- Markdown rendering (`react-markdown`, `remark-gfm`)
- Resizable layout panel system
- Icons and toast/notifications

Note:
- New app should keep architecture modular and avoid direct old file copy.
- Rebuild behavior via reusable components in `packages/*`.

## 8) Implementation Phases (Execution Order)
### Phase 1: Structural Match
- Build exact 3-panel inbox skeleton
- Add mode switch (`chat`, `dossier`, `marketplace`)
- Add resizable left/right panels with persisted width

### Phase 2: Sidebar + Filtering
- Implement conversation list + search + filters + date range
- Group and archive filtering rules
- Selection logic and empty states

### Phase 3: Chat Engine UI
- Message rendering, markdown, statuses
- Reply context, context menu, typing indicators
- Composer interactions + local optimistic updates

### Phase 4: Realtime Sync
- Socket event handling
- Merge, dedupe, stable reorder
- Non-jitter updates while active chat is open

### Phase 5: Overlay System
- Add all slideovers and settings overlays
- Profile detail + mission control coupling

### Phase 6: Polish + Parity
- Micro interactions, motion timing, spacing parity
- Keyboard shortcuts
- Mobile drawer behavior parity

## 9) Acceptance Criteria
- Inbox desktop layout mirrors old project behavior
- Mode transitions are functional and stable
- Realtime updates do not break ordering or selection
- Chat interactions (reply/menu/typing/markdown) work end-to-end
- Right panel toggle and left resize behave predictably
- Mobile behavior remains usable with drawer-based side panels

## 10) File Placement for New Build
Primary new design target:
- `cluaiz/Apps/app/apps/shell-web`

Shared reusable system:
- `cluaiz/Apps/app/packages/ui-core`
- `cluaiz/Apps/app/packages/app-core`
- `cluaiz/Apps/app/packages/protocol`
