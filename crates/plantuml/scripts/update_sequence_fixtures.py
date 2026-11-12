#!/usr/bin/env python3
from __future__ import annotations

import argparse
import html
import json
import re
import shutil
import subprocess
import sys
import tempfile
import urllib.request
from html.parser import HTMLParser
from pathlib import Path


DEFAULT_URL = "https://plantuml.com/ja/sequence-diagram"


class SequenceExampleParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self._in_pre = False
        self._pre_chunks: list[str] = []
        self.examples: list[str] = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        if tag == "pre":
            self._in_pre = True
            self._pre_chunks = []

    def handle_endtag(self, tag: str) -> None:
        if tag != "pre" or not self._in_pre:
            return

        source = "".join(self._pre_chunks).replace("\r\n", "\n").replace("\r", "\n")
        source = source.strip("\n") + "\n"
        stripped = source.strip()
        if stripped.startswith("@startuml") and stripped.endswith("@enduml"):
            self.examples.append(source)

        self._in_pre = False
        self._pre_chunks = []

    def handle_data(self, data: str) -> None:
        if self._in_pre:
            self._pre_chunks.append(data)


def main() -> int:
    args = parse_args()
    repo_root = Path(__file__).resolve().parents[3]
    fixtures_dir = args.fixtures_dir or repo_root / "crates/plantuml/tests/fixtures/sequence"
    plantuml_jar = args.plantuml_jar or repo_root / "plantuml-mit-1.2026.2.jar"
    renderer_bin = args.renderer_bin or repo_root / "target/debug/oriha-plantuml"

    if args.index_only:
        count = write_visual_index(fixtures_dir)
        print(f"wrote visual index for {count} sequence fixtures to {fixtures_dir / 'index.html'}")
        return 0

    if args.from_json:
        fixtures = read_json_fixtures(fixtures_dir)
    else:
        html_source = read_html(args.url, args.html)
        examples = extract_examples(html_source)
        if not examples:
            raise SystemExit("no @startuml examples were found")

        if not plantuml_jar.is_file():
            raise SystemExit(f"PlantUML jar not found: {plantuml_jar}")

        fixtures = []
        for index, source in enumerate(examples, start=1):
            name = f"{index:03d}"
            print(f"rendering PlantUML {name}/{len(examples):03d}", file=sys.stderr, flush=True)
            golden = render_plantuml_svg(plantuml_jar, source, name, args.timeout_seconds)
            fixtures.append({"name": name, "source": source, "golden": golden})

    if not args.skip_renderer_build:
        build_renderer(repo_root)
    if not renderer_bin.is_file():
        raise SystemExit(f"oriha-plantuml renderer binary not found: {renderer_bin}")

    for index, fixture in enumerate(fixtures, start=1):
        print(
            f"rendering oriha-plantuml {fixture['name']}/{len(fixtures):03d}",
            file=sys.stderr,
            flush=True,
        )
        fixture["rendered"] = render_oriha_svg(
            renderer_bin,
            fixture["source"],
            fixture["name"],
            args.timeout_seconds,
        )

    write_fixture_files(fixtures_dir, fixtures, write_json=not args.no_json)
    count = write_visual_index(fixtures_dir)
    print(f"wrote {len(fixtures)} sequence fixtures to {fixtures_dir}")
    print(f"wrote visual index for {count} sequence fixtures to {fixtures_dir / 'index.html'}")
    return 0


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Fetch or split PlantUML sequence examples and write numbered fixtures."
    )
    parser.add_argument("--url", default=DEFAULT_URL, help="PlantUML documentation URL")
    parser.add_argument(
        "--from-json",
        action="store_true",
        help="Split existing numbered JSON fixtures instead of fetching --url",
    )
    parser.add_argument(
        "--html",
        type=Path,
        help="Use a local HTML file instead of fetching --url",
    )
    parser.add_argument(
        "--fixtures-dir",
        type=Path,
        help="Directory that receives 001.json, 002.json, ...",
    )
    parser.add_argument(
        "--plantuml-jar",
        type=Path,
        help="Path to plantuml-mit jar used for golden SVG generation",
    )
    parser.add_argument(
        "--renderer-bin",
        type=Path,
        help="Path to oriha-plantuml CLI binary used for local SVG generation",
    )
    parser.add_argument(
        "--skip-renderer-build",
        action="store_true",
        help="Do not run cargo build -p oriha-plantuml before rendering local SVGs",
    )
    parser.add_argument(
        "--no-json",
        action="store_true",
        help="Do not write compatibility JSON fixtures when fetching from URL",
    )
    parser.add_argument(
        "--index-only",
        action="store_true",
        help="Only regenerate index.html from existing puml/svg fixture pairs",
    )
    parser.add_argument(
        "--timeout-seconds",
        type=float,
        default=20.0,
        help="Maximum seconds allowed for each PlantUML render",
    )
    return parser.parse_args()


