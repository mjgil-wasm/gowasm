import { normalizeWorkspacePath } from "./browser-workspace.js";

export function collectDiagnosticSourceSections(diagnostics, files) {
  const sections = [];
  for (const [diagnosticIndex, diagnostic] of (diagnostics ?? []).entries()) {
    const links = collectDiagnosticLinks(diagnostic, files);
    if (links.length === 0) {
      continue;
    }
    sections.push({
      heading: diagnosticHeading(diagnostic, diagnosticIndex),
      links,
    });
  }
  return sections;
}

export function applyEditorSelection(textarea, fileContents, target) {
  const selection = selectionOffsetsForTarget(fileContents, target);
  if (!selection) {
    return false;
  }

  textarea.focus();
  textarea.setSelectionRange(selection.start, selection.end);
  textarea.scrollTop = approximateScrollTop(fileContents, selection.start);
  return true;
}

export function renderDiagnosticSourcePanel(ctx) {
  const sections = collectDiagnosticSourceSections(ctx.diagnostics, ctx.files);
  ctx.linksElement.replaceChildren();
  ctx.panelElement.hidden = sections.length === 0;
  if (sections.length === 0) {
    return;
  }

  for (const section of sections) {
    const group = document.createElement("article");
    group.className = "source-link-group";

    const heading = document.createElement("p");
    heading.className = "source-link-heading";
    heading.textContent = section.heading;
    group.append(heading);

    for (const link of section.links) {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "source-link-button secondary";
      button.textContent = link.label;
      button.addEventListener("click", () => {
        ctx.onSelectLink(link);
      });
      group.append(button);
    }

    ctx.linksElement.append(group);
  }
}

function collectDiagnosticLinks(diagnostic, files) {
  const links = [];
  const seen = new Set();

  appendLink(
    links,
    seen,
    sourceTargetFromDiagnostic(diagnostic, files),
  );

  for (const [frameIndex, frame] of (diagnostic?.runtime?.stack_trace ?? []).entries()) {
    appendLink(
      links,
      seen,
      sourceTargetFromRuntimeFrame(frame, files, frameIndex),
    );
  }

  return links;
}

function appendLink(links, seen, link) {
  if (!link) {
    return;
  }
  const key = [
    link.path,
    link.startLine,
    link.startColumn,
    link.endLine,
    link.endColumn,
    link.label,
  ].join(":");
  if (seen.has(key)) {
    return;
  }
  seen.add(key);
  links.push(link);
}

function sourceTargetFromDiagnostic(diagnostic, files) {
  const path = normalizeWorkspacePath(diagnostic?.file_path ?? "");
  if (!path) {
    return null;
  }

  if (diagnostic?.source_span) {
    return {
      label: `primary: ${formatLocation(path, diagnostic.source_span.start.line, diagnostic.source_span.start.column)}`,
      path,
      startLine: diagnostic.source_span.start.line,
      startColumn: diagnostic.source_span.start.column,
      endLine: diagnostic.source_span.end.line,
      endColumn: diagnostic.source_span.end.column,
    };
  }

  if (diagnostic?.position) {
    return {
      label: `primary: ${formatLocation(path, diagnostic.position.line, diagnostic.position.column)}`,
      path,
      startLine: diagnostic.position.line,
      startColumn: diagnostic.position.column,
      endLine: diagnostic.position.line,
      endColumn: diagnostic.position.column,
    };
  }

  const source = fileContentsForPath(files, path);
  if (!diagnostic?.source_excerpt || source === null) {
    return null;
  }
  return targetFromExcerpt(path, diagnostic.source_excerpt, source, "primary");
}

function sourceTargetFromRuntimeFrame(frame, files, frameIndex) {
  const runtimeLocation = frame?.source_location;
  if (runtimeLocation?.path) {
    const path = normalizeWorkspacePath(runtimeLocation.path);
    return {
      label: `frame ${frameIndex + 1}: ${frame.function} (${formatLocation(path, runtimeLocation.line, runtimeLocation.column)})`,
      path,
      startLine: runtimeLocation.line,
      startColumn: runtimeLocation.column,
      endLine: runtimeLocation.end_line,
      endColumn: runtimeLocation.end_column,
    };
  }

  const runtimeSpan = frame?.source_span;
  if (!runtimeSpan?.path) {
    return null;
  }

  const path = normalizeWorkspacePath(runtimeSpan.path);
  const source = fileContentsForPath(files, path);
  if (source === null) {
    return null;
  }

  const spanRange = lineColumnRangeForOffsets(source, runtimeSpan.start, runtimeSpan.end);
  if (!spanRange) {
    return null;
  }

  return {
    label: `frame ${frameIndex + 1}: ${frame.function} (${formatLocation(path, spanRange.startLine, spanRange.startColumn)})`,
    path,
    ...spanRange,
  };
}

