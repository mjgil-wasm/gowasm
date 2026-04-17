export function isCancellableRequestKind(kind) {
  return kind === "run" || kind === "test_package" || kind === "test_snippet";
}

export function cancellationPendingStatus(kind) {
  if (kind === "test_snippet") {
    return "Cancelling snippet test…";
  }
  if (kind === "test_package") {
    return "Cancelling package tests…";
  }
  return "Cancelling run…";
}

export function cancelledRequestView(kind) {
  if (kind === "test_snippet") {
    return {
      statusText: "Snippet test cancelled",
      outputText: "Snippet test cancelled. Worker ready for another request.",
    };
  }
  if (kind === "test_package") {
    return {
      statusText: "Package test cancelled",
      outputText: "Package test cancelled. Worker ready for another request.",
    };
  }
  return {
    statusText: "Run cancelled",
    outputText: "Execution cancelled. Worker ready for another request.",
  };
}

export function cancellationTimeoutView(kind) {
  const label = requestLabel(kind);
  return {
    outputText:
      `${label} cancellation timed out before the worker yielded. `
      + "Restarting worker for recovery…",
    readySuffix: "recovered after cancellation timeout",
    statusText: `Restarting worker after ${label.toLowerCase()} cancellation timeout...`,
  };
}

function requestLabel(kind) {
  if (kind === "test_snippet") {
    return "Snippet test";
  }
  if (kind === "test_package") {
    return "Package test";
  }
  return "Run";
}
