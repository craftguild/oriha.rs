use oriha_plantuml::__private::{
    layout_sequence, parse_sequence, render_sequence_svg, LayoutOptions, LineStyle, PlantUmlTheme,
    SequenceDiagram, SequenceEvent, SequenceMessage, SequenceSemanticEvent,
};
use std::{collections::BTreeSet, fs, path::Path};

#[derive(Debug)]
struct FixtureGroup {
    name: &'static str,
    active: &'static [SequenceFixture],
    future: &'static [SequenceFixture],
}

#[derive(Debug)]
struct SequenceFixture {
    name: &'static str,
    source: &'static str,
    expected_participants: &'static [&'static str],
    expected_messages: &'static [ExpectedMessage],
    expected_timeline_markers: &'static [&'static str],
    expected_svg_markers: &'static [&'static str],
}

#[derive(Debug)]
struct ExpectedMessage {
    from: &'static str,
    to: &'static str,
    label: &'static str,
    line_style: LineStyle,
}

#[derive(Debug)]
struct OfficialSequenceFixture {
    source: String,
    golden: String,
}

#[derive(Debug)]
struct OfficialSequenceFixtureCase {
    name: String,
    fixture: OfficialSequenceFixture,
}

#[derive(Debug, Default)]
struct SoftFailures {
    failures: Vec<String>,
}

impl SoftFailures {
    fn push(&mut self, fixture_name: &str, message: impl Into<String>) {
        self.failures
            .push(format!("{fixture_name}: {}", message.into()));
    }

    fn push_svg_snapshot_mismatch(&mut self, fixture_name: &str, expected: &str, actual: &str) {
        self.push(
            fixture_name,
            format!(
                "rendered SVG should match checked-in snapshot byte-for-byte; {}",
                svg_diff_summary(expected, actual)
            ),
        );
    }

    fn finish(self, context: &str) {
        assert!(
            self.failures.is_empty(),
            "{context} failed for {} case(s):\n{}",
            self.failures.len(),
            self.failures.join("\n")
        );
    }
}

const EXPECTED_SUPPORTED_OFFICIAL_SEQUENCE_FIXTURE_COUNT: usize = 87;
const EXPECTED_UNSUPPORTED_OFFICIAL_SEQUENCE_FIXTURES: &[&str] = &[];

const PARTICIPANT_ACTIVE: &[SequenceFixture] = &[SequenceFixture {
    name: "implicit_participants_from_messages",
    source: r#"@startuml
Alice -> Bob: Authentication Request
Bob --> Alice: Authentication Response
Alice -> Bob: Another authentication Request
Alice <-- Bob: Another authentication Response
@enduml
"#,
    expected_participants: &["Alice", "Bob"],
    expected_messages: &[
        ExpectedMessage {
            from: "Alice",
            to: "Bob",
            label: "Authentication Request",
            line_style: LineStyle::Solid,
        },
        ExpectedMessage {
            from: "Bob",
            to: "Alice",
            label: "Authentication Response",
            line_style: LineStyle::Dashed,
        },
        ExpectedMessage {
            from: "Alice",
            to: "Bob",
            label: "Another authentication Request",
            line_style: LineStyle::Solid,
        },
        ExpectedMessage {
            from: "Bob",
            to: "Alice",
            label: "Another authentication Response",
            line_style: LineStyle::Dashed,
        },
    ],
    expected_timeline_markers: &[],
    expected_svg_markers: &[
        "data-diagram-type=\"SEQUENCE\"",
        "class=\"participant\"",
        "class=\"message\"",
        "Alice",
        "Bob",
        "Authentication Request",
        "Authentication Response",
    ],
}];

const PARTICIPANT_FUTURE: &[SequenceFixture] = &[SequenceFixture {
    name: "participant_declarations_aliases_and_kinds",
    source: r#"@startuml
participant Participant as Foo
actor Actor as Foo1
boundary Boundary as Foo2
control Control as Foo3
entity Entity as Foo4
database Database as Foo5
collections Collections as Foo6
queue Queue as Foo7
Foo -> Foo1 : To actor
Foo -> Foo2 : To boundary
Foo -> Foo3 : To control
Foo -> Foo4 : To entity
Foo -> Foo5 : To database
Foo -> Foo6 : To collections
Foo -> Foo7: To queue
@enduml
"#,
    expected_participants: &[
        "Foo", "Foo1", "Foo2", "Foo3", "Foo4", "Foo5", "Foo6", "Foo7",
    ],
    expected_messages: &[
        ExpectedMessage {
            from: "Foo",
            to: "Foo1",
            label: "To actor",
            line_style: LineStyle::Solid,
        },
        ExpectedMessage {
            from: "Foo",
            to: "Foo2",
            label: "To boundary",
            line_style: LineStyle::Solid,
        },
        ExpectedMessage {
            from: "Foo",
            to: "Foo3",
            label: "To control",
            line_style: LineStyle::Solid,
        },
        ExpectedMessage {
            from: "Foo",
            to: "Foo4",
            label: "To entity",
            line_style: LineStyle::Solid,
        },
        ExpectedMessage {
            from: "Foo",
            to: "Foo5",
            label: "To database",
            line_style: LineStyle::Solid,
        },
        ExpectedMessage {
            from: "Foo",
            to: "Foo6",
            label: "To collections",
            line_style: LineStyle::Solid,
        },
        ExpectedMessage {
            from: "Foo",
            to: "Foo7",
            label: "To queue",
            line_style: LineStyle::Solid,
        },
    ],
    expected_timeline_markers: &[],
    expected_svg_markers: &[
        "Participant",
        "Actor",
        "Boundary",
        "Control",
        "Entity",
        "Database",
        "Collections",
        "Queue",
    ],
}];

const ARROW_ACTIVE: &[SequenceFixture] = &[SequenceFixture {
    name: "supported_forward_reverse_and_dashed_arrows",
    source: r#"@startuml
Alice -> Bob: solid request
Bob --> Alice: dashed response
Alice <- Bob: reverse solid response
Alice <-- Bob: reverse dashed response
@enduml
"#,
    expected_participants: &["Alice", "Bob"],
    expected_messages: &[
        ExpectedMessage {
            from: "Alice",
            to: "Bob",
            label: "solid request",
            line_style: LineStyle::Solid,
        },
        ExpectedMessage {
            from: "Bob",
            to: "Alice",
            label: "dashed response",
            line_style: LineStyle::Dashed,
        },
        ExpectedMessage {
            from: "Bob",
            to: "Alice",
            label: "reverse solid response",
            line_style: LineStyle::Solid,
        },
        ExpectedMessage {
            from: "Bob",
            to: "Alice",
            label: "reverse dashed response",
            line_style: LineStyle::Dashed,
        },
    ],
    expected_timeline_markers: &[],
    expected_svg_markers: &[
        "solid request",
        "dashed response",
        "reverse solid response",
        "reverse dashed response",
        "data-line-style=\"solid\"",
        "data-line-style=\"dashed\"",
    ],
}];

