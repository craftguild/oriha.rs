# oriha.rs documentation

## Guides

- [Development, build and install](guides/development.md)
- [Verification methods](guides/verification.md)
- [Expo API](guides/expo.md)
- [Source-aware parsing and annotations](guides/0001-source-aware-parsing.md)
- [Structured and rendered block output](guides/0002-block-output.md)
- [Prepared responsive images](guides/0003-image-presentation.md)
- [Oriha rename migration](guides/0004-oriha-rename.md)
- [Code block options](guides/code-block-options.md)
- [Release preparation](guides/releases.md)
- [Trusted publishing status](guides/trusted-publishing.md)

## Design

- [Engine architecture](design/oriha.md)
- [Block derivation](design/block-derivation.md)
- [Rendering support](design/rendering-support.md)
- [Typed native contract](design/native-bridge-contract.md)
- [JSON compatibility](design/json-contract.md)
- [PlantUML layout](design/plantuml-layout.md)
- [Distribution, SBOM and provenance](design/distribution.md)

## Decisions and standards

[Accepted ADRs](adr/README.md) retain their original numbers and dispositions.
`make prime` renders shared contracts from the verified engine fixed in
[standards/engine.json](../standards/engine.json); use `make setup` after cloning.
The [local Skill](../.agents/skills/craftguild-standardize/SKILL.md) explains topic selection.
[craftguild.toml](../craftguild.toml) selects this product's artifacts, release
groups, quality checks and source roles. Private evaluation logs and goal prompts
are not build inputs or substitutes for the public contracts and fixtures.
