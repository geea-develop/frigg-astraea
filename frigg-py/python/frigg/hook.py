"""Frigg Claude Code hook adapter.

Reads PreToolUse JSON from stdin, checks against Frigg rules,
outputs a hook decision JSON to stdout.

Usage in .claude/settings.json:
    {
      "hooks": {
        "PreToolUse": [{
          "matcher": "*",
          "hooks": [{
            "type": "command",
            "command": "python -m frigg.hook --config rules.yaml --log frigg.log"
          }]
        }]
      }
    }
"""

import json
import sys
import argparse

from frigg import Frigg


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", default="rules.yaml")
    parser.add_argument("--log", default="frigg.log")
    args = parser.parse_args()

    try:
        frigg = Frigg.from_config(args.config, args.log)
    except Exception as e:
        print(f"Frigg init error: {e}", file=sys.stderr)
        sys.exit(0)  # non-blocking error

    hook_input = json.load(sys.stdin)
    tool_name = hook_input.get("tool_name", "")
    tool_input = hook_input.get("tool_input", {})

    # Map to Frigg action: "ToolName:arg_summary" for richer matching
    action_name = tool_name.lower()
    if isinstance(tool_input, dict):
        if "command" in tool_input:
            action_name = f"{tool_name.lower()}:{tool_input['command']}"
        elif "file_path" in tool_input:
            action_name = f"{tool_name.lower()}:{tool_input['file_path']}"

    result = frigg.check({"name": action_name, "params": tool_input if isinstance(tool_input, dict) else {}})

    if result["decision"] == "blocked":
        output = {
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "deny",
                "permissionDecisionReason": result.get("reason", "Blocked by Frigg"),
            }
        }
        json.dump(output, sys.stdout)
        sys.exit(0)

    if result["decision"] == "warned":
        output = {
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "additionalContext": f"[Frigg warning] {result.get('reason', '')}",
            }
        }
        json.dump(output, sys.stdout)

    sys.exit(0)


if __name__ == "__main__":
    main()
