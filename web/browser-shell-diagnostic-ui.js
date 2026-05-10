import { normalizeWorkspacePath } from "./browser-workspace.js";
import {
  applyEditorSelection,
  renderDiagnosticSourcePanel,
} from "./browser-shell-source-links.js";

export function createBrowserShellDiagnosticUi({
  diagnosticSummaryElement,
  diagnosticSummaryPanelElement,
  getDisplayFiles,
  linksElement,
  outputElement,
  renderWorkspace,
  setSelectedFilePath,
  sourceElement,
  sourceLinksPanelElement,
  statusElement,
}) {
  return {
    setOutputView(text, diagnostics = [], auxiliaryIssues = []) {
      outputElement.textContent = text;
      renderStructuredIssuePanel({
        diagnostics,
        auxiliaryIssues,
        listElement: diagnosticSummaryElement,
        panelElement: diagnosticSummaryPanelElement,
      });
      renderDiagnosticSourcePanel({
        diagnostics,
        files: getDisplayFiles(),
        linksElement,
        onSelectLink(link) {
          jumpToSourceLink(link, {
            getDisplayFiles,
            renderWorkspace,
            setSelectedFilePath,
            sourceElement,
            statusElement,
          });
        },
        panelElement: sourceLinksPanelElement,
      });
    },
  };
}

export function createToolingIssue(message, options = {}) {
  return createAuxiliaryIssue("tooling", message, options);
}

export function createProtocolIssue(message, options = {}) {
  return createAuxiliaryIssue("protocol_error", message, options);
}

export function createHostIssue(message, options = {}) {
  return createAuxiliaryIssue("host_error", message, options);
}

export function collectStructuredIssues({ diagnostics = [], auxiliaryIssues = [] }) {
  return [...(diagnostics ?? []), ...(auxiliaryIssues ?? [])]
    .map(structuredIssueFromSource)
    .filter(Boolean);
}

export function renderStructuredIssuePanel({
  diagnostics = [],
  auxiliaryIssues = [],
  listElement,
  panelElement,
}) {
  const issues = collectStructuredIssues({ diagnostics, auxiliaryIssues });
  listElement.replaceChildren();
  panelElement.hidden = issues.length === 0;
  if (issues.length === 0) {
    return;
  }

  for (const issue of issues) {
    listElement.append(buildIssueCard(issue));
  }
}

function buildIssueCard(issue) {
  const card = document.createElement("article");
  card.className = `diagnostic-card diagnostic-card-${issue.severity}`;
  card.dataset.category = issue.category;
  card.dataset.severity = issue.severity;

  const header = document.createElement("div");
  header.className = "diagnostic-card-header";

  const title = document.createElement("p");
  title.className = "diagnostic-card-title";
  title.textContent = issue.title;
  header.append(title);

  const badges = document.createElement("div");
  badges.className = "diagnostic-badges";
  badges.append(
    createBadge(`diagnostic-badge diagnostic-badge-${issue.severity}`, issue.severity),
    createBadge("diagnostic-badge diagnostic-badge-category", issue.category),
  );
  header.append(badges);
  card.append(header);

  const meta = document.createElement("dl");
  meta.className = "diagnostic-meta";
  appendMetaRow(meta, "Location", issue.location);
  appendMetaRow(meta, "Next step", issue.suggestedAction);
  if (meta.childElementCount > 0) {
    card.append(meta);
  }

  if (issue.excerpt) {
    const excerpt = document.createElement("pre");
    excerpt.className = "diagnostic-excerpt";
    excerpt.textContent = issue.excerpt;
    card.append(excerpt);
  }

  if (issue.stackLines.length > 0) {
    const stack = document.createElement("div");
    stack.className = "diagnostic-stack";

    const stackLabel = document.createElement("strong");
    stackLabel.textContent = "Stack";
    stack.append(stackLabel);

    const stackList = document.createElement("ol");
    stackList.className = "diagnostic-stack-list";
    for (const line of issue.stackLines) {
      const item = document.createElement("li");
      item.textContent = line;
      stackList.append(item);
    }
    stack.append(stackList);
    card.append(stack);
  }

  return card;
}

function createBadge(className, text) {
  const badge = document.createElement("span");
  badge.className = className;
  badge.textContent = text;
  return badge;
}

function appendMetaRow(meta, termText, valueText) {
  if (!valueText) {
    return;
  }
  const row = document.createElement("div");
  const term = document.createElement("dt");
  term.textContent = termText;
  const value = document.createElement("dd");
  value.textContent = valueText;
  row.append(term, value);
  meta.append(row);
}

function structuredIssueFromSource(source) {
  if (!source || typeof source !== "object") {
    return null;
  }

  return {
    title: issueLeadMessage(source),
    category: issueCategory(source),
    severity: normalizeSeverity(source.severity),
    location: issueLocation(source),
    suggestedAction: issueSuggestedAction(source),
    excerpt: formatSourceExcerptBlock(source.source_excerpt),
    stackLines: issueStackLines(source),
  };
}

