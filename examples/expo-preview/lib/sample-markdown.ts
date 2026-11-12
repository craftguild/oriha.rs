export const sampleMarkdown = `# Markdown + GFM Complete Preview

This document is the Expo preview stress sample. It intentionally mixes
CommonMark blocks, GFM extensions, renderer callouts, syntax highlighting, raw
HTML, PlantUML, Mermaid, and upcoming LaTeX syntax so the mobile preview
exercises the same paths as a dense real-world technical document.

:::note Preview target
The sample should be long enough to test scrolling, source mapping, typography,
table wrapping, syntax themes, callout chrome, and PlantUML text measurement.
:::

## Contents

- [Inline markup](#inline-markup)
- [Headings](#headings)
- [Paragraphs and breaks](#paragraphs-and-breaks)
- [Images](#images)
- [Lists and task lists](#lists-and-task-lists)
- [Block quotes and alerts](#block-quotes-and-alerts)
- [Tables](#tables)
- [Code blocks](#code-blocks)
- [Raw HTML](#raw-html)
- [PlantUML](#plantuml)
- [Mermaid](#mermaid)
- [LaTeX](#latex)
- [Footnotes](#footnotes)
- [Reference definitions](#reference-definitions)

## Inline Markup

Inline coverage includes *emphasis*, _underscore emphasis_, **strong text**,
__underscore strong text__, ***strong emphasis***, ~~GFM strikethrough~~,
\`inline code\`, escaped punctuation like \\*literal stars\\*, an entity
&amp;, smart-looking punctuation that should stay literal, and mixed English /
日本語 / numbers 12345.

Links include [an inline link](https://example.com/docs "Example docs"),
[a reference link][engine-docs], [a collapsed reference][collapsed], and
[a shortcut reference]. Autolinks include <https://example.com/path?q=gfm>,
<user@example.com>, plus GFM bare autolinks: https://example.org,
www.example.net, and support@example.org.

An inline image exercises image syntax without network access: ![Inline SVG badge](data:image/svg+xml,%3Csvg%20xmlns%3D%27http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%27%20width%3D%27220%27%20height%3D%2764%27%20viewBox%3D%270%200%20220%2064%27%3E%3Crect%20width%3D%27220%27%20height%3D%2764%27%20rx%3D%278%27%20fill%3D%27%23eef2ff%27%2F%3E%3Ctext%20x%3D%2716%27%20y%3D%2739%27%20font-size%3D%2720%27%20fill%3D%27%23312e81%27%3EMarkdown%20Preview%3C%2Ftext%3E%3C%2Fsvg%3E "Inline SVG data image")

## Images

The next images use remote Unsplash image URLs so the preview can exercise
real image loading, intrinsic dimensions, alt text, and narrow-screen scaling.

![Wide white sand dunes under a pale sky](https://images.unsplash.com/photo-1765498069015-aaf7e6bc1759?ixlib=rb-4.1.0&auto=format&fit=crop&w=1600&h=900&q=80 "Wide Unsplash sample: white dunes")

Source: [white sand dunes by Royce Fonseca](https://unsplash.com/photos/bsWh6w4nQio).

![Portrait pink magnolia blossoms against a pale sky](https://images.unsplash.com/photo-1775312766921-61f840e3d6f8?ixlib=rb-4.1.0&auto=format&fit=crop&w=900&h=1350&q=80 "Portrait Unsplash sample: pink magnolia")

Source: [pink magnolia blossoms by Natalia Grela](https://unsplash.com/photos/-6XD-t-jTKI).

## Headings

# H1 ATX duplicate check

## H2 ATX duplicate check

### H3 ATX With Closing Hashes ###

#### H4 ATX

##### H5 ATX

###### H6 ATX

Setext Level 1
==============

Setext Level 2
--------------

## Paragraphs And Breaks

This paragraph contains a soft
line break that should remain part of the same paragraph.

This line ends with two spaces.  
The next line should render after a hard break.

Backslash hard break follows.\\
The next line should also render after a hard break.

Thematic breaks:

---

***

___

## Lists And Task Lists

- Tight unordered item
- Tight unordered item with \`inline code\`
- Tight unordered item with nested ordered content:
  1. First nested ordered item
  2. Second nested ordered item
     - Deep unordered child
     - Deep unordered child with [reference link][engine-docs]

- Loose unordered item with a paragraph.

  The continuation paragraph should stay inside the item and keep its own
  spacing.

- Final loose item.

1. Ordered list item one
2. Ordered list item two
   1. Nested ordered item
   2. Nested ordered item with a task list:
      - [x] Completed nested task
      - [ ] Pending nested task
      - [X] Uppercase checked marker
3. Ordered list item three

42. Ordered list with explicit start
43. The second item should keep numbering semantics.

- [x] GFM task list item
- [ ] GFM incomplete task item
- [x] Task item with **bold**, ~~struck~~, and \`code\`

## Block Quotes And Alerts

> A normal block quote with *inline emphasis*.
>
> - quoted unordered item
> - quoted item with a nested quote:
>
>   > Nested quote body.

> Lazy continuation starts here
and this line should remain inside the quote.

> [!NOTE]
>
> GitHub / MDN-style alert syntax should render as a callout when supported.

> [!TIP] Custom title
> Alerts may include a custom title, a table, and a task list.
>
> | Item | Done |
> | ---- | :--: |
> | Measure text | yes |
> | Inspect output | yes |
>
> - [x] Nested task inside alert

:::warning Fenced callout
Fenced callouts are an engine extension, but this sample keeps them near the
GFM alert cases because they share renderer chrome.

1. Ordered item inside a callout
2. Another item with \`inline code\`
:::

:::custom-kind Custom extension kind
Unknown fenced kinds should still render predictably.
:::

## Tables

| Feature | Markdown | GFM | Notes |
| :------ | :------: | --: | ----- |
| Paragraphs | yes | yes | soft and hard breaks |
| Tables | no | yes | alignment markers are included |
| Task lists | no | yes | checked and unchecked items |
| Strikethrough | no | yes | ~~text~~ |
| Autolinks | partial | yes | bare URLs and email addresses |
| Escaped pipe | yes | yes | a \\| b |

| Wide content | Result |
| ------------ | ------ |
| Long text | This row intentionally contains a long sentence so the preview can show wrapping behavior inside table cells on narrow mobile screens. |
| Inline markup | **strong**, *emphasis*, \`code\`, [link][engine-docs] |

## Code Blocks

Indented code:

    let indented = true;
    println!("indented code block");

Fenced TypeScript:

~~~typescript
type RenderState = 'idle' | 'measuring' | 'rendered';

export function nextState(hasPlantUml: boolean): RenderState {
  if (hasPlantUml) {
    return 'measuring';
  }
  return 'rendered';
}
~~~

Fenced Rust:

~~~rust
#[derive(Debug, Clone, PartialEq, Eq)]
struct DocumentBlock {
    kind: &'static str,
    line: usize,
}

fn block_count(blocks: &[DocumentBlock]) -> usize {
    blocks.len()
}
~~~

Fenced JSON:

~~~json
{
  "mode": "styledHtml",
  "features": {
    "plantuml": true,
    "tables": true,
    "taskLists": true
  }
}
~~~

Fenced diff:

~~~diff
- Google Fonts request path
+ Local font installer path
+ WebView uses installed font metrics
~~~

Fenced Markdown:

~~~markdown
## Nested Markdown Example

- [x] task
- [ ] another task

| A | B |
| - | - |
| 1 | 2 |
~~~

## Raw HTML

Inline HTML: <span data-inline="true">inline span</span>.

<details>
<summary>Raw HTML disclosure</summary>
Raw HTML block content should remain stable under the renderer policy.
</details>

<div data-sample="raw-html">
  <p>Block HTML with nested tags and attributes.</p>
</div>

## PlantUML

~~~plantuml
@startuml
title Complex Markdown Preview / 日本語 mixed text

actor "Reader\\n利用者" as Reader
participant "Expo App\\nReact Native" as App
participant "Rust Core\\nMarkdown" as Core
participant "Hidden WebView\\nText Metrics" as Metrics

Reader -> App : Open preview
App -> Core : prepare_markdown_layout
Core -> Metrics : measure PlantUML text runs

note right of Metrics
  <back:lavender>background highlight</back> と <w:#FF33FF>波線 mixed text</w>
  English, 日本語, numbers 12345, and <size:18>larger text</size>
  <b>bold</b> / <i>italic</i> / <monospaced>monospace</monospaced>
end note

Metrics --> Core : width + run rects
Core --> App : final HTML + SVG
App --> Reader : rendered document

note over App, Core
  The note width and decoration range should use local WebView metrics.
  <back:#FFF2A8>local font measurement</back> / <w:#0088CC>precise wave</w>
end note
@enduml
~~~

## Mermaid

Flowchart example:

~~~mermaid
flowchart TD
  Start([Open Markdown]) --> Parse[Parse blocks and inline spans]
  Parse --> HasDiagram{Diagram fence?}
  HasDiagram -- Mermaid --> RenderMermaid[Render Mermaid SVG]
  HasDiagram -- PlantUML --> RenderPlantUml[Render PlantUML SVG]
  HasDiagram -- No --> RenderHtml[Render semantic HTML]
  RenderMermaid --> Preview[Preview WebView]
  RenderPlantUml --> Preview
  RenderHtml --> Preview
~~~

Sequence diagram example:

~~~mermaid
sequenceDiagram
  autonumber
  participant User
  participant App as Expo App
  participant Core as Rust Core
  participant WebView as Hidden WebView
  User->>App: Open document
  App->>Core: prepareMarkdownLayout(markdown)
  Core-->>App: measurement requests
  App->>WebView: measure text batch
  WebView-->>App: width / height results
  App->>Core: renderPreparedMarkdown(metrics)
  Core-->>User: final HTML
~~~

State diagram example:

~~~mermaid
stateDiagram-v2
  [*] --> Editing
  Editing --> Measuring: contains measured diagram
  Editing --> Rendered: no measurements required
  Measuring --> Rendered: metrics returned
  Rendered --> Editing: markdown changed
  Rendered --> [*]
~~~

## LaTeX

Inline LaTeX examples should eventually render inside paragraphs: $E = mc^2$,
$\\alpha + \\beta = \\gamma$, $\\sqrt{x^2 + 1}$, and
$\\sum_{i=1}^{n} i = \\frac{n(n+1)}{2}$.

Display LaTeX with dollar fences:

$$
\\int_0^1 x^2\\,dx = \\frac{1}{3}
$$

$$
\\begin{aligned}
f(x) &= ax^2 + bx + c \\\\
f'(x) &= 2ax + b
\\end{aligned}
$$

Fenced LaTeX block:

~~~latex
\\begin{bmatrix}
1 & 0 & 0 \\\\
0 & \\cos\\theta & -\\sin\\theta \\\\
0 & \\sin\\theta & \\cos\\theta
\\end{bmatrix}
~~~

~~~latex
\\begin{align*}
x^2 + y^2 &= 1 \\\\
y &= \\sqrt{1 - x^2}
\\end{align*}
~~~

## Footnotes

Footnotes should round-trip from inline references to a collected note section.[^local-fonts]
Multiple references can point to the same note without duplicating the definition.[^local-fonts]

[^local-fonts]: This footnote confirms GFM-style footnote references and definitions render in the Expo preview.

## Reference Definitions

[engine-docs]: https://example.com/markdown-engine "Markdown engine docs"
[collapsed]: https://example.com/collapsed
[shortcut reference]: https://example.com/shortcut
`;
