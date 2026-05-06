export class CancelledRunError extends Error {}

export async function buildCapabilityResumeRequest(response, runState) {
  if (runState?.cancelled) {
    throw new CancelledRunError();
  }
  if (response.capability?.kind === "clock_now") {
    return {
      kind: "resume",
      run_id: response.run_id,
      capability: {
        kind: "clock_now",
        unix_millis: Date.now(),
      },
    };
  }
  if (response.capability?.kind === "sleep") {
    await waitForAbortableDelay(response.capability.duration_millis, runState);
    return {
      kind: "resume",
      run_id: response.run_id,
      capability: {
        kind: "sleep",
        unix_millis: Date.now(),
      },
    };
  }
  if (response.capability?.kind === "fetch") {
    return {
      kind: "resume",
      run_id: response.run_id,
      capability: {
        kind: "fetch",
        result: await performFetchCapability(response.capability.request, runState),
      },
    };
  }
  if (response.capability?.kind === "fetch_start") {
    return {
      kind: "resume",
      run_id: response.run_id,
      capability: await performFetchStartCapability(response.capability.request, runState),
    };
  }
  if (response.capability?.kind === "fetch_body_chunk") {
    return {
      kind: "resume",
      run_id: response.run_id,
      capability: await performFetchBodyChunkCapability(response.capability.request, runState),
    };
  }
  if (response.capability?.kind === "fetch_body_complete") {
    return {
      kind: "resume",
      run_id: response.run_id,
      capability: await performFetchBodyCompleteCapability(
        response.capability.request,
        runState,
      ),
    };
  }
  if (response.capability?.kind === "fetch_body_abort") {
    return {
      kind: "resume",
      run_id: response.run_id,
      capability: await performFetchBodyAbortCapability(response.capability.request, runState),
    };
  }
  if (response.capability?.kind === "fetch_response_chunk") {
    return {
      kind: "resume",
      run_id: response.run_id,
      capability: await performFetchResponseChunkCapability(
        response.capability.request,
        runState,
      ),
    };
  }
  if (response.capability?.kind === "fetch_response_close") {
    return {
      kind: "resume",
      run_id: response.run_id,
      capability: await performFetchResponseCloseCapability(
        response.capability.request,
        runState,
      ),
    };
  }
  if (response.capability?.kind === "yield") {
    await waitForAbortableDelay(0, runState);
    return {
      kind: "resume",
      run_id: response.run_id,
      capability: {
        kind: "yield",
      },
    };
  }
  throw new Error(`unsupported engine capability request: ${JSON.stringify(response)}`);
}

export function disposeFetchSessions(runState) {
  for (const sessionId of Array.from(runState?.fetchSessions?.keys?.() ?? [])) {
    const session = clearFetchSession(runState, sessionId);
    try {
      session.bodyController?.error(new Error("fetch session disposed"));
    } catch {
      // Ignore repeated errors after the stream is already closed.
    }
    try {
      void session.responseReader?.cancel("fetch session disposed");
    } catch {
      // Ignore repeated cancellation after the reader is already closed.
    }
    session.controller.abort();
  }
}

export function withHostClock(request) {
  if (request?.kind !== "run") {
    return request;
  }
  if (
    request.host_time_unix_nanos !== undefined ||
    request.host_time_unix_millis !== undefined
  ) {
    return request;
  }
  return {
    ...request,
    host_time_unix_millis: Date.now(),
  };
}

export function formatError(error) {
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}

async function performFetchCapability(request, runState) {
  const controller = new AbortController();
  const abortCapability = () => controller.abort();
  const deadlineAbort = installFetchDeadlineAbort(
    controller,
    request.context_deadline_unix_millis,
  );
  let clearAbortCapability = () => {};
  try {
    clearAbortCapability = installAbortableCapability(runState, abortCapability);

    const init = {
      method: request.method,
      headers: buildFetchRequestHeaders(request.headers),
      signal: controller.signal,
    };
    if (Array.isArray(request.body) && request.body.length > 0) {
      if (request.method === "GET" || request.method === "HEAD") {
        throw new Error(`fetch rejected ${request.method} body payload`);
      }
      init.body = new Uint8Array(request.body);
    }

    const response = await fetch(request.url, init);
    const body = fetchResponseMustNotHaveBody(request.method, response.status)
      ? new Uint8Array(0)
      : new Uint8Array(await response.arrayBuffer());
    return {
      kind: "response",
      response: {
        status_code: response.status,
        status: formatFetchStatus(response.status, response.statusText),
        url: normalizeFetchResponseUrl(response.url, request.url),
        headers: collectFetchResponseHeaders(response.headers),
        body: Array.from(body),
      },
    };
  } catch (error) {
    if (runState?.cancelled) {
      throw new CancelledRunError();
    }
    if (deadlineAbort.didExpire()) {
      return {
        kind: "error",
        message: "context deadline exceeded",
      };
    }
    return {
      kind: "error",
      message: `fetch failed for ${request.method} ${request.url}: ${formatError(error)}`,
    };
  } finally {
    deadlineAbort.clear();
    clearAbortCapability();
  }
}

