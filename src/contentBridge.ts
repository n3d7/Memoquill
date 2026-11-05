// Compatibility facade. Transport belongs to the native application adapter.
export * from './application/contentTypes';
export { failureMessage, isConfirmedRejection } from './application/errors';
export { library, searchResources, applyContent, readDocument, saveDocument, readingPosition, saveReadingPosition, annotations, saveAnnotation } from './application/nativeContentPort';
