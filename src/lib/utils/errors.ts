export type ErrorSeverity = 'error' | 'warning' | 'info';

export interface AppError {
	message: string;
	severity: ErrorSeverity;
	originalError?: unknown;
}

const ERROR_MESSAGE_MAP: Record<string, string> = {
	'Database error': 'Failed to save changes. Please try again.',
	'No such file or directory': 'The requested file could not be found.',
	'Permission denied': 'You do not have permission to perform this action.',
	'Connection refused': 'Could not connect to the server. Please check your connection.',
	'Entity not found': 'The requested item could not be found.',
	'UNIQUE constraint failed': 'An item with this name already exists.',
	'Foreign key constraint failed': 'This item is referenced by other data and cannot be modified.',
	'timeout': 'The operation took too long. Please try again.',
};

export function formatError(error: unknown): AppError {
	let message = 'An unexpected error occurred. Please try again.';
	let severity: ErrorSeverity = 'error';

	if (error instanceof Error) {
		message = error.message;
	} else if (typeof error === 'string') {
		message = error;
	}

	for (const [pattern, friendlyMessage] of Object.entries(ERROR_MESSAGE_MAP)) {
		if (message.toLowerCase().includes(pattern.toLowerCase())) {
			message = friendlyMessage;
			break;
		}
	}

	if (message.includes('warning')) {
		severity = 'warning';
	}

	return {
		message,
		severity,
		originalError: error,
	};
}

export function getErrorMessage(error: unknown): string {
	return formatError(error).message;
}

export function isNetworkError(error: unknown): boolean {
	const message = error instanceof Error ? error.message : String(error);
	return (
		message.toLowerCase().includes('connection') ||
		message.toLowerCase().includes('network') ||
		message.toLowerCase().includes('fetch') ||
		message.toLowerCase().includes('timeout')
	);
}

export function isNotFoundError(error: unknown): boolean {
	const message = error instanceof Error ? error.message : String(error);
	return (
		message.toLowerCase().includes('not found') ||
		message.toLowerCase().includes('no such file') ||
		message.toLowerCase().includes('entity not found')
	);
}
