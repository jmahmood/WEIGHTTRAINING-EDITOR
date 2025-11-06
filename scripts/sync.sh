#!/usr/bin/env bash
set -Eeuo pipefail

# sync.sh — SCP + USB filesystem sync helper for Linux devices
#
# Subcommands:
#   send                      Send local outbox -> remote inbox
#   receive                   Receive remote outbox -> local inbox
#   setup-remote              Create remote dir tree under remote_root
#   discover-remote-dirs      List candidate writable dirs on remote host (SSH)
#   auto                      Choose transport (usb-fs if healthy else ssh)
#
# All configuration is passed by the application via flags; no manual configs.
#
# Example:
#   scripts/sync.sh send \
#     --transport ssh \
#     --local-root /home/user/.config/weightlifting/plans \
#     --remote-host sync@device.local \
#     --remote-root /home/sync/plans
#

############## Defaults and Globals ##############

TRANSPORT="auto"           # ssh | usb-fs | auto
LOCAL_ROOT=""
REMOTE_HOST=""            # user@host
REMOTE_PORT=""            # optional
REMOTE_ROOT=""
USB_MOUNT=""              # mountpoint for usb-fs
DRY_RUN=0
TIMEOUT=30
VERBOSE=0
ARCHIVE=1
ACK_REMOTE=0
REFRESH_CACHE=0
JOBS=1
SCP_LAST_ERR=""

SUBCOMMAND=""

# XDG state dir for auto-generated caches
XDG_STATE_HOME_DEFAULT="$HOME/.local/state"
STATE_HOME="${XDG_STATE_HOME:-$XDG_STATE_HOME_DEFAULT}"
STATE_DIR="$STATE_HOME/weightlifting/sync"
REMOTE_DIR_CACHE_DIR="$STATE_DIR/remote_dirs"

START_TS_NS=$(date +%s%3N 2>/dev/null || true)
if [[ -z "${START_TS_NS:-}" ]]; then
  # Fallback when %3N isn't supported
  START_TS_S=$(date +%s)
  START_TS_NS=$(( START_TS_S * 1000 ))
fi

############## Utilities ##############

err() { echo "[sync] $*" >&2; }
vlog() { if [[ "$VERBOSE" -eq 1 ]]; then err "$*"; fi; }

json_escape() {
  # Escapes a string for safe JSON embedding
  local s
  s=$(printf '%s' "$1" | sed -E "s/\\\\/\\\\\\\\/g; s/\"/\\\"/g; s/\t/\\t/g; s/\r/\\r/g; s/\n/\\n/g")
  printf '%s' "$s"
}

q_remote() {
  # Single-quote for remote shell (handles embedded single quotes)
  local s="$1"
  printf "'%s'" "${s//\'/\'\\\'\'}"
}

duration_ms() {
  local end
  end=$(date +%s%3N 2>/dev/null || true)
  if [[ -z "$end" ]]; then end=$(( $(date +%s) * 1000 )); fi
  echo $(( end - START_TS_NS ))
}

ensure_state_dirs() {
  mkdir -p "$STATE_DIR" "$REMOTE_DIR_CACHE_DIR" || true
}

ensure_local_tree() {
  local root="$1"
  mkdir -p "$root/inbox" "$root/outbox" "$root/processed" "$root/archive" "$root/.state" "$root/logs" \
    || error_exit "LOCAL_ROOT_UNWRITABLE" "Cannot create local sync tree at $root"
}

hash_cmd_local() {
  if command -v sha256sum >/dev/null 2>&1; then
    echo "sha256sum"
  elif command -v openssl >/dev/null 2>&1; then
    echo "openssl"
  else
    echo ""; return 1
  fi
}

sha256_file() {
  local path="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum -b "$path" | awk '{print $1}'
  else
    # openssl fallback
    openssl dgst -sha256 -r "$path" | awk '{print $1}'
  fi
}

remote_sha256_file() {
  local host="$1"; shift
  local rpath="$1"; shift
  local port_opt=()
  if [[ -n "$REMOTE_PORT" ]]; then port_opt=("-p" "$REMOTE_PORT"); fi
  ssh -o BatchMode=yes -o StrictHostKeyChecking=accept-new "${port_opt[@]}" "$host" "\
    if command -v sha256sum >/dev/null 2>&1; then sha256sum -b $(printf %s "$(q_remote "$rpath")"); \
    elif command -v openssl >/dev/null 2>&1; then openssl dgst -sha256 -r $(printf %s "$(q_remote "$rpath")"); \
    else echo MISSING_HASH_TOOL; exit 127; fi" 2>/dev/null | awk '{print $1}'
}

