# Plugin Implementation Validation

## Summary

Both plugins are implemented **correctly** according to official documentation.

## ✅ OpenCode Plugin - CORRECT

**Location:** `plugins/opencode-cli-web-search/`

**Structure:**
```
opencode-cli-web-search/
├── package.json         # npm package metadata
├── index.ts             # Plugin implementation
├── README.md            # Documentation
└── LICENSE              # Apache-2.0
```

**Implementation Details:**
- ✅ Exports `WebSearchPlugin` function matching `Plugin` type
- ✅ Uses `@opencode-ai/plugin` SDK correctly
- ✅ Returns object with `tool` property
- ✅ Custom tools use `tool()` helper with Zod schema
- ✅ Executes CLI via Bun's `$` shell API
- ✅ Uses `client.app.log()` for structured logging
- ✅ Proper error handling with descriptive messages

**Self-Contained:** ✅ YES
- Single TypeScript file (can be copied standalone)
- Only requires `@opencode-ai/plugin` (peer dependency)
- No external file dependencies
- Can be published to npm as-is

## ✅ Claude Code Plugin - CORRECT

**Location:** `plugins/claude-cli-web-search/`

**Structure:**
```
claude-cli-web-search/
├── .claude-plugin/
│   └── plugin.json      # Plugin manifest (required)
├── skills/              # Agent Skills (model-invoked)
│   ├── web-search/
│   │   └── SKILL.md
│   └── fetch-url/
│       └── SKILL.md
├── README.md
└── LICENSE
```

**Implementation Details:**
- ✅ Has `.claude-plugin/plugin.json` manifest at root
- ✅ Uses `skills/` directory for Agent Skills
- ✅ Each skill is a directory with `SKILL.md`
- ✅ Skills have proper YAML frontmatter (`name`, `description`)
- ✅ Skills are model-invoked (Claude uses automatically)
- ✅ Plugin name becomes namespace (`cli-web-search:*`)

**Self-Contained:** ✅ YES
- Complete plugin directory with all files
- No external file dependencies
- Can be distributed as a directory
- Ready for marketplace or `--plugin-dir` use

## Key Differences Explained

### OpenCode
- **Type**: Custom Tools
- **Invocation**: Explicitly called by OpenCode
- **Language**: TypeScript
- **Distribution**: npm package
- **Dependencies**: `@opencode-ai/plugin` (peer)

### Claude Code
- **Type**: Agent Skills
- **Invocation**: Automatically by Claude (model-invoked)
- **Language**: Markdown with YAML frontmatter
- **Distribution**: Directory with manifest
- **Dependencies**: None (pure configuration)

## Common Dependency: cli-web-search

Both plugins are "self-contained" in that they include all plugin code, BUT:

**External Requirement:** Both require `cli-web-search` binary in PATH
- This is **expected and correct**
- Plugins are integration layers, not replacements
- Similar to how LSP plugins require language servers
- Documented in plugin `README.md` files

**Why this is correct:**
1. Plugins shouldn't duplicate cli-web-search functionality
2. Users get updates to cli-web-search independently
3. Follows separation of concerns (plugin = integration, CLI = functionality)
4. Common pattern: MCP plugins require MCP servers, LSP plugins require LSP servers

## Distribution

### OpenCode Plugin

**Option 1: npm (recommended)**
```bash
cd plugins/opencode-cli-web-search
npm publish
```

Users install with:
```json
{
  "plugin": ["opencode-cli-web-search"]
}
```

**Option 2: Local file**
```bash
cp plugins/opencode-cli-web-search/index.ts ~/.config/opencode/plugins/
```

### Claude Code Plugin

**Option 1: Marketplace (recommended when available)**
```bash
/plugin install cli-web-search
```

**Option 2: Local directory**
```bash
cp -r plugins/claude-cli-web-search ~/.claude-plugins/cli-web-search
claude --plugin-dir ~/.claude-plugins/cli-web-search
```

**Option 3: Direct use**
```bash
cd plugins/claude-cli-web-search
claude --plugin-dir .
```

## Validation Checklist

### OpenCode Plugin ✅

- [x] Exports named plugin function
- [x] Uses correct TypeScript types
- [x] Returns object with `tool` property
- [x] Tools use `tool()` helper
- [x] Has package.json with metadata
- [x] Has complete README
- [x] Has LICENSE file
- [x] Can be published to npm
- [x] Self-contained (one file)

### Claude Code Plugin ✅

- [x] Has `.claude-plugin/plugin.json` manifest
- [x] Plugin.json has required fields (name, version, description)
- [x] Plugin.json is valid JSON
- [x] Skills are in `skills/` directory
- [x] Each skill has `SKILL.md` file
- [x] Skills have proper YAML frontmatter
- [x] Skills have `name` and `description` fields
- [x] Has complete README
- [x] Has LICENSE file
- [x] Can be used with `--plugin-dir`
- [x] Self-contained (complete directory)

## Conclusion

✅ **Both plugins are correctly implemented** according to official documentation.

✅ **Both plugins are self-contained** - they include all plugin code and configuration.

✅ **External dependency (cli-web-search) is expected and correct** - plugins integrate with external tools, they don't replace them.

✅ **Ready for distribution** - Both can be shared via their respective channels (npm for OpenCode, marketplace/directory for Claude Code).

## Testing

### OpenCode
```bash
cd plugins/opencode-cli-web-search
cp index.ts ~/.config/opencode/plugins/cli-web-search.ts
opencode
# Ask: "Search for Rust tutorials"
```

### Claude Code
```bash
cd plugins/claude-cli-web-search
claude --plugin-dir .
# Then: /skills
# Ask: "Search for recent Tokio updates"
```

Both should work correctly if cli-web-search is installed and configured.
