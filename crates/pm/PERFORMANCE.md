# Performance Optimization Notes

## Current Status

**Test case**: Next.js project with 342 packages
**Bun**: ~1.3 seconds
**Deka PM**: ~4.2 seconds

We're now **3.2x slower than Bun** (down from ~45x slower initially).

## Optimizations Implemented

### 1. Async Parallel Downloads (tokio)
- Converted from sequential blocking downloads to async/await
- Uses `tokio::task::JoinSet` to manage concurrent downloads
- **Semaphore limit**: 100 concurrent downloads (matches Bun's aggressive parallelism)
- Downloads happen in parallel while dependencies are being resolved

**Impact**: Massive - reduced from ~180ms per package to ~25ms per package

### 2. Global Package Cache
- Location: `~/.config/deka/pm/cache/`
- Structure:
  - `node/` - Extracted package contents
  - `archives/` - Downloaded tarballs
  - `meta/` - Package metadata (integrity, resolved URL)
- Packages are downloaded once and reused across projects

**Impact**: First install still slow, but subsequent installs benefit from cached tarballs

### 3. Copy-on-Write Filesystem Features

#### macOS (APFS)
- Uses `clonefile(2)` system call for instant CoW clones
- Falls back to hardlinks if clonefile fails
- Falls back to copy if hardlinks fail

#### Linux (future)
- Should use `copy_file_range()` or `reflink` for btrfs/XFS
- Currently falls back to hardlinks

**Implementation**: `crates/pm/src/cache.rs:174-219`

```rust
fn hardlink_dir(source: &Path, destination: &Path) -> Result<()> {
    // Recursively creates directory structure
    // For each file:
    //   1. Try clonefile (macOS APFS) - instant CoW
    //   2. Try hardlink - instant, same inode
    //   3. Fall back to copy - slow
}
```

**Impact**: Moderate - on APFS, copies are nearly instant. On other filesystems, hardlinks are still fast.

### 4. Parallel Copy Operations
- Copy operations run in parallel using `tokio::task::spawn_blocking`
- Downloads complete → spawn copy task → immediately spawn dependencies
- All copy tasks run on tokio's blocking thread pool

**Impact**: Moderate - prevents blocking the async event loop during I/O

### 5. Eager Dependency Spawning
- After each package completes, immediately spawn ALL queued dependencies
- No longer spawning one at a time
- Maximizes parallelism throughout the dependency tree

**Code**: `crates/pm/src/install.rs:159-185`

## Performance Bottlenecks Remaining

### 1. Network I/O (~1.5s overhead)
Even with 100 concurrent downloads, we're still slower than Bun. Possible causes:
- **HTTP/1.1 vs HTTP/2**: Bun might be using HTTP/2 multiplexing
- **Connection pooling**: We might be creating new connections for each request
- **DNS lookups**: Not cached, each package does its own DNS resolution
- **reqwest overhead**: Might be slower than Bun's native HTTP client

**Future fix**: Use HTTP/2, persistent connections, DNS caching

### 2. Metadata Fetching
We fetch full package metadata from npm registry for each package:
- `https://registry.npmjs.org/react` returns ~200KB of JSON (all versions)
- Bun likely uses a more efficient protocol or caches this better

**Current approach**:
```rust
pub async fn fetch_npm_metadata(name: &str) -> Result<Value> {
    let url = format!("https://registry.npmjs.org/{encoded}");
    let response = reqwest::get(&url).await?;
    response.json::<Value>().await
}
```

**Future fix**:
- Only fetch needed version metadata using npm registry API
- Cache metadata in-memory across package installs
- Consider using npm's "abbreviated metadata" endpoint

### 3. Lockfile Updates
Currently writing to lockfile synchronously for each package:
```rust
lock::update_lock_entry("node", &name, descriptor, resolved, metadata, integrity)?;
```

**Future fix**:
- Batch lockfile updates
- Write once at the end with all packages
- Use async file I/O

### 4. File System Operations
Even with clonefile/hardlinks, we still:
- Create many directories (`fs::create_dir_all`)
- Recursively walk directory trees
- Make many small syscalls

**Future fix**:
- Batch directory creation
- Use `rayon` for parallel directory walking
- Consider memory-mapped I/O for metadata files

## Ideas for Future Sessions

### High Priority

1. **HTTP/2 Connection Pooling**
   - Configure reqwest with persistent HTTP/2 connections
   - Reuse connections across all npm registry requests
   - Should reduce connection overhead significantly

2. **In-Memory Metadata Cache**
   - Cache npm metadata in a `HashMap<String, Value>` during install
   - Avoid re-fetching metadata for already-seen packages
   - Clear cache after install completes

3. **Batch Lockfile Updates**
   - Collect all package updates in memory
   - Write lockfile once at the end
   - Use `serde_json::to_writer_pretty` for formatted output

4. **Better Progress Indication**
   - Show current download/copy progress
   - Display package count (e.g., "152/342 packages")
   - Only in debug mode to keep output clean

### Medium Priority

5. **Linux Reflink Support**
   - Detect filesystem type (btrfs, XFS)
   - Use `ioctl(FICLONE)` for reflink copies
   - Same performance as macOS clonefile

6. **Compression Support**
   - npm tarballs are gzipped
   - We're extracting with tar command
   - Could use Rust `flate2` + `tar` crates for better control

7. **Parallel Tarball Extraction**
   - Currently extracts serially in `extract_tarball()`
   - Could spawn extraction tasks in parallel with downloads
   - Use tokio blocking pool

8. **Content-Addressable Storage**
   - Store files by hash instead of package name/version
   - Deduplicate across packages (e.g., same LICENSE file)
   - Similar to Git's object store

### Low Priority

9. **Binary Manifest Cache**
   - Pre-parse package.json files
   - Store in binary format (e.g., MessagePack)
   - Faster than re-parsing JSON each time

10. **Integrity Check Optimization**
    - Currently computing SHA-512 on every install
    - Could trust cache and only verify on first download
    - Add `--verify` flag for paranoid mode

11. **Dependency Resolution Cache**
    - Cache resolved dependency trees
    - Skip resolution if package.json hasn't changed
    - Similar to yarn's resolution cache

12. **Custom npm Registry Client**
    - Replace reqwest with a purpose-built client
    - HTTP/2 multiplexing
    - Built-in retry logic and rate limiting
    - Streaming JSON parsing

## Architecture Notes

### Current Flow

1. **Parse package.json** → resolve specs from dependencies
2. **Load bun.lock** → get pinned versions
3. **Spawn initial tasks** → enqueue top-level dependencies
4. **Download loop**:
   - Fetch metadata from npm registry
   - Resolve version from lockfile or dist-tags
   - Download tarball (or use cache)
   - Extract to cache directory
   - Parse package.json for dependencies
   - Enqueue dependencies
   - Spawn new tasks
5. **Copy loop** (runs in parallel):
   - Determine install path based on lock_key
   - Copy from cache to node_modules
   - Update lockfile entry
6. **Wait for all tasks** to complete
7. **Print summary**

### Why We're Still Slower Than Bun

Bun is written in Zig and uses:
- **Zig's async I/O** - Lower overhead than tokio
- **JavaScriptCore** - Fast JSON parsing
- **Custom HTTP client** - Optimized for npm registry
- **Optimized syscalls** - Direct system calls, no abstraction layers
- **Hardlink by default** - Always uses hardlinks, never copies
- **Better caching** - Likely caches more aggressively

We're using:
- **Tokio** - Great async runtime, but more overhead
- **serde_json** - Fast, but not as fast as JSC for parsing
- **reqwest** - General-purpose HTTP client
- **std::fs** - Standard library file operations
- **Fallback strategy** - Try clonefile → hardlink → copy

## Conclusion

We've made excellent progress:
- **58 seconds → 4.2 seconds** (13.8x faster)
- Now only **3.2x slower than Bun** (vs 45x initially)

The remaining gap is mostly:
1. Network overhead (~40%)
2. Metadata fetching/parsing (~30%)
3. File system operations (~20%)
4. Lockfile updates (~10%)

With the optimizations listed above (especially HTTP/2 pooling and metadata caching), we should be able to get within 2x of Bun's performance, which would be excellent for a Rust implementation.
