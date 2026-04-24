# Plan: Optimize oxo-call Skill Commands for Small Model Compatibility

## Context

The user wants to optimize the oxo-call skill series commands to integrate check-skill knowledge for reliable skill generation. Focus: making this work well with ollama small models (without web search and strong model capabilities).

**User Requirements:**
- Integration level: New skill-generator skill
- Small model focus: Structured workflow
- Scope: Full skill series (create, verify, polish)
- Workflow structure: Mirror check-skill 7-step

## Current Implementation Analysis

### Existing Prompts (src/llm/prompt.rs)
- `build_skill_generate_prompt`: Basic prompt asking LLM to generate skill - **lacks workflow knowledge**
- `skill_reviewer_system_prompt`: Contains format requirements - used for verify/polish
- `mini_skill_generation_system_prompt`: For mini-skill from docs

### Missing Capabilities
1. No embedded check-skill workflow knowledge in generate prompt
2. No local --help fetching for skill generation
3. No structured step-by-step guidance for small models

## Solution Design

### 1. Create `skills/skill-generator.md`

Embeds check-skill workflow knowledge as a skill that guides LLM skill creation.

**Key Content:**
```markdown
---
name: skill-generator
category: utility
description: Generate comprehensive skill.md files using structured 7-step workflow
tags: [skill, creation, generation, workflow, template]
---

## Concepts
- Skill file format: YAML front-matter + Markdown sections
- Minimum requirements: 5 examples, 3 concepts, 3 pitfalls
- Subcommand-first vs flags-first pattern (critical for many tools)
- Example format: ### task → **Args:** `args` → **Explanation:** text
- Case-insensitive tool lookup: STAR/star, R/r
- Documentation hierarchy: local --help > source_url > web search
- Key flag extraction from help output
- Common workflow dependencies (index before align)

## Pitfalls
- Starting args with flags instead of subcommand
- Vague concepts/pitfalls without actionable guidance
- Fabricating unverified flags
- Case mismatch between skill name and binary
- Missing minimum depth requirements (5/3/3)
- Incorrect example format
- Overwriting existing correct content

## Examples
- Generate skill for tool installed locally (samtools)
- Generate skill for tool not installed locally
- Generate skill for multi-subcommand tool (bwa, gatk)
- Generate skill for single-command tool (fastp)

## Workflow (Step-by-Step)
### Step 1: Identify tool name and check installation
- Try PATH lookup (case-insensitive)
- Try pixi global list
- Try conda environments
- Note: if not found, skill generation will be template-only

### Step 2: Fetch local help documentation
- Run `tool --help`, `tool -h`, `tool help`, bare invocation
- For subcommand tools: enumerate all subcommands
- Extract: flags, subcommands, I/O formats

### Step 3: Analyze tool structure
- Determine: subcommand-first or flags-first?
- Identify: most common operations
- Note: threading options, output format flags

### Step 4: Write Concepts section
- Focus on: data model, I/O formats, key behaviors
- Include: workflow dependencies, threading
- Minimum: 3 concepts

### Step 5: Write Pitfalls section
- Focus on: common mistakes + consequences
- Include: argument ordering, version issues
- Minimum: 3 pitfalls

### Step 6: Write Examples section
- Cover: basic, intermediate, advanced usage
- Format: ### task → **Args:** `args` → **Explanation:**
- Args NEVER start with tool name
- Minimum: 5 examples

### Step 7: Validate and output
- Check minimum requirements met
- Verify format correctness
- Output complete skill.md
```

### 2. Enhance `build_skill_generate_prompt` (src/llm/prompt.rs)

**New Design:**
```rust
pub fn build_skill_generate_prompt(tool: &str, help_output: Option<&str>, generator_skill: &str) -> String {
    // Include skill-generator skill content as guidance
    // Include local --help output if available
    // Provide structured step-by-step instructions
}
```

**Key Changes:**
- Load skill-generator skill content
- Inject fetched --help output
- Provide explicit 7-step workflow
- Include validation criteria

### 3. Update `skill create --llm` Flow (src/skill.rs)

**New Flow:**
1. Load skill-generator skill
2. Fetch local --help for tool (if available)
3. Build enhanced prompt with:
   - skill-generator workflow
   - local --help content
4. Call LLM
5. Parse and validate output
6. Save to output path (if -o specified)

### 4. Minor Enhancements to verify/polish

- Already use skill_reviewer_system_prompt - keep existing behavior
- Ensure prompts reference check-skill concepts where helpful

## Implementation Order

### Phase 1: Core skill-generator.md
1. Create `skills/skill-generator.md` with complete workflow
2. Add to BUILTIN_SKILLS in src/skill.rs

### Phase 2: Enhanced prompt building
3. Modify `build_skill_generate_prompt` to accept help_output and generator_skill
4. Add `build_skill_generate_prompt_enhanced` function

### Phase 3: Integration in skill.rs
5. Add `load_skill_generator()` function
6. Add `fetch_tool_help_for_skill()` function
7. Modify skill create --llm handler to use enhanced flow

### Phase 4: Testing
8. Test with local tool (samtools)
9. Test with unavailable tool
10. Validate small model output quality

## Files to Modify

| File | Changes |
|------|---------|
| `skills/skill-generator.md` | NEW - core workflow skill |
| `src/skill.rs` | Add load_skill_generator(), fetch_tool_help(), enhance create flow |
| `src/llm/prompt.rs` | Enhance build_skill_generate_prompt() |
| `src/lib.rs` or handler | Update skill create command handler |

## Verification

1. `skill create --llm samtools` should:
   - Load skill-generator skill
   - Fetch samtools --help locally
   - Generate skill meeting minimum requirements

2. `skill create --llm non-existent-tool` should:
   - Generate template-only skill
   - Still meet minimum format requirements

3. Test with ollama small model:
   - Verify structured workflow is followed
   - Verify output format is correct