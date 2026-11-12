# Trusted publishing status

Automated npm publication is not configured in this repository. Release
preparation and local artifact builds are separate from publishing.

An owner may later adopt npm's trusted publisher mechanism after reviewing the
registry settings, exact `craftguild/oriha.rs` workflow/environment identity,
current npm requirements and reproducible native artifact delivery. That would
need an explicit publishing workflow and authorization. Do not infer that such a
publisher exists from this document or add registry credentials to ordinary CI.

The current procedure is [release preparation](releases.md).
