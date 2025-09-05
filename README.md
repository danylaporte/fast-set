# fast-set


### Tests

Run dhat-memory

```bash
DHAT_FILE=heap.json cargo test --test dhat_memory
```

Run miri

```bash
cargo miri test --test miri_unsafe
```
