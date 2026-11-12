export type PackageKind = 'nativeFull' | 'portableContent';
/** Backend-validated manifests are rendered passively, including future optional fields. */
export interface PackageManifest { kind: string; scope: string; sourceCourseId: string; complete?: boolean; fullRecovery?: boolean; missingAssets?: string[]; omissions: string[]; [field: string]: unknown }
export interface PackageSummary { kind: PackageKind; manifest: PackageManifest }
export interface PackageReview extends PackageSummary { reviewId: string }
export interface ConfirmPackage { reviewId: string; operationId: string; title: string; allowIncomplete: boolean }
export interface ImportResult { kind: PackageKind; report: { sourceCourseId: string; courseId: string; omissions: string[]; [field: string]: unknown } }
export interface PackagePort {
 exportPackage(kind: PackageKind, includeOriginals: boolean): Promise<PackageSummary | null>;
 inspect(): Promise<PackageReview | null>;
 confirm(request: ConfirmPackage): Promise<ImportResult>;
 discard(reviewId: string): Promise<void>;
}
