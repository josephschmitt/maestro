import type { FocusRegionId } from './types.js';
import { registerElement, unregisterElement, setActiveRegion } from './context.js';

export interface FocusableOptions {
	id: string;
	region: FocusRegionId;
	role?: string;
}

let counter = 0;

function generateId(): string {
	return `focusable-${++counter}`;
}

export function focusable(node: HTMLElement, options: FocusableOptions) {
	const elementId = options.id || generateId();
	let currentRegion = options.region;

	node.setAttribute('tabindex', '0');
	if (options.role) {
		node.setAttribute('role', options.role);
	}

	function handleFocus() {
		setActiveRegion(currentRegion);
	}

	node.addEventListener('focus', handleFocus);

	registerElement({
		id: elementId,
		region: currentRegion,
		element: node
	});

	return {
		update(newOptions: FocusableOptions) {
			if (newOptions.region !== currentRegion) {
				unregisterElement(elementId, currentRegion);
				currentRegion = newOptions.region;
				registerElement({
					id: elementId,
					region: currentRegion,
					element: node
				});
			}
			if (newOptions.role) {
				node.setAttribute('role', newOptions.role);
			}
		},
		destroy() {
			node.removeEventListener('focus', handleFocus);
			unregisterElement(elementId, currentRegion);
		}
	};
}
