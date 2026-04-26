const TEXT_ENCODER = new TextEncoder();
const CRC32_TABLE = buildCrc32Table();

export function createArchiveDataUrl(entries) {
  const bytes = createStoredZipBytes(entries);
  return `data:application/zip;base64,${base64Encode(bytes)}`;
}

export function createArchiveFile(view, name, entries) {
  return new view.File([createStoredZipBytes(entries)], name, {
    type: "application/zip",
  });
}

function createStoredZipBytes(entries) {
  const normalizedEntries = (entries ?? []).map((entry) => normalizeArchiveEntry(entry));
  const localParts = [];
  const centralParts = [];
  let offset = 0;

  for (const entry of normalizedEntries) {
    const localHeader = encodeLocalFileHeader(entry);
    localParts.push(localHeader, entry.pathBytes, entry.contentsBytes);

    const centralHeader = encodeCentralDirectoryHeader(entry, offset);
    centralParts.push(centralHeader, entry.pathBytes);
    offset += localHeader.length + entry.pathBytes.length + entry.contentsBytes.length;
  }

  const centralDirectoryOffset = offset;
  const centralDirectorySize = centralParts.reduce((size, part) => size + part.length, 0);
  const endRecord = encodeEndOfCentralDirectory(
    normalizedEntries.length,
    centralDirectorySize,
    centralDirectoryOffset,
  );

  return concatUint8Arrays([...localParts, ...centralParts, endRecord]);
}

function normalizeArchiveEntry(entry) {
  const pathBytes = TEXT_ENCODER.encode(String(entry?.path ?? ""));
  const contentsBytes =
    entry?.contents instanceof Uint8Array
      ? entry.contents
      : TEXT_ENCODER.encode(String(entry?.contents ?? ""));

  return {
    compressedSize: contentsBytes.length,
    contentsBytes,
    crc32: crc32(contentsBytes),
    pathBytes,
    uncompressedSize: contentsBytes.length,
  };
}

function encodeLocalFileHeader(entry) {
  const header = new Uint8Array(30);
  const view = new DataView(header.buffer);
  view.setUint32(0, 0x04034b50, true);
  view.setUint16(4, 20, true);
  view.setUint16(6, 0x0800, true);
  view.setUint16(8, 0, true);
  view.setUint16(10, 0, true);
  view.setUint16(12, 0, true);
  view.setUint32(14, entry.crc32, true);
  view.setUint32(18, entry.compressedSize, true);
  view.setUint32(22, entry.uncompressedSize, true);
  view.setUint16(26, entry.pathBytes.length, true);
  view.setUint16(28, 0, true);
  return header;
}

function encodeCentralDirectoryHeader(entry, localHeaderOffset) {
  const header = new Uint8Array(46);
  const view = new DataView(header.buffer);
  view.setUint32(0, 0x02014b50, true);
  view.setUint16(4, 20, true);
  view.setUint16(6, 20, true);
  view.setUint16(8, 0x0800, true);
  view.setUint16(10, 0, true);
  view.setUint16(12, 0, true);
  view.setUint16(14, 0, true);
  view.setUint32(16, entry.crc32, true);
  view.setUint32(20, entry.compressedSize, true);
  view.setUint32(24, entry.uncompressedSize, true);
  view.setUint16(28, entry.pathBytes.length, true);
  view.setUint16(30, 0, true);
  view.setUint16(32, 0, true);
  view.setUint16(34, 0, true);
  view.setUint16(36, 0, true);
  view.setUint32(38, 0, true);
  view.setUint32(42, localHeaderOffset, true);
  return header;
}

function encodeEndOfCentralDirectory(entryCount, centralDirectorySize, centralDirectoryOffset) {
  const record = new Uint8Array(22);
  const view = new DataView(record.buffer);
  view.setUint32(0, 0x06054b50, true);
  view.setUint16(4, 0, true);
  view.setUint16(6, 0, true);
  view.setUint16(8, entryCount, true);
  view.setUint16(10, entryCount, true);
  view.setUint32(12, centralDirectorySize, true);
  view.setUint32(16, centralDirectoryOffset, true);
  view.setUint16(20, 0, true);
  return record;
}

function concatUint8Arrays(parts) {
  const totalLength = parts.reduce((size, part) => size + part.length, 0);
  const combined = new Uint8Array(totalLength);
  let offset = 0;
  for (const part of parts) {
    combined.set(part, offset);
    offset += part.length;
  }
  return combined;
}

function crc32(bytes) {
  let crc = 0xffffffff;
  for (const byte of bytes) {
    crc = (crc >>> 8) ^ CRC32_TABLE[(crc ^ byte) & 0xff];
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function buildCrc32Table() {
  const table = new Uint32Array(256);
  for (let index = 0; index < table.length; index += 1) {
    let value = index;
    for (let bit = 0; bit < 8; bit += 1) {
      value = (value & 1) !== 0 ? 0xedb88320 ^ (value >>> 1) : value >>> 1;
    }
    table[index] = value >>> 0;
  }
  return table;
}

function base64Encode(bytes) {
  let binary = "";
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }
  return btoa(binary);
}