size_file_bytes() {
  local path="$1"
  if stat --version >/dev/null 2>&1; then
    stat -c '%s' "$path"
  else
    # BSD/macOS fallback (not expected here, but harmless)
    stat -f '%z' "$path"
  fi
}

require_tool() {
  local t="$1"
  command -v "$t" >/dev/null 2>&1 || error_exit "MISSING_TOOL" "Required tool '$t' not found in PATH"
}

log_file_path=""
open_log() {
  local root="$1"
  mkdir -p "$root/logs"
  local ts=$(date +%Y%m%d-%H%M%S)
  log_file_path="$root/logs/transfer-$ts.log"
  : > "$log_file_path" || true
}

wlog() {
  # Write to log if available
  if [[ -n "$log_file_path" ]]; then
    printf '%s\n' "$(date -Is) $*" >> "$log_file_path" || true
  fi
}

error_exit() {
  local code="$1"; shift
  local msg="$1"; shift || true
  local dur=$(duration_ms)
  local jmsg=$(json_escape "$msg")
  local logf
  logf=${log_file_path:-}
  if [[ -n "$logf" ]]; then
    printf '{"status":"error","code":"%s","message":"%s","log_path":"%s","duration_ms":%s}\n' "$code" "$jmsg" "$(json_escape "$logf")" "$dur"
  else
    printf '{"status":"error","code":"%s","message":"%s","duration_ms":%s}\n' "$code" "$jmsg" "$dur"
  fi
  exit 1
}

ok_exit() {
  local payload="$1"
  local dur=$(duration_ms)
  local logf
  logf=${log_file_path:-}
  # Expect payload to be a partial JSON object without outer braces
  if [[ -n "$logf" ]]; then
    printf '{"status":"ok",%s,"log_path":"%s","duration_ms":%s}\n' "$payload" "$(json_escape "$logf")" "$dur"
  else
    printf '{"status":"ok",%s,"duration_ms":%s}\n' "$payload" "$dur"
  fi
}

############## Argument Parsing ##############

usage() {
  cat <<'USAGE'
Usage: sync.sh <subcommand> [options]

Subcommands:
  send | receive | setup-remote | discover-remote-dirs | auto

Common options:
  --transport <ssh|usb-fs|auto>
  --local-root <path>
  --remote-host <user@host>
  --remote-port <port>
  --remote-root <path>
  --usb-mount <mountpoint>
  --dry-run (0|1)
  --timeout <seconds>
  --verbose (0|1)
  --archive (0|1)
  --ack-remote (0|1)
  --refresh-cache (0|1)
  --jobs <N>
USAGE
}

parse_args() {
  if [[ $# -lt 1 ]]; then usage; exit 2; fi
  SUBCOMMAND="$1"; shift
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --transport) TRANSPORT="$2"; shift 2;;
      --local-root) LOCAL_ROOT="$2"; shift 2;;
      --remote-host) REMOTE_HOST="$2"; shift 2;;
      --remote-port) REMOTE_PORT="$2"; shift 2;;
      --remote-root) REMOTE_ROOT="$2"; shift 2;;
      --usb-mount) USB_MOUNT="$2"; shift 2;;
      --dry-run) DRY_RUN="$2"; shift 2;;
      --timeout) TIMEOUT="$2"; shift 2;;
      --verbose) VERBOSE="$2"; shift 2;;
      --archive) ARCHIVE="$2"; shift 2;;
      --ack-remote) ACK_REMOTE="$2"; shift 2;;
      --refresh-cache) REFRESH_CACHE="$2"; shift 2;;
      --jobs) JOBS="$2"; shift 2;;
      -h|--help) usage; exit 0;;
      *) err "Unknown argument: $1"; usage; exit 2;;
    esac
  done
}

############## Locking ##############

