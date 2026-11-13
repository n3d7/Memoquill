import type { Envelope } from './contentTypes';

export interface SelectedSource { digest: string; byteLength: number; filename: string }
export interface NewSource {
  id: string; resourceId: string; digest: string; filename: string; mediaType: string;
  attribution: string | null; license: string | null; originUrl: string | null;
}
export interface OriginalSource extends NewSource { revision: number; byteLength: number }
export interface SourceList { revision: number; sources: OriginalSource[] }
export interface SourcePreview { source: OriginalSource; text: string | null; hex: string | null; truncated: boolean; extraction?: SourceExtraction }
export interface SourceExtraction {
  contractVersion: number;
  format: 'text' | 'markdown' | 'csv' | 'json' | 'unsupported';
  status: 'complete' | 'partial' | 'attachmentOnly' | 'failed';
  records: { locator: { kind: 'line' | 'csvRecord' | 'jsonPointer'; value: string }; fields: string[]; properties?: Record<string, string>; duplicateProperties?: string[] }[];
  diagnostic: 'unsupportedFormat' | 'inputLimit' | 'invalidUtf8' | 'malformedInput' | 'recordLimit' | 'fieldLimit' | null;
}
export type SaveSource = Envelope & { source: NewSource };
