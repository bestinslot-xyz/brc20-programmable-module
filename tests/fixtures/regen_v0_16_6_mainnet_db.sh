#!/usr/bin/env bash
# Regenerates tests/fixtures/v0_16_6_mainnet_db: the mainnet consensus fixture workload run and
# committed by brc20-prog v0.16.6, the last release with the per-database layout.
#
# It builds v0.16.6 in a temporary git worktree, points its test server at a fixed data dir,
# commits after the workload, runs the mainnet consensus fixture, and copies the data dir here
# without the RocksDB LOG and LOCK files. The snapshot test must still pass in v0.16.6.
#
# The block databases hold wall-clock processing times, so a regenerated fixture differs from
# the committed one in those values; every consensus output is the same.
#
# usage: tests/fixtures/regen_v0_16_6_mainnet_db.sh [output dir]
#        (default output: tests/fixtures/v0_16_6_mainnet_db)
set -euo pipefail

repo=$(git rev-parse --show-toplevel)
out="${1:-$repo/tests/fixtures/v0_16_6_mainnet_db}"
work=$(mktemp -d)
trap 'git -C "$repo" worktree remove --force "$work/src" >/dev/null 2>&1 || true; rm -rf "$work"' EXIT

git -C "$repo" worktree add --detach "$work/src" v0.16.6
cd "$work/src"
git apply <<'PATCH'
--- a/test_utils/src/utils.rs
+++ b/test_utils/src/utils.rs
@@ -21,9 +21,11 @@

 pub async fn spawn_test_server(config: Brc20ProgConfig) -> (ServerHandle, HttpClient) {
     let db_path = TempDir::new().unwrap();
+    let db_path_str = std::env::var("FIXTURE_DB")
+        .unwrap_or_else(|_| db_path.path().to_str().unwrap().to_string());
     let server_address = format!("127.0.0.1:{}", get_free_port());
     let server = start(Brc20ProgConfig {
-        db_path: db_path.path().to_str().unwrap().to_string(),
+        db_path: db_path_str,
         brc20_prog_rpc_server_url: server_address.clone(),
         fail_on_bitcoin_rpc_error: false,
         evm_record_traces: true,
--- a/test_utils/src/consensus_fixture.rs
+++ b/test_utils/src/consensus_fixture.rs
@@ -520,6 +520,11 @@
     for era in &spec.eras {
         runner.run_era(era).await;
     }
+    runner
+        .client
+        .brc20_commit_to_database()
+        .await
+        .expect("commit");
     let report = runner.render().await;
     server.stop().expect("stop server");
     report
PATCH

FIXTURE_DB="$work/db" cargo test --all-features --test consensus_fixture_mainnet

rm -rf "$out"
mkdir -p "$out"
rsync -a --exclude 'LOG*' --exclude LOCK "$work/db/" "$out/"
echo "Wrote $out"
