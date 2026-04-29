import {
  isSupportedEditableWorkspacePath,
  normalizeWorkspacePath,
} from "./browser-workspace.js";
const TEXT_DECODER = new TextDecoder("utf-8", { fatal: true });

export async function importProjectArchiveUrl(url) {
  const trimmedUrl = String(url ?? "").trim();
  if (!trimmedUrl) {
    throw new Error("Archive URL is required.");
  }

  const response = await fetch(trimmedUrl);
  if (!response.ok) {
    throw new Error(`Archive download failed: ${response.status} ${response.statusText}`.trim());
  }

  const buffer = await response.arrayBuffer();
  return importProjectArchiveBytes(buffer, { sourceLabel: trimmedUrl });
}

export async function importProjectArchiveBytes(bytes, { sourceLabel = "archive.zip" } = {}) {
  const entries = await parseZipEntries(bytes);
  if (entries.length === 0) {
    throw new Error("Archive import failed: the ZIP archive did not contain any files.");
  }

  const decodedEntries = [];
  for (const entry of entries) {
    if (entry.isDirectory) {
      continue;
    }

    const path = normalizeArchiveEntryPath(entry.path);
    const contents = decodeArchiveText(entry.bytes, path);
    validateSupportedArchiveFile(path, contents);
    decodedEntries.push({ path, contents });
  }

  if (decodedEntries.length === 0) {
    throw new Error("Archive import failed: the ZIP archive only contained directories.");
  }

  const imported = stripArchiveImportPrefix(decodedEntries);
  const orderedFiles = imported.files
    .slice()
    .sort((left, right) => left.path.localeCompare(right.path));
  const entryPath = detectEntryPath(orderedFiles);

  return {
    files: orderedFiles,
    sourceLabel,
    strippedPrefix: imported.strippedPrefix,
    ignoredPaths: imported.ignoredPaths,
    entryPath,
    packageTargetPath: entryPath && entryPath.endsWith(".go") ? entryPath : "",
  };
}

export function formatArchiveImportSummary(result) {
  const lines = [
    `Imported ${result.files.length} file(s) from ${result.sourceLabel}.`,
  ];
  if (result.strippedPrefix) {
    lines.push(`Stripped archive prefix: ${result.strippedPrefix}`);
  }
  if (result.entryPath) {
    lines.push(`Selected entry path: ${result.entryPath}`);
  }
  if (result.ignoredPaths.length > 0) {
    lines.push(`Ignored ${result.ignoredPaths.length} file(s) outside the detected module root.`);
  }
  lines.push("");
  lines.push("Imported files:");
  for (const file of result.files) {
    lines.push(`- ${file.path}`);
  }
  return lines.join("\n");
}

function normalizeArchiveEntryPath(rawPath) {
  const trimmedPath = String(rawPath ?? "").trim().replace(/\\/g, "/");
  if (!trimmedPath) {
    throw new Error("Archive import failed: archive entries must have a path.");
  }
  if (/^[A-Za-z]:\//.test(trimmedPath) || trimmedPath.startsWith("/")) {
    throw new Error(`Archive import failed: absolute archive path ${JSON.stringify(trimmedPath)} is not allowed.`);
  }

  const normalizedSegments = [];
  for (const segment of trimmedPath.split("/")) {
    if (!segment || segment === ".") {
      continue;
    }
    if (segment === "..") {
      throw new Error(
        `Archive import failed: path traversal entry ${JSON.stringify(trimmedPath)} is not allowed.`,
      );
    }
    normalizedSegments.push(segment);
  }

  const normalizedPath = normalizeWorkspacePath(normalizedSegments.join("/"));
  if (!normalizedPath) {
    throw new Error("Archive import failed: archive entries cannot normalize to an empty path.");
  }
  if (normalizedPath.startsWith("__module_cache__/")) {
    throw new Error(
      `Archive import failed: ${JSON.stringify(normalizedPath)} is reserved for projected module-cache files.`,
    );
  }
  return normalizedPath;
}