const ARROW_FUTURE: &[SequenceFixture] = &[SequenceFixture {
    name: "arrow_style_variants",
    source: r#"@startuml
Bob ->x Alice
Bob -> Alice
Bob ->> Alice
Bob -\ Alice
Bob \\- Alice
Bob //-- Alice
Bob ->o Alice
Bob o\-- Alice
Bob <-> Alice
Bob <->o Alice
@enduml
"#,
    expected_participants: &["Bob", "Alice"],
    expected_messages: &[
        ExpectedMessage {
            from: "Bob",
            to: "Alice",
            label: "",
            line_style: LineStyle::Solid,
        },
        ExpectedMessage {
            from: "Bob",
            to: "Alice",
            label: "",
            line_style: LineStyle::Solid,
        },
        ExpectedMessage {
            from: "Bob",
            to: "Alice",
            label: "",
            line_style: LineStyle::Solid,
        },
        ExpectedMessage {
            from: "Bob",
            to: "Alice",
            label: "",
            line_style: LineStyle::Solid,
        },
        ExpectedMessage {
            from: "Alice",
            to: "Bob",
            label: "",
            line_style: LineStyle::Solid,
        },
        ExpectedMessage {
            from: "Alice",
            to: "Bob",
            label: "",
            line_style: LineStyle::Dashed,
        },
        ExpectedMessage {
            from: "Bob",
            to: "Alice",
            label: "",
            line_style: LineStyle::Solid,
        },
        ExpectedMessage {
            from: "Alice",
            to: "Bob",
            label: "",
            line_style: LineStyle::Dashed,
        },
        ExpectedMessage {
            from: "Bob",
            to: "Alice",
            label: "",
            line_style: LineStyle::Solid,
        },
        ExpectedMessage {
            from: "Bob",
            to: "Alice",
            label: "",
            line_style: LineStyle::Solid,
        },
    ],
    expected_timeline_markers: &[],
    expected_svg_markers: &[
        "lost-message",
        "thin-arrow",
        "half-arrow",
        "open-circle",
        "bidirectional",
    ],
}];

const NOTES_GROUPS_ACTIVE: &[SequenceFixture] = &[];

const NOTES_GROUPS_FUTURE: &[SequenceFixture] = &[SequenceFixture {
    name: "multiline_notes_alt_else_and_loop",
    source: r#"@startuml
Alice -> Bob: Authentication Request
note left of Alice
  This is a first note.
  It spans multiple lines.
end note
alt successful case
    Bob -> Alice: Authentication Accepted
else some kind of failure
    Bob -> Alice: Authentication Failure
end
loop 1000 times
    Alice -> Bob: DNS Attack
end
@enduml
"#,
    expected_participants: &["Alice", "Bob"],
    expected_messages: &[
        ExpectedMessage {
            from: "Alice",
            to: "Bob",
            label: "Authentication Request",
            line_style: LineStyle::Solid,
        },
        ExpectedMessage {
            from: "Bob",
            to: "Alice",
            label: "Authentication Accepted",
            line_style: LineStyle::Solid,
        },
        ExpectedMessage {
            from: "Bob",
            to: "Alice",
            label: "Authentication Failure",
            line_style: LineStyle::Solid,
        },
        ExpectedMessage {
            from: "Alice",
            to: "Bob",
            label: "DNS Attack",
            line_style: LineStyle::Solid,
        },
    ],
    expected_timeline_markers: &[
        "note left of Alice",
        "alt successful case",
        "else some kind of failure",
        "loop 1000 times",
    ],
    expected_svg_markers: &[
        "This is a first note.",
        "It spans multiple lines.",
        "successful case",
        "some kind of failure",
        "1000 times",
    ],
}];

const CONTROL_ACTIVE: &[SequenceFixture] = &[];

const CONTROL_FUTURE: &[SequenceFixture] = &[SequenceFixture {
    name: "divider_delay_activation_return_create_destroy",
    source: r#"@startuml
participant User
== Initialization ==
User -> A: DoWork
activate A
A -> B: << createRequest >>
activate B
...5 minutes later...
create Other
B -> Other: new
B --> A: RequestCreated
deactivate B
A -> User: Done
return bye
destroy A
@enduml
"#,
    expected_participants: &["User", "A", "B", "Other"],
    expected_messages: &[
        ExpectedMessage {
            from: "User",
            to: "A",
            label: "DoWork",
            line_style: LineStyle::Solid,
        },
        ExpectedMessage {
            from: "A",
            to: "B",
            label: "<< createRequest >>",
            line_style: LineStyle::Solid,
        },
        ExpectedMessage {
            from: "B",
            to: "Other",
            label: "new",
            line_style: LineStyle::Solid,
        },
        ExpectedMessage {
            from: "B",
            to: "A",
            label: "RequestCreated",
            line_style: LineStyle::Dashed,
        },
        ExpectedMessage {
            from: "A",
            to: "User",
            label: "Done",
            line_style: LineStyle::Solid,
        },
    ],
    expected_timeline_markers: &[
        "== Initialization ==",
        "activate A",
        "...5 minutes later...",
        "create Other",
        "return bye",
        "destroy A",
    ],
    expected_svg_markers: &[
        "Initialization",
        "5 minutes later",
        "createRequest",
        "RequestCreated",
        "bye",
    ],
}];

const STYLING_ACTIVE: &[SequenceFixture] = &[];

const STYLING_FUTURE: &[SequenceFixture] = &[SequenceFixture {
    name: "skinparam_box_and_teoz_pragma",
    source: r#"@startuml
!pragma teoz true
skinparam sequenceMessageAlign right
skinparam actorStyle awesome
box "Internal Service" #LightBlue
participant Bob
participant Alice
end box
participant Other
Bob -> Alice : hello
Alice -> Other : hello
@enduml
"#,
    expected_participants: &["Bob", "Alice", "Other"],
    expected_messages: &[
        ExpectedMessage {
            from: "Bob",
            to: "Alice",
            label: "hello",
            line_style: LineStyle::Solid,
        },
        ExpectedMessage {
            from: "Alice",
            to: "Other",
            label: "hello",
            line_style: LineStyle::Solid,
        },
    ],
    expected_timeline_markers: &[
        "!pragma teoz true",
        "skinparam sequenceMessageAlign right",
        "skinparam actorStyle awesome",
        "box \"Internal Service\" #LightBlue",
    ],
    expected_svg_markers: &["Internal Service", "hello"],
}];

