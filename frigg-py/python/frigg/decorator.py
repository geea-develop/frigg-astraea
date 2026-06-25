"""Generic decorator to guard any Python function with Frigg rules.

Usage:
    from frigg import Frigg
    from frigg.decorator import guard

    frigg = Frigg.from_config("rules.yaml", "frigg.log")

    @guard(frigg)
    def send_email(to, subject):
        ...

    @guard(frigg, action_name="http_post")
    def call_api(url):
        ...
"""

import functools
import sys


def guard(frigg_instance, action_name=None):
    """Decorator that checks Frigg rules before executing the function.

    Args:
        frigg_instance: A Frigg instance (from Frigg.from_config)
        action_name: Override action name. Defaults to function name.
    """
    def decorator(fn):
        @functools.wraps(fn)
        def wrapper(*args, **kwargs):
            name = action_name or fn.__name__
            result = frigg_instance.check({"name": name, "params": {}})

            if result["decision"] == "blocked":
                raise PermissionError(
                    f"[Frigg] Action '{name}' blocked: {result.get('reason', 'rule violation')}"
                )
            if result["decision"] == "warned":
                print(f"[Frigg] Warning for '{name}': {result.get('reason', '')}", file=sys.stderr)

            return fn(*args, **kwargs)
        return wrapper
    return decorator