LOCK_DIR_LOCAL=""
acquire_lock() {
  local root="$1"
  ensure_local_tree "$root"
  LOCK_DIR_LOCAL="$root/.state/sync.lock"
  if mkdir "$LOCK_DIR_LOCAL" 2>/dev/null; then
    echo $$ > "$LOCK_DIR_LOCAL/pid" || true
    trap 'release_lock' EXIT
  else
    error_exit "LOCK_HELD" "Another sync is in progress (lock at $LOCK_DIR_LOCAL)"
  fi
}

release_lock() {
  if [[ -n "$LOCK_DIR_LOCAL" && -d "$LOCK_DIR_LOCAL" ]]; then
    rm -f "$LOCK_DIR_LOCAL/pid" 2>/dev/null || true
    rmdir "$LOCK_DIR_LOCAL" 2>/dev/null || true
  fi
}

############## Transport Health ##############

usb_healthy() {
  [[ -n "$USB_MOUNT" && -d "$USB_MOUNT" && -w "$USB_MOUNT" ]]
}

select_transport() {
  if [[ "$TRANSPORT" == "auto" ]]; then
    if usb_healthy; then TRANSPORT="usb-fs"; else TRANSPORT="ssh"; fi
  fi
}

############## Remote Helpers (SSH) ##############

ssh_base() {
  # Emit SSH option arguments as NUL-delimited items so callers can
  # reliably reconstruct the argv array without losing word boundaries.
  local args=("-o" "BatchMode=yes" "-o" "StrictHostKeyChecking=accept-new")
  if [[ -n "$REMOTE_PORT" ]]; then args+=("-p" "$REMOTE_PORT"); fi
  printf '%s\0' "${args[@]}"
}

# scp uses -P (uppercase) for port, unlike ssh which uses -p.
scp_base() {
  local args=("-o" "BatchMode=yes" "-o" "StrictHostKeyChecking=accept-new")
  if [[ -n "$REMOTE_PORT" ]]; then args+=("-P" "$REMOTE_PORT"); fi
  printf '%s\0' "${args[@]}"
}

ssh_run() {
  local cmd="$1"
  local -a base
  mapfile -d '' -t base < <(ssh_base)
  wlog "CMD: ssh ${base[*]} $REMOTE_HOST -- $cmd"
  if [[ "$DRY_RUN" -eq 1 ]]; then
    vlog "[dry-run] ssh ${base[*]} $REMOTE_HOST -- $cmd"
    return 0
  fi
  timeout "$TIMEOUT" ssh ${base[@]} "$REMOTE_HOST" "$cmd"
}

scp_copy_up() {
  local src="$1"; local dest="$2" # dest is remote absolute filepath
  local -a base
  mapfile -d '' -t base < <(scp_base)
  local dest_target="$REMOTE_HOST:$(printf %s "$(q_remote "$dest")")"
  if [[ "$DRY_RUN" -eq 1 ]]; then
    vlog "[dry-run] scp ${base[*]} -C $src $dest_target"
    return 0
  fi
  SCP_LAST_ERR=""
  local out ec
  local -a scp_args
  scp_args=(-C)
  if [[ "$VERBOSE" -eq 1 ]]; then scp_args+=(-v); fi
  wlog "CMD: scp ${scp_args[*]} ${base[*]} -- $src $dest_target"
  out=$(timeout "$TIMEOUT" scp ${scp_args[@]} ${base[@]} -- "$src" "$dest_target" 2>&1)
  ec=$?
  if [[ $ec -ne 0 ]]; then
    # Retry with legacy SCP protocol for older servers (e.g., Dropbear)
    wlog "CMD: scp -O ${scp_args[*]} ${base[*]} -- $src $dest_target"
    local out2
    out2=$(timeout "$TIMEOUT" scp -O ${scp_args[@]} ${base[@]} -- "$src" "$dest_target" 2>&1)
    local ec2=$?
    if [[ $ec2 -eq 0 ]]; then
      return 0
    else
      wlog "ERR: scp upload failed: $out | legacy:-O -> $out2"
      SCP_LAST_ERR="$out | legacy:-O -> $out2"
      return $ec2
    fi
  fi
  return 0
}

