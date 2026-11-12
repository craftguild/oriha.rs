# oriha-stats-viewer

`oriha-stats-viewer` summarizes raw JSONL records written by
`oriha-stats-ingest`.

```bash
cargo run -p oriha-stats-viewer -- /opt/apprunner/oriha-stats-ingest
cargo run -p oriha-stats-viewer -- --group-by day --from 2026-06-01 --to 2026-06-15 ./data/stats
cargo run -p oriha-stats-viewer -- --last 7d ./data/stats
cargo run -p oriha-stats-viewer -- --format json ./data/stats
```

By default, the viewer reads sealed `.jsonl` chunks only. Pass
`--include-open` when you also want to include the active `.jsonl.open` chunk for
the current rotation window.

Use `--last` for relative windows ending at the current UTC time. Supported
units are hours (`h`), days (`d`), weeks (`w`), and months (`m` or `mo`).
`--last` is mutually exclusive with `--from` and `--to`.