const AUTONUMBER_PAGE_HEADER_FOOTER_ACTIVE: &[SequenceFixture] = &[
    SequenceFixture {
        name: "page_title_basic",
        source: r#"@startuml
title Example Title
Alice -> Bob : message 1
Alice -> Bob : message 2
@enduml
"#,
        expected_participants: &["Alice", "Bob"],
        expected_messages: &[
            ExpectedMessage {
                from: "Alice",
                to: "Bob",
                label: "message 1",
                line_style: LineStyle::Solid,
            },
            ExpectedMessage {
                from: "Alice",
                to: "Bob",
                label: "message 2",
                line_style: LineStyle::Solid,
            },
        ],
        expected_timeline_markers: &["title Example Title"],
        expected_svg_markers: &["data-diagram-type=\"SEQUENCE\"", "message 1", "message 2"],
    },
    SequenceFixture {
        name: "page_header_footer_variables",
        source: r#"@startuml
header Page Header
footer Page %page% of %lastpage%
title Example Title
Alice -> Bob : message 1
Bob -> Alice : message 2
@enduml
"#,
        expected_participants: &["Alice", "Bob"],
        expected_messages: &[
            ExpectedMessage {
                from: "Alice",
                to: "Bob",
                label: "message 1",
                line_style: LineStyle::Solid,
            },
            ExpectedMessage {
                from: "Bob",
                to: "Alice",
                label: "message 2",
                line_style: LineStyle::Solid,
            },
        ],
        expected_timeline_markers: &[
            "header Page Header",
            "footer Page %page% of %lastpage%",
            "title Example Title",
        ],
        expected_svg_markers: &["Page Header", "Page 1 of 1", "Example Title"],
    },
];

const AUTONUMBER_PAGE_HEADER_FOOTER_FUTURE: &[SequenceFixture] = &[];

const REF_DELAY_SPACE_ACTIVE: &[SequenceFixture] = &[
    SequenceFixture {
        name: "delay_with_optional_message_text",
        source: r#"@startuml
Alice -> Bob: Authentication Request
...
Bob --> Alice: Authentication Response
...5 minutes later...
Bob --> Alice: Good Bye !
@enduml
"#,
        expected_participants: &["Alice", "Bob"],
        expected_messages: &[
            ExpectedMessage {
                from: "Alice",
                to: "Bob",
                label: "Authentication Request",
                line_style: LineStyle::Solid,
            },
            ExpectedMessage {
                from: "Bob",
                to: "Alice",
                label: "Authentication Response",
                line_style: LineStyle::Dashed,
            },
            ExpectedMessage {
                from: "Bob",
                to: "Alice",
                label: "Good Bye !",
                line_style: LineStyle::Dashed,
            },
        ],
        expected_timeline_markers: &["...", "...5 minutes later..."],
        expected_svg_markers: &["data-event=\"delay\"", "5 minutes later", "Good Bye !"],
    },
    SequenceFixture {
        name: "reference_over_participants",
        source: r#"@startuml
participant Alice
actor Bob

ref over Alice, Bob : init

Alice -> Bob : hello
ref over Bob
  This can be on
  several lines
end ref
@enduml
"#,
        expected_participants: &["Alice", "Bob"],
        expected_messages: &[ExpectedMessage {
            from: "Alice",
            to: "Bob",
            label: "hello",
            line_style: LineStyle::Solid,
        }],
        expected_timeline_markers: &["ref over Alice, Bob : init", "ref over Bob"],
        expected_svg_markers: &["init", "This can be on", "several lines", "hello"],
    },
    SequenceFixture {
        name: "space_separators",
        source: r#"@startuml
Alice -> Bob: message 1
Bob --> Alice: ok
|||
Alice -> Bob: message 2
Bob --> Alice: ok
||45||
Alice -> Bob: message 3
Bob --> Alice: ok
@enduml
"#,
        expected_participants: &["Alice", "Bob"],
        expected_messages: &[
            ExpectedMessage {
                from: "Alice",
                to: "Bob",
                label: "message 1",
                line_style: LineStyle::Solid,
            },
            ExpectedMessage {
                from: "Bob",
                to: "Alice",
                label: "ok",
                line_style: LineStyle::Dashed,
            },
            ExpectedMessage {
                from: "Alice",
                to: "Bob",
                label: "message 2",
                line_style: LineStyle::Solid,
            },
            ExpectedMessage {
                from: "Bob",
                to: "Alice",
                label: "ok",
                line_style: LineStyle::Dashed,
            },
            ExpectedMessage {
                from: "Alice",
                to: "Bob",
                label: "message 3",
                line_style: LineStyle::Solid,
            },
            ExpectedMessage {
                from: "Bob",
                to: "Alice",
                label: "ok",
                line_style: LineStyle::Dashed,
            },
        ],
        expected_timeline_markers: &["|||", "||45||"],
        expected_svg_markers: &["message 1", "message 2", "message 3"],
    },
];

const REF_DELAY_SPACE_FUTURE: &[SequenceFixture] = &[];

const ACTIVATION_SHORTCUTS_ACTIVE: &[SequenceFixture] = &[];

const ACTIVATION_SHORTCUTS_FUTURE: &[SequenceFixture] = &[];

const INCOMING_OUTGOING_ACTIVE: &[SequenceFixture] = &[];

const INCOMING_OUTGOING_FUTURE: &[SequenceFixture] = &[];

const STEREOTYPES_SPOTS_ACTIVE: &[SequenceFixture] = &[];

const STEREOTYPES_SPOTS_FUTURE: &[SequenceFixture] = &[];

const TEXT_MARKUP_ACTIVE: &[SequenceFixture] = &[];

const TEXT_MARKUP_FUTURE: &[SequenceFixture] = &[];

const MAINFRAME_ACTIVE: &[SequenceFixture] = &[SequenceFixture {
    name: "mainframe_title",
    source: r#"@startuml
mainframe This is a **mainframe**
Alice->Bob : Hello
@enduml
"#,
    expected_participants: &["Alice", "Bob"],
    expected_messages: &[ExpectedMessage {
        from: "Alice",
        to: "Bob",
        label: "Hello",
        line_style: LineStyle::Solid,
    }],
    expected_timeline_markers: &["mainframe This is a **mainframe**"],
    expected_svg_markers: &["This is a **mainframe**", "Hello"],
}];