def read_html(url: str, html_path: Path | None) -> str:
    if html_path is not None:
        return html_path.read_text(encoding="utf-8")

    request = urllib.request.Request(url, headers={"User-Agent": "oriha-fixture-updater"})
    with urllib.request.urlopen(request, timeout=30) as response:
        charset = response.headers.get_content_charset() or "utf-8"
        return response.read().decode(charset)


def extract_examples(html: str) -> list[str]:
    parser = SequenceExampleParser()
    parser.feed(html)
    return parser.examples


def read_json_fixtures(fixtures_dir: Path) -> list[dict[str, str]]:
    fixtures = []
    for path in sorted(fixtures_dir.glob("[0-9][0-9][0-9].json")):
        payload = json.loads(path.read_text(encoding="utf-8"))
        source = payload.get("source")
        golden = payload.get("golden")
        if not isinstance(source, str) or not isinstance(golden, str):
            raise SystemExit(f"{path} should contain string source and golden fields")
        fixtures.append({"name": path.stem, "source": source, "golden": golden})
    if not fixtures:
        raise SystemExit(f"no numbered JSON fixtures found in {fixtures_dir}")
    return fixtures


def build_renderer(repo_root: Path) -> None:
    completed = subprocess.run(
        ["cargo", "build", "-p", "oriha-plantuml"],
        cwd=repo_root,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if completed.returncode != 0:
        raise SystemExit(
            "failed to build oriha-plantuml CLI:\n"
            f"{completed.stdout}\n{completed.stderr}"
        )


def render_plantuml_svg(
    plantuml_jar: Path,
    source: str,
    fixture_name: str,
    timeout_seconds: float,
) -> str:
    command = ["java", "-jar", str(plantuml_jar), "--svg", "--pipe"]
    try:
        completed = subprocess.run(
            command,
            input=source,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            timeout=timeout_seconds,
        )
    except subprocess.TimeoutExpired as error:
        raise SystemExit(
            f"PlantUML timed out for fixture {fixture_name} after "
            f"{timeout_seconds:g}s"
        ) from error
    if completed.returncode != 0:
        raise SystemExit(
            f"PlantUML failed for fixture {fixture_name} with exit code "
            f"{completed.returncode}:\n{completed.stderr}"
        )
    if "<svg" not in completed.stdout or "</svg>" not in completed.stdout:
        raise SystemExit(f"PlantUML did not emit SVG for fixture {fixture_name}")
    return completed.stdout


def render_oriha_svg(
    renderer_bin: Path,
    source: str,
    fixture_name: str,
    timeout_seconds: float,
) -> str:
    try:
        completed = subprocess.run(
            [str(renderer_bin), "-"],
            input=source,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            timeout=timeout_seconds,
        )
    except subprocess.TimeoutExpired as error:
        return error_svg(fixture_name, f"oriha-plantuml timed out after {timeout_seconds:g}s")

    if completed.returncode != 0:
        return error_svg(
            fixture_name,
            f"oriha-plantuml failed with exit code {completed.returncode}:\n{completed.stderr}",
        )
    if "<svg" not in completed.stdout or "</svg>" not in completed.stdout:
        return error_svg(fixture_name, "oriha-plantuml did not emit SVG")
    return completed.stdout


def error_svg(fixture_name: str, message: str) -> str:
    escaped_name = html.escape(fixture_name)
    escaped_message = html.escape(message.strip() or "unknown error")
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" data-diagram-type="SEQUENCE" '
        'data-render-status="error" viewBox="0 0 720 180" width="720" height="180">'
        '<rect width="100%" height="100%" fill="#fff5f5" stroke="#dc2626"/>'
        f'<text x="24" y="36" fill="#991b1b" font-family="monospace" font-size="16">'
        f"{escaped_name}: unsupported by oriha-plantuml</text>"
        f'<foreignObject x="24" y="56" width="672" height="104">'
        f'<pre xmlns="http://www.w3.org/1999/xhtml" '
        f'style="margin:0;white-space:pre-wrap;font:12px monospace;color:#7f1d1d;">'
        f"{escaped_message}</pre></foreignObject></svg>"
    )


def write_fixture_files(
    fixtures_dir: Path,
    fixtures: list[dict[str, str]],
    *,
    write_json: bool,
) -> None:
    fixtures_dir.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="sequence-fixtures-", dir=fixtures_dir) as tmp:
        tmp_dir = Path(tmp)
        for fixture in fixtures:
            name = fixture["name"]
            (tmp_dir / f"{name}.puml").write_text(fixture["source"], encoding="utf-8")
            (tmp_dir / f"{name}-golden.svg").write_text(fixture["golden"], encoding="utf-8")
            (tmp_dir / f"{name}.svg").write_text(fixture["rendered"], encoding="utf-8")
            if not write_json:
                continue

            payload = {"source": fixture["source"], "golden": fixture["golden"]}
            (tmp_dir / f"{name}.json").write_text(
                json.dumps(payload, ensure_ascii=False, indent=2) + "\n",
                encoding="utf-8",
            )

        for pattern in (
            "[0-9][0-9][0-9].puml",
            "[0-9][0-9][0-9]-golden.svg",
            "[0-9][0-9][0-9].svg",
        ):
            for path in fixtures_dir.glob(pattern):
                path.unlink()
        if write_json:
            for path in fixtures_dir.glob("[0-9][0-9][0-9].json"):
                path.unlink()

        for path in sorted(tmp_dir.iterdir()):
            shutil.move(str(path), fixtures_dir / path.name)


