# qr
A qr code generator built using only the rust standard library.

Supports all versions, all error correction levels, and byte, alphanumeric, and numeric modes.

```
Usage: qr "message" [options]

options:
    --ec [low|medium|quartile|high]
    --mask [0-7]
    --min-version [1-40]
```
