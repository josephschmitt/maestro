export type FocusRegionId = 'sidebar' | 'board' | 'card-detail' | 'dialog';

export interface FocusElement {
	id: string;
	region: FocusRegionId;
	element: HTMLElement;
}

export interface FocusStackEntry {
	region: FocusRegionId;
	previouslyFocusedElement: HTMLElement | null;
}

export interface FocusState {
	activeRegion: FocusRegionId | null;
	stack: FocusStackEntry[];
	elementsByRegion: Map<FocusRegionId, FocusElement[]>;
}
