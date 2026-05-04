# Option C: Spec-First (MD-Only Control Plane) - Most Scalable for Authoring

This model treats each skill as a **single source of truth in Markdown**.
Runtime artifacts (`logic.wasm`, `connector.mcp`, `state.kv-cache`) are optional build outputs.

## Folder Shape

```text
skills/
  registry/
    skills.index.json
    tags.index.json
  packs/
    <skill-id>/
      SKILL.md              # Only required file
      examples.md           # Optional
      changelog.md          # Optional
      artifacts/            # Optional generated outputs
        manifest.json
        connector.mcp
        logic.wasm
        state.kv-cache
```

## SKILL.md Contract (required sections)

```md
# skill: github-manager
version: 1.0.0
category: dev-tools
runtime: mcp|prompt|workflow|hybrid
permissions: [repo.read, repo.write]
triggers: ["github", "pull request", "review"]

## Intent
What this skill is for.

## Prompt Policy
System instructions, constraints, tone, refusal rules.

## Workflow
Step-by-step deterministic flow.

## Tools
Declared external tools/APIs and expected inputs/outputs.

## Memory
What to store, retention, redaction.

## Eval
Success criteria + test prompts + expected behaviors.

## Security
Allowed actions, blocked actions, escalation policy.
```

## Why this is scalable

1. Fast authoring: one file per skill for teams.
2. Easy review: PR diff stays human-readable.
3. Works with AI generation: models can create/update SKILL.md safely.
4. Build pipeline can compile MD -> manifest/wasm/mcp later.
5. Perfect for large registries where discovery + governance matters.

## Tradeoff

1. Raw runtime speed depends on later compilation step.
2. Needs strict schema validation for SKILL.md headers.

## Best Use

Use this if your priority is:
- massive skill count,
- contributor velocity,
- governance/audit,
- and AI-assisted authoring.

If your priority is pure runtime performance first, keep Option A/B runtime-native layout.
