# Full Torch .venv Workspace acceptance

Status: pending execution. This follows the paused issue #71 three-sample proof on the current v0.1.4 implementation. The original task, failed attempts, native measurements and uncommitted evidence remain untouched.

The [frozen plan](plan.json) preserves the original tar workload, Create/Exec/Commit timing boundaries, fresh Store per sample, 2 CPUs, 2 GiB RAM, no swap, 256 PIDs, full independent content/metadata oracle, healthy Commit presentation, clean End and owned-container removal. macOS owns SDK, SQLite and publication; Docker owns daemon/FUSE/workload. Archive delivery, setup and independent verification are outside the timers. Samples have uncontrolled caches and no eligible untouched-release comparator; no speedup or formal benchmark admission is claimed.

The existing probe is copied from the paused task and compiled against current workspace crates. The existing source archive must match its recorded SHA-256; the source tree is checked against the archive before execution. Samples are serialized by the shared measurement lock. Each outer workload deadline is 480 seconds, including the existing 300-second Exec deadline; each independent verification deadline is 180 seconds. No gates or limits are relaxed.
