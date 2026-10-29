# ALIV Agent cookbook

Drive ALIV Agent from your own code. Each recipe starts `aliv cli agent run`
as a child process and talks to it over its `stream-json` channel: one JSON
record per line on stdout, and, for interactive runs, one JSON message per
line on stdin. Every recipe uses only the standard library of its language.

The complete, runnable sources live in [`examples/adk/`](../../examples/adk).
They run on every test pass against the real `aliv` binary and a scripted
model (`src-tauri/aliv-cli/tests/adk_examples.rs`), and every excerpt on this
page is checked to be a verbatim quote of them.

## Before you start

- Install the `aliv` CLI, or point `ALIV_BIN` at a build of it.
- Configure a provider once, for example the desktop app's local server:

  ```sh
  aliv config set --provider aliv --base-url http://localhost:1337/v1 --model <model>
  ```

  or any OpenAI-compatible endpoint with `--api-key` and `--base-url`.
- Set `ALIV_MODEL` to choose a model per run.

Each recipe runs the agent in a fresh temporary project with `--project`, so
the agent's own file and shell tools cannot touch the directory you ran it
from.

## Records you will read

| `type`               | When                         | Fields used here                                |
| -------------------- | ---------------------------- | ----------------------------------------------- |
| `init`               | First record of a run        | `session_id`, `model`, `tool_specs`             |
| `token`              | Each streamed text fragment  | `text`                                          |
| `permission_request` | A tool needs approval        | `request_id`, `tool_name`                       |
| `tool_request`       | The model called a host tool | `request_id`, `tool_name`, `args`               |
| `result`             | Last record of a run         | `is_error`, `error`, `stop_reason`, `num_turns` |

## Recipe 1: run one task and stream the answer

Start the run, print `token` text as it arrives, and keep the final `result`
record to report success or failure.

### Node.js

```js filename="examples/adk/js/first-turn.mjs"
const run = spawn(aliv, args, { stdio: ['ignore', 'pipe', 'inherit'] })
const exited = new Promise((resolve) => run.on('close', resolve))

let result = null
for await (const line of createInterface({ input: run.stdout })) {
  const record = JSON.parse(line)
  if (record.type === 'init') {
    console.error(`[session ${record.session_id} on ${record.model}]`)
  } else if (record.type === 'token') {
    process.stdout.write(record.text)
  } else if (record.type === 'result') {
    result = record
  }
}
const code = await exited
```

### Python

```python filename="examples/adk/python/first_turn.py"
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
```

Run it:

```sh
node examples/adk/js/first-turn.mjs "Explain what a mutex is in one sentence."
python3 examples/adk/python/first_turn.py "Explain what a mutex is in one sentence."
```

## Recipe 2: give the agent a tool your code runs

Declare tools in a JSON file and pass it with `--host-tools`. The model sees
each one as `host__<name>`. When the model calls it, the run emits a
`tool_request`; your code runs the handler and answers with a `tool_result`
on stdin. Starting the run with `--safe` makes every side effect, including
host tools, wait for a `permission` answer from your code.

### Declare the tool

```js filename="examples/adk/js/host-tool.mjs"
const tools = [
  {
    name: 'get_weather',
    description: 'Current weather for a city.',
    parameters: {
      type: 'object',
      properties: { city: { type: 'string' } },
      required: ['city'],
      additionalProperties: false,
    },
  },
]
```

```python filename="examples/adk/python/host_tool.py"
TOOLS = [
    {
        "name": "get_weather",
        "description": "Current weather for a city.",
        "parameters": {
            "type": "object",
            "properties": {"city": {"type": "string"}},
            "required": ["city"],
            "additionalProperties": False,
        },
    }
]
```

### Start the run with stdin open

```js filename="examples/adk/js/host-tool.mjs"
const args = [
  'cli', 'agent', 'run',
  '--project', project,
  '--safe',
  '--output-format', 'stream-json',
  '--input-format', 'stream-json',
  '--host-tools', toolsFile,
]
if (process.env.ALIV_MODEL) args.push('--model', process.env.ALIV_MODEL)
args.push(task)
```

### Answer permission prompts and tool calls

```js filename="examples/adk/js/host-tool.mjs"
    case 'permission_request':
      // Under `--safe` a host tool is prompted like an MCP tool, and so are
      // writes and shell commands. Approve ours; refuse the rest.
      send({
        type: 'permission',
        request_id: record.request_id,
        decision: record.tool_name.startsWith('host__') ? 'allow_once' : 'deny',
      })
      break
    case 'tool_request': {
      // `tool_name` is the name declared above, without `host__`.
      let content
      let isError = false
      try {
        const handler = handlers[record.tool_name]
        if (!handler) throw new Error(`no handler for ${record.tool_name}`)
        content = JSON.stringify(handler(record.args))
      } catch (error) {
        // The model sees this as the tool's error.
        content = `${error.name}: ${error.message}`
        isError = true
      }
      console.error(`\n[${record.tool_name}(${JSON.stringify(record.args)}) -> ${content}]`)
      send({ type: 'tool_result', request_id: record.request_id, content, is_error: isError })
      break
    }
```

```python filename="examples/adk/python/host_tool.py"
            elif kind == "permission_request":
                # Under `--safe` a host tool is prompted like an MCP tool, and
                # so are writes and shell commands. Approve ours; refuse the rest.
                ours = record["tool_name"].startswith("host__")
                send({
                    "type": "permission",
                    "request_id": record["request_id"],
                    "decision": "allow_once" if ours else "deny",
                })
            elif kind == "tool_request":
                # `tool_name` is the name declared above, without `host__`.
                handler = HANDLERS.get(record["tool_name"])
                try:
                    if handler is None:
                        raise KeyError(f"no handler for {record['tool_name']}")
                    content, is_error = json.dumps(handler(record["args"])), False
                except Exception as error:  # the model sees this as the tool's error
                    content, is_error = f"{type(error).__name__}: {error}", True
                print(f"\n[{record['tool_name']}({json.dumps(record['args'])}) -> {content}]", file=sys.stderr)
                send({
                    "type": "tool_result",
                    "request_id": record["request_id"],
                    "content": content,
                    "is_error": is_error,
                })
```

If a declared tool is missing from `init.tool_specs`, policy withheld it (a
deny list or plan mode) and the model will never call it.

Run it:

```sh
node examples/adk/js/host-tool.mjs "What's the weather in Hanoi?"
python3 examples/adk/python/host_tool.py "What's the weather in Hanoi?"
```
