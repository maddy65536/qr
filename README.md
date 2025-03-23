# qr
A qr code generator built using only the rust standard library.

Supports all versions, all error correction levels, and byte, alphanumeric, and numeric modes.

```
Usage: qr "message" [options]

options:
    -e / --ec [low|medium|quartile|high]
    -m / --mask [0-7]
    -v / --min-version [1-40]
    -o / --output (path)
```

![qr code containing the text "this qr code was generated using this project!"](./example.png)