const MAINFRAME_FUTURE: &[SequenceFixture] = &[];

const TEOZ_PARALLEL_ACTIVE: &[SequenceFixture] = &[SequenceFixture {
    name: "parallel_messages_with_teoz",
    source: r#"@startuml
!pragma teoz true
Alice -> Bob : hello
& Bob -> Charlie : hi
@enduml
"#,
    expected_participants: &["Alice", "Bob", "Charlie"],
    expected_messages: &[
        ExpectedMessage {
            from: "Alice",
            to: "Bob",
            label: "hello",
            line_style: LineStyle::Solid,
        },
        ExpectedMessage {
            from: "Bob",
            to: "Charlie",
            label: "hi",
            line_style: LineStyle::Solid,
        },
    ],
    expected_timeline_markers: &["!pragma teoz true", "& Bob -> Charlie : hi"],
    expected_svg_markers: &["hello", "hi", "parallel"],
}];

const TEOZ_PARALLEL_FUTURE: &[SequenceFixture] = &[];

const SKINPARAM_STYLE_ACTIVE: &[SequenceFixture] = &[SequenceFixture {
    name: "skinparam_style_strictuml_with_delay",
    source: r#"@startuml
skinparam style strictuml
Bob -> Alice : hello
...
Alice -> Bob : ok
@enduml
"#,
    expected_participants: &["Bob", "Alice"],
    expected_messages: &[
        ExpectedMessage {
            from: "Bob",
            to: "Alice",
            label: "hello",
            line_style: LineStyle::Solid,
        },
        ExpectedMessage {
            from: "Alice",
            to: "Bob",
            label: "ok",
            line_style: LineStyle::Solid,
        },
    ],
    expected_timeline_markers: &["skinparam style strictuml", "..."],
    expected_svg_markers: &["hello", "ok", "data-event=\"delay\""],
}];

const SKINPARAM_STYLE_FUTURE: &[SequenceFixture] = &[];

const FIXTURE_GROUPS: &[FixtureGroup] = &[
    FixtureGroup {
        name: "participants",
        active: PARTICIPANT_ACTIVE,
        future: PARTICIPANT_FUTURE,
    },
    FixtureGroup {
        name: "arrows",
        active: ARROW_ACTIVE,
        future: ARROW_FUTURE,
    },
    FixtureGroup {
        name: "notes_groups",
        active: NOTES_GROUPS_ACTIVE,
        future: NOTES_GROUPS_FUTURE,
    },
    FixtureGroup {
        name: "control",
        active: CONTROL_ACTIVE,
        future: CONTROL_FUTURE,
    },
    FixtureGroup {
        name: "styling",
        active: STYLING_ACTIVE,
        future: STYLING_FUTURE,
    },
    FixtureGroup {
        name: "autonumber_page_header_footer",
        active: AUTONUMBER_PAGE_HEADER_FOOTER_ACTIVE,
        future: AUTONUMBER_PAGE_HEADER_FOOTER_FUTURE,
    },
    FixtureGroup {
        name: "ref_delay_space",
        active: REF_DELAY_SPACE_ACTIVE,
        future: REF_DELAY_SPACE_FUTURE,
    },
    FixtureGroup {
        name: "activation_shortcuts",
        active: ACTIVATION_SHORTCUTS_ACTIVE,
        future: ACTIVATION_SHORTCUTS_FUTURE,
    },
    FixtureGroup {
        name: "incoming_outgoing",
        active: INCOMING_OUTGOING_ACTIVE,
        future: INCOMING_OUTGOING_FUTURE,
    },
    FixtureGroup {
        name: "stereotypes_spots",
        active: STEREOTYPES_SPOTS_ACTIVE,
        future: STEREOTYPES_SPOTS_FUTURE,
    },
    FixtureGroup {
        name: "text_markup",
        active: TEXT_MARKUP_ACTIVE,
        future: TEXT_MARKUP_FUTURE,
    },
    FixtureGroup {
        name: "mainframe",
        active: MAINFRAME_ACTIVE,
        future: MAINFRAME_FUTURE,
    },
    FixtureGroup {
        name: "teoz_parallel",
        active: TEOZ_PARALLEL_ACTIVE,
        future: TEOZ_PARALLEL_FUTURE,
    },
    FixtureGroup {
        name: "skinparam_style",
        active: SKINPARAM_STYLE_ACTIVE,
        future: SKINPARAM_STYLE_FUTURE,
    },
];

#[test]
fn official_sequence_fixture_semantic_inputs_are_well_formed() {
    let mut failures = SoftFailures::default();

    for case in official_sequence_fixture_cases() {
        if !case.fixture.source.starts_with("@startuml\n") {
            failures.push(&case.name, "source should start with @startuml");
        }
        if !case.fixture.source.trim_end().ends_with("@enduml") {
            failures.push(&case.name, "source should end with @enduml");
        }
        if !case.fixture.golden.contains("<svg") || !case.fixture.golden.contains("</svg>") {
            failures.push(&case.name, "golden should contain SVG");
        }

        let golden_summary = match try_svg_semantic_summary(&case.fixture.golden) {
            Ok(summary) => summary,
            Err(error) => {
                failures.push(
                    &case.name,
                    format!("golden SVG should parse as XML: {error}"),
                );
                continue;
            }
        };

        if golden_summary.visible_text.is_empty() {
            failures.push(&case.name, "golden SVG should expose visible text");
        }
        if !golden_summary.participants.is_empty()
            && golden_summary.lifeline_count == 0
            && golden_summary.message_count == 0
        {
            failures.push(
                &case.name,
                "golden SVG summary should expose participant-related structure",
            );
        }
    }

    failures.finish("official sequence fixture semantic input checks");
}