function targetFromExcerpt(path, excerpt, source, prefix) {
  const lineText = excerpt?.text ?? "";
  const startLine = excerpt?.line ?? 0;
  const startColumn = excerpt?.highlight_start_column ?? 0;
  const endColumn = excerpt?.highlight_end_column ?? startColumn;
  if (startLine <= 0 || startColumn <= 0 || endColumn <= 0) {
    return null;
  }

  const sourceLine = sourceLineText(source, startLine);
  if (sourceLine === null || sourceLine !== lineText) {
    return null;
  }

  return {
    label: `${prefix}: ${formatLocation(path, startLine, startColumn)}`,
    path,
    startLine,
    startColumn,
    endLine: startLine,
    endColumn,
  };
}

function diagnosticHeading(diagnostic, diagnosticIndex) {
  const message = diagnostic?.runtime?.root_message || diagnostic?.message || "";
  const firstLine = message.split("\n", 1)[0]?.trim() || `Diagnostic ${diagnosticIndex + 1}`;
  return firstLine;
}

function selectionOffsetsForTarget(source, target) {
  const start = byteOffsetForLineColumn(source, target.startLine, target.startColumn);
  const end = byteOffsetForLineColumn(source, target.endLine, target.endColumn);
  if (start === null || end === null) {
    return null;
  }
  return {
    start,
    end: Math.max(start, end),
  };
}

function lineColumnRangeForOffsets(source, startOffset, endOffset) {
  const start = lineColumnForOffset(source, startOffset);
  const end = lineColumnForOffset(source, Math.max(startOffset, endOffset));
  if (!start || !end) {
    return null;
  }
  return {
    startLine: start.line,
    startColumn: start.column,
    endLine: end.line,
    endColumn: end.column,
  };
}

function lineColumnForOffset(source, offset) {
  if (offset < 0 || offset > source.length) {
    return null;
  }

  let line = 1;
  let lineStart = 0;
  forEachChar(source, (index, ch) => {
    if (index >= offset) {
      return true;
    }
    if (ch === "\n") {
      line += 1;
      lineStart = index + 1;
    }
    return false;
  });

  return {
    line,
    column: Array.from(source.slice(lineStart, offset)).length + 1,
  };
}

function byteOffsetForLineColumn(source, line, column) {
  if (line <= 0 || column <= 0) {
    return null;
  }

  let lineStart = 0;
  if (line > 1) {
    let currentLine = 1;
    let found = false;
    forEachChar(source, (index, ch) => {
      if (ch === "\n") {
        currentLine += 1;
        if (currentLine === line) {
          lineStart = index + 1;
          found = true;
          return true;
        }
      }
      return false;
    });
    if (!found) {
      return null;
    }
  }

  if (column === 1) {
    return lineStart;
  }

  const lineText = sourceLineText(source, line);
  if (lineText === null) {
    return null;
  }

  const lineEndOffset = lineStart + lineText.length;
  const lineCharCount = Array.from(lineText).length;
  if (column === lineCharCount + 1) {
    return lineEndOffset;
  }

  let charsSeen = 1;
  let resolvedOffset = null;
  forEachChar(lineText, (relative) => {
    if (charsSeen === column) {
      resolvedOffset = lineStart + relative;
      return true;
    }
    charsSeen += 1;
    return false;
  });
  if (resolvedOffset !== null) {
    return resolvedOffset;
  }
  return null;
}

function sourceLineText(source, line) {
  if (line <= 0) {
    return null;
  }

  let currentLine = 1;
  let lineStart = 0;
  if (line > 1) {
    let found = false;
    forEachChar(source, (index, ch) => {
      if (ch === "\n") {
        currentLine += 1;
        if (currentLine === line) {
          lineStart = index + 1;
          found = true;
          return true;
        }
      }
      return false;
    });
    if (!found) {
      return null;
    }
  }

  const lineEndRelative = source.slice(lineStart).indexOf("\n");
  const lineEnd = lineEndRelative < 0 ? source.length : lineStart + lineEndRelative;
  return source.slice(lineStart, lineEnd);
}

function approximateScrollTop(source, offset) {
  const prefix = source.slice(0, offset);
  const lineCount = prefix.split("\n").length;
  return Math.max(0, (lineCount - 3) * 20);
}

function fileContentsForPath(files, path) {
  const normalizedPath = normalizeWorkspacePath(path);
  const file = (files ?? []).find((candidate) => normalizeWorkspacePath(candidate?.path) === normalizedPath);
  return file ? String(file.contents ?? "") : null;
}

function formatLocation(path, line, column) {
  return `${path}:${line}:${column}`;
}

function forEachChar(source, visitor) {
  for (let index = 0; index < source.length;) {
    const codePoint = source.codePointAt(index);
    const ch = String.fromCodePoint(codePoint);
    if (visitor(index, ch)) {
      break;
    }
    index += ch.length;
  }
}