function decodeArchiveText(bytes, path) {
  let contents;
  try {
    contents = TEXT_DECODER.decode(bytes);
  } catch {
    throw new Error(
      `Archive import failed: ${path} is not valid UTF-8 text and falls outside the supported browser workspace boundary.`,
    );
  }

  if (contents.includes("\u0000")) {
    throw new Error(
      `Archive import failed: ${path} contains NUL bytes and falls outside the supported browser workspace boundary.`,
    );
  }
  return contents;
}

function validateSupportedArchiveFile(path, contents) {
  if (!isSupportedEditableWorkspacePath(path)) {
    throw new Error(
      `Archive import failed: ${path} is not a supported editable workspace file type.`,
    );
  }

  if (/[\u0001-\u0008\u000b\u000c\u000e-\u001f]/.test(contents)) {
    throw new Error(
      `Archive import failed: ${path} contains unsupported control bytes and is treated as a non-text file.`,
    );
  }
}

function stripArchiveImportPrefix(entries) {
  const rootGoMod = entries.some((entry) => entry.path === "go.mod");
  if (rootGoMod) {
    return { files: entries, strippedPrefix: "", ignoredPaths: [] };
  }

  const goModEntries = entries.filter((entry) => entry.path.endsWith("/go.mod"));
  if (goModEntries.length > 1) {
    throw new Error(
      "Archive import failed: multiple nested go.mod roots were found; import a single-module project archive.",
    );
  }
  if (goModEntries.length === 1) {
    const prefix = goModEntries[0].path.slice(0, -"/go.mod".length);
    return stripPrefix(entries, prefix);
  }

  const commonPrefix = commonDirectoryPrefix(entries.map((entry) => entry.path));
  if (!commonPrefix) {
    return { files: entries, strippedPrefix: "", ignoredPaths: [] };
  }
  return stripPrefix(entries, commonPrefix);
}

function stripPrefix(entries, prefix) {
  const normalizedPrefix = normalizeWorkspacePath(prefix);
  const prefixMarker = normalizedPrefix ? `${normalizedPrefix}/` : "";
  const strippedFiles = [];
  const ignoredPaths = [];

  for (const entry of entries) {
    if (!prefixMarker || entry.path === normalizedPrefix || entry.path.startsWith(prefixMarker)) {
      const strippedPath = prefixMarker ? entry.path.slice(prefixMarker.length) : entry.path;
      if (!strippedPath || strippedPath === "go.mod" || strippedPath.includes("/")) {
        strippedFiles.push({
          path: strippedPath || entry.path,
          contents: entry.contents,
        });
      } else {
        strippedFiles.push({ path: strippedPath, contents: entry.contents });
      }
      continue;
    }
    ignoredPaths.push(entry.path);
  }

  const normalizedFiles = strippedFiles.map((entry) => ({
    path: normalizeWorkspacePath(entry.path),
    contents: entry.contents,
  }));
  if (normalizedFiles.length === 0) {
    throw new Error("Archive import failed: the detected module root did not contain any files.");
  }
  if (normalizedFiles.some((entry) => !entry.path)) {
    throw new Error("Archive import failed: stripping the archive root produced an empty file path.");
  }

  return {
    files: normalizedFiles,
    strippedPrefix: prefixMarker,
    ignoredPaths,
  };
}

function commonDirectoryPrefix(paths) {
  const segments = paths.map((path) => path.split("/"));
  if (segments.length === 0) {
    return "";
  }

  const prefix = [];
  for (let index = 0; ; index += 1) {
    const segment = segments[0][index];
    if (!segment) {
      break;
    }
    if (segments.some((parts) => parts.length <= index + 1 || parts[index] !== segment)) {
      break;
    }
    prefix.push(segment);
  }

  return prefix.join("/");
}