#[test]
fn official_sequence_fixture_source_structure_support_is_reported() {
    let mut failures = SoftFailures::default();
    let mut supported = Vec::new();
    let mut unsupported = Vec::new();
    let mut unsupported_names = Vec::new();
    let cases = official_sequence_fixture_cases();
    let total = cases.len();

    for case in cases {
        match parse_sequence(&case.fixture.source) {
            Ok(diagram) => {
                if diagram.events.is_empty() {
                    failures.push(&case.name, "parsed diagram should retain source events");
                }
                for failure in parsed_diagram_source_message_label_failures(&case, &diagram) {
                    failures.push(&case.name, failure);
                }
                supported.push(case.name);
            }
            Err(error) => {
                unsupported_names.push(case.name.clone());
                unsupported.push(format!("{}: {error}", case.name));
            }
        }
    }

    eprintln!(
        "official sequence fixture parser support: {} supported, {} unsupported",
        supported.len(),
        unsupported.len()
    );
    if !unsupported.is_empty() {
        eprintln!(
            "unsupported official sequence fixtures:\n{}",
            unsupported.join("\n")
        );
    }

    if supported.is_empty() {
        failures.push(
            "official sequence corpus",
            "at least one fixture should parse",
        );
    }
    if supported.len() + unsupported.len() != total {
        failures.push(
            "official sequence corpus",
            format!(
                "all fixtures should be evaluated; evaluated {}, expected {total}",
                supported.len() + unsupported.len()
            ),
        );
    }
    if supported.len() != EXPECTED_SUPPORTED_OFFICIAL_SEQUENCE_FIXTURE_COUNT {
        failures.push(
            "official sequence corpus",
            format!(
                "parser support count changed: got {}, expected {}; update the baseline if intentional",
                supported.len(),
                EXPECTED_SUPPORTED_OFFICIAL_SEQUENCE_FIXTURE_COUNT
            ),
        );
    }
    if unsupported_names != EXPECTED_UNSUPPORTED_OFFICIAL_SEQUENCE_FIXTURES {
        failures.push(
            "official sequence corpus",
            format!(
                "unsupported fixture set changed: got {:?}, expected {:?}; update the baseline if intentional",
                unsupported_names,
                EXPECTED_UNSUPPORTED_OFFICIAL_SEQUENCE_FIXTURES
            ),
        );
    }

    failures.finish("official sequence fixture source structure support");
}

#[test]
fn official_sequence_fixture_golden_svg_semantic_summary_tracks_source_text() {
    let mut failures = SoftFailures::default();

    for case in official_sequence_fixture_cases() {
        let source_text = source_semantic_text_summary(&case.fixture.source);
        let golden_text = match try_svg_semantic_summary(&case.fixture.golden) {
            Ok(summary) => summary,
            Err(error) => {
                failures.push(
                    &case.name,
                    format!("golden SVG should parse as XML: {error}"),
                );
                continue;
            }
        };
        let confirmed_texts = confirmed_source_texts(&source_text, &golden_text);

        if golden_text.visible_text.is_empty() {
            failures.push(&case.name, "golden SVG should yield visible text");
        }
        if !source_text.candidate_texts.is_empty() && confirmed_texts.is_empty() {
            failures.push(
                &case.name,
                format!(
                    "golden SVG should confirm at least one source semantic text; source candidates: {:?}; visible text: {:?}",
                    source_text.candidate_texts,
                    golden_text.visible_text
                ),
            );
        }
    }

    failures.finish("official sequence fixture golden SVG semantic summary");
}

#[test]
fn official_sequence_fixture_supported_rust_svg_contains_source_semantic_text() {
    let mut supported = 0;
    let mut failures = SoftFailures::default();

    for case in official_sequence_fixture_cases() {
        let Ok(diagram) = parse_sequence(&case.fixture.source) else {
            continue;
        };
        supported += 1;

        let layout = layout_sequence(&diagram, &LayoutOptions::default());
        let rendered = render_sequence_svg(&layout, &PlantUmlTheme::default());
        let source_text = source_semantic_text_summary(&case.fixture.source);
        let golden_text = match try_svg_semantic_summary(&case.fixture.golden) {
            Ok(summary) => summary,
            Err(error) => {
                failures.push(
                    &case.name,
                    format!("golden SVG should parse as XML: {error}"),
                );
                continue;
            }
        };
        let rendered_text = match try_svg_semantic_summary(&rendered.svg) {
            Ok(summary) => summary,
            Err(error) => {
                failures.push(&case.name, format!("Rust SVG should parse as XML: {error}"));
                continue;
            }
        };
        let confirmed_texts = confirmed_source_texts(&source_text, &golden_text);

        for expected in &confirmed_texts {
            if !rendered_text.contains(expected) {
                failures.push(
                    &case.name,
                    format!(
                        "Rust SVG text should contain source semantic text {expected:?}; rendered text: {:?}",
                        rendered_text.visible_text
                    ),
                );
            }
        }
        if rendered_text.message_count != diagram.messages.len() {
            failures.push(
                &case.name,
                format!(
                    "Rust SVG message groups should match parsed IR messages; got {}, expected {}",
                    rendered_text.message_count,
                    diagram.messages.len()
                ),
            );
        }
    }

    if supported == 0 {
        failures.push(
            "official sequence corpus",
            "at least one fixture should parse and render",
        );
    }

    failures.finish("official sequence fixture supported Rust SVG semantic summary");
}

#[test]
fn checked_in_sequence_fixture_svg_snapshots_match_current_renderer_output() {
    let mut failures = SoftFailures::default();

    for case in official_sequence_fixture_cases() {
        let expected_path = checked_in_sequence_fixture_snapshot_svg_path(&case.name);
        let expected = match fs::read_to_string(&expected_path) {
            Ok(expected) => expected,
            Err(error) => {
                failures.push(
                    &case.name,
                    format!(
                        "failed to read checked-in SVG snapshot {}: {error}",
                        expected_path.display()
                    ),
                );
                continue;
            }
        };
        let diagram = match parse_sequence(&case.fixture.source) {
            Ok(diagram) => diagram,
            Err(error) => {
                failures.push(&case.name, format!("source should parse: {error}"));
                continue;
            }
        };

        let layout = layout_sequence(&diagram, &LayoutOptions::default());
        let rendered = render_sequence_svg(&layout, &PlantUmlTheme::default());
        if expected != rendered.svg {
            failures.push_svg_snapshot_mismatch(&case.name, &expected, &rendered.svg);
        }
    }

    failures.finish("checked-in sequence fixture SVG snapshot comparison");
}

#[test]
fn in_memory_sequence_fixture_contracts_are_named_and_documented() {
    for group in FIXTURE_GROUPS {
        assert!(!group.name.is_empty(), "fixture group should be named");
        for fixture in group.active.iter().chain(group.future.iter()) {
            assert_timeline_markers_are_documented_in_source(fixture);
        }
    }
}

#[test]
fn parses_currently_supported_sequence_fixtures() {
    for fixture in active_fixtures() {
        let diagram = parse_sequence(fixture.source)
            .unwrap_or_else(|error| panic!("{} should parse: {error}", fixture.name));

        assert_parsed_fixture(fixture, &diagram);
    }
}

