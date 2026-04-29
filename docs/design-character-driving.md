# Character Driving Roadmap

This document captures the agreed direction for Animus after the initial project
review. It is written to be split into GitHub issues.

## Goal

Build Animus into a usable local role-play tool with strong character driving:

- Structured character data instead of a single loose persona blob.
- Stable prompt assembly with clear sections and placeholders.
- Strong first messages and style examples.
- Per-persona response length and sampling controls.
- Fast iteration tools: redo, redo with note, and edit last assistant reply.
- Local-first storage for images and backups.
- Later: lorebook/world info and deeper memory tools.

## GitHub Labels

Suggested labels:

- `area:backend`
- `area:frontend`
- `area:db`
- `area:llm`
- `area:docs`
- `area:ux`
- `type:feature`
- `type:refactor`
- `type:test`
- `type:chore`
- `priority:p0`
- `priority:p1`
- `priority:p2`
- `phase:1-character-driving`
- `phase:2-local-data`
- `phase:3-lorebook-memory`

## Core Decisions

### Source of Truth

Animus should use its own structured persona model as the DB/API source of
truth. Character Card v2 remains an import/export compatibility format.
Animus-specific fields can be exported through Character Card `extensions`.

### Migration Strategy

Use a minimal but structurally meaningful first migration. Add fields that
directly improve generation before adding lorebook tables.

First wave fields:

- `model_instructions`
- `appearance`
- `speech_style`
- `character_goals`
- `post_history_instructions`
- `response_length_limit`
- `temperature`
- `repeat_penalty`
- `instruction_template`

Keep existing fields for now:

- `description`
- `personality`
- `scenario`
- `first_message`
- `message_example`
- `content_rating`
- `model`
- `avatar_url`
- `background_url`
- `raw_card`

### Prompt Shape

Prompt construction should move from a loose interpolated paragraph to strict
sections.

Target system message:

```text
# Role
You are {{char}}.

# Model Instructions
...

# Character
Appearance:
...

Description:
...

Personality:
...

Speech Style:
...

# Scenario
...

# Character Goals
...

# Style Examples
Use these as style references, not current conversation events.
...

# Response Contract
...
```

Then append:

- optional `system` memory summary
- recent real conversation history
- optional final `system` post-history instructions
- optional temporary redo instruction

### Templates and Placeholders

Model instructions should be stored as editable text. The UI should offer
templates, but the server should resolve placeholders at prompt-build time.

Supported MVP placeholders:

- `{{char}}`
- `{{user}}`
- `{{response_length_limit}}`
- `{{response_length_example}}`

Default template:

```text
Write as {{char}} in a character-driven roleplay scene.
Use vivid physical actions in *asterisks* and spoken dialogue in "quotes".
Stay in {{char}}'s perspective and preserve {{char}}'s personality, goals, speech patterns, and boundaries.
Drive the scene proactively through concrete actions, choices, tension, and sensory detail.
Keep the response under {{response_length_limit}} characters.
```

NSFW template:

```text
Write as {{char}} in a mature, intimate roleplay scene.
Use vivid physical actions in *asterisks* and spoken dialogue in "quotes".
Stay in {{char}}'s perspective and preserve {{char}}'s personality, desires, goals, speech patterns, and boundaries.
Build tension through pacing, sensory detail, consent-aware reactions, and proactive character choices.
Keep the response under {{response_length_limit}} characters.
```

Template selection should be explicit, with automatic preselection:

- `default`
- `nsfw`
- `custom`

When creating an NSFW persona, preselect `nsfw`. Do not silently overwrite
existing custom instructions when a user changes content rating.

### Response Length

Store `response_length_limit` as characters. Use it in two ways:

- Prompt instruction: `Keep the response under {{response_length_limit}} characters.`
- Ollama option: estimate `num_predict` from the character limit.

Recommended estimate:

```text
num_predict = ceil(response_length_limit / 4) + 50
```

The UI should use a slider and show a generated example below it.

Suggested MVP range:

- minimum: `400`
- default: `1200`
- maximum: `4000`

### Sampling Params

Expose simple persona-level response style first, but store explicit values.

DB/API fields:

- `temperature`
- `repeat_penalty`

UI presets:

- `Stable`: `temperature = 0.65`, `repeat_penalty = 1.12`
- `Balanced`: `temperature = 0.8`, `repeat_penalty = 1.08`
- `Creative`: `temperature = 1.0`, `repeat_penalty = 1.05`

Recommended default: `Stable`.

