import type { Envelope } from './contentTypes';

export interface SelectedSource { digest: string; byteLength: number; filename: string }
export interface NewSource {
  id: string; resourceId: string; digest: string; filename: string; mediaType: string;
  attribution: string | null; license: string | null; originUrl: string | null;
}
export interface OriginalSource extends NewSource { revision: number; byteLength: number }
export interface SourceList { revision: number; sources: OriginalSource[] }
export interface SourcePreview { source: OriginalSource; text: string | null; hex: string | null; truncated: boolean }
export type SaveSource = Envelope & { source: NewSource };