scp_copy_down() {
  local src_remote="$1"; local dest_local="$2"
  local -a base
  mapfile -d '' -t base < <(scp_base)
  local src_spec="$REMOTE_HOST:$(printf %s "$(q_remote "$src_remote")")"
  if [[ "$DRY_RUN" -eq 1 ]]; then
    vlog "[dry-run] scp ${base[*]} -C $src_spec $dest_local"
    return 0
  fi
  SCP_LAST_ERR=""
  local out ec
  local -a scp_args
  scp_args=(-C)
  if [[ "$VERBOSE" -eq 1 ]]; then scp_args+=(-v); fi
  wlog "CMD: scp ${scp_args[*]} ${base[*]} -- $src_spec $dest_local"
  out=$(timeout "$TIMEOUT" scp ${scp_args[@]} ${base[@]} -- "$src_spec" "$dest_local" 2>&1)
  ec=$?
  if [[ $ec -ne 0 ]]; then
    # Retry with legacy SCP protocol
    wlog "CMD: scp -O ${scp_args[*]} ${base[*]} -- $src_spec $dest_local"
    local out2
    out2=$(timeout "$TIMEOUT" scp -O ${scp_args[@]} ${base[@]} -- "$src_spec" "$dest_local" 2>&1)
    local ec2=$?
    if [[ $ec2 -eq 0 ]]; then
      return 0
    else
      wlog "ERR: scp download failed: $out | legacy:-O -> $out2"
      SCP_LAST_ERR="$out | legacy:-O -> $out2"
      return $ec2
    fi
  fi
  return 0
}

ensure_remote_tree() {
  local root="$1"
  local cmd="mkdir -p $(q_remote "$root/inbox") $(q_remote "$root/outbox") $(q_remote "$root/processed") $(q_remote "$root/archive") $(q_remote "$root/.state")"
  ssh_run "$cmd" || error_exit "SSH_CMD_FAILED" "Failed to create remote tree at $root"
}

############## Send (SSH + USB) ##############

send_ssh() {
  require_tool ssh; require_tool scp
  [[ -n "$REMOTE_HOST" ]] || error_exit "MISSING_ARG" "--remote-host is required for ssh transport"
  [[ -n "$REMOTE_ROOT" ]] || error_exit "MISSING_ARG" "--remote-root is required for ssh transport"

  ensure_remote_tree "$REMOTE_ROOT"
  local outbox="$LOCAL_ROOT/outbox"
  local archive_dir="$LOCAL_ROOT/archive"
  mkdir -p "$outbox" "$archive_dir"

  local files=()
  while IFS= read -r -d '' f; do files+=("$f"); done < <(find "$outbox" -maxdepth 1 -type f -print0)

  local files_json="["; local sep=""; local total_bytes=0
  for f in "${files[@]}"; do
    local base
    base=$(basename "$f")
    local bytes; bytes=$(size_file_bytes "$f")
    local sha; sha=$(sha256_file "$f")
    total_bytes=$(( total_bytes + bytes ))
    local remote_inbox="$REMOTE_ROOT/inbox"
    local remote_part="$remote_inbox/$base.part"
    local remote_final="$remote_inbox/$base"

    # Upload to .part
    scp_copy_up "$f" "$remote_part"; ec=$?
    if [[ $ec -ne 0 ]]; then
      # Include captured scp stderr for context
      error_exit "SCP_FAILED" "Upload failed for $base (exit=$ec) - scp: $SCP_LAST_ERR"
    fi

    # Verify checksum remotely
    local rsha; rsha=$(remote_sha256_file "$REMOTE_HOST" "$remote_part") || true
    if [[ "$DRY_RUN" -eq 0 ]]; then
      if [[ -z "$rsha" || "$rsha" == "MISSING_HASH_TOOL" ]]; then
        error_exit "CHECKSUM_UNAVAILABLE" "Remote cannot compute SHA-256"
      fi
      if [[ "$rsha" != "$sha" ]]; then
        error_exit "CHECKSUM_MISMATCH" "Checksum mismatch after upload for $base"
      fi
    fi

    # Atomic rename
    ssh_run "mv $(q_remote "$remote_part") $(q_remote "$remote_final")" || error_exit "SSH_CMD_FAILED" "Failed to finalize $base on remote"

    # Archive local
    if [[ "$ARCHIVE" -eq 1 && "$DRY_RUN" -eq 0 ]]; then
      local ts=$(date +%Y%m%d-%H%M%S)
      mv "$f" "$archive_dir/$ts-$base" || true
    fi

    # Append to files_json
    local item
    item=$(printf '{"name":"%s","bytes":%s,"sha256":"%s","action":"sent"}' "$(json_escape "$base")" "$bytes" "$sha")
    files_json+="$sep$item"; sep="," 
  done
  files_json+="]"
  ok_exit $(printf '"transport":"ssh","files":%s,"bytes":%s' "$files_json" "$total_bytes")
}

