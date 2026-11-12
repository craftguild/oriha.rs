// Compose distinct target graphs without merging architecture-specific edges.
export function appendGraph(bom, source, namespace) {
  const root = source.metadata.component;
  const components = [root, ...(source.components ?? [])];
  const references = new Map(components.map((component) => [
    component["bom-ref"], `urn:oriha:${namespace}:${encodeURIComponent(component["bom-ref"])}`,
  ]));
  if (references.size !== components.length || references.has(undefined)) {
    throw new Error(`Ambiguous component references in ${namespace}`);
  }
  for (const component of components) {
    bom.components.push({
      ...component,
      "bom-ref": references.get(component["bom-ref"]),
      properties: [...(component.properties ?? []), { name: "oriha:artifact-scope", value: namespace }],
    });
  }
  const mapped = (reference) => {
    const result = references.get(reference);
    if (!result) throw new Error(`Unresolved dependency in ${namespace}: ${reference}`);
    return result;
  };
  for (const dependency of source.dependencies ?? []) {
    bom.dependencies.push({
      ref: mapped(dependency.ref),
      dependsOn: (dependency.dependsOn ?? []).map(mapped),
    });
  }
  return mapped(root["bom-ref"]);
}

export function packageBom(manifest) {
  const reference = `urn:oriha:npm:${manifest.name}:${manifest.version}`;
  return {
    $schema: "http://cyclonedx.org/schema/bom-1.5.schema.json",
    bomFormat: "CycloneDX", specVersion: "1.5", version: 1,
    metadata: {
      component: {
        type: "library", "bom-ref": reference, name: manifest.name, version: manifest.version,
        purl: `pkg:npm/${manifest.name.replace("@", "%40")}@${manifest.version}`,
        licenses: [{ expression: manifest.license.replace(/^\((.*)\)$/, "$1") }],
      },
      properties: [
        { name: "oriha:scope", value: "Actual npm payload; JavaScript bundle modules and five selected native runtime graphs. Build/dev/proc-macro dependencies and system SDKs are excluded." },
        { name: "oriha:external-peer-requirements", value: JSON.stringify(manifest.peerDependencies) },
        { name: "oriha:peer-scope", value: "Peers are supplied by the consuming application; their eventual versions and dependency graphs are not asserted by this library." },
        { name: "oriha:inventory", value: "artifacts.json hashes payload files outside sbom/. SBOM and association files exclude their own bytes to avoid self-hashing cycles." },
      ],
    },
    components: [], dependencies: [{ ref: reference, dependsOn: [] }],
  };
}