Future candidates:

- `top_p`
- `top_k`
- `min_p`
- `mirostat`

### Redo and Edit

MVP scope: only the latest assistant message can be regenerated or edited.

Redo behavior:

- Button visible only on the latest assistant message.
- Server rebuilds prompt from history up to the preceding user message.
- Server replaces the existing assistant message content in the DB.
- Keep the same `message_id`.
- UI updates the existing message bubble.

Redo with note:

- Instruction is temporary and not persisted as visible conversation history.
- Inject it as a final system instruction for that one regeneration.

Suggested temporary instruction:

```text
Regenerate the previous assistant response with this direction:
{{redo_instruction}}
```

Manual edit:

- Allow editing only the latest assistant message in the MVP.
- Save edited content with `PATCH /api/messages/:id`.
- Edited replies stay in history to help steer later generations.

### Delete Persona

Persona deletion should remain cascading, but deletion must be guarded.

Required behavior:

- UI confirmation before deletion.
- Server creates a backup before deleting.
- Server deletes persona after backup succeeds.
- SQLite foreign keys must be explicitly enabled for runtime connections.

Backup content:

- persona
- conversations
- messages

Format:

```json
{
  "format": "animus_backup",
  "version": 1,
  "exported_at": "2026-04-29T00:00:00Z",
  "persona": {},
  "conversations": [
    {
      "id": "...",
      "created_at": 0,
      "updated_at": 0,
      "messages": []
    }
  ]
}
```

Importing backups can be implemented later, but the format should be importable.

### Local Assets

Avatars and backgrounds should be files on disk, not data URLs in SQLite.

Recommended layout:

```text
~/.animus/assets/personas/{persona_id}/avatar.png
~/.animus/assets/personas/{persona_id}/background.png
~/.animus/backups/personas/{persona_slug}-{timestamp}.json
```

API can still accept frontend data URLs initially, but the server should decode
and write files locally. DB should store a stable app URL or relative asset path.

### Settings

Add a small Settings screen and settings API.

MVP settings:

- `user_name`
- `default_model`
- `ollama_url` read-only initially
- `assets_dir` read-only
- `backups_dir` read-only

Storage:

```sql
CREATE TABLE app_settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
```

Default `user_name`: `You`.

Use `user_name` when resolving `{{user}}` for first messages and prompts.
Do not rewrite old messages when the setting changes.

### Lorebook

Lorebook/world info is phase 3, not MVP.

Expected future shape:

- dedicated table
- keyword matching
- priority/order
- token budget
- enabled/disabled flag
- prompt injection section
- UI to manage entries per persona

Do not model lorebook as a single JSON text blob unless used only as a temporary
prototype.

## Phase 1: Structured Character Driving

### Issue 1: Add Structured Persona Fields

Labels:

- `area:db`
- `area:backend`
- `type:feature`
- `priority:p0`
- `phase:1-character-driving`

Summary:

Add the first wave of structured persona fields to the database, domain model,
API DTOs, and repository layer.

Tasks:

- Add a new SQL migration.
- Update `Persona` in `crates/animus-core/src/persona.rs`.
- Update `CreatePersonaRequest`, `UpdatePersonaRequest`, and `PersonaResponse`.
- Update `PersonaRepo::insert`, `find_by_id`, `find_all`, and `update`.
- Preserve existing personas with sensible defaults.

Fields:

- `model_instructions TEXT NOT NULL DEFAULT ''`
- `appearance TEXT NOT NULL DEFAULT ''`
- `speech_style TEXT NOT NULL DEFAULT ''`
- `character_goals TEXT NOT NULL DEFAULT ''`
- `post_history_instructions TEXT NOT NULL DEFAULT ''`
- `response_length_limit INTEGER NOT NULL DEFAULT 1200`
- `temperature REAL NOT NULL DEFAULT 0.65`
- `repeat_penalty REAL NOT NULL DEFAULT 1.12`
- `instruction_template TEXT NOT NULL DEFAULT 'default'`

Acceptance criteria:

- Existing tests pass.
- Creating a minimal persona still works.
- Fetching an old persona returns defaults for new fields.
- Patch requests can update all new fields.
- Duplicate name handling still works.

Dependencies:

- None.

### Issue 2: Add App Settings Storage and API

Labels:

- `area:db`
- `area:backend`
- `type:feature`
- `priority:p0`
- `phase:1-character-driving`

Summary:

Add local app settings for global user name and default model.

Tasks:

- Add `app_settings` migration.
- Add settings repository in `animus-db`.
- Add shared settings DTO/domain type if useful.
- Add `GET /api/settings`.
- Add `PATCH /api/settings`.
- Initialize defaults when settings are absent.
- Use settings in server state or fetch where needed.

MVP settings:

- `user_name`
- `default_model`

Read-only response fields:

- `ollama_url`
- `assets_dir`
- `backups_dir`

Acceptance criteria:

- `GET /api/settings` returns defaults on a fresh DB.
- `PATCH /api/settings` updates `user_name` and `default_model`.
- Empty `user_name` is rejected.
- Existing `OLLAMA_MODEL` env value can seed or override the default model.

Dependencies:

- None.

### Issue 3: Implement Structured Prompt Builder

Labels:

- `area:llm`
- `area:backend`
- `type:refactor`
- `priority:p0`
- `phase:1-character-driving`

Summary:

Refactor `build_prompt` to produce strict prompt sections and resolve supported
placeholders.

Tasks:

- Add a prompt context object containing persona, settings, summary, history,
  and optional redo instruction.
- Resolve `{{char}}`, `{{user}}`, `{{response_length_limit}}`, and
  `{{response_length_example}}`.
- Include style examples in the system message.
- Include summary as `# Memory Summary`.
- Add final post-history system instructions when configured.
- Keep first-message behavior compatible.

Acceptance criteria:

- Prompt unit tests cover default persona, NSFW template, summary, examples,
  post-history instructions, and response length placeholders.
- No unresolved known placeholders remain in generated prompts.
- Existing chat still streams replies.

Dependencies:

- Issue 1
- Issue 2

### Issue 4: Send Ollama Sampling Options

Labels:

- `area:llm`
- `area:backend`
- `type:feature`
- `priority:p0`
- `phase:1-character-driving`

Summary:

Send per-persona generation options to Ollama, including estimated `num_predict`.

Tasks:

- Extend Ollama client request payloads with `options`.
- Add `temperature`, `repeat_penalty`, and `num_predict`.
- Use persona model override if present, otherwise settings default model.
- Apply behavior consistently for streaming and non-streaming paths.

Acceptance criteria:

- Streaming endpoint sends options to `/api/chat`.
- Non-streaming endpoint sends options to `/api/generate`.
- Tests cover `num_predict` estimate from `response_length_limit`.
- Persona model override continues to work or is explicitly fixed if currently
  unused.

Dependencies:

- Issue 1
- Issue 2

### Issue 5: Update Persona Form for Character Driving Fields

Labels:

- `area:frontend`
- `area:ux`
- `type:feature`
- `priority:p0`
- `phase:1-character-driving`

Summary:

Expose new persona fields in the create/edit form without making the UI feel
like a raw database editor.

Tasks:

- Add fields for model instructions, appearance, speech style, and goals.
- Add template selector: `default`, `nsfw`, `custom`.
- Auto-select `nsfw` template for new NSFW personas.
- Add response length slider and example preview.
- Add response style preset selector: `Stable`, `Balanced`, `Creative`.
- Map presets to explicit `temperature` and `repeat_penalty`.
- Keep existing fields and import flow working.

Acceptance criteria:

- Create persona can set all new fields.
- Edit persona loads and saves all new fields.
- Changing rating to NSFW on an untouched new persona selects the NSFW template.
- Changing rating does not overwrite custom edited instructions.
- Slider displays current character limit and example text.

Dependencies:

- Issue 1

### Issue 6: Add Settings Screen

Labels:

- `area:frontend`
- `area:ux`
- `type:feature`
- `priority:p1`
- `phase:1-character-driving`

Summary:

Add a compact settings page for global user name and default model.

Tasks:

- Add route/page for Settings.
- Link Settings from the existing app navigation.
- Add fields for `user_name` and `default_model`.
- Show read-only `ollama_url`, `assets_dir`, and `backups_dir`.
- Add save/error/loading states.

Acceptance criteria:

- User can update global user name.
- User can update default model.
- Empty user name cannot be saved.
- `{{user}}` in new prompts uses updated user name after save.

Dependencies:

- Issue 2

## Phase 2: Iteration Tools and Local Data

### Issue 7: Add Message Update Repository Methods

Labels:

- `area:db`
- `area:backend`
- `type:feature`
- `priority:p0`
- `phase:2-local-data`

Summary:

Add repository support needed for redo/edit of the latest assistant message.

Tasks:

- Add `find_by_id`.
- Add `find_latest_by_conversation`.
- Add `find_before` or equivalent history helper.
- Add `update_content`.
- Add tests for latest message and update behavior.

Acceptance criteria:

- Can identify latest assistant message for a conversation.
- Can update a message body without changing its ID.
- Cannot accidentally update a message from another conversation.

Dependencies:

- None.

### Issue 8: Add Redo Latest Assistant Message API

Labels:

- `area:backend`
- `area:llm`
- `type:feature`
- `priority:p0`
- `phase:2-local-data`

Summary:

Add an endpoint to regenerate and replace the latest assistant message.

Proposed endpoint:

```text
POST /api/messages/:id/regenerate
```

Request:

```json
{
  "instruction": "make it shorter and more teasing"
}
```

Tasks:

- Validate that the target message is the latest assistant message.
- Rebuild prompt from history ending before that assistant message.
- Inject optional redo instruction as temporary post-history instruction.
- Stream replacement text to the frontend.
- Replace existing message content in DB on completion.

Acceptance criteria:

- Non-latest assistant messages are rejected.
- User messages are rejected.
- Regeneration keeps the same message ID.
- Optional note affects only the regeneration prompt and is not added as a
  visible message.

Dependencies:

- Issue 3
- Issue 4
- Issue 7

### Issue 9: Add Edit Latest Assistant Message API

Labels:

- `area:backend`
- `type:feature`
- `priority:p0`
- `phase:2-local-data`

Summary:

Allow manual editing of the latest assistant message.

Proposed endpoint:

```text
PATCH /api/messages/:id
```

Request:

```json
{
  "content": "Edited assistant response"
}
```

Tasks:

- Validate that message exists.
- Validate it is the latest assistant message.
- Reject empty content.
- Update message content.

Acceptance criteria:

- Latest assistant message can be edited.
- Older assistant messages cannot be edited in MVP.
- User messages cannot be edited through this endpoint.
- Updated content appears in subsequent `GET /api/conversations/:id`.

Dependencies:

- Issue 7

### Issue 10: Add Redo/Edit Controls in Chat UI

Labels:

- `area:frontend`
- `area:ux`
- `type:feature`
- `priority:p0`
- `phase:2-local-data`

Summary:

Add UI controls for redo, redo with note, and edit on the latest assistant
message.

Tasks:

- Detect latest assistant message.
- Add compact controls on that message only.
- Add redo action.
- Add redo-with-note dialog or inline popover.
- Add edit mode with save/cancel.
- Update existing message bubble after redo/edit.
- Disable controls while streaming.

Acceptance criteria:

- Controls appear only on the latest assistant message.
- Redo replaces the message in place.
- Redo with note sends temporary instruction.
- Edit saves changed content and keeps conversation order.
- Error states are visible and recoverable.

Dependencies:

- Issue 8
- Issue 9

### Issue 11: Store Persona Assets as Local Files

Labels:

- `area:backend`
- `area:db`
- `type:feature`
- `priority:p1`
- `phase:2-local-data`

Summary:

Move avatar and background storage from DB data URLs to local files.

Tasks:

- Define asset directory config.
- Decode incoming data URLs server-side.
- Write avatar/background files under persona-specific directory.
- Store stable relative paths or app asset URLs in DB.
- Serve local assets from API/static route.
- Keep existing data URL personas readable during migration.

Acceptance criteria:

- Creating a persona with avatar writes an image file.
- DB stores a path/reference, not a data URL, for new uploads.
- Existing data URL avatars continue to render.
- Editing/replacing an avatar updates the local file.

Dependencies:

- Issue 2 for exposing `assets_dir` in settings response.

### Issue 12: Add Backup Before Persona Deletion

Labels:

- `area:backend`
- `area:db`
- `type:feature`
- `priority:p0`
- `phase:2-local-data`

Summary:

Before deleting a persona, export persona, conversations, and messages to local
JSON.

Tasks:

- Ensure SQLite foreign keys are enabled for runtime connections.
- Add repo methods to list conversations and messages for a persona.
- Implement backup writer.
- Update delete handler to backup before delete.
- Return useful error if backup fails.

Acceptance criteria:

- Deleting a persona creates a JSON backup file.
- Backup contains persona, conversations, and messages.
- Delete does not happen if backup write fails.
- Conversations and messages are deleted after persona delete.
- Tests or integration checks cover backup JSON shape.

Dependencies:

- Issue 2 for exposing `backups_dir` in settings response.

