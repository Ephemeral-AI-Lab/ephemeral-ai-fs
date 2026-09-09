//! Pinned #72 importer/observer, plus the approved storage smoke filesystem operations.
use super::{hex, Result, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, FileTimes};
use std::io::{Read, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    mode: u32,
    oid: String,
    size: u64,
}
fn decode(s: &str) -> Result<Vec<u8>> {
    if !s.is_ascii() || s.len() % 2 != 0 {
        return Err("odd path hex".into());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(Into::into))
        .collect()
}
fn manifest(path: &Path) -> Result<BTreeMap<PathBuf, Entry>> {
    let mut result = BTreeMap::new();
    for line in fs::read_to_string(path)?.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 4 {
            return Err("manifest fields".into());
        }
        let path = PathBuf::from(std::ffi::OsString::from_vec(decode(fields[3])?));
        if path.as_os_str().is_empty()
            || path
                .components()
                .any(|x| !matches!(x, Component::Normal(_)))
            || path.as_os_str().as_bytes().contains(&0)
        {
            return Err("unsafe input path".into());
        }
        let mode = u32::from_str_radix(fields[0], 8)?;
        if ![0o100644, 0o100755, 0o120000].contains(&mode)
            || fields[1].len() != 40
            || !fields[1].bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err("unsupported entry".into());
        }
        if result
            .insert(
                path,
                Entry {
                    mode,
                    oid: fields[1].into(),
                    size: fields[2].parse()?,
                },
            )
            .is_some()
        {
            return Err("duplicate path".into());
        }
    }
    for path in result.keys() {
        for parent in path.ancestors().skip(1) {
            if result.contains_key(parent) {
                return Err("file is ancestor".into());
            }
        }
    }
    Ok(result)
}
fn dirs(entries: &BTreeMap<PathBuf, Entry>) -> BTreeSet<PathBuf> {
    entries
        .keys()
        .flat_map(|p| {
            p.ancestors()
                .skip(1)
                .filter(|p| !p.as_os_str().is_empty())
                .map(Path::to_path_buf)
        })
        .collect()
}
fn normalize(path: &Path, mode: u32) -> Result<()> {
    fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    File::open(path)?.set_times(
        FileTimes::new().set_modified(UNIX_EPOCH + Duration::from_secs(1_000_000_000)),
    )?;
    Ok(())
}
pub fn import(input: &Path, root: &Path) -> Result<()> {
    let old = manifest(&input.join("previous.tsv"))?;
    let new = manifest(&input.join("manifest.tsv"))?;
    let old_dirs = dirs(&old);
    let new_dirs = dirs(&new);
    for (path, entry) in &old {
        if new.get(path).is_none_or(|n| {
            (n.mode == 0o120000) != (entry.mode == 0o120000)
                || (n.mode == 0o120000 && n.oid != entry.oid)
        }) {
            fs::remove_file(root.join(path))?;
        }
    }
    let mut removed: Vec<_> = old_dirs.difference(&new_dirs).collect();
    removed.sort_by_key(|p| std::cmp::Reverse(p.components().count()));
    for path in removed {
        fs::remove_dir(root.join(path))?;
    }
    let mut ordered: Vec<_> = new_dirs.iter().collect();
    ordered.sort_by_key(|p| p.components().count());
    for path in &ordered {
        if !old_dirs.contains(*path) {
            fs::create_dir(root.join(path))?;
        }
    }
    for (path, entry) in &new {
        if old.get(path) == Some(entry) {
            continue;
        }
        let dest = root.join(path);
        // Validate every parent before opening a destination; never follow an imported link.
        for parent in path
            .ancestors()
            .skip(1)
            .filter(|p| !p.as_os_str().is_empty())
        {
            if !fs::symlink_metadata(root.join(parent))?.is_dir() {
                return Err("non-directory parent".into());
            }
        }
        if entry.mode == 0o120000 {
            let bytes = fs::read(input.join("blobs").join(&entry.oid))?;
            if bytes.len() as u64 != entry.size || bytes.contains(&0) {
                return Err("symlink payload".into());
            }
            symlink(std::ffi::OsString::from_vec(bytes), dest)?;
        } else {
            if fs::symlink_metadata(&dest).is_ok_and(|m| !m.is_file()) {
                return Err("non-file destination".into());
            }
            if old
                .get(path)
                .is_none_or(|o| o.oid != entry.oid || o.mode == 0o120000)
            {
                let size = fs::copy(input.join("blobs").join(&entry.oid), &dest)?;
                if size != entry.size {
                    return Err("file payload length".into());
                }
            }
            normalize(&dest, entry.mode & 0o777)?;
        }
    }
    for path in ordered.into_iter().rev() {
        normalize(&root.join(path), 0o755)?;
    }
    normalize(root, 0o755)?;
    println!("history-import-ok files={}", new.len());
    Ok(())
}
pub fn observe(root: &Path, output: &Path) -> Result<()> {
    let mut out = File::create(output)?;
    let mut queue = vec![PathBuf::new()];
    let mut count = 0;
    while let Some(dir) = queue.pop() {
        for entry in fs::read_dir(root.join(&dir))? {
            let entry = entry?;
            let path = dir.join(entry.file_name());
            let m = fs::symlink_metadata(entry.path())?;
            let (mode, size, digest) = if m.is_dir() {
                queue.push(path.clone());
                (0o040755, 0, "-".into())
            } else if m.file_type().is_symlink() {
                let target = fs::read_link(entry.path())?;
                let bytes = target.as_os_str().as_bytes();
                let mut h = Sha256::new();
                h.update(bytes);
                (0o120000, bytes.len() as u64, hex(&h.finish()))
            } else if m.is_file() {
                let mut f = File::open(entry.path())?;
                let mut h = Sha256::new();
                let mut buf = [0; 65536];
                loop {
                    let n = f.read(&mut buf)?;
                    if n == 0 {
                        break;
                    }
                    h.update(&buf[..n]);
                }
                (
                    0o100000 | (m.permissions().mode() & 0o777),
                    m.len(),
                    hex(&h.finish()),
                )
            } else {
                return Err("unexpected filesystem type".into());
            };
            if m.is_dir() && m.permissions().mode() & 0o777 != 0o755 {
                return Err("directory mode".into());
            }
            writeln!(
                out,
                "{:o}\t{}\t{}\t{}",
                mode,
                size,
                digest,
                hex(path.as_os_str().as_bytes())
            )?;
            count += 1;
        }
    }
    out.sync_all()?;
    println!("history-observe-ok entries={count}");
    Ok(())
}