#[test]
fn renders_currently_supported_sequence_fixture_svg_markers() {
    for fixture in active_fixtures() {
        let diagram = parse_sequence(fixture.source)
            .unwrap_or_else(|error| panic!("{} should parse: {error}", fixture.name));
        let layout = layout_sequence(&diagram, &LayoutOptions::default());
        let rendered = render_sequence_svg(&layout, &PlantUmlTheme::default());

        for marker in fixture.expected_svg_markers {
            assert!(
                rendered_svg_contains_marker(&rendered.svg, marker),
                "{} rendered SVG should contain marker {marker:?}",
                fixture.name
            );
        }
    }
}

#[test]
fn parses_future_sequence_fixture_contracts() {
    for fixture in future_fixtures() {
        let diagram = parse_sequence(fixture.source)
            .unwrap_or_else(|error| panic!("{} should parse: {error}", fixture.name));

        assert_parsed_fixture(fixture, &diagram);
    }
}

#[test]
fn renders_future_sequence_fixture_svg_markers() {
    for fixture in future_fixtures() {
        let diagram = parse_sequence(fixture.source)
            .unwrap_or_else(|error| panic!("{} should parse: {error}", fixture.name));
        let layout = layout_sequence(&diagram, &LayoutOptions::default());
        let rendered = render_sequence_svg(&layout, &PlantUmlTheme::default());

        for marker in fixture.expected_svg_markers {
            assert!(
                rendered_svg_contains_marker(&rendered.svg, marker),
                "{} rendered SVG should contain future marker {marker:?}",
                fixture.name
            );
        }
    }
}

fn active_fixtures() -> impl Iterator<Item = &'static SequenceFixture> {
    FIXTURE_GROUPS.iter().flat_map(|group| group.active.iter())
}

fn future_fixtures() -> impl Iterator<Item = &'static SequenceFixture> {
    FIXTURE_GROUPS.iter().flat_map(|group| group.future.iter())
}

fn rendered_svg_contains_marker(svg: &str, marker: &str) -> bool {
    match marker {
        "class=\"message\"" => svg_has_class(svg, "message"),
        "class=\"participant\"" => svg_has_class(svg, "participant"),
        _ if marker.contains("**") || marker.contains("//") || marker.contains("<U+") => {
            svg.contains(marker)
                || try_svg_semantic_summary(svg).is_ok_and(|summary| summary.contains(marker))
        }
        _ => svg.contains(marker),
    }
}

fn svg_has_class(svg: &str, class_name: &str) -> bool {
    svg.match_indices("class=\"").any(|(start, _)| {
        let value_start = start + "class=\"".len();
        let Some(value_end) = svg[value_start..].find('"') else {
            return false;
        };
        svg[value_start..value_start + value_end]
            .split_ascii_whitespace()
            .any(|name| name == class_name)
    })
}

fn checked_in_sequence_fixture_snapshot_svg_path(case_name: &str) -> std::path::PathBuf {
    let snapshot_name = case_name
        .strip_suffix(".puml")
        .map(|stem| format!("{stem}.svg"))
        .unwrap_or_else(|| format!("{case_name}.svg"));

    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sequence")
        .join(snapshot_name)
}

fn svg_diff_summary(expected: &str, actual: &str) -> String {
    let first_diff_byte = expected
        .as_bytes()
        .iter()
        .zip(actual.as_bytes())
        .position(|(expected, actual)| expected != actual)
        .unwrap_or_else(|| expected.len().min(actual.len()));
    let first_diff_char = expected
        .chars()
        .zip(actual.chars())
        .position(|(expected, actual)| expected != actual)
        .unwrap_or_else(|| expected.chars().count().min(actual.chars().count()));

    format!(
        "expected_len={} bytes/{} chars; actual_len={} bytes/{} chars; first_diff_byte={}; first_diff_char={}; expected_fragment={:?}; actual_fragment={:?}",
        expected.len(),
        expected.chars().count(),
        actual.len(),
        actual.chars().count(),
        first_diff_byte,
        first_diff_char,
        svg_diff_fragment(expected, first_diff_char),
        svg_diff_fragment(actual, first_diff_char)
    )
}

fn svg_diff_fragment(value: &str, center_char: usize) -> String {
    const CONTEXT_CHARS: usize = 48;

    let total_chars = value.chars().count();
    let start = center_char.saturating_sub(CONTEXT_CHARS);
    let end = center_char.saturating_add(CONTEXT_CHARS).min(total_chars);
    let mut fragment = value
        .chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect::<String>();

    fragment = fragment
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    if start > 0 {
        fragment.insert_str(0, "...");
    }
    if end < total_chars {
        fragment.push_str("...");
    }
    fragment
}

fn official_sequence_fixture_cases() -> Vec<OfficialSequenceFixtureCase> {
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sequence");
    let mut paths = fs::read_dir(&fixture_dir)
        .unwrap_or_else(|error| {
            panic!(
                "failed to read fixture directory {}: {error}",
                fixture_dir.display()
            )
        })
        .map(|entry| entry.expect("fixture directory entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "puml")
        })
        .collect::<Vec<_>>();
    paths.sort();

    assert!(
        !paths.is_empty(),
        "official sequence fixture 1/2 corpus should contain puml files"
    );

    paths
        .into_iter()
        .enumerate()
        .map(|(offset, path)| {
            let expected_name = format!("{:03}.puml", offset + 1);
            assert_eq!(
                path.file_name().and_then(|name| name.to_str()),
                Some(expected_name.as_str()),
                "sequence fixture puml files should be numbered contiguously"
            );

            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            let golden_path = path.with_file_name(format!("{:03}-golden.svg", offset + 1));
            let golden = fs::read_to_string(&golden_path).unwrap_or_else(|error| {
                panic!("failed to read {}: {error}", golden_path.display())
            });

            OfficialSequenceFixtureCase {
                name: expected_name,
                fixture: OfficialSequenceFixture { source, golden },
            }
        })
        .collect()
}

#[derive(Debug)]
struct SourceTextSummary {
    candidate_texts: Vec<String>,
    message_labels: Vec<String>,
}

#[derive(Debug)]
struct SvgSemanticSummary {
    visible_text: Vec<String>,
    normalized_text: String,
    participants: BTreeSet<String>,
    message_count: usize,
    lifeline_count: usize,
}

impl SvgSemanticSummary {
    fn contains(&self, expected: &str) -> bool {
        let expected = normalize_semantic_text(expected);
        expected.is_empty() || self.normalized_text.contains(&expected)
    }
}

