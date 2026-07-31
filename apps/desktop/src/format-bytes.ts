import type { FormattedByteSize } from "@partman/ui";

const IEC_UNITS = [
  { bytes: 1n, symbol: "B" },
  { bytes: 1_024n, symbol: "KiB" },
  { bytes: 1_048_576n, symbol: "MiB" },
  { bytes: 1_073_741_824n, symbol: "GiB" },
  { bytes: 1_099_511_627_776n, symbol: "TiB" },
  { bytes: 1_125_899_906_842_624n, symbol: "PiB" },
  { bytes: 1_152_921_504_606_846_976n, symbol: "EiB" },
] as const;

function requireUnsigned(bytes: bigint): void {
  if (bytes < 0n) {
    throw new RangeError("bytes must be non-negative");
  }
}

function roundToTenths(bytes: bigint, unitBytes: bigint): bigint {
  return (bytes * 10n + unitBytes / 2n) / unitBytes;
}

function formatTenths(tenths: bigint, locale: string): string {
  const digits = new Intl.NumberFormat(locale, { useGrouping: false });
  const decimal =
    new Intl.NumberFormat(locale)
      .formatToParts(1.1)
      .find((part) => part.type === "decimal")?.value ?? ".";
  return `${digits.format(tenths / 10n)}${decimal}${digits.format(tenths % 10n)}`;
}

export function formatIecBytes(
  bytes: bigint,
  locale = "en-US",
): string {
  requireUnsigned(bytes);

  if (bytes < IEC_UNITS[1].bytes) {
    return `${new Intl.NumberFormat(locale, { useGrouping: false }).format(bytes)} B`;
  }

  let unit: (typeof IEC_UNITS)[number] = IEC_UNITS[1];
  let nextUnit: (typeof IEC_UNITS)[number] | undefined;
  for (const candidate of IEC_UNITS.slice(2)) {
    if (bytes < candidate.bytes) {
      nextUnit = candidate;
      break;
    }
    unit = candidate;
  }

  let tenths = roundToTenths(bytes, unit.bytes);

  // Promote a value that rounds to 1024.0 so the display never emits that
  // non-canonical boundary spelling (for example, 1024.0 MiB).
  if (tenths >= 10_240n && nextUnit !== undefined) {
    unit = nextUnit;
    tenths = roundToTenths(bytes, unit.bytes);
  }

  return `${formatTenths(tenths, locale)} ${unit.symbol}`;
}

export function formatExactBytes(
  bytes: bigint,
  locale = "en-US",
): string {
  requireUnsigned(bytes);

  return `${new Intl.NumberFormat(locale, {
    maximumFractionDigits: 0,
    useGrouping: true,
  }).format(bytes)} B`;
}

export function formatByteSize(
  bytes: bigint,
  locale = "en-US",
): FormattedByteSize {
  return {
    displaySize: formatIecBytes(bytes, locale),
    exactBytes: formatExactBytes(bytes, locale),
  };
}
