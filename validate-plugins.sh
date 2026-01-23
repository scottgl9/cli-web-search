#!/bin/bash

# Plugin Validation Script for cli-web-search
# This script validates that both OpenCode and Claude Code plugins are properly configured

set -e

COLOR_GREEN='\033[0;32m'
COLOR_RED='\033[0;31m'
COLOR_YELLOW='\033[1;33m'
COLOR_BLUE='\033[0;34m'
COLOR_RESET='\033[0m'

echo -e "${COLOR_BLUE}=== cli-web-search Plugin Validator ===${COLOR_RESET}\n"

# Track validation status
all_valid=true

# Function to check if a file exists
check_file() {
    if [ -f "$1" ]; then
        echo -e "${COLOR_GREEN}✓${COLOR_RESET} $1"
        return 0
    else
        echo -e "${COLOR_RED}✗${COLOR_RESET} $1 (missing)"
        all_valid=false
        return 1
    fi
}

# Function to check if a directory exists
check_dir() {
    if [ -d "$1" ]; then
        echo -e "${COLOR_GREEN}✓${COLOR_RESET} $1/"
        return 0
    else
        echo -e "${COLOR_RED}✗${COLOR_RESET} $1/ (missing)"
        all_valid=false
        return 1
    fi
}

# Check cli-web-search binary
echo -e "${COLOR_BLUE}Checking cli-web-search installation:${COLOR_RESET}"
if command -v cli-web-search &> /dev/null; then
    version=$(cli-web-search --version 2>&1 || echo "unknown")
    echo -e "${COLOR_GREEN}✓${COLOR_RESET} cli-web-search is installed: $version"
else
    echo -e "${COLOR_RED}✗${COLOR_RESET} cli-web-search not found in PATH"
    echo -e "  Install with: cargo install --git https://github.com/scottgl9/cli-web-search.git"
    all_valid=false
fi

# Check provider configuration
echo -e "\n${COLOR_BLUE}Checking provider configuration:${COLOR_RESET}"
if command -v cli-web-search &> /dev/null; then
    if cli-web-search providers &> /dev/null; then
        provider_count=$(cli-web-search providers 2>/dev/null | grep -c "✓" || echo "0")
        if [ "$provider_count" -gt 0 ]; then
            echo -e "${COLOR_GREEN}✓${COLOR_RESET} At least one provider is configured"
        else
            echo -e "${COLOR_YELLOW}!${COLOR_RESET} No providers configured"
            echo -e "  Configure one with: cli-web-search config set providers.duckduckgo.enabled true"
        fi
    fi
fi

# Check OpenCode plugin
echo -e "\n${COLOR_BLUE}OpenCode Plugin Structure:${COLOR_RESET}"
check_dir ".opencode"
check_dir ".opencode/plugins"
check_file ".opencode/plugins/web-search.ts"
check_file ".opencode/README.md"

# Validate OpenCode plugin syntax
if [ -f ".opencode/plugins/web-search.ts" ]; then
    if grep -q "export const WebSearchPlugin" ".opencode/plugins/web-search.ts"; then
        echo -e "${COLOR_GREEN}✓${COLOR_RESET} Plugin exports WebSearchPlugin"
    else
        echo -e "${COLOR_RED}✗${COLOR_RESET} Plugin missing WebSearchPlugin export"
        all_valid=false
    fi
    
    if grep -q "web_search:" ".opencode/plugins/web-search.ts"; then
        echo -e "${COLOR_GREEN}✓${COLOR_RESET} Plugin defines web_search tool"
    else
        echo -e "${COLOR_RED}✗${COLOR_RESET} Plugin missing web_search tool"
        all_valid=false
    fi
    
    if grep -q "fetch_url:" ".opencode/plugins/web-search.ts"; then
        echo -e "${COLOR_GREEN}✓${COLOR_RESET} Plugin defines fetch_url tool"
    else
        echo -e "${COLOR_RED}✗${COLOR_RESET} Plugin missing fetch_url tool"
        all_valid=false
    fi
fi

# Check Claude Code plugin
echo -e "\n${COLOR_BLUE}Claude Code Plugin Structure:${COLOR_RESET}"
check_dir ".claude-plugin"
check_file ".claude-plugin/plugin.json"
check_file ".claude-plugin/README.md"
check_dir "skills"
check_dir "skills/web-search"
check_file "skills/web-search/SKILL.md"
check_dir "skills/fetch-url"
check_file "skills/fetch-url/SKILL.md"

# Validate Claude Code plugin.json
if [ -f ".claude-plugin/plugin.json" ]; then
    if command -v jq &> /dev/null; then
        if jq empty ".claude-plugin/plugin.json" 2>/dev/null; then
            echo -e "${COLOR_GREEN}✓${COLOR_RESET} plugin.json is valid JSON"
            
            name=$(jq -r '.name' ".claude-plugin/plugin.json")
            if [ "$name" != "null" ] && [ -n "$name" ]; then
                echo -e "${COLOR_GREEN}✓${COLOR_RESET} Plugin name: $name"
            else
                echo -e "${COLOR_RED}✗${COLOR_RESET} Plugin name missing or invalid"
                all_valid=false
            fi
        else
            echo -e "${COLOR_RED}✗${COLOR_RESET} plugin.json is invalid JSON"
            all_valid=false
        fi
    else
        echo -e "${COLOR_YELLOW}!${COLOR_RESET} jq not found, skipping JSON validation"
    fi
fi

# Validate skill files
echo -e "\n${COLOR_BLUE}Validating Claude Code Skills:${COLOR_RESET}"
for skill in skills/*/SKILL.md; do
    if [ -f "$skill" ]; then
        skill_name=$(basename $(dirname "$skill"))
        
        # Check for frontmatter
        if head -5 "$skill" | grep -q "^---$"; then
            echo -e "${COLOR_GREEN}✓${COLOR_RESET} $skill_name: Has frontmatter"
            
            # Check for required fields
            if grep -q "^name:" "$skill"; then
                echo -e "${COLOR_GREEN}✓${COLOR_RESET} $skill_name: Has name field"
            else
                echo -e "${COLOR_RED}✗${COLOR_RESET} $skill_name: Missing name field"
                all_valid=false
            fi
            
            if grep -q "^description:" "$skill"; then
                echo -e "${COLOR_GREEN}✓${COLOR_RESET} $skill_name: Has description field"
            else
                echo -e "${COLOR_RED}✗${COLOR_RESET} $skill_name: Missing description field"
                all_valid=false
            fi
        else
            echo -e "${COLOR_RED}✗${COLOR_RESET} $skill_name: Missing frontmatter"
            all_valid=false
        fi
    fi
done

# Check documentation
echo -e "\n${COLOR_BLUE}Documentation:${COLOR_RESET}"
check_file "PLUGINS.md"
check_file "QUICK_START_PLUGINS.md"

# Final summary
echo -e "\n${COLOR_BLUE}=== Validation Summary ===${COLOR_RESET}"
if [ "$all_valid" = true ]; then
    echo -e "${COLOR_GREEN}✓ All validation checks passed!${COLOR_RESET}"
    echo -e "\nYou can now use the plugins:"
    echo -e "  ${COLOR_BLUE}OpenCode:${COLOR_RESET}     cd $(pwd) && opencode"
    echo -e "  ${COLOR_BLUE}Claude Code:${COLOR_RESET}  cd $(pwd) && claude --plugin-dir ."
    exit 0
else
    echo -e "${COLOR_RED}✗ Some validation checks failed${COLOR_RESET}"
    echo -e "\nPlease fix the issues above before using the plugins."
    exit 1
fi
