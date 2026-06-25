"""Frigg middleware for LangChain v1 agents.

Usage:
    from frigg.middleware import FriggMiddleware
    from langchain.agents import create_agent

    agent = create_agent(
        model="...",
        tools=[...],
        middleware=[FriggMiddleware("rules.yaml", "frigg.log")],
    )
"""

from frigg import Frigg


try:
    from langchain.agents.middleware import AgentMiddleware
    from langchain.messages import ToolMessage
    from langchain.tools.tool_node import ToolCallRequest

    class FriggMiddleware(AgentMiddleware):
        """LangChain v1 middleware that checks tool calls against Frigg rules."""

        def __init__(self, config_path: str, log_path: str = "frigg.log"):
            super().__init__()
            self._frigg = Frigg.from_config(config_path, log_path)

        def wrap_tool_call(self, request: ToolCallRequest, handler):
            tool_name = request.tool_call["name"]
            tool_args = request.tool_call.get("args", {})
            action = {"name": tool_name, "params": tool_args if isinstance(tool_args, dict) else {}}
            result = self._frigg.check(action)

            if result["decision"] == "blocked":
                return ToolMessage(
                    content=f"[Frigg] Blocked: {result.get('reason', 'rule violation')}",
                    tool_call_id=request.tool_call.get("id", ""),
                )
            if result["decision"] == "warned":
                import sys
                print(f"[Frigg] Warning: {result.get('reason', '')}", file=sys.stderr)

            return handler(request)

except ImportError:
    # LangChain not installed — expose a stub so import doesn't crash
    class FriggMiddleware:  # type: ignore[no-redef]
        def __init__(self, *args, **kwargs):
            raise ImportError("langchain is required for FriggMiddleware. Install with: pip install langchain")
