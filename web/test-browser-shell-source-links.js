import {
  applyEditorSelection,
  collectDiagnosticSourceSections,
} from "./browser-shell-source-links.js";

export function testCompileDiagnosticSourceLinkResolution() {
  const files = [
    {
      path: "main.go",
      contents: `package main

func main() {
\tundeclaredFunction()
}
`,
    },
  ];
  const diagnostics = [
    {
      message: "undefined: undeclaredFunction",
      severity: "error",
      file_path: "main.go",
      source_span: {
        start: { line: 4, column: 2 },
        end: { line: 4, column: 20 },
      },
    },
  ];

  const sections = collectDiagnosticSourceSections(diagnostics, files);
  const link = sections[0]?.links[0];
  if (!link || link.path !== "main.go" || link.startLine !== 4 || link.startColumn !== 2) {
    throw new Error(`unexpected compile link: ${JSON.stringify(sections)}`);
  }

  const textarea = fakeTextarea(files[0].contents);
  if (!applyEditorSelection(textarea, files[0].contents, link)) {
    throw new Error("failed to apply compile selection");
  }
  if (!selectedText(textarea).includes("undeclaredFunction")) {
    throw new Error(`unexpected compile selection: ${JSON.stringify(selectedText(textarea))}`);
  }
}

export function testRuntimeInstructionSpanResolution() {
  const files = [
    {
      path: "__module_cache__/example.com/remote/@v1.0.0/panicmod/panicmod.go",
      contents: `package panicmod

func Run() {
\texplode(0)
}

func explode(value int) int {
\treturn 1 / value
}
`,
    },
  ];
  const highlight = "\treturn 1 / value";
  const start = files[0].contents.indexOf(highlight);
  const end = start + highlight.length;
  const diagnostics = [
    {
      message: "division by zero",
      severity: "error",
      runtime: {
        root_message: "division by zero",
        stack_trace: [
          {
            function: "explode",
            source_span: {
              path: files[0].path,
              start,
              end,
            },
          },
        ],
      },
    },
  ];

  const sections = collectDiagnosticSourceSections(diagnostics, files);
  const link = sections[0]?.links[0];
  if (!link || link.path !== files[0].path || link.startLine !== 8 || link.startColumn !== 1) {
    throw new Error(`unexpected runtime link: ${JSON.stringify(sections)}`);
  }

  const textarea = fakeTextarea(files[0].contents);
  if (!applyEditorSelection(textarea, files[0].contents, link)) {
    throw new Error("failed to apply runtime selection");
  }
  if (!selectedText(textarea).includes("1 / value")) {
    throw new Error(`unexpected runtime selection: ${JSON.stringify(selectedText(textarea))}`);
  }
}

function fakeTextarea(value) {
  return {
    value,
    selectionStart: 0,
    selectionEnd: 0,
    scrollTop: 0,
    focus() {},
    setSelectionRange(start, end) {
      this.selectionStart = start;
      this.selectionEnd = end;
    },
  };
}

function selectedText(textarea) {
  return textarea.value.slice(textarea.selectionStart, textarea.selectionEnd);
}
