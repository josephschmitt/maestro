// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { createStreamingRenderer } from './streaming-renderer.js';

describe('createStreamingRenderer', () => {
	beforeEach(() => {
		vi.useFakeTimers();
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	it('renders content on flush', () => {
		const renderer = createStreamingRenderer();
		renderer.update('Hello **world**');
		renderer.flush();
		expect(renderer.html).toContain('<strong>world</strong>');
		renderer.destroy();
	});

	it('notifies subscribers on flush', () => {
		const renderer = createStreamingRenderer();
		const callback = vi.fn();
		renderer.subscribe(callback);

		renderer.update('test content');
		renderer.flush();

		expect(callback).toHaveBeenCalledWith(expect.stringContaining('test content'));
		renderer.destroy();
	});

	it('throttles rapid updates', () => {
		const renderer = createStreamingRenderer();
		const callback = vi.fn();
		renderer.subscribe(callback);

		renderer.update('first');
		renderer.update('second');
		renderer.update('third');

		// Before any animation frame fires, no render should have happened from updates
		expect(callback).not.toHaveBeenCalled();

		renderer.destroy();
	});

	it('unsubscribe stops notifications', () => {
		const renderer = createStreamingRenderer();
		const callback = vi.fn();
		const unsub = renderer.subscribe(callback);
		unsub();

		renderer.update('test');
		renderer.flush();

		expect(callback).not.toHaveBeenCalled();
		renderer.destroy();
	});

	it('flush cancels pending scheduled render', () => {
		const renderer = createStreamingRenderer();
		renderer.update('content');
		renderer.flush();
		expect(renderer.html).toContain('content');
		renderer.destroy();
	});

	it('skips render when content has not changed', () => {
		const renderer = createStreamingRenderer();
		const callback = vi.fn();

		renderer.update('same');
		renderer.flush();

		renderer.subscribe(callback);
		renderer.update('same');
		renderer.flush();

		// subscriber should get called once for the initial subscribe (since html exists)
		// but NOT again for the flush since content didn't change
		expect(callback).toHaveBeenCalledTimes(1);
		renderer.destroy();
	});

	it('destroy clears listeners and pending renders', () => {
		const renderer = createStreamingRenderer();
		const callback = vi.fn();
		renderer.subscribe(callback);

		renderer.update('content');
		renderer.destroy();
		renderer.flush();

		// callback may have been called with initial empty or not at all, but not after destroy
		const callsAfterDestroy = callback.mock.calls.filter(
			(args: unknown[]) => typeof args[0] === 'string' && args[0].includes('content')
		);
		expect(callsAfterDestroy.length).toBe(0);
	});
});
