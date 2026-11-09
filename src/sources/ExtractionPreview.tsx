import type { SourceExtraction } from '../application/sourceTypes';

const statuses = {
  complete: 'Extraction complete for this source.',
  partial: 'Extraction is partial. Some source content is outside this preview.',
  attachmentOnly: 'Original only. This format has no qualified extraction handler yet.',
  failed: 'Extraction failed. The preserved original is still available.',
};
const diagnostics = {
  unsupportedFormat: 'You can still inspect the preserved original.',
  inputLimit: 'The file exceeds the extraction preview byte limit.',
  invalidUtf8: 'This handler requires valid UTF-8 text.',
  malformedInput: 'The file could not be parsed in the selected format.',
  recordLimit: 'The record limit was reached; more records may exist.',
  fieldLimit: 'A record exceeds the supported field limit.',
};
const locators = { line: 'Line', csvRecord: 'CSV record', jsonPointer: 'JSON Pointer' };

/** Derived records are always passive text, including formulas and markup. */
export function ExtractionPreview({ extraction }: { extraction: SourceExtraction }) {
  if (extraction.contractVersion !== 1 || !Object.hasOwn(statuses, extraction.status)) {
    return <p>Extraction preview is unavailable in this version. The raw original preview remains available.</p>;
  }
  return <section aria-label="Extracted source preview">
    <h5>Extracted source preview</h5>
    <p>{statuses[extraction.status]}</p>
    {extraction.diagnostic && <p>{diagnostics[extraction.diagnostic] ?? 'Extraction has an unsupported diagnostic. Inspect the original.'}</p>}
    <p>Review these source records before creating or updating learning material.</p>
    <ol>{extraction.records.slice(0, 128).map((record, index) => <li key={index}>
      <p>{locators[record.locator.kind] ?? 'Source location'} {record.locator.value || '(root)'}</p>
      {record.fields.slice(0, 64).map((field, fieldIndex) => <pre key={fieldIndex}>{field}</pre>)}
    </li>)}</ol>
  </section>;
}