send_usb() {
  [[ -n "$USB_MOUNT" ]] || error_exit "MISSING_ARG" "--usb-mount is required for usb-fs transport"
  [[ -d "$USB_MOUNT" && -w "$USB_MOUNT" ]] || error_exit "PATH_NOT_WRITABLE" "USB mount not writable: $USB_MOUNT"
  [[ -n "$REMOTE_ROOT" ]] || error_exit "MISSING_ARG" "--remote-root is required (path under mount)"
  local remote_root_on_mount="$USB_MOUNT$REMOTE_ROOT"
  local remote_inbox="$remote_root_on_mount/inbox"
  mkdir -p "$remote_inbox"

  local outbox="$LOCAL_ROOT/outbox"
  local archive_dir="$LOCAL_ROOT/archive"
  mkdir -p "$outbox" "$archive_dir"

  local files=()
  while IFS= read -r -d '' f; do files+=("$f"); done < <(find "$outbox" -maxdepth 1 -type f -print0)

  local files_json="["; local sep=""; local total_bytes=0
  for f in "${files[@]}"; do
    local base; base=$(basename "$f")
    local bytes; bytes=$(size_file_bytes "$f")
    local sha; sha=$(sha256_file "$f")
    total_bytes=$(( total_bytes + bytes ))
    local part="$remote_inbox/$base.part"
    local final="$remote_inbox/$base"

    if [[ "$DRY_RUN" -eq 0 ]]; then
      install -D "$f" "$part"
      local rsha; rsha=$(sha256_file "$part")
      if [[ "$rsha" != "$sha" ]]; then
        error_exit "CHECKSUM_MISMATCH" "Checksum mismatch after USB copy for $base"
      fi
      mv "$part" "$final"
      if [[ "$ARCHIVE" -eq 1 ]]; then
        local ts=$(date +%Y%m%d-%H%M%S)
        mv "$f" "$archive_dir/$ts-$base" || true
      fi
    else
      vlog "[dry-run] USB copy $f -> $final"
    fi

    local item
    item=$(printf '{"name":"%s","bytes":%s,"sha256":"%s","action":"sent"}' "$(json_escape "$base")" "$bytes" "$sha")
    files_json+="$sep$item"; sep="," 
  done
  files_json+="]"
  ok_exit $(printf '"transport":"usb-fs","files":%s,"bytes":%s' "$files_json" "$total_bytes")
}

############## Receive (SSH + USB) ##############

receive_ssh() {
  require_tool ssh; require_tool scp
  [[ -n "$REMOTE_HOST" ]] || error_exit "MISSING_ARG" "--remote-host is required for ssh transport"
  [[ -n "$REMOTE_ROOT" ]] || error_exit "MISSING_ARG" "--remote-root is required for ssh transport"
  ensure_remote_tree "$REMOTE_ROOT"

  local inbox="$LOCAL_ROOT/inbox"
  local remote_outbox="$REMOTE_ROOT/outbox"
  local remote_archive="$REMOTE_ROOT/archive"
  mkdir -p "$inbox"

  # List remote files (NUL-delimited to handle spaces)
  local -a base
  mapfile -d '' -t base < <(ssh_base)
  local list_cmd="find $(q_remote "$remote_outbox") -maxdepth 1 -type f -print0"

  local files_json="["; local sep=""; local total_bytes=0

  if [[ "$DRY_RUN" -eq 0 ]]; then
    while IFS= read -r -d '' rfile; do
      local rbase; rbase=$(basename "$rfile")
      local dest_part="$inbox/$rbase.part"
      local dest_final="$inbox/$rbase"

      scp_copy_down "$rfile" "$dest_part"; ec=$?
      if [[ $ec -ne 0 ]]; then
        error_exit "SCP_FAILED" "Download failed for $rbase (exit=$ec) - scp: $SCP_LAST_ERR"
      fi

      local lsha=""; local rsha=""
      lsha=$(sha256_file "$dest_part")
      rsha=$(remote_sha256_file "$REMOTE_HOST" "$rfile") || true
      if [[ -z "$rsha" || "$rsha" == "MISSING_HASH_TOOL" ]]; then
        error_exit "CHECKSUM_UNAVAILABLE" "Remote cannot compute SHA-256 for $rbase"
      fi
      if [[ "$lsha" != "$rsha" ]]; then
        error_exit "CHECKSUM_MISMATCH" "Checksum mismatch after download for $rbase"
      fi
      mv "$dest_part" "$dest_final"
      local bytes; bytes=$(size_file_bytes "$dest_final")
      total_bytes=$(( total_bytes + bytes ))
      if [[ "$ACK_REMOTE" -eq 1 ]]; then
        ssh_run "mkdir -p $(q_remote "$remote_archive") && mv $(q_remote "$rfile") $(q_remote "$remote_archive/$(basename "$rfile")")"
      fi

      local item
      item=$(printf '{"name":"%s","bytes":%s,"sha256":"%s","action":"received"}' "$(json_escape "$rbase")" "${bytes:-0}" "${lsha:-}" )
      files_json+="$sep$item"; sep="," 
    done < <(timeout "$TIMEOUT" ssh ${base[@]} "$REMOTE_HOST" "$list_cmd")
  else
    vlog "[dry-run] would list and download files from $remote_outbox"
  fi

  files_json+="]"
  ok_exit $(printf '"transport":"ssh","files":%s,"bytes":%s' "$files_json" "$total_bytes")
}

