"""The ALIV ADK for Python.

One long-lived ``aliv cli agent rpc`` runtime, many addressable sessions, and the
host tools your own process runs - with no Node and no third-party packages.

    from aliv_adk import AlivRuntime

    with AlivRuntime.start() as runtime:
        session = runtime.create_session(model="gpt-4o-mini", ephemeral=True)
        turn = session.prompt("Summarise this table in one line.")
        for event in turn:
            if event["type"] == "token":
                print(event["text"], end="", flush=True)
        print(turn.result().stop_reason)

See ``README.md`` for host tools, images and process ownership.
"""

from ._generated import EVENT_TAGS, PROTOCOL_VERSION
from .errors import AlivRpcError, AlivRuntimeError
from .install import (
    MANIFEST_URL,
    PLATFORM_KEYS,
    InstalledRuntime,
    AlivInstallError,
    bin_name,
    find_runtime,
    install_runtime,
    platform_key,
    runtime_root,
)
from .runtime import CLIENT_NAME, CLIENT_VERSION, AlivRuntime
from .session import HostTool, HostToolCall, AlivSession, AlivTurn, TurnResult

__version__ = CLIENT_VERSION

__all__ = [
    "CLIENT_NAME",
    "CLIENT_VERSION",
    "EVENT_TAGS",
    "MANIFEST_URL",
    "PLATFORM_KEYS",
    "PROTOCOL_VERSION",
    "HostTool",
    "HostToolCall",
    "InstalledRuntime",
    "AlivInstallError",
    "AlivRpcError",
    "AlivRuntime",
    "AlivRuntimeError",
    "AlivSession",
    "AlivTurn",
    "TurnResult",
    "__version__",
    "bin_name",
    "find_runtime",
    "install_runtime",
    "platform_key",
    "runtime_root",
]