fn try_svg_semantic_summary(svg: &str) -> Result<SvgSemanticSummary, roxmltree::Error> {
    let document = roxmltree::Document::parse(svg)?;
    let mut visible_text = Vec::new();
    let mut participants = BTreeSet::new();
    let mut message_count = 0;
    let mut lifeline_count = 0;

    for node in document.descendants().filter(|node| node.is_element()) {
        if let Some(participant) = node
            .attribute("data-qualified-name")
            .or_else(|| node.attribute("data-participant"))
        {
            let participant = normalize_semantic_text(participant);
            if !participant.is_empty() {
                participants.insert(participant);
            }
        }

        if has_svg_class(node, "message") {
            message_count += 1;
        }
        if has_svg_class(node, "participant-lifeline") {
            lifeline_count += 1;
        }
    }

    for node in document.descendants().filter(|node| node.is_text()) {
        let Some(parent) = node.parent() else {
            continue;
        };
        if !parent.has_tag_name("text") && !parent.has_tag_name("tspan") {
            continue;
        }

        let text = normalize_semantic_text(node.text().unwrap_or_default());
        if !text.is_empty() {
            visible_text.push(text);
        }
    }

    let normalized_text = visible_text.join(" ");
    Ok(SvgSemanticSummary {
        visible_text,
        normalized_text,
        participants,
        message_count,
        lifeline_count,
    })
}

fn has_svg_class(node: roxmltree::Node<'_, '_>, class_name: &str) -> bool {
    node.attribute("class")
        .is_some_and(|classes| classes.split_whitespace().any(|class| class == class_name))
}

fn confirmed_source_texts(
    source_text: &SourceTextSummary,
    golden_text: &SvgSemanticSummary,
) -> Vec<String> {
    source_text
        .candidate_texts
        .iter()
        .filter(|text| golden_text.contains(text))
        .cloned()
        .collect()
}

fn source_semantic_text_summary(source: &str) -> SourceTextSummary {
    let mut texts = BTreeSet::new();
    let mut message_labels = BTreeSet::new();
    let mut in_note = false;
    let mut in_ignored_block = false;

    for raw_line in source.lines() {
        let line = raw_line.trim();
        if line.is_empty()
            || line.eq_ignore_ascii_case("@startuml")
            || line.eq_ignore_ascii_case("@enduml")
            || line.starts_with('\'')
        {
            continue;
        }

        if in_ignored_block {
            if line.eq_ignore_ascii_case("</style>") || line.eq_ignore_ascii_case("endskinparam") {
                in_ignored_block = false;
            }
            continue;
        }
        if line.eq_ignore_ascii_case("<style>") || line.eq_ignore_ascii_case("skinparam") {
            in_ignored_block = true;
            continue;
        }

        if in_note {
            if line.eq_ignore_ascii_case("end note") {
                in_note = false;
            } else {
                push_source_text(&mut texts, line);
            }
            continue;
        }
        if line.to_ascii_lowercase().starts_with("note ") {
            if let Some((_, text)) = line.split_once(':') {
                push_source_text(&mut texts, text);
            } else {
                in_note = true;
            }
            continue;
        }

        if let Some(text) = line
            .strip_prefix("title ")
            .or_else(|| line.strip_prefix("mainframe "))
        {
            push_source_text(&mut texts, text);
            continue;
        }

        if let Some(text) = divider_text(line) {
            push_source_text(&mut texts, text);
            continue;
        }

        if let Some(text) = group_text(line) {
            push_source_text(&mut texts, text);
            continue;
        }

        if let Some((_, text)) = line.split_once(':') {
            if source_line_has_message_arrow(line) {
                push_source_text(&mut message_labels, text);
            }
            push_source_text(&mut texts, text);
        }

        collect_participant_display_text(line, &mut texts);
    }

    SourceTextSummary {
        candidate_texts: texts.into_iter().collect(),
        message_labels: message_labels.into_iter().collect(),
    }
}

fn parsed_diagram_source_message_label_failures(
    case: &OfficialSequenceFixtureCase,
    diagram: &SequenceDiagram,
) -> Vec<String> {
    let source_text = source_semantic_text_summary(&case.fixture.source);
    let parsed_message_labels = diagram
        .messages
        .iter()
        .map(|message| normalize_semantic_text(&message.label))
        .collect::<BTreeSet<_>>();
    let parsed_message_text = parsed_message_labels
        .iter()
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");

    let mut failures = Vec::new();
    for expected in source_text.message_labels {
        if !parsed_message_text.contains(&expected) {
            failures.push(format!(
                "parsed IR should preserve source message label {expected:?}; parsed labels: {:?}",
                parsed_message_labels
            ));
        }
    }

    failures
}

fn source_line_has_message_arrow(line: &str) -> bool {
    let line = line.trim();
    [
        "->", "-->", "<-", "<--", "->>", "<->", "\\-", "-\\", "/-", "-/", "->o", "o->",
    ]
    .iter()
    .any(|arrow| line.contains(arrow))
}

fn push_source_text(texts: &mut BTreeSet<String>, text: &str) {
    for part in text.split("\\n") {
        push_source_text_part(texts, part);
    }
}

fn push_source_text_part(texts: &mut BTreeSet<String>, text: &str) {
    if text.contains('%') || text.contains("<U+") {
        return;
    }
    let text = normalize_semantic_text(&strip_empty_quoted_fragments(text));
    if !text.is_empty()
        && !text.contains('%')
        && text.chars().any(|character| character.is_alphanumeric())
    {
        texts.insert(text);
    }
}

fn strip_empty_quoted_fragments(value: &str) -> String {
    let mut text = value.to_owned();
    while let Some(start) = text.find("\"\"") {
        let after_start = start + 2;
        let Some(relative_end) = text[after_start..].find("\"\"") else {
            break;
        };
        let end = after_start + relative_end + 2;
        text.replace_range(start..end, "");
    }
    text
}

fn divider_text(line: &str) -> Option<&str> {
    line.strip_prefix("==")
        .and_then(|text| text.strip_suffix("=="))
        .map(str::trim)
        .filter(|text| !text.is_empty())
}

fn group_text(line: &str) -> Option<&str> {
    for keyword in [
        "alt", "else", "opt", "loop", "par", "break", "critical", "group",
    ] {
        if line == keyword {
            return Some(keyword);
        }
        if let Some(text) = line
            .strip_prefix(keyword)
            .and_then(|text| text.strip_prefix(' '))
        {
            return Some(text);
        }
    }
    None
}

