# System Architecture

## Purpose and boundary

oxo-call is a Rust command-line tool that turns a natural-language request into
**one reviewable tool invocation**. It grounds generation in the target tool's
documentation and skills, can preview or execute that invocation, and records
its provenance.

oxo-call deliberately does not own dependency graphs, multi-step workflow
state, scheduler integration, resumability, or pipeline reporting. Those
concerns belong to [oxo-flow](https://github.com/Traitome/oxo-flow). A shell
pipeline, redirection, or `--input-list` fan-out is text the user can inspect;
it is not an oxo-call workflow graph.

## Workspace

| Crate | Purpose | Published |
|-------|---------|-----------|
| `oxo-call` (root) | End-user command-generation CLI and Rust API | Yes |
| `crates/license-issuer` | Maintainer-only license-signing utility | No |
| `crates/oxo-bench` | Command-generation evaluation suite | No |

## Command path

```text
CLI request
    │
    ├── License gate
    ├── Documentation resolution and structured extraction
    ├── Skill resolution (user → community → MCP → built-in)
    ├── Context assembly and command-generation mode selection
    ├── LLM generation and response validation
    ├── Command preview or execution
    ├── Optional retry / advisory verification
    └── JSONL provenance history
```

Documentation is resolved before skills and LLM generation. This
docs-first ordering supplies real flags and examples to the model, reducing
unsupported-flag generation.

### Generation modes

The internal command-generation pipeline can choose a fast or quality mode.
Quality mode may make several internal LLM calls—for task normalization or
documentation-derived context—but it still produces one command invocation.
This is a quality mechanism, not workflow orchestration.

`--scenario` selects a named grounding context (`bare`, `prompt`, `doc`,
`skill`, or `full`) for one generation request.

### Preview, execution, and batch fan-out

`dry-run` renders the generated command without starting a process. `run`
starts it after risk checks and optional confirmation. With `--input-list` or
`--input-items`, oxo-call generates one command template and applies it
independently to each supplied item. It does not infer dependencies among
items; users must make ordering and downstream semantics explicit in their
own scripts or use oxo-flow.

## Main modules

```text
main.rs                    command dispatch and license gate
cli.rs                     Clap command definitions
runner/
  core.rs                  docs → skill → LLM → preview/execute orchestration
  batch.rs                 independent input-item fan-out
  retry.rs                 bounded command retry
  utils.rs                 tool detection and runner support
command_pipeline.rs        fast/quality command-generation pipeline
context_scenario.rs        named command-generation context scenarios
docs.rs                    documentation resolution and caching
doc_processor.rs           flag and example extraction
skill.rs, mcp.rs           built-in, user, community, and MCP skills
context.rs                 LLM prompt-context assembly
llm/                       provider abstraction and response types
history.rs                 JSONL command provenance
sanitize.rs                sensitive-data handling for LLM contexts
config.rs                  TOML configuration and environment overrides
license.rs                 Ed25519 offline license verification
```

Other modules provide interactive chat, remote-server and job support,
formatting, cache management, tool discovery, and error handling. They support
the command-generation interface; none establishes a DAG execution model.

## Design invariants

1. **One generated invocation:** one `run` or `dry-run` request has one target
   tool invocation to preview or execute.
2. **Docs before model:** tool documentation is collected before loading skills
   or asking the LLM for arguments.
3. **Human-reviewable execution:** generated shell text is shown or available
   before execution; dangerous commands require confirmation.
4. **Portable provenance:** command history records the request, generated
   command, model context, timestamps, and execution result.
5. **Explicit orchestration boundary:** DAGs, retries across dependent steps,
   scheduling, environments, and resume semantics are delegated to oxo-flow.

## Extensibility

- **Providers:** `llm/` provides a common interface for supported LLM backends.
- **Skills:** `SkillManager` preserves the precedence order user → community →
  MCP → built-in.
- **Documentation:** the resolver caches local and remote help material for
  reproducible grounding.
- **Rust API:** `lib.rs` exports the components intended for programmatic
  integrations.

For release direction, compatibility commitments, and the oxo-flow handoff,
see the repository [Roadmap](https://github.com/Traitome/oxo-call/blob/main/ROADMAP.md).
