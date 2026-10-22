#!/usr/bin/env bash
# The rootless NAT lab (M9 N8c): four homes, each a player behind its own
# NAT router, on a small "internet" that holds the relay and the lab's
# STUN and TURN server (glasnost-turn). One game of four net-clients, one
# per home, played as a split soak (one soak per home, the room code passed
# through a file), then `soak compare` over the four halves.
#
# Usage: crates/netlab/lab/nat-lab.sh SCENARIO OUT [MINUTES]
#   SCENARIO  eim-stun | sym-one-stun | sym-both-stun | sym-turn | no-ice |
#             udp-blocked | udp-flap | wss | all
#   OUT       the folder for the scenario's folders (counts and timings)
#   MINUTES   game minutes per scenario (default 3; udp-flap takes 4)
# GLASNOST_BIN names the folder of glasnost-headless, glasnost-relay and
# glasnost-turn (default target/release). Synthetic install only.
#
# No root: the script runs itself again under `unshare -rnm` (a user, a
# network and a mount namespace), and every router and player is a network
# namespace inside it, joined by veth pairs. Nothing outside changes, and
# everything goes when the script ends.
#
# The NATs are nftables masquerade on each router's internet side:
#   - plain `masquerade` keeps a socket's port for every destination: an
#     endpoint-independent mapping with port-dependent filtering (a
#     port-restricted cone);
#   - `masquerade fully-random` picks a new port per destination: an
#     endpoint-dependent mapping (a symmetric NAT).
# Each home's NAT is checked first with `glasnost-turn probe` from the
# player, against the server's two addresses.
#
# A router drops unsolicited UDP from the internet in its input hook (a
# home router's firewall does the same): a hole opens only by the player's
# own send, so two players punch through when both send.
#
# Path evidence: each router counts the UDP it forwards to the other homes'
# public addresses (direct), to the TURN server (STUN and TURN), and the
# TCP to the relay; glasnost-turn counts what it relayed. Our own harness;
# no spec applies.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../../.." && pwd)
bin=${GLASNOST_BIN:-$root/target/release}
scenario=${1:?usage: nat-lab.sh SCENARIO OUT [MINUTES]}
out=${2:?usage: nat-lab.sh SCENARIO OUT [MINUTES]}
minutes=${3:-3}

if [ "${GLASNOST_LAB_INSIDE:-}" != 1 ]; then
  mkdir -p "$out"
  out=$(cd "$out" && pwd)
  exec env GLASNOST_LAB_INSIDE=1 GLASNOST_BIN="$bin" \
    unshare -rnm --propagation private "$0" "$scenario" "$out" "$minutes"
fi

if [ "$scenario" = all ]; then
  for s in eim-stun sym-one-stun sym-both-stun sym-turn no-ice udp-blocked udp-flap wss; do
    "$0" "$s" "$out" "$minutes" || echo "scenario $s failed"
  done
  exit 0
fi

# The internet: 203.0.113.0/24 (a documentation range). The server is .1
# and .2 (the probe needs two addresses); home K's router is .(10+K).
# Home K's player is 10.0.K.2 behind 10.0.K.1.
SERVER=203.0.113.1
SERVER2=203.0.113.2
RELAY_PORT=7400
TLS_PORT=7443
TURN_PORT=3478
HOMES=4

pids=()
cleanup() {
  for p in "${pids[@]}"; do kill "$p" 2>/dev/null || true; done
  sleep 2
  for p in "${pids[@]}"; do kill -KILL "$p" 2>/dev/null || true; done
  wait 2>/dev/null || true
}
trap cleanup EXIT

mount -t tmpfs none /run
mkdir -p /run/netns
ip link set lo up

ns() { ip netns exec "$@"; }

ip netns add inet
ns inet ip link set lo up
ns inet ip link add br0 type bridge
ns inet ip addr add $SERVER/24 dev br0
ns inet ip addr add $SERVER2/24 dev br0
ns inet ip link set br0 up

# The NAT of each home, by scenario: eim or sym.
nat_of() {
  case "$scenario" in
    sym-one-stun) [ "$1" = 0 ] && echo sym || echo eim ;;
    sym-both-stun) echo sym ;;
    sym-turn) { [ "$1" = 0 ] || [ "$1" = 1 ]; } && echo sym || echo eim ;;
    *) echo eim ;;
  esac
}