receive_usb() {
  [[ -n "$USB_MOUNT" ]] || error_exit "MISSING_ARG" "--usb-mount is required for usb-fs transport"
  [[ -n "$REMOTE_ROOT" ]] || error_exit "MISSING_ARG" "--remote-root is required (path under mount)"
  local remote_root_on_mount="$USB_MOUNT$REMOTE_ROOT"
  local remote_outbox="$remote_root_on_mount/outbox"
  local remote_archive="$remote_root_on_mount/archive"

  [[ -d "$remote_outbox" ]] || error_exit "PATH_NOT_FOUND" "USB outbox not found: $remote_outbox"

  local inbox="$LOCAL_ROOT/inbox"
  mkdir -p "$inbox" "$remote_archive"

  local files=()
  while IFS= read -r -d '' f; do files+=("$f"); done < <(find "$remote_outbox" -maxdepth 1 -type f -print0)

  local files_json="["; local sep=""; local total_bytes=0
  for rf in "${files[@]}"; do
    local base; base=$(basename "$rf")
    local part="$inbox/$base.part"
    local final="$inbox/$base"
    if [[ "$DRY_RUN" -eq 0 ]]; then
      install -D "$rf" "$part"
      local lsha; lsha=$(sha256_file "$part")
      local rsha; rsha=$(sha256_file "$rf")
      if [[ "$lsha" != "$rsha" ]]; then
        error_exit "CHECKSUM_MISMATCH" "Checksum mismatch after USB receive for $base"
      fi
      mv "$part" "$final"
      local bytes; bytes=$(size_file_bytes "$final")
      total_bytes=$(( total_bytes + bytes ))
      if [[ "$ACK_REMOTE" -eq 1 ]]; then
        mkdir -p "$remote_archive"
        mv "$rf" "$remote_archive/$base" || true
      fi
    else
      vlog "[dry-run] would copy $rf -> $final"
    fi

    local item
    item=$(printf '{"name":"%s","bytes":%s,"sha256":"%s","action":"received"}' "$(json_escape "$base")" "${bytes:-0}" "${lsha:-}" )
    files_json+="$sep$item"; sep="," 
  done
  files_json+="]"
  ok_exit $(printf '"transport":"usb-fs","files":%s,"bytes":%s' "$files_json" "$total_bytes")
}

############## Setup Remote Tree ##############

cmd_setup_remote() {
  select_transport
  if [[ "$TRANSPORT" != "ssh" ]]; then
    error_exit "UNSUPPORTED" "setup-remote only applies to ssh transport"
  fi
  require_tool ssh
  [[ -n "$REMOTE_HOST" && -n "$REMOTE_ROOT" ]] || error_exit "MISSING_ARG" "--remote-host and --remote-root are required"
  ensure_remote_tree "$REMOTE_ROOT"
  ok_exit $(printf '"transport":"ssh","setup":"%s"' "created")
}

############## Discover Remote Dirs (SSH) ##############

