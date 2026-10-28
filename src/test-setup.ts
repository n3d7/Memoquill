import '@testing-library/jest-dom/vitest';
import { afterEach } from 'vitest';
import { cleanup } from '@testing-library/react';

afterEach(cleanup);

// jsdom has no native modal top layer; native focus trapping is qualified in RPM.
HTMLDialogElement.prototype.showModal = function () { this.open = true; };
HTMLDialogElement.prototype.close = function () { this.open = false; this.dispatchEvent(new Event('close')); };
Range.prototype.getClientRects = () => Object.assign([], { item: () => null });
Range.prototype.getBoundingClientRect = () => new DOMRect();
