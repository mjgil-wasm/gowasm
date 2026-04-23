export function formatDiagnostics(diagnostics) {
  return (diagnostics || []).map(formatDiagnostic).join("\n\n");
}

export function formatRunResult(stdout, diagnostics) {
  const diagnosticsText = formatDiagnostics(diagnostics);
  if (stdout && diagnosticsText) {
    return `${stdout}${stdout.endsWith("\n") ? "" : "\n"}${diagnosticsText}`;
  }
  return stdout || diagnosticsText || "";
}

export function testRunnerLabel(runner) {
  return runner === "package" ? "Package tests" : "Snippet test";
}

export function formatTestResult(runner, passed, stdout, diagnostics, resultDetails) {
  const summary = `${testRunnerLabel(runner)} ${passed ? "passed" : "failed"}.`;
  const structuredDetails = formatTestDetails(runner, resultDetails);
  const outputDetails = formatRunResult(stdout, diagnostics);
  const sections = [summary];
  if (structuredDetails) {
    sections.push(structuredDetails);
  }
  if (outputDetails) {
    sections.push(outputDetails);
  }
  return sections.join("\n\n");
}

export function loadedModuleVirtualFileCount(bundles) {
  return (bundles ?? []).reduce(
    (count, bundle) => count + (Array.isArray(bundle?.files) ? bundle.files.length : 0),
    0,
  );
}

export function formatLoadedModulesOutput(bundles) {
  if (!bundles || bundles.length === 0) {
    return "No remote modules loaded.";
  }

  const lines = [
    `Loaded ${bundles.length} remote module bundle(s).`,
    `Projected ${loadedModuleVirtualFileCount(bundles)} virtual file(s) into __module_cache__/ for run and test requests.`,
  ];
  for (const bundle of bundles) {
    lines.push(
      `- ${bundle.module.module_path}@${bundle.module.version} from ${bundle.origin_url}`,
    );
  }
  return lines.join("\n");
}

export function describeModuleStatus({
  modules,
  errors,
  isLoading,
  requestedModuleCount,
  loadedBundles,
  loadedBundlesStale,
  configuredModulesMatchLoaded,
  lastLoadError,
}) {
  if (errors.length > 0) {
    return errors.join("\n");
  }

  const summary = [];
  if (modules.length === 0) {
    summary.push("No remote modules configured.");
  } else {
    summary.push(`${modules.length} remote module root(s) configured.`);
  }

  if (isLoading) {
    summary.push(`Loading ${requestedModuleCount} module bundle(s) through the worker boundary…`);
  } else if (lastLoadError) {
    summary.push(`Last module load failed: ${lastLoadError}`);
    summary.push(
      loadedBundlesStale && loadedBundles.length > 0
        ? "Previously loaded bundles are stale; retry Load Modules or run/test to refresh them."
        : "Use Load Modules now or let run/test retry the configured bundles.",
    );
  } else if (modules.length === 0) {
    summary.push(
      loadedBundles.length > 0
        ? "Loaded bundles are idle until module roots are configured again."
        : "Run and test requests use only the editable workspace files.",
    );
  } else if (configuredModulesMatchLoaded && loadedBundles.length > 0) {
    summary.push(
      `Loaded ${loadedBundles.length} bundle(s) projecting ${loadedModuleVirtualFileCount(loadedBundles)} virtual file(s).`,
    );
    for (const bundle of loadedBundles) {
      summary.push(`- ${bundle.module.module_path}@${bundle.module.version}`);
    }
  } else if (loadedBundlesStale && loadedBundles.length > 0) {
    summary.push("Previously loaded bundles are stale; retry Load Modules or run/test to refresh them.");
  } else if (loadedBundles.length > 0) {
    summary.push("Module roots changed; the next run/test will reload bundles automatically.");
  } else {
    summary.push("Use Load Modules now or let run/test autoload the configured bundles.");
  }

  return summary.join("\n");
}

function formatRuntimeFrame(frame) {
  const location = frame?.source_location;
  if (location) {
    return `  at ${frame.function} (${location.path}:${location.line}:${location.column})`;
  }
  return `  at ${frame.function}`;
}

function formatSourceExcerpt(excerpt) {
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

function decorateDiagnosticMessage(diagnostic, message) {
  if (!message) {
    return "";
  }
  switch (diagnostic?.severity) {
    case "warning":
      return `warning: ${message}`;
    case "info":
      return `info: ${message}`;
    default:
      return message;
  }
}

function diagnosticLeadMessage(diagnostic) {
  const runtime = diagnostic?.runtime;
  if (runtime?.root_message) {
    return runtime.root_message;
  }
  const message = diagnostic?.message || "";
  if (diagnostic?.source_excerpt && message.includes("\n")) {
    return message.split("\n", 1)[0];
  }
  return message;
}

function formatTestDetails(runner, details) {
  if (!details || typeof details !== "object") {
    return "";
  }

  const lines = [];
  if (details.subject_path) {
    lines.push(`${runner === "package" ? "Target" : "Entry"}: ${details.subject_path}`);
  }

  const plannedTests = Array.isArray(details.planned_tests) ? details.planned_tests : [];
  const completedTests = Array.isArray(details.completed_tests) ? details.completed_tests : [];
  const activeTest = details.active_test || null;

  if (plannedTests.length > 0) {
    lines.push(`Planned: ${plannedTests.join(", ")}`);
  }
  if (completedTests.length > 0) {
    lines.push(`Completed: ${completedTests.join(", ")}`);
  }
  if (activeTest) {
    lines.push(`Stopped in: ${activeTest}`);
  }

  const pendingTests = plannedTests.filter(
    (name) => !completedTests.includes(name) && name !== activeTest,
  );
  if (pendingTests.length > 0) {
    lines.push(`Not run: ${pendingTests.join(", ")}`);
  }

  return lines.join("\n");
}

function formatDiagnostic(diagnostic) {
  const runtime = diagnostic?.runtime;
  const sections = [decorateDiagnosticMessage(diagnostic, diagnosticLeadMessage(diagnostic))];

  const excerpt = formatSourceExcerpt(diagnostic?.source_excerpt);
  if (excerpt) {
    sections.push(excerpt);
  }

  if (diagnostic?.suggested_action) {
    sections.push(`suggestion: ${diagnostic.suggested_action}`);
  }

  if (runtime && Array.isArray(runtime.stack_trace) && runtime.stack_trace.length > 0) {
    sections.push("stack trace:");
    sections.push(...runtime.stack_trace.map(formatRuntimeFrame));
  }

  return sections.filter(Boolean).join("\n");
}
