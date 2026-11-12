# stats-ingest deployment

`oriha-stats-ingest` は `apprunner@.service` の instance として起動する。

## 初回のみ

```bash
sudo useradd --system --home /nonexistent --shell /usr/sbin/nologin apprunner
sudo install -d -m 0755 /etc/default/apprunner
sudo install -d -m 0755 /opt/apprunner
```

`/etc/systemd/system/apprunner@.service` を作成する。

```ini
[Unit]
Description=App runner: %i
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=apprunner
Group=apprunner
EnvironmentFile=/etc/default/apprunner/%i.env
WorkingDirectory=/opt/apprunner/%i
ExecStart=/opt/apprunner/%i/%i
Restart=always
RestartSec=5

NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/apprunner/%i/raw

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
```

## oriha-stats-ingest の配置

```bash
cargo build --release -p oriha-stats-ingest

sudo install -d -o root -g root -m 0755 /opt/apprunner/oriha-stats-ingest
sudo install -d -o apprunner -g apprunner -m 0750 /opt/apprunner/oriha-stats-ingest/raw
sudo install -o root -g root -m 0755 \
  target/release/oriha-stats-ingest \
  /opt/apprunner/oriha-stats-ingest/oriha-stats-ingest
```

`/etc/default/apprunner/oriha-stats-ingest.env` を作成する。

```dotenv
ORIHA_STATS_BIND_ADDR=127.0.0.1:5000
ORIHA_STATS_DATA_DIR=/opt/apprunner/oriha-stats-ingest/
ORIHA_STATS_HMAC_SECRET_FILE=/etc/default/apprunner/oriha-stats-ingest.hmac
ORIHA_STATS_API_KEY_FILE=/etc/default/apprunner/oriha-stats-ingest.apikey
RUST_LOG=info
```

secret file を作成する。

```bash
sudo install -o root -g apprunner -m 0640 /dev/null /etc/default/apprunner/oriha-stats-ingest.hmac
sudo install -o root -g apprunner -m 0640 /dev/null /etc/default/apprunner/oriha-stats-ingest.apikey
sudoedit /etc/default/apprunner/oriha-stats-ingest.hmac
sudoedit /etc/default/apprunner/oriha-stats-ingest.apikey
```

起動する。

```bash
sudo systemctl enable --now apprunner@oriha-stats-ingest
sudo systemctl status apprunner@oriha-stats-ingest
```

ログ確認。

```bash
journalctl -u apprunner@oriha-stats-ingest -f
```

## 収集データの集計表示

raw JSONL の集計確認には `oriha-stats-viewer` を使う。

```bash
cargo run -p oriha-stats-viewer -- /opt/apprunner/oriha-stats-ingest
cargo run -p oriha-stats-viewer -- --group-by day --from 2026-06-01 --to 2026-06-15 /opt/apprunner/oriha-stats-ingest
cargo run -p oriha-stats-viewer -- --last 7d /opt/apprunner/oriha-stats-ingest
```
