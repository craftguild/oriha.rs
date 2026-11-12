#!/usr/bin/env python3
"""Browser regression for unchanged HTML from actual release Rust image APIs."""
import argparse
import hashlib
import html
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def verify(chrome, work):
    fixtures_path = work / "fixtures.json"
    subprocess.run([
        "cargo", "test", "-p", "oriha-core", "--test", "image_presentation_urls",
        "data_image_candidates_preserve_syntax_in_all_real_api_paths",
        "--release", "--locked", "--", "--exact",
    ], cwd=ROOT, env=dict(os.environ, ORIHA_DATA_URL_BROWSER_FIXTURES=str(fixtures_path)), check=True)
    fixtures = json.loads(fixtures_path.read_text())
    assert len(fixtures) == 24
    sections = []
    metadata = []
    for fixture in fixtures:
        # Embed the actual output verbatim; neither HTML nor URLs are repaired.
        sections.append(f'<div data-case="{fixture["id"]}">{fixture["html"]}</div>')
        metadata.append({key: value for key, value in fixture.items() if key != "html"})
    source = fixtures[0]["expectedSource"]
    broken = source.replace(",", "%2C", 1)
    sections.append(
        f'<img id="broken-control" src="{html.escape(source, quote=True)}" '
        f'srcset="{html.escape(broken, quote=True)} 1w" sizes="1px" width="1" height="1">'
    )
    script = """
    (async () => {
        const results = [];
        for (const fixture of FIXTURES) {
            const images = document.querySelector(`[data-case="${fixture.id}"]`).querySelectorAll('img');
            if (images.length !== 2) throw Error('missing figure/inline image');
            for (const [index, image] of images.entries()) {
                let error = null;
                try { await image.decode(); } catch (e) { error = e.name; }
                const selected = new URL(fixture.expectedSelected, document.baseURI).href;
                const src = image.getAttribute('src');
                const srcset = image.getAttribute('srcset');
                results.push({id: fixture.id, index, src, srcset, currentSrc: image.currentSrc,
                    naturalWidth: image.naturalWidth, naturalHeight: image.naturalHeight, error,
                    ok: error === null && image.naturalWidth === 1 && image.naturalHeight === 1 &&
                        image.currentSrc === selected && src === fixture.expectedSource &&
                        srcset === fixture.expectedSrcset});
            }
        }
        const control = document.getElementById('broken-control');
        let error = null;
        try { await control.decode(); } catch (e) { error = e.name; }
        const negative = {error, currentSrc: control.currentSrc,
            naturalWidth: control.naturalWidth, naturalHeight: control.naturalHeight};
        document.getElementById('result').textContent = JSON.stringify({results, negative});
    })().catch(e => { document.getElementById('result').textContent = JSON.stringify({error: String(e)}); });
    """.replace("FIXTURES", json.dumps(metadata).replace("<", "\\u003c"))
    page = work / "index.html"
    page.write_text('<!doctype html><meta charset="utf-8">' + "\n".join(sections) +
                    '<pre id="result"></pre><script>' + script + '</script>')
    try:
        process = subprocess.run([
            "node", str(ROOT / "tools/browser-result.mjs"), str(chrome), str(page),
        ], text=True, capture_output=True, timeout=55, check=True)
    except subprocess.CalledProcessError as error:
        raise RuntimeError(f"Browser assertions failed:\n{error.stderr.strip()}") from error
    result = json.loads(process.stdout)
    assert "error" not in result, result
    assert len(result["results"]) == 48, result
    assert all(row["ok"] for row in result["results"]), result
    negative = result["negative"]
    assert negative["error"] == "EncodingError" and negative["naturalWidth"] == 0, result
    result["browser"] = subprocess.check_output([str(chrome), "--version"], text=True).strip()
    result["fixtureHtmlSha256"] = {
        f["id"]: hashlib.sha256(f["html"].encode()).hexdigest() for f in fixtures
    }
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--chrome", type=Path, default=shutil.which("google-chrome") or
                        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome")
    parser.add_argument("--output", type=Path, help="write the actual browser result as JSON")
    args = parser.parse_args()
    if not args.chrome.is_file():
        parser.error("Chrome is required; specify its executable with --chrome")
    with tempfile.TemporaryDirectory(prefix="oriha-data-url-browser-") as directory:
        result = verify(args.chrome.resolve(), Path(directory).resolve())
    if args.output:
        args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(f'PASS: {len(result["results"])} real API images decoded with expected currentSrc; '
          f'broken-comma control rejected; {result["browser"]}')


if __name__ == "__main__":
    main()