def write_visual_index(fixtures_dir: Path) -> int:
    cases = visual_fixture_cases(fixtures_dir)
    if not cases:
        raise SystemExit(f"no numbered puml fixtures found in {fixtures_dir}")

    index_path = fixtures_dir / "index.html"
    index_path.write_text(visual_index_html(cases), encoding="utf-8")
    return len(cases)


def visual_fixture_cases(fixtures_dir: Path) -> list[dict[str, str]]:
    cases = []
    for source_path in sorted(fixtures_dir.glob("[0-9][0-9][0-9].puml")):
        name = source_path.stem
        golden_path = fixtures_dir / f"{name}-golden.svg"
        rendered_path = fixtures_dir / f"{name}.svg"
        if not golden_path.is_file():
            raise SystemExit(f"missing golden SVG for fixture {name}: {golden_path}")
        if not rendered_path.is_file():
            raise SystemExit(f"missing rendered SVG for fixture {name}: {rendered_path}")

        source = source_path.read_text(encoding="utf-8")
        rendered_svg = rendered_path.read_text(encoding="utf-8")
        status = "unsupported" if 'data-render-status="error"' in rendered_svg else "supported"
        cases.append(
            {
                "name": name,
                "status": status,
                "source": source,
                "summary": source_summary(source),
                "error": render_error_summary(rendered_svg) if status == "unsupported" else "",
                "golden_svg": golden_path.name,
                "rendered_svg": rendered_path.name,
                "source_file": source_path.name,
            }
        )
    return cases


def source_summary(source: str) -> str:
    for line in source.splitlines():
        stripped = line.strip()
        if stripped and not stripped.startswith("'") and stripped not in {"@startuml", "@enduml"}:
            return stripped
    return "empty diagram"


def render_error_summary(rendered_svg: str) -> str:
    match = re.search(r"<pre\b[^>]*>(.*?)</pre>", rendered_svg, flags=re.DOTALL)
    if not match:
        return "render failed"
    return " ".join(html.unescape(match.group(1)).split())


def visual_index_html(cases: list[dict[str, str]]) -> str:
    supported = sum(1 for case in cases if case["status"] == "supported")
    unsupported = len(cases) - supported
    case_markup = "\n".join(visual_case_html(case) for case in cases)
    return Path(__file__).with_name("sequence_index.html").read_text(encoding="utf-8").format(
        total=len(cases),
        supported=supported,
        unsupported=unsupported,
        case_markup=case_markup,
    )


def visual_case_html(case: dict[str, str]) -> str:
    name = html.escape(case["name"])
    status = html.escape(case["status"])
    summary = html.escape(case["summary"])
    source = html.escape(case["source"])
    golden_svg = html.escape(case["golden_svg"])
    rendered_svg = html.escape(case["rendered_svg"])
    source_file = html.escape(case["source_file"])
    error = case["error"]
    error_html = ""
    if error:
        error_html = f'\n      <p class="error">{html.escape(error)}</p>'

    return f"""      <section class="case" id="case-{name}" data-status="{status}">
        <header class="case-header">
          <div>
            <div class="case-title">
              <h2><a href="#case-{name}">{name}</a></h2>
              <span class="badge {status}">{status}</span>
            </div>
            <p class="summary">{summary}</p>
          </div>
          <nav class="links" aria-label="{name} files">
            <a href="{source_file}">PUML</a>
            <a href="{golden_svg}">PlantUML SVG</a>
            <a href="{rendered_svg}">oriha SVG</a>
          </nav>
        </header>
        <div class="pair">
          <figure>
            <figcaption>PlantUML</figcaption>
            <div class="svg-frame">
              <img loading="lazy" src="{golden_svg}" alt="{name} PlantUML golden SVG">
            </div>
          </figure>
          <figure>
            <figcaption>oriha-plantuml</figcaption>
            <div class="svg-frame">
              <img loading="lazy" src="{rendered_svg}" alt="{name} oriha-plantuml SVG">
            </div>
          </figure>
        </div>{error_html}
        <details>
          <summary>Source</summary>
          <pre>{source}</pre>
        </details>
      </section>"""


if __name__ == "__main__":
    sys.exit(main())
