#!/usr/bin/env bash
set -euo pipefail

# Starts/stops OPA instances used by the branch-opa-test benchmark cases
# in benches/host_call.rs.
#
# Mapping (must match benchmark code):
#   8181 -> allow.yaml
#   8182 -> argument-1-all-defined.yaml
#   8183 -> argument-1.yaml
#   8184 -> argument-3.yaml
#   8185 -> argument-all-no-constraint.yaml
#   8186 -> argument-all.yaml
#   8187 -> function.yaml
#
# Usage:
#   ./scripts/opa_test_instances.sh start
#   ./scripts/opa_test_instances.sh stop
#   ./scripts/opa_test_instances.sh restart
#   ./scripts/opa_test_instances.sh status

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RULES_FILE="${ROOT_DIR}/rules.rego"
PID_DIR="${ROOT_DIR}/.opa"
LOG_DIR="${ROOT_DIR}/.opa/logs"
PID_FILE="${PID_DIR}/opa_test_instances.pids"

NAMES=(
  "allow"
  "argument-1-all-defined"
  "argument-1"
  "argument-3"
  "argument-all-no-constraint"
  "argument-all"
  "function"
)

PORTS=(8181 8182 8183 8184 8185 8186 8187)
YAMLS=(
  "allow.yaml"
  "argument-1-all-defined.yaml"
  "argument-1.yaml"
  "argument-3.yaml"
  "argument-all-no-constraint.yaml"
  "argument-all.yaml"
  "function.yaml"
)

require_tools() {
  command -v opa >/dev/null 2>&1 || {
    echo "error: 'opa' binary not found in PATH" >&2
    exit 1
  }
  command -v curl >/dev/null 2>&1 || {
    echo "error: 'curl' binary not found in PATH" >&2
    exit 1
  }
}

is_pid_running() {
  local pid="$1"
  kill -0 "${pid}" >/dev/null 2>&1
}

wait_ready() {
  local port="$1"
  local retries=50
  local sleep_s=0.1

  for _ in $(seq 1 "${retries}"); do
    if curl -fsS "http://127.0.0.1:${port}/v1/policies" >/dev/null 2>&1; then
      return 0
    fi
    sleep "${sleep_s}"
  done

  return 1
}

start_instances() {
  require_tools

  [[ -f "${RULES_FILE}" ]] || {
    echo "error: rules file not found: ${RULES_FILE}" >&2
    exit 1
  }

  mkdir -p "${PID_DIR}" "${LOG_DIR}"

  if [[ -f "${PID_FILE}" ]]; then
    echo "PID file exists: ${PID_FILE}"
    echo "If instances are stale, run: $0 stop"
    exit 1
  fi

  : >"${PID_FILE}"

  local i
  for i in "${!PORTS[@]}"; do
    local name="${NAMES[i]}"
    local port="${PORTS[i]}"
    local yaml="${ROOT_DIR}/${YAMLS[i]}"
    local log_file="${LOG_DIR}/${name}.log"

    [[ -f "${yaml}" ]] || {
      echo "error: yaml not found: ${yaml}" >&2
      rm -f "${PID_FILE}"
      exit 1
    }

    # opa run in server mode. Load both policy and data file.
    # Data is branch-case-specific, matched by port.
    opa run \
      --server \
      --addr "127.0.0.1:${port}" \
      -l error \
      "${RULES_FILE}" \
      "${yaml}" \
      >"${log_file}" 2>&1 &

    local pid=$!
    echo "${pid}:${port}:${name}" >>"${PID_FILE}"

    if ! wait_ready "${port}"; then
      echo "error: OPA instance '${name}' on port ${port} did not become ready" >&2
      stop_instances || true
      exit 1
    fi

    echo "started OPA '${name}' on :${port} (pid ${pid})"
  done

  echo "all OPA test instances started"
}

stop_instances() {
  if [[ ! -f "${PID_FILE}" ]]; then
    echo "no PID file found (${PID_FILE}); nothing to stop"
    return 0
  fi

  local line pid port name
  while IFS=: read -r pid port name; do
    [[ -n "${pid}" ]] || continue
    if is_pid_running "${pid}"; then
      kill "${pid}" >/dev/null 2>&1 || true
      echo "stopped OPA '${name}' on :${port} (pid ${pid})"
    else
      echo "OPA '${name}' on :${port} already stopped (pid ${pid})"
    fi
  done <"${PID_FILE}"

  rm -f "${PID_FILE}"
}

status_instances() {
  if [[ ! -f "${PID_FILE}" ]]; then
    echo "no PID file found (${PID_FILE}); instances likely not running"
    return 0
  fi

  local line pid port name
  while IFS=: read -r pid port name; do
    [[ -n "${pid}" ]] || continue
    if is_pid_running "${pid}"; then
      echo "running: OPA '${name}' on :${port} (pid ${pid})"
    else
      echo "stale:   OPA '${name}' on :${port} (pid ${pid})"
    fi
  done <"${PID_FILE}"
}

case "${1:-}" in
  start)
    start_instances
    ;;
  stop)
    stop_instances
    ;;
  restart)
    stop_instances || true
    start_instances
    ;;
  status)
    status_instances
    ;;
  *)
    echo "usage: $0 {start|stop|restart|status}" >&2
    exit 1
    ;;
esac