fn mounted(root: &Path) -> Result<()> {
    let text = fs::read_to_string("/proc/self/mountinfo")?;
    let found = text.lines().any(|line| {
        let Some((left, right)) = line.split_once(" - ") else {
            return false;
        };
        left.split_whitespace().nth(4) == root.to_str()
            && right
                .split_whitespace()
                .next()
                .is_some_and(|kind| kind.starts_with("fuse"))
    });
    if !found {
        return Err("storage smoke target is not a FUSE mount".into());
    }
    println!("storage_smoke_mount=fuse");
    Ok(())
}

fn read_tree(root: &Path) -> Result<()> {
    let mut dirs = vec![root.to_path_buf()];
    let mut files = Vec::new();
    while let Some(dir) = dirs.pop() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_dir() {
                dirs.push(entry.path());
            } else if kind.is_file() {
                files.push(entry.path());
            }
        }
    }
    files.sort();
    let mut bytes = 0_u64;
    let mut buffer = [0; 65536];
    for path in files {
        let mut file = File::open(path)?;
        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            std::hint::black_box(&buffer[..n]);
            bytes = bytes.checked_add(n as u64).ok_or("read byte count")?;
        }
    }
    println!("read_bytes={bytes}");
    Ok(())
}

fn file_change(root: &Path, input: &Path, step: usize) -> Result<()> {
    use std::io::{Seek, SeekFrom};
    use std::os::unix::fs::MetadataExt;
    let path = root.join("file");
    let before = fs::symlink_metadata(&path)?;
    if !before.is_file() {
        return Err("edit requires regular file".into());
    }
    match step {
        1 => {
            let mut file = fs::OpenOptions::new().write(true).open(&path)?;
            file.seek(SeekFrom::Start(before.len() / 4))?;
            file.write_all(&[b'B'; 4096])?;
        }
        2 | 4 => {
            let mut source = File::open(input.join(format!("state-{step}")))?;
            let mut dest = File::create(&path)?;
            std::io::copy(&mut source, &mut dest)?;
        }
        3 => {
            let temp = root.join("save.tmp");
            let mut source = File::open(input.join("state-3"))?;
            let mut dest = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp)?;
            std::io::copy(&mut source, &mut dest)?;
            drop(dest);
            fs::rename(temp, &path)?;
        }
        _ => return Err("ordinary edit step".into()),
    }
    normalize(&path, 0o644)?;
    normalize(root, 0o755)?;
    println!(
        "initial_inode={}\nfinal_inode={}\nwrite_route={}",
        before.ino(),
        fs::metadata(&path)?.ino(),
        match step {
            1 => "in-place",
            3 => "tempfile-rename",
            _ => "truncate-rewrite",
        }
    );
    Ok(())
}