fn collect_participant_display_text(line: &str, texts: &mut BTreeSet<String>) {
    for keyword in [
        "participant",
        "actor",
        "boundary",
        "control",
        "entity",
        "database",
        "collections",
        "queue",
    ] {
        let Some(rest) = line
            .strip_prefix(keyword)
            .and_then(|text| text.strip_prefix(' '))
        else {
            continue;
        };
        let display = rest
            .split_once(" as ")
            .map_or(rest, |(display, _)| display)
            .trim()
            .trim_matches('"');
        push_source_text(texts, strip_participant_declaration_options(display));
        return;
    }
}

fn normalize_semantic_text(value: &str) -> String {
    let text = strip_plantuml_markup(value)
        .replace("\\n", " ")
        .replace('«', "<<")
        .replace('»', ">>");
    let text = text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .trim_matches('"')
        .to_owned();
    text.replace("<< ", "<<").replace(" >>", ">>")
}

fn strip_participant_declaration_options(value: &str) -> &str {
    value
        .split(['#', '['])
        .next()
        .unwrap_or(value)
        .split(" order ")
        .next()
        .unwrap_or(value)
        .trim()
        .trim_matches('"')
}

fn strip_plantuml_markup(value: &str) -> String {
    let value = value
        .replace("**", "")
        .replace("__", "")
        .replace("//", "")
        .replace("~~", "");
    let mut output = String::new();
    let mut chars = value.chars().peekable();

    while let Some(character) = chars.next() {
        if character == '<' {
            let next = chars.peek().copied();
            if next == Some('<') {
                output.push(character);
                output.push(chars.next().expect("peeked character should exist"));
                continue;
            }
            if next.is_some_and(|next| next == '/' || next.is_ascii_alphabetic()) {
                for character in chars.by_ref() {
                    if character == '>' {
                        break;
                    }
                }
                continue;
            }
        }
        output.push(character);
    }

    output
}

fn assert_parsed_fixture(fixture: &SequenceFixture, diagram: &SequenceDiagram) {
    let participant_ids: Vec<&str> = diagram
        .participants
        .iter()
        .map(|participant| participant.id.as_str())
        .collect();
    assert_eq!(
        participant_ids, fixture.expected_participants,
        "{} participants should match fixture",
        fixture.name
    );

    assert_eq!(
        diagram.messages.len(),
        fixture.expected_messages.len(),
        "{} message count should match fixture",
        fixture.name
    );
    for (actual, expected) in diagram
        .messages
        .iter()
        .zip(fixture.expected_messages.iter())
    {
        assert_eq!(
            (
                actual.from.as_str(),
                actual.to.as_str(),
                actual.label.as_str()
            ),
            (expected.from, expected.to, expected.label),
            "{} message endpoint/label should match fixture",
            fixture.name
        );
        assert_eq!(
            actual.line_style, expected.line_style,
            "{} message line style should match fixture",
            fixture.name
        );
    }

    for marker in fixture.expected_timeline_markers {
        assert!(
            diagram_has_timeline_marker(diagram, marker),
            "{} timeline should contain marker {marker:?}",
            fixture.name
        );
    }
}

fn assert_timeline_markers_are_documented_in_source(fixture: &SequenceFixture) {
    for marker in fixture.expected_timeline_markers {
        assert!(
            fixture.source.contains(marker),
            "{} source should contain timeline marker {marker:?}",
            fixture.name
        );
    }
}

fn diagram_has_timeline_marker(diagram: &SequenceDiagram, marker: &str) -> bool {
    diagram.events.iter().any(|event| match event {
        SequenceEvent::ParticipantDeclaration(declaration) => declaration.raw == marker,
        SequenceEvent::Divider(divider) => marker.contains(&divider.text),
        SequenceEvent::Delay(delay) => delay
            .text
            .as_deref()
            .map_or(marker == "...", |text| marker.contains(text)),
        SequenceEvent::Activation(activation) => marker.contains(&activation.participant),
        SequenceEvent::Return(return_event) => return_event
            .label
            .as_deref()
            .is_some_and(|label| marker.contains(label)),
        SequenceEvent::Note(note) => marker.contains("note") || marker.contains(&note.text),
        SequenceEvent::GroupStart(group) => group
            .label
            .as_deref()
            .is_some_and(|label| marker.contains(label)),
        SequenceEvent::GroupElse(group_else) => group_else
            .label
            .as_deref()
            .is_some_and(|label| marker.contains(label)),
        SequenceEvent::BoxStart(box_block) => box_block.raw == marker,
        SequenceEvent::Title(title) => marker.contains(&title.text),
        SequenceEvent::HideFootbox(_) => marker.eq_ignore_ascii_case("hide footbox"),
        SequenceEvent::SkinParam(skinparam) => skinparam.raw == marker,
        SequenceEvent::Pragma(pragma) => pragma.raw == marker,
        SequenceEvent::Message(message) => message_has_marker(message, marker),
        _ => false,
    }) || diagram
        .semantic_events
        .iter()
        .any(|event| semantic_event_has_marker(event, marker))
}

fn semantic_event_has_marker(event: &SequenceSemanticEvent, marker: &str) -> bool {
    match event {
        SequenceSemanticEvent::Autonumber(event) => event.raw == marker,
        SequenceSemanticEvent::PageBreak(event) => event.raw == marker,
        SequenceSemanticEvent::HeaderFooter(event) => event.raw == marker,
        SequenceSemanticEvent::Reference(event) => {
            event.raw == marker
                || event
                    .text
                    .as_deref()
                    .is_some_and(|text| marker.contains(text))
        }
        SequenceSemanticEvent::Space(event) => event.raw == marker,
        SequenceSemanticEvent::Mainframe(event) => event.raw == marker,
        SequenceSemanticEvent::StyleBlock(event) => {
            marker == "<style>" || event.content.contains(marker)
        }
        SequenceSemanticEvent::HideUnlinked(event) => event.raw == marker,
        SequenceSemanticEvent::Autoactivate(event) => event.raw == marker,
    }
}

fn message_has_marker(message: &SequenceMessage, marker: &str) -> bool {
    let marker = marker.trim();
    let (requires_parallel, marker) = marker
        .strip_prefix('&')
        .map_or((false, marker), |rest| (true, rest.trim_start()));

    let Some((participants, label)) = marker.split_once(':') else {
        return false;
    };
    let mut parts = participants.split_whitespace();
    let (Some(from), Some(_arrow), Some(to), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };

    message.from.id == from
        && message.to.id == to
        && message.label == label.trim()
        && (!requires_parallel || message.arrow.features.parallel)
}
