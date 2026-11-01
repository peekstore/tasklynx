# AI Setup

AI is opt-in. The default build has no networking dependency and the default provider is `none`.

## Build with AI support

```bash
cargo build --features ai
```

The `ollama` and `openai-compatible` providers are compiled only with the `ai` feature. Core task management and offline triage work without it.

## Configure a provider

Add an `[ai]` table to `config.toml`:

```toml
[ai]
provider = "none"
model = "llama3.1"
endpoint = "http://127.0.0.1:11434"
max_context_tasks = 50
include_descriptions = true
redact = true
```

| Key | Default | Purpose |
| --- | --- | --- |
| `provider` | `"none"` | `none`, `ollama`, or `openai-compatible` |
| `model` | `"llama3.1"` | Model name sent to the provider |
| `endpoint` | `"http://127.0.0.1:11434"` | Provider base URL |
| `max_context_tasks` | `50` | Maximum open tasks included in prompt context |
| `include_descriptions` | `true` | Include task descriptions in context |
| `redact` | `true` | Redact common secret patterns before sending prompts |

Set `TASKLYNX_AI_API_KEY` for hosted OpenAI-compatible endpoints. API keys are never written to `config.toml`.

## Ollama

```toml
[ai]
provider = "ollama"
model = "llama3.1"
endpoint = "http://127.0.0.1:11434"
```

```bash
tx ai doctor
```

## OpenAI-compatible endpoints

```toml
[ai]
provider = "openai-compatible"
model = "gpt-4.1-mini"
endpoint = "https://api.openai.com/v1"
```

```bash
export TASKLYNX_AI_API_KEY="your-api-key"
tx ai doctor
```

Include the API version path in `endpoint`, such as `/v1` or `/v1beta/openai`.

## Google Gemini

```toml
[ai]
provider = "openai-compatible"
model = "gemini-2.5-flash"
endpoint = "https://generativelanguage.googleapis.com/v1beta/openai"
```

```bash
export TASKLYNX_AI_API_KEY="your-ai-studio-key"
tx ai doctor
```

## Commands

```bash
tx ai plan "Ship the billing API"
tx ai plan "Ship the billing API" --apply

tx ai breakdown <id>
tx ai breakdown <id> --apply

tx ai triage
tx ai standup --days 1
tx ai review feature-auth
tx ai ask "What should I do next?"
```

- Mutating commands preview by default and require `--apply`.
- `tx ai triage` falls back to deterministic `tx next` ranking when no provider is configured.
- `--limit N` and `--no-description` tighten context.
- CLI, TUI, and web expose the same preview and apply behavior.

## Privacy and safety

- AI commands are the only code path that can send task data to a provider.
- Context is limited to the selected project and branch.
- Common secrets are redacted when `ai.redact` is enabled.
- Prompt contents are never logged. `TASKLYNX_AI_LOG=1` logs prompt size and provider latency only.
- Responses are validated against a strict schema before tasks are created.
- Blocked tasks are excluded from triage unless explicitly requested.
- AI output never auto-completes, auto-deletes, or auto-commits anything.

## Troubleshooting

```bash
tx ai doctor
```

Use `tx ai doctor` to check provider reachability, model availability, and configuration. If the binary was built without the `ai` feature, provider-backed commands explain how to rebuild.