for k in $(seq 0 $((HOMES - 1))); do
  r=r$k
  h=h$k
  ip netns add $r
  ip netns add $h
  ns $r ip link set lo up
  ns $h ip link set lo up
  # The router's internet side, on the bridge.
  ip link add wan$k netns $r type veth peer name port$k netns inet
  ns inet ip link set port$k master br0 up
  ns $r ip addr add 203.0.113.$((10 + k))/24 dev wan$k
  ns $r ip link set wan$k up
  # The home's side.
  ip link add lan$k netns $r type veth peer name eth0 netns $h
  ns $r ip addr add 10.0.$k.1/24 dev lan$k
  ns $r ip link set lan$k up
  ns $h ip addr add 10.0.$k.2/24 dev eth0
  ns $h ip link set eth0 up
  ns $h ip route add default via 10.0.$k.1
  ns $r sysctl -qw net.ipv4.ip_forward=1
  if [ "$(nat_of $k)" = sym ]; then masq="masquerade fully-random"; else masq="masquerade"; fi
  peers=""
  for j in $(seq 0 $((HOMES - 1))); do
    [ "$j" = "$k" ] && continue
    peers="$peers${peers:+, }203.0.113.$((10 + j))"
  done
  ns $r nft -f - <<EOF
table ip lab {
  counter direct {}
  counter turn {}
  counter relay {}
  counter blocked {}
  counter unsolicited {}
  chain block {}
  chain forward {
    type filter hook forward priority 0; policy accept;
    jump block
    oifname "wan$k" ip daddr { $peers } meta l4proto udp counter name direct
    oifname "wan$k" ip daddr { $SERVER, $SERVER2 } udp dport $TURN_PORT counter name turn
    oifname "wan$k" ip daddr $SERVER tcp dport { $RELAY_PORT, $TLS_PORT } counter name relay
  }
  # Unsolicited datagrams from the internet stop here, as on a home
  # router's firewall, before their connection entry is confirmed: a
  # confirmed entry would take the port a player's own later send needs,
  # and masquerade would then pick another (the lab's first run).
  chain input {
    type filter hook input priority 0; policy accept;
    iifname "wan$k" meta l4proto udp ct state new counter name unsolicited drop
  }
  chain postrouting {
    type nat hook postrouting priority 100; policy accept;
    oifname "wan$k" $masq
  }
}
EOF
done

block_udp() {
  for k in $(seq 0 $((HOMES - 1))); do
    ns r$k nft add rule ip lab block meta l4proto udp counter name blocked drop
  done
  echo "udp blocked at $(date +%s)" >>"$dir/events.txt"
}
unblock_udp() {
  for k in $(seq 0 $((HOMES - 1))); do
    ns r$k nft flush chain ip lab block
  done
  echo "udp unblocked at $(date +%s)" >>"$dir/events.txt"
}

dir="$out/$scenario"
rm -rf "$dir"
mkdir -p "$dir"
{
  echo "scenario $scenario minutes $minutes"
  for k in $(seq 0 $((HOMES - 1))); do echo "home $k nat $(nat_of $k)"; done
} >"$dir/lab.txt"

# The server side: the relay, the STUN and TURN server, and for wss a TLS
# proxy with a lab certificate authority.
ns inet nice -n 19 "$bin/glasnost-relay" --listen $SERVER:$RELAY_PORT --stats 30 >"$dir/relay.txt" 2>&1 &
pids+=($!)
ns inet nice -n 19 "$bin/glasnost-turn" serve --listen $SERVER:$TURN_PORT --listen $SERVER2:$TURN_PORT \
  --relay-ip $SERVER --ports 49160-49200 --user lab=labword --stats 10 >"$dir/turn.txt" 2>&1 &