async function performFetchStartCapability(request, runState) {
  const controller = new AbortController();
  const session = {
    bodyController: null,
    clearAbortCapability: null,
    controller,
    deadlineExceeded: false,
    requestLabel: `${request.method} ${request.url}`,
    responseDone: false,
    responsePending: new Uint8Array(0),
    responseReader: null,
    responsePromise: null,
  };
  const body = new ReadableStream({
    start(controller) {
      session.bodyController = controller;
    },
  });
  const abortSession = () => {
    try {
      session.bodyController?.error(new Error("request body stream aborted"));
    } catch {
      // Ignore repeated aborts after the stream is already closed.
    }
    try {
      void session.responseReader?.cancel("fetch session aborted");
    } catch {
      // Ignore repeated aborts after the reader is already closed.
    }
    controller.abort();
  };
  const clearAbortCapability = installAbortableCapability(runState, abortSession);
  const clearDeadlineAbort = installFetchDeadlineAbort(
    controller,
    request.context_deadline_unix_millis,
    () => {
      session.deadlineExceeded = true;
    },
  );
  const init = {
    method: request.method,
    headers: buildFetchRequestHeaders(request.headers),
    signal: controller.signal,
    body,
    duplex: "half",
  };

  session.clearAbortCapability = () => {
    clearDeadlineAbort.clear();
    clearAbortCapability();
  };
  session.responsePromise = fetch(request.url, init)
    .then(async (response) => {
      const responseStart = {
        status_code: response.status,
        status: formatFetchStatus(response.status, response.statusText),
        url: normalizeFetchResponseUrl(response.url, request.url),
        headers: collectFetchResponseHeaders(response.headers),
      };
      if (
        fetchResponseMustNotHaveBody(request.method, response.status) ||
        !response.body ||
        typeof response.body.getReader !== "function"
      ) {
        return {
          kind: "response",
          response: {
            ...responseStart,
            body: [],
          },
        };
      }
      session.responseReader = response.body.getReader();
      return {
        kind: "response_start",
        response: responseStart,
      };
    })
    .catch((error) => {
      if (session.deadlineExceeded) {
        return {
          kind: "error",
          message: "context deadline exceeded",
        };
      }
      return {
        kind: "error",
        message: `fetch failed for ${request.method} ${request.url}: ${formatError(error)}`,
      };
    });

  runState.fetchSessions.set(request.session_id, session);
  return { kind: "fetch_start" };
}

async function performFetchBodyChunkCapability(request, runState) {
  const session = requireFetchSession(runState, request.session_id);
  if (session.deadlineExceeded) {
    return { kind: "fetch_body_chunk" };
  }
  const chunk = Array.isArray(request.chunk) ? request.chunk : [];
  if (chunk.length > 0) {
    session.bodyController?.enqueue(new Uint8Array(chunk));
  }
  return { kind: "fetch_body_chunk" };
}

async function performFetchBodyCompleteCapability(request, runState) {
  const session = requireFetchSession(runState, request.session_id);
  session.bodyController?.close();
  const result = await session.responsePromise;
  if (result.kind !== "response_start") {
    clearFetchSession(runState, request.session_id);
  }
  if (runState?.cancelled) {
    throw new CancelledRunError();
  }
  return {
    kind: "fetch_body_complete",
    result,
  };
}

async function performFetchBodyAbortCapability(request, runState) {
  const session = clearFetchSession(runState, request.session_id);
  try {
    session.bodyController?.error(new Error("request body stream aborted"));
  } catch {
    // Ignore repeated aborts after the stream is already closed.
  }
  session.controller.abort();
  return { kind: "fetch_body_abort" };
}

async function performFetchResponseChunkCapability(request, runState) {
  const session = requireFetchSession(runState, request.session_id);
  const result = await nextFetchResponseChunk(
    session,
    normalizeFetchResponseMaxBytes(request.max_bytes),
  );
  if (result.kind === "error" || (result.kind === "chunk" && result.eof)) {
    clearFetchSession(runState, request.session_id);
  }
  return {
    kind: "fetch_response_chunk",
    result,
  };
}

async function performFetchResponseCloseCapability(request, runState) {
  const session = clearFetchSession(runState, request.session_id);
  try {
    await session.responseReader?.cancel("response body closed");
  } catch {
    // Ignore repeated close/cancel after the reader is already closed.
  }
  session.controller.abort();
  return { kind: "fetch_response_close" };
}

function normalizeFetchResponseMaxBytes(maxBytes) {
  const numeric = Math.trunc(Number(maxBytes));
  if (!Number.isFinite(numeric) || numeric <= 0) {
    return 0;
  }
  return numeric;
}