### Issue 13: Add Delete Confirmation in Persona UI

Labels:

- `area:frontend`
- `area:ux`
- `type:feature`
- `priority:p1`
- `phase:2-local-data`

Summary:

Add safe persona deletion from the UI with explicit confirmation.

Tasks:

- Add delete action on persona detail/card/edit screen.
- Show confirmation with persona name.
- Explain that a backup will be created.
- Call existing delete endpoint.
- Refresh persona list after success.

Acceptance criteria:

- User cannot delete with a single accidental click.
- Success removes persona from list.
- Error state is shown if backup/delete fails.

Dependencies:

- Issue 12

## Phase 3: Lorebook and Memory

### Issue 14: Design Lorebook Data Model

Labels:

- `area:db`
- `area:llm`
- `type:feature`
- `priority:p2`
- `phase:3-lorebook-memory`

Summary:

Design and migrate dedicated lorebook/world info tables.

Suggested fields:

- `id`
- `persona_id`
- `name`
- `keywords`
- `content`
- `priority`
- `enabled`
- `insertion_order`
- `created_at`
- `updated_at`

Acceptance criteria:

- Lore entries are stored separately from persona text.
- A persona can have multiple entries.
- Entries can be enabled/disabled.
- Keywords can be matched deterministically.

Dependencies:

- Phase 1 prompt builder.

### Issue 15: Implement Lorebook Injection

Labels:

- `area:llm`
- `area:backend`
- `type:feature`
- `priority:p2`
- `phase:3-lorebook-memory`

Summary:

Inject relevant lore entries into the prompt based on recent conversation
keywords.

Acceptance criteria:

- Matching considers recent user and assistant messages.
- Injected lore appears in a clear prompt section.
- Injection respects priority and budget.
- Unit tests cover keyword matching and ordering.

Dependencies:

- Issue 14

### Issue 16: Improve Summary for Role-play Memory

Labels:

- `area:llm`
- `area:backend`
- `type:refactor`
- `priority:p2`
- `phase:3-lorebook-memory`

Summary:

Refine summary generation to preserve role-play continuity rather than generic
conversation compression.

Target summary sections:

- established facts
- relationship state
- current scene state
- open tensions/goals
- user persona details
- continuity constraints

Acceptance criteria:

- Summary prompt asks for structured RP memory.
- Existing summary drawer still works.
- Prompt builder injects summary as `# Memory Summary`.

Dependencies:

- Issue 3

## Suggested Milestones

### Milestone: Phase 1 - Character Driving Foundation

Issues:

- Issue 1: Add Structured Persona Fields
- Issue 2: Add App Settings Storage and API
- Issue 3: Implement Structured Prompt Builder
- Issue 4: Send Ollama Sampling Options
- Issue 5: Update Persona Form for Character Driving Fields
- Issue 6: Add Settings Screen

Outcome:

Personas have structured prompt-driving fields, templates, response length, and
sampling controls. `{{user}}` is resolved from global settings.

### Milestone: Phase 2 - Iteration and Local Safety

Issues:

- Issue 7: Add Message Update Repository Methods
- Issue 8: Add Redo Latest Assistant Message API
- Issue 9: Add Edit Latest Assistant Message API
- Issue 10: Add Redo/Edit Controls in Chat UI
- Issue 11: Store Persona Assets as Local Files
- Issue 12: Add Backup Before Persona Deletion
- Issue 13: Add Delete Confirmation in Persona UI

Outcome:

User can iterate on the latest assistant reply, keep assets out of SQLite, and
delete personas without losing data accidentally.

### Milestone: Phase 3 - Lorebook and Memory

Issues:

- Issue 14: Design Lorebook Data Model
- Issue 15: Implement Lorebook Injection
- Issue 16: Improve Summary for Role-play Memory

Outcome:

Animus gains keyword-triggered world info and more useful long-session memory.

## Recommended Implementation Order

1. Issue 1
2. Issue 2
3. Issue 3
4. Issue 4
5. Issue 5
6. Issue 6
7. Issue 7
8. Issue 8
9. Issue 9
10. Issue 10
11. Issue 12
12. Issue 13
13. Issue 11
14. Phase 3 issues

Rationale:

- Prompt-driving fields unblock the core product value.
- Settings are needed before proper `{{user}}` resolution and default model
  handling.
- Redo/edit depends on prompt generation and message update helpers.
- Backup before deletion should land before making delete prominent in the UI.
- Local assets are important, but less blocking than generation quality and
  iteration tools.