function issueLeadMessage(source) {
  const runtimeMessage = normalizeString(source.runtime?.root_message);
  if (runtimeMessage) {
    return runtimeMessage;
  }
  const message = normalizeString(source.message);
  if (!message) {
    return "Uncategorized issue";
  }
  return message.split("\n", 1)[0];
}

function issueLocation(source) {
  const path = normalizeWorkspacePath(normalizeString(source.file_path));
  if (path && source.source_span?.start) {
    return `${path}:${source.source_span.start.line}:${source.source_span.start.column}`;
  }
  if (path && source.position) {
    return `${path}:${source.position.line}:${source.position.column}`;
  }
  if (path && source.source_excerpt?.line) {
    return `${path}:${source.source_excerpt.line}:${Math.max(1, source.source_excerpt.highlight_start_column ?? 1)}`;
  }
  if (path) {
    return path;
  }

  const firstFrame = source.runtime?.stack_trace?.[0];
  const runtimeLocation = firstFrame?.source_location;
  if (runtimeLocation?.path) {
    return `${runtimeLocation.path}:${runtimeLocation.line}:${runtimeLocation.column}`;
  }

  return "";
}

function issueStackLines(source) {
  if (Array.isArray(source.runtime?.stack_trace) && source.runtime.stack_trace.length > 0) {
    return source.runtime.stack_trace.map((frame) => {
      const location = frame?.source_location;
      if (location?.path) {
        return `${frame.function} (${location.path}:${location.line}:${location.column})`;
      }
      return frame?.function || "unknown";
    });
  }

  if (Array.isArray(source.stack_lines)) {
    return source.stack_lines
      .map((line) => normalizeString(line))
      .filter(Boolean);
  }

  return [];
}

function issueCategory(source) {
  const explicit = normalizeCategory(source.runtime?.category || source.category);
  if (explicit !== "uncategorized") {
    return explicit;
  }
  if (normalizeSeverity(source.severity) === "warning") {
    return "tooling";
  }
  if (normalizeWorkspacePath(normalizeString(source.file_path))) {
    return "compile_error";
  }
  return explicit;
}

function issueSuggestedAction(source) {
  const explicit = normalizeString(source.suggested_action);
  if (explicit) {
    return explicit;
  }
  if (Array.isArray(source.runtime?.stack_trace) && source.runtime.stack_trace.length > 0) {
    return "Fix the failing code path and rerun the current request.";
  }
  if (normalizeWorkspacePath(normalizeString(source.file_path))) {
    return "Fix the source error and compile again.";
  }
  return "";
}

function normalizeCategory(category) {
  const normalized = normalizeString(category);
  return normalized || "uncategorized";
}

function normalizeSeverity(severity) {
  const normalized = normalizeString(severity);
  return normalized || "error";
}

function formatSourceExcerptBlock(excerpt) {
  if (!excerpt || typeof excerpt !== "object") {
    return "";
  }

  const lineNumber = excerpt.line ?? 0;
  const text = excerpt.text ?? "";
  const startColumn = Math.max(1, excerpt.highlight_start_column ?? 1);
  const endColumn = Math.max(startColumn, excerpt.highlight_end_column ?? startColumn);
  const gutter = `${lineNumber} | `;
  const underlineWidth = Math.max(1, endColumn - startColumn + 1);
  return [
    `${gutter}${text}`,
    `${" ".repeat(gutter.length)}${" ".repeat(startColumn - 1)}${"^".repeat(underlineWidth)}`,
  ].join("\n");
}

function jumpToSourceLink(
  link,
  { getDisplayFiles, renderWorkspace, setSelectedFilePath, sourceElement, statusElement },
) {
  const targetPath = normalizeWorkspacePath(link.path);
  if (!targetPath) {
    return;
  }

  setSelectedFilePath(targetPath);
  renderWorkspace();

  const selected = getDisplayFiles().find((file) => file.path === targetPath);
  if (!selected) {
    statusElement.textContent = "Source jump target unavailable";
    return;
  }

  applyEditorSelection(sourceElement, selected.contents, link);
  statusElement.textContent = `Jumped to ${link.path}:${link.startLine}:${link.startColumn}`;
}

function createAuxiliaryIssue(category, message, options) {
  const filePath = normalizeString(options.filePath);
  const line = Number.isFinite(options.line) && options.line > 0 ? options.line : null;
  const column = Number.isFinite(options.column) && options.column > 0 ? options.column : null;
  return {
    message,
    severity: options.severity || "error",
    category,
    file_path: filePath || null,
    position:
      filePath && line
        ? {
            line,
            column: column ?? 1,
          }
        : null,
    source_span: null,
    source_excerpt: null,
    suggested_action: options.suggestedAction || null,
    runtime: null,
    stack_lines: parseStackLines(options.stackText),
  };
}

function parseStackLines(stackText) {
  const text = normalizeString(stackText);
  if (!text) {
    return [];
  }
  return text
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line && !line.startsWith("Error:"));
}

function normalizeString(value) {
  return typeof value === "string" ? value : "";
}