turn_pid=$!
pids+=($turn_pid)
relay_url=ws://$SERVER:$RELAY_PORT/
if [ "$scenario" = wss ]; then
  ca=/run/lab-ca && mkdir -p "$ca" # the mount namespace's own /run
  openssl req -x509 -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 -nodes -days 1 \
    -subj /CN=glasnost-lab-ca -keyout "$ca/ca.key" -out "$ca/ca.pem" 2>/dev/null
  openssl req -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 -nodes \
    -subj /CN=$SERVER -keyout "$ca/srv.key" -out "$ca/srv.csr" 2>/dev/null
  printf 'subjectAltName=IP:%s\n' $SERVER >"$ca/ext.txt"
  openssl x509 -req -in "$ca/srv.csr" -CA "$ca/ca.pem" -CAkey "$ca/ca.key" -CAcreateserial \
    -days 1 -extfile "$ca/ext.txt" -out "$ca/srv.pem" 2>/dev/null
  cat "$ca/srv.pem" "$ca/srv.key" >"$ca/srv-full.pem"
  ns inet socat OPENSSL-LISTEN:$TLS_PORT,bind=$SERVER,reuseaddr,fork,cert="$ca/srv-full.pem",verify=0 \
    TCP:$SERVER:$RELAY_PORT 2>"$dir/tls.txt" &
  pids+=($!)
  export SSL_CERT_FILE="$ca/ca.pem"
  relay_url=wss://$SERVER:$TLS_PORT/
fi
sleep 1

# The NATs, as a player sees them.
for k in $(seq 0 $((HOMES - 1))); do
  echo "home $k $(ns h$k "$bin/glasnost-turn" probe --server $SERVER:$TURN_PORT --server $SERVER2:$TURN_PORT 2>&1)" >>"$dir/lab.txt"
done

case "$scenario" in
  no-ice) ice="" ;;
  sym-turn | udp-blocked) ice="stun:$SERVER:$TURN_PORT turn:$SERVER:$TURN_PORT username=lab credential=labword" ;;
  *) ice="stun:$SERVER:$TURN_PORT" ;;
esac
[ "$scenario" = udp-flap ] && [ "$minutes" -lt 4 ] && minutes=4

[ "$scenario" = udp-blocked ] && block_udp

# One soak per home. Home 0 hosts; each home writes the code once its row
# is seated and the next home waits for that file, so the rows join in
# order (the room seats a joiner in the next free row).
start=$(date +%s)
soaks=()
for k in $(seq 0 $((HOMES - 1))); do
  args=(soak --headless "$bin/glasnost-headless" --out "$dir/home$k" --rows $k
    --relay-url "$relay_url" --no-proxy --transport rtc --clean --no-trouble
    --minutes "$minutes" --synthetic --row-arg "$k:--ice" --row-arg "$k:$ice"
    --code-out "$dir/code$k.txt")
  [ "$k" != 0 ] && args+=(--code-in "$dir/code$((k - 1)).txt")
  # A home that never gets its game does not hold the lab for ever.
  ns h$k timeout $((minutes * 60 + 600)) nice -n 19 "$bin/glasnost-headless" "${args[@]}" >"$dir/soak$k.txt" 2>&1 &
  soaks+=($!)
  pids+=($!)
done

if [ "$scenario" = udp-flap ]; then
  # Block once every link is open, for long enough that the first retry
  # (15 s after the fallback) fails and the next (30 s later) lands after
  # the unblock (docs/development.md, "Network lab (N8c)").
  (sleep 75 && block_udp && sleep 40 && unblock_udp) &
  pids+=($!)
fi

status=0
for p in "${soaks[@]}"; do wait "$p" || status=1; done
end=$(date +%s)
echo "wall $((end - start)) s soaks $([ $status = 0 ] && echo ok || echo failed)" >>"$dir/lab.txt"

for k in $(seq 0 $((HOMES - 1))); do
  echo "home $k counters $(ns r$k nft list counters table ip lab | tr -s ' \n\t' ' ' | sed 's/table ip lab //')" >>"$dir/lab.txt"
done
# SIGTERM: a background job of a script ignores SIGINT.
kill -TERM "$turn_pid" 2>/dev/null || true
sleep 1
"$bin/glasnost-headless" soak compare --minutes "$minutes" "$dir"/home0 "$dir"/home1 "$dir"/home2 "$dir"/home3 >"$dir/compare.txt" 2>&1 || status=1
cat "$dir/compare.txt"
exit $status
