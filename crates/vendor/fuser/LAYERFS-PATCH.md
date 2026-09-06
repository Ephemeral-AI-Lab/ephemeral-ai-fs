# LayerFS fuser 0.18.0 patch

Upstream crate checksum: `b82b6597d216503555ead6b358f341ef748869bf5c6fbae6a0cb9dd231baecfd`.
Upstream VCS revision: `9c957f74efe715112049298cdf1d601781829c8d`.
The retained normalized Cargo manifest, source, examples, build script and license come from that released crate. The standalone test lock comes from the same package; product builds use the repository root Cargo.lock. The root crates.io patch selects this directory, which is excluded from workspace membership. Existing image COPY and source/product sealing include `crates/` recursively.

Only `src/session.rs` and `src/read_buf.rs` change upstream source:

- A single configured receive loop executes directly in `Session::run`; `Session::spawn` remains its one background thread. Existing multithread behavior, decoder, replies, notifications, mount guard and exactly-once destroy remain.
- Linux INIT uses an 8 KiB usable buffer. Successful negotiation retains the required receive capacity for the event loop; other platforms retain the previous capacity.
- Linux receive payload allowance is the maximum of negotiated max_write, `max(negotiated max_pages, 32) * page_size`, and 64 KiB, plus 4096 bytes for headers/names. Alignment adds at most 7 owned bytes on the tested ABI without reducing usable capacity. Optional security-context/supplementary-group creation extensions retain the legacy allocation; LayerFS does not negotiate them.

## Request-size audit

Linux 6.12.76 is the retained deployment reference. Its device reader requires at least 8192 bytes and enough room for the negotiated write plus fixed headers. Batch-forget count is derived from the actual supplied buffer, and retrieve replies are capped by max_write. This makes shrinking negotiation **and** the actual allocation safe for these classes. [Kernel device reader](https://raw.githubusercontent.com/gregkh/linux/v6.12.76/fs/fuse/dev.c).

Ioctl input page counts cannot exceed the connection max_pages; accounting for pages is necessary even with small max_write. The connection begins with its default 32-page allowance. The patch conservatively keeps that floor when deriving capacity. [Kernel ioctl](https://raw.githubusercontent.com/gregkh/linux/v6.12.76/fs/fuse/ioctl.c), [connection negotiation](https://raw.githubusercontent.com/gregkh/linux/v6.12.76/fs/fuse/inode.c).

SETXATTR carries a value bounded by Linux's 64 KiB xattr limit plus a name/header. GETXATTR/list operations receive their large data in replies, not requests. Ordinary lookup/create/mkdir/unlink/link/rename/symlink carry bounded names and a path-sized symlink target; 4096 bytes of extra room on top of the 64 KiB minimum payload covers them. Stat/attribute/open/release/read/readdir/flush/fsync/lock/access/poll/bmap/fallocate/lseek/copy-range/interrupt/destroy requests have fixed-size bodies. Optional extended creation payloads use the conservative legacy allowance. CUSE remains outside LayerFS support. [Kernel xattrs](https://raw.githubusercontent.com/gregkh/linux/v6.12.76/fs/fuse/xattr.c), [fuser ABI](src/ll/fuse_abi.rs).

For LayerFS's unchanged 1 MiB write/readahead negotiation and a 4 KiB Linux page, one ingress allocation is 1,052,679 bytes including alignment, versus 16 MiB + 4096 previously. 100 such allocations are 105,267,900 bytes (100.391293 MiB), before stacks/allocator/kernel/socket/live data. These are source-derived capacities, not RSS or measured 100-mount results. Different page sizes use the actual arithmetic. No TTL, permission, writeback or kernel change is included.

## Focused checks

The socket-backed Session test exercises real fuser INIT/decode/reply/run/destroy without mounting: negotiated max_write, callback on the run thread and exactly-once destroy. Buffer tests check full usable aligned capacity, 1 MiB and legacy write sizes, small-write/default ioctl-page floor, 64 KiB pages and checked overflow.

Native command (macOS compile-only mount feature; no macFUSE support claim):

```sh
CARGO_TARGET_DIR="$PWD/target" cargo +1.85.1 test \
  --manifest-path crates/vendor/fuser/Cargo.toml --features macos-no-mount \
  --lib layerfs_session_tests -j2
CARGO_TARGET_DIR="$PWD/target" cargo +1.85.1 test \
  --manifest-path crates/vendor/fuser/Cargo.toml --features macos-no-mount \
  --lib read_buf::tests -j2
```

Use the shared host measurement lock as for other checks. The same tests can run on Linux without the macOS feature. Real Docker/FUSE integration is a separate P3/P6 obligation; these checks do not establish it. Preserve this narrow diff when updating fuser, and retire it when an upstream release supplies equivalent behavior.
