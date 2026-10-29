#!/usr/bin/env bash
# End-to-end OTel check: a real `aliv` binary, a real OpenTelemetry Collector
# (Docker), and a fake OpenAI-compatible model, so it needs no provider key.
#
#   scripts/otel-e2e.sh [path/to/aliv]
#
# ALIV_E2E_REAL_MODEL=1   use your own ~/.aliv config and default model instead
# LANGFUSE_PUBLIC_KEY / LANGFUSE_SECRET_KEY / LANGFUSE_BASE_URL
#                        also forward traces to Langfuse (keys never logged)
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
aliv="${1:-$here/../src-tauri/target/debug/aliv}"
[ -x "$aliv" ] || { echo "no aliv binary at $aliv (build src-tauri/aliv-cli first)"; exit 2; }
command -v docker >/dev/null || { echo "docker required"; exit 2; }

work="$(mktemp -d)"
name="aliv-otel-e2e-$$"
port_otlp="${ALIV_E2E_OTLP_PORT:-14318}"
port_model="${ALIV_E2E_MODEL_PORT:-18765}"
cleanup() {
  docker rm -f "$name" >/dev/null 2>&1 || true
  [ -n "${model_pid:-}" ] && kill "$model_pid" 2>/dev/null || true
  rm -rf "$work"
}
trap cleanup EXIT
mkdir -p "$work/out" "$work/proj" "$work/home/.aliv"
chmod 777 "$work/out"

config="$here/otel-e2e/collector.yaml"
docker_env=()
if [ -n "${LANGFUSE_PUBLIC_KEY:-}" ] && [ -n "${LANGFUSE_SECRET_KEY:-}" ]; then
  config="$here/otel-e2e/collector.langfuse.yaml"
  printf 'LANGFUSE_OTLP_ENDPOINT=%s/api/public/otel\nLANGFUSE_AUTH=%s\n' \
    "${LANGFUSE_BASE_URL:-https://cloud.langfuse.com}" \
    "$(printf '%s:%s' "$LANGFUSE_PUBLIC_KEY" "$LANGFUSE_SECRET_KEY" | base64 | tr -d '\n')" \
    > "$work/lf.env"
  docker_env=(--env-file "$work/lf.env")
  echo "forwarding traces to Langfuse"
fi
docker run -d --name "$name" -p "127.0.0.1:$port_otlp:4318" \
  -v "$config:/etc/otelcol-contrib/config.yaml:ro" -v "$work/out:/out" \
  ${docker_env[@]+"${docker_env[@]}"} \
  otel/opentelemetry-collector-contrib:0.161.0 >/dev/null
for _ in $(seq 50); do curl -s -o /dev/null "http://127.0.0.1:$port_otlp" && break; sleep 0.2; done

export ALIV_AGENT_ENABLE_TELEMETRY=1 OTEL_TRACES_EXPORTER=otlp
export OTEL_EXPORTER_OTLP_ENDPOINT="http://127.0.0.1:$port_otlp"
export OTEL_EXPORTER_OTLP_PROTOCOL="${OTEL_EXPORTER_OTLP_PROTOCOL:-http/protobuf}"
export OTEL_RESOURCE_ATTRIBUTES="deployment.environment=otel-e2e"

args=(cli agent run --output-format stream-json)
if [ "${ALIV_E2E_REAL_MODEL:-0}" = 1 ]; then
  prompt="Run the shell command: echo otel-e2e. Then reply done."
  (cd "$work/proj" && "$aliv" "${args[@]}" "$prompt" >"$work/run.jsonl" 2>"$work/run.err") || true
else
  python3 "$here/otel-e2e/fake_model.py" "$port_model" & model_pid=$!
  cat > "$work/home/.aliv/config.toml" <<TOML
default_model = "fake-model"
[providers.fake]
base_url = "http://127.0.0.1:$port_model/v1"
models = ["fake-model"]
TOML
  sleep 0.5
  (cd "$work/proj" && HOME="$work/home" "$aliv" "${args[@]}" "run echo" \
    >"$work/run.jsonl" 2>"$work/run.err") || true
fi
sleep 3   # collector batch timeout + file flush
if [ ! -s "$work/out/otel.jsonl" ]; then
  echo "collector received nothing"; tail -20 "$work/run.err"; exit 1
fi
python3 "$here/otel-e2e/check.py" "$work/out/otel.jsonl" || { tail -20 "$work/run.err"; exit 1; }