async function nextFetchResponseChunk(session, maxBytes) {
  if (maxBytes <= 0) {
    return {
      kind: "chunk",
      chunk: [],
      eof: false,
    };
  }

  for (;;) {
    if (session.responsePending.length > 0) {
      const chunk = session.responsePending.slice(0, maxBytes);
      session.responsePending = session.responsePending.slice(chunk.length);
      return {
        kind: "chunk",
        chunk: Array.from(chunk),
        eof: false,
      };
    }

    if (session.responseDone || !session.responseReader) {
      session.responseDone = true;
      return {
        kind: "chunk",
        chunk: [],
        eof: true,
      };
    }

    try {
      const { value, done } = await session.responseReader.read();
      if (done) {
        session.responseDone = true;
        continue;
      }

      const chunk =
        value instanceof Uint8Array ? value : new Uint8Array(value ?? []);
      if (chunk.length === 0) {
        continue;
      }
      session.responsePending = chunk;
    } catch (error) {
      if (session.deadlineExceeded) {
        return {
          kind: "error",
          message: "context deadline exceeded",
        };
      }
      return {
        kind: "error",
        message: `fetch response body read failed for ${session.requestLabel}: ${formatError(error)}`,
      };
    }
  }
}

function requireFetchSession(runState, sessionId) {
  const session = runState?.fetchSessions?.get(sessionId);
  if (!session) {
    throw new Error(`missing fetch session ${sessionId}`);
  }
  return session;
}

function clearFetchSession(runState, sessionId) {
  const session = requireFetchSession(runState, sessionId);
  runState.fetchSessions.delete(sessionId);
  session.clearAbortCapability?.();
  return session;
}

function buildFetchRequestHeaders(headers) {
  const result = new Headers();
  for (const header of headers ?? []) {
    const values = Array.isArray(header?.values) ? header.values : [];
    if (values.length === 0) {
      result.append(header.name, "");
      continue;
    }
    for (const value of values) {
      result.append(header.name, value);
    }
  }
  return result;
}

function collectFetchResponseHeaders(headers) {
  const collected = new Map();
  for (const [name, value] of headers.entries()) {
    const existing = collected.get(name);
    if (existing) {
      existing.push(value);
    } else {
      collected.set(name, [value]);
    }
  }
  const setCookieValues = getFetchSetCookieValues(headers);
  if (setCookieValues.length > 0) {
    collected.set("set-cookie", setCookieValues);
  }
  return Array.from(collected, ([name, values]) => ({ name, values }));
}

function getFetchSetCookieValues(headers) {
  if (typeof headers?.getSetCookie !== "function") {
    return [];
  }
  const values = headers.getSetCookie();
  if (!Array.isArray(values)) {
    return [];
  }
  return values.map((value) => String(value));
}

function normalizeFetchResponseUrl(responseUrl, fallbackUrl) {
  if (typeof responseUrl === "string" && responseUrl.length > 0) {
    return responseUrl;
  }
  return String(fallbackUrl ?? "");
}

function fetchResponseMustNotHaveBody(method, statusCode) {
  if (method === "HEAD") {
    return true;
  }
  return (
    (statusCode >= 100 && statusCode < 200) ||
    statusCode === 204 ||
    statusCode === 205 ||
    statusCode === 304
  );
}

function formatFetchStatus(statusCode, statusText) {
  if (!statusText) {
    return "";
  }
  return `${statusCode} ${statusText}`;
}

function waitForAbortableDelay(durationMillis, runState) {
  return new Promise((resolve, reject) => {
    const timerId = self.setTimeout(() => {
      clearAbortCapability();
      if (runState?.cancelled) {
        reject(new CancelledRunError());
        return;
      }
      resolve();
    }, Math.max(0, durationMillis ?? 0));

    function abortDelay() {
      self.clearTimeout(timerId);
      clearAbortCapability();
      reject(new CancelledRunError());
    }

    let clearAbortCapability;
    try {
      clearAbortCapability = installAbortableCapability(runState, abortDelay);
    } catch (error) {
      self.clearTimeout(timerId);
      reject(error);
    }
  });
}

function installAbortableCapability(runState, abortCapability) {
  if (!runState) {
    return () => {};
  }
  if (runState.cancelled) {
    throw new CancelledRunError();
  }
  runState.abortCapability = abortCapability;
  return () => {
    if (runState.abortCapability === abortCapability) {
      runState.abortCapability = null;
    }
  };
}

function installFetchDeadlineAbort(
  controller,
  deadlineUnixMillis,
  onExpire = () => {},
) {
  const normalizedDeadline = normalizeContextDeadlineUnixMillis(deadlineUnixMillis);
  if (normalizedDeadline === null) {
    return {
      clear() {},
      didExpire() {
        return false;
      },
    };
  }

  let expired = false;
  const expire = () => {
    if (expired) {
      return;
    }
    expired = true;
    onExpire();
    controller.abort();
  };

  const delayMillis = normalizedDeadline - Date.now();
  let timer = null;
  if (delayMillis <= 0) {
    expire();
  } else {
    timer = setTimeout(expire, delayMillis);
  }

  return {
    clear() {
      if (timer !== null) {
        clearTimeout(timer);
        timer = null;
      }
    },
    didExpire() {
      return expired;
    },
  };
}

function normalizeContextDeadlineUnixMillis(deadlineUnixMillis) {
  if (deadlineUnixMillis === null || deadlineUnixMillis === undefined) {
    return null;
  }
  const numeric = Math.trunc(Number(deadlineUnixMillis));
  if (!Number.isFinite(numeric)) {
    return null;
  }
  return numeric;
}
