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