fn small_change(root: &Path, step: usize) -> Result<()> {
    let growing = root.join("d0/f032");
    match step {
        1 => fs::OpenOptions::new()
            .append(true)
            .open(&growing)?
            .write_all(b"gg")?,
        2 => fs::set_permissions(root.join("d0/f040"), fs::Permissions::from_mode(0o644))?,
        3 => fs::OpenOptions::new()
            .write(true)
            .open(&growing)?
            .set_len(8191)?,
        _ => return Err("small-file change step".into()),
    }
    Ok(())
}

pub fn dispatch(args: &[String]) -> Result<()> {
    match args {
        [command, root, op, path, offset, length] if command == "storage-smoke-access" => {
            access(Path::new(root), op, path, offset.parse()?, length.parse()?)
        }
        [command, input, root] if command == "storage-smoke-import" => {
            mounted(Path::new(root))?;
            import(Path::new(input), Path::new(root))
        }
        [command, root, output] if command == "storage-smoke-observe" => {
            mounted(Path::new(root))?;
            observe(Path::new(root), Path::new(output))
        }
        [command, root] if command == "storage-smoke-read" => {
            mounted(Path::new(root))?;
            read_tree(Path::new(root))
        }
        [command, root, input, step] if command == "storage-smoke-file-change" => {
            mounted(Path::new(root))?;
            file_change(Path::new(root), Path::new(input), step.parse()?)
        }
        [command, root, step] if command == "storage-smoke-small-change" => {
            mounted(Path::new(root))?;
            small_change(Path::new(root), step.parse()?)
        }
        _ => Err("storage smoke workload arguments".into()),
    }
}


fn access(root: &Path, op: &str, relative: &str, offset: u64, length: usize) -> Result<()> {
    use std::os::unix::fs::{FileExt, MetadataExt};
    mounted(root)?;
    let relative = Path::new(relative);
    if relative != Path::new(".") && (relative.as_os_str().is_empty() || relative.components().any(|c| !matches!(c, Component::Normal(_)))) {
        return Err("unsafe access path".into());
    }
    if length > 1024 * 1024 || offset.checked_add(length as u64).is_none() {
        return Err("access range budget".into());
    }
    let path = root.join(relative);
    let mut bytes = vec![0; if op == "read" { length } else { 0 }];
    let mut names = Vec::new();
    let mut metadata = None;
    let start = std::time::Instant::now();
    match op {
        "stat" => metadata = Some(fs::symlink_metadata(path)?),
        "directory" => {
            for entry in fs::read_dir(path)? {
                if names.len() == 128 { return Err("directory bound exceeded".into()); }
                names.push(entry?.file_name().as_bytes().to_vec());
            }
        }
        "read" => File::open(path)?.read_exact_at(&mut bytes, offset)?,
        _ => return Err("unknown access operation".into()),
    }
    let ns = start.elapsed().as_nanos();
    println!("access_operation_ns={ns}\naccess_completed_count=1\naccess_returned_bytes={}", bytes.len());
    if let Some(m) = metadata {
        println!("access_mode={:o}\naccess_size={}\naccess_mtime={}\naccess_mtime_nsec={}", m.mode(), m.size(), m.mtime(), m.mtime_nsec());
    } else if op == "directory" {
        names.sort();
        println!("access_names={}", names.iter().map(|n| hex(n)).collect::<Vec<_>>().join(","));
    } else {
        let mut hash = Sha256::new(); hash.update(&bytes);
        println!("access_sha256={}", hex(&hash.finish()));
    }
    Ok(())
}
