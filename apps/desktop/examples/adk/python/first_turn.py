"""Run one ALIV Agent task from Python and stream the answer.

Starts `aliv cli agent run` with `--output-format stream-json`, prints the answer
as it streams, and reads the terminal `result` record. Standard library only.

    python3 first_turn.py "Explain what a mutex is in one sentence."

Set ALIV_BIN to use a aliv binary that is not on PATH, and ALIV_MODEL to pick a model.
"""

import json
import os
import subprocess
import sys
import tempfile

ALIV = os.environ.get("ALIV_BIN", "aliv")


def main() -> int:
    task = sys.argv[1] if len(sys.argv) > 1 else "Reply with one short sentence: what is ALIV Agent?"
    # A fresh, empty project: the agent's own file and shell tools are confined
    # to it, so an example cannot touch the directory you ran it from.
    with tempfile.TemporaryDirectory() as project:
        cmd = [ALIV, "cli", "agent", "run", "--project", project, "--output-format", "stream-json"]
        if os.environ.get("ALIV_MODEL"):
            cmd += ["--model", os.environ["ALIV_MODEL"]]
        cmd.append(task)

        run = subprocess.Popen(cmd, stdout=subprocess.PIPE, text=True)
        result = None
        for line in run.stdout:
            record = json.loads(line)
            if record["type"] == "init":
                print(f"[session {record['session_id']} on {record['model']}]", file=sys.stderr)
            elif record["type"] == "token":
                print(record["text"], end="", flush=True)
            elif record["type"] == "result":
                result = record
        run.wait()

    print()
    if result is None:
        print(f"aliv exited with {run.returncode} before printing a result", file=sys.stderr)
        return 1
    if result["is_error"]:
        print(f"run failed: {result['error']['code']}: {result['error']['message']}", file=sys.stderr)
        return 1
    print(f"[{result['stop_reason']} after {result['num_turns']} turn(s)]", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
