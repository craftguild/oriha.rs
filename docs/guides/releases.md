# Release preparation

`craftguild.toml` owns three release groups:

| Group | Current version authority | Tag prefix | Scope |
| --- | --- | --- | --- |
| `oriha` | Cargo workspace and Expo package, both 0.1.0 | `v` | All 15 Rust members and the Expo npm package/lockfile |
| `font-registry` | Private registry package, 0.1.0 | `font-registry-v` | Independent registry deployment |
| `expo-preview` | Private demo package, 0.1.0 | `expo-preview-v` | Independent example app |

The standards engine is pinned to v0.6.2; its normative rules baseline remains 0.1.0. Neither sets product versions. Cargo members inherit the workspace version. Font registry and demo manifests are not bumped with the Oriha package. The demo's local package link is resolved within this repository, without a separately published adoption revision.

## Local preparation

Begin with reviewed source committed using OpenPGP signatures, full history and
all applicable tags. The common contract rejects unrelated dirty work, invalid
Conventional Commits, missing/invalid signatures and version/tag disagreement.
Historical migration and any public-history rewrite are separate reviewed work;
they are not performed by `make release`.

When intentionally preparing a release:

```sh
make release RELEASE_ARGS=--json
make release
```

The first command previews. The second updates local version manifests,
lockfiles and English notes with an immutable receipt. Repeating the same
preparation does not increment again. `RELEASE_GROUP=font-registry` or
`RELEASE_GROUP=expo-preview` selects another group. The package's `npm run release`
is a compatibility entry for the Oriha group, with the same preparation behavior.
It no longer builds artifacts or uses a separate Node versioning engine.

The common Conventional Commit implementation selects one increment since the
previous group tag: breaking/feat selects MINOR and fix selects PATCH below 1.0.0;
breaking selects MAJOR from 1.0.0. Use `--major` for an intentional major request.
Initial preparation retains the configured initial version. Read the matching
contract with `make prime PRIME_ARGS="--topic releases"` before changing ownership.

Inspect the prepared diff, then create and verify the intended signed commit.
A signing/verification failure stops the workflow. Build final artifacts from the
clean reviewed commit and run `make acceptance`, `make check` and
`make build-expo` before a separately authorized signed tag/publication. No entry
implicitly commits, tags, pushes or publishes.

## Package publication

The package configuration prepares public npm access. This does not publish a new
version or change an existing registry release. Publication stays an explicit
owner operation. Preserve the existing release security preflight: registry
signature audit, version/tag alignment, credential/lifecycle-hook checks and
packed-file inspection. Run its inspection before authenticating for publication.
Do not place long-lived npm publishing tokens in CI.

The shipped `sbom/` directory contains the composed package graph, actual
JavaScript graph, five native graphs and the payload/hash association inventory.
The separate font registry deployment retains its own fonts and licenses. A
package SBOM cannot describe the eventual consuming application's peer graph.