sanitize_host() {
  printf '%s' "$1" | tr -c 'A-Za-z0-9._-' '_'
}

discover_remote_dirs() {
  require_tool ssh
  [[ -n "$REMOTE_HOST" ]] || error_exit "MISSING_ARG" "--remote-host is required"
  ensure_state_dirs
  local host_key; host_key=$(sanitize_host "$REMOTE_HOST")
  local cache_file="$REMOTE_DIR_CACHE_DIR/$host_key.json"
  local ttl=300
  if [[ "$REFRESH_CACHE" -eq 0 && -f "$cache_file" ]]; then
    # Check age
    local now=$(date +%s)
    local mtime=$(stat -c %Y "$cache_file" 2>/dev/null || echo 0)
    local age=$(( now - mtime ))
    if (( age < ttl )); then
      local dur=$(duration_ms)
      # Return cached file content as field 'dirs'
      local cached
      cached=$(tr -d '\n' < "$cache_file" 2>/dev/null || echo '[]')
      printf '{"status":"ok","transport":"ssh","dirs":%s,"cache":"hit","duration_ms":%s}\n' "$cached" "$dur"
      exit 0
    fi
  fi

  local script='
set -e
for d in /storage/weightlifting; do
  [ -d "$d" ] || continue
  find "$d" -maxdepth 2 -type d -print 2>/dev/null | while IFS= read -r p; do
    fb=$(df -P "$p" 2>/dev/null | awk "NR==2{print \$4}")
    [ -z "$fb" ] && fb=0
    owner=$(stat -c %U "$p" 2>/dev/null || echo "")
    if [ -w "$p" ]; then w=true; else w=false; fi
    printf "{\"path\":\"%s\",\"free_bytes\":%s,\"owner\":\"%s\",\"writable\":%s}\n" "$p" "$fb" "$owner" "$w" | tr -d "\n"; echo
  done
done
'

  local -a base
  mapfile -d '' -t base < <(ssh_base)
  local out
  # Run SSH discovery and capture exit status explicitly so we can surface errors
  set +e
  out=$(timeout "$TIMEOUT" ssh ${base[@]} "$REMOTE_HOST" "$script" 2>/dev/null)
  ssh_status=$?
  set -e
  if [[ $ssh_status -ne 0 ]]; then
    error_exit "SSH_UNREACHABLE" "Failed to connect or run discovery via SSH (exit=$ssh_status) on $REMOTE_HOST"
  fi

  # Build JSON array
  local arr="["
  local first=1
  while IFS= read -r line; do
    [[ -z "$line" ]] && continue
    if (( first )); then arr+="$line"; first=0; else arr+=",$line"; fi
  done <<< "$out"
  arr+="]"

  # Cache and return
  printf '%s' "$arr" > "$cache_file" 2>/dev/null || true
  ok_exit $(printf '"transport":"ssh","dirs":%s,"cache":"%s"' "$arr" "miss")
}

############## Auto ##############

cmd_auto() {
  select_transport
  ok_exit $(printf '"transport":"%s"' "$TRANSPORT")
}

############## Main Dispatch ##############

main() {
  parse_args "$@"
  [[ -n "$LOCAL_ROOT" ]] || error_exit "MISSING_ARG" "--local-root is required"
  ensure_state_dirs
  ensure_local_tree "$LOCAL_ROOT"
  open_log "$LOCAL_ROOT"
  acquire_lock "$LOCAL_ROOT"

  case "$SUBCOMMAND" in
    send)
      select_transport
      if [[ "$TRANSPORT" == "ssh" ]]; then send_ssh; elif [[ "$TRANSPORT" == "usb-fs" ]]; then send_usb; else error_exit "NO_TRANSPORT_AVAILABLE" "No transport available"; fi
      ;;
    receive)
      select_transport
      if [[ "$TRANSPORT" == "ssh" ]]; then receive_ssh; elif [[ "$TRANSPORT" == "usb-fs" ]]; then receive_usb; else error_exit "NO_TRANSPORT_AVAILABLE" "No transport available"; fi
      ;;
    setup-remote)
      cmd_setup_remote
      ;;
    discover-remote-dirs)
      discover_remote_dirs
      ;;
    auto)
      cmd_auto
      ;;
    *)
      usage; exit 2;
      ;;
  esac
}

main "$@"