function detectEntryPath(files) {
  if (files.some((file) => file.path === "main.go")) {
    return "main.go";
  }

  const mainCandidates = files
    .filter((file) => file.path.endsWith("/main.go"))
    .map((file) => file.path)
    .sort();
  if (mainCandidates.length > 0) {
    return mainCandidates[0];
  }

  const goCandidates = files
    .filter((file) => file.path.endsWith(".go"))
    .map((file) => file.path)
    .sort();
  return goCandidates[0] ?? files[0]?.path ?? "";
}

async function parseZipEntries(bytes) {
  const buffer = bytes instanceof ArrayBuffer ? bytes : await bytes.arrayBuffer();
  const view = new DataView(buffer);
  const endRecordOffset = findEndOfCentralDirectory(view);
  const centralDirectoryOffset = view.getUint32(endRecordOffset + 16, true);
  const centralDirectoryEntries = view.getUint16(endRecordOffset + 10, true);

  const entries = [];
  let cursor = centralDirectoryOffset;
  for (let index = 0; index < centralDirectoryEntries; index += 1) {
    if (view.getUint32(cursor, true) !== 0x02014b50) {
      throw new Error("Archive import failed: invalid ZIP central directory signature.");
    }

    const flags = view.getUint16(cursor + 8, true);
    const compressionMethod = view.getUint16(cursor + 10, true);
    const compressedSize = view.getUint32(cursor + 20, true);
    const fileNameLength = view.getUint16(cursor + 28, true);
    const extraFieldLength = view.getUint16(cursor + 30, true);
    const fileCommentLength = view.getUint16(cursor + 32, true);
    const localHeaderOffset = view.getUint32(cursor + 42, true);
    const fileNameBytes = new Uint8Array(buffer, cursor + 46, fileNameLength);
    const filePath = decodeZipPath(fileNameBytes, flags);

    const localFile = await readLocalFileRecord(
      buffer,
      localHeaderOffset,
      compressionMethod,
      compressedSize,
    );
    entries.push({
      path: filePath,
      isDirectory: filePath.endsWith("/"),
      bytes: localFile,
    });
    cursor += 46 + fileNameLength + extraFieldLength + fileCommentLength;
  }
  return entries;
}

function findEndOfCentralDirectory(view) {
  const minimumSize = 22;
  const scanStart = Math.max(0, view.byteLength - 0xffff - minimumSize);
  for (let offset = view.byteLength - minimumSize; offset >= scanStart; offset -= 1) {
    if (view.getUint32(offset, true) === 0x06054b50) {
      return offset;
    }
  }
  throw new Error("Archive import failed: ZIP end-of-central-directory record not found.");
}

function decodeZipPath(bytes, flags) {
  if ((flags & 0x800) !== 0) {
    return TEXT_DECODER.decode(bytes);
  }

  let path = "";
  for (const byte of bytes) {
    path += String.fromCharCode(byte);
  }
  return path;
}

async function readLocalFileRecord(buffer, offset, compressionMethod, compressedSize) {
  const view = new DataView(buffer);
  if (view.getUint32(offset, true) !== 0x04034b50) {
    throw new Error("Archive import failed: invalid ZIP local-file signature.");
  }

  const fileNameLength = view.getUint16(offset + 26, true);
  const extraFieldLength = view.getUint16(offset + 28, true);
  const dataOffset = offset + 30 + fileNameLength + extraFieldLength;
  const compressedBytes = new Uint8Array(buffer, dataOffset, compressedSize);

  switch (compressionMethod) {
    case 0:
      return new Uint8Array(compressedBytes);
    case 8:
      return inflateRaw(compressedBytes);
    default:
      throw new Error(
        `Archive import failed: ZIP compression method ${compressionMethod} is not supported.`,
      );
  }
}

async function inflateRaw(bytes) {
  const stream = new Blob([bytes]).stream().pipeThrough(new DecompressionStream("deflate-raw"));
  const response = new Response(stream);
  const buffer = await response.arrayBuffer();
  return new Uint8Array(buffer);
}
