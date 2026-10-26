# Windows room-store platform decision — 2026-10-24

Primary references checked:
- https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-flushfilebuffers — requires GENERIC_WRITE; a read-only directory handle is unsuitable for this call.
- https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw — provides REPLACE_EXISTING and WRITE_THROUGH. Its explicit flush wording covers copy/delete moves; it does not establish Unix directory-fsync equivalence for every filesystem.
- https://learn.microsoft.com/en-us/windows/win32/fileio/obtaining-a-handle-to-a-directory — directory handles support a documented subset of file APIs.

Implementation uses file.sync_all, closes the temporary file, then same-directory MoveFileExW(REPLACE_EXISTING | WRITE_THROUGH), without COPY_ALLOWED. Canonicalized source and destination parent produce Windows extended paths, encoded as validated NUL-terminated UTF-16. Failed calls with uncertain source existence report replacement_visible=true. No Windows directory handle FlushFileBuffers call remains.

Windows directory creation and unlink have no Unix directory-fsync promise here. Two Windows-only tests cover Unicode/long paths/replacement and NUL/blocked destination preservation. They were added but not executed on this macOS host. Platform runtime, filesystems and power-loss behavior remain unverified.
