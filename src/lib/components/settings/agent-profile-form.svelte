<script lang="ts">
	import {
		Dialog,
		DialogContent,
		DialogDescription,
		DialogFooter,
		DialogHeader,
		DialogTitle
	} from '$lib/components/ui/dialog/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import type { AgentProfileConfig, AgentProfileInput } from '$lib/types/index.js';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import TrashIcon from '@lucide/svelte/icons/trash-2';

	let {
		open = $bindable(false),
		profile = null,
		existingNames = [],
		onSave
	}: {
		open: boolean;
		profile: AgentProfileConfig | null;
		existingNames: string[];
		onSave: (profile: AgentProfileInput, originalName?: string) => Promise<void>;
	} = $props();

	let name = $state('');
	let binary = $state('');
	let flags: string[] = $state([]);
	let customCommand = $state('');
	let envVars: Array<{ key: string; value: string }> = $state([]);
	let isSaving = $state(false);
	let error = $state('');
	let newFlag = $state('');
	let newEnvKey = $state('');
	let newEnvValue = $state('');

	const isEditing = $derived(profile !== null);
	const title = $derived(isEditing ? 'Edit Agent Profile' : 'Add Agent Profile');

	$effect(() => {
		if (open) {
			if (profile) {
				name = profile.name;
				binary = profile.binary;
				flags = [...profile.flags];
				customCommand = profile.custom_command ?? '';
				envVars = profile.env_vars
					? Object.entries(profile.env_vars).map(([key, value]) => ({ key, value }))
					: [];
			} else {
				name = '';
				binary = '';
				flags = [];
				customCommand = '';
				envVars = [];
			}
			newFlag = '';
			newEnvKey = '';
			newEnvValue = '';
			error = '';
		}
	});

	function validateName(): string | null {
		const trimmedName = name.trim();
		if (!trimmedName) {
			return 'Name is required';
		}
		const otherNames = isEditing ? existingNames.filter((n) => n !== profile?.name) : existingNames;
		if (otherNames.includes(trimmedName)) {
			return 'An agent with this name already exists';
		}
		return null;
	}

	function validateBinary(): string | null {
		const trimmedBinary = binary.trim();
		const trimmedCustom = customCommand.trim();
		if (!trimmedBinary && !trimmedCustom) {
			return 'Binary path or custom command is required';
		}
		return null;
	}

	function addFlag() {
		const trimmed = newFlag.trim();
		if (trimmed && !flags.includes(trimmed)) {
			flags = [...flags, trimmed];
			newFlag = '';
		}
	}

	function removeFlag(index: number) {
		flags = flags.filter((_, i) => i !== index);
	}

	function addEnvVar() {
		const trimmedKey = newEnvKey.trim();
		const trimmedValue = newEnvValue.trim();
		if (trimmedKey && !envVars.some((e) => e.key === trimmedKey)) {
			envVars = [...envVars, { key: trimmedKey, value: trimmedValue }];
			newEnvKey = '';
			newEnvValue = '';
		}
	}

	function removeEnvVar(index: number) {
		envVars = envVars.filter((_, i) => i !== index);
	}

	async function handleSave() {
		error = '';

		const nameError = validateName();
		if (nameError) {
			error = nameError;
			return;
		}

		const binaryError = validateBinary();
		if (binaryError) {
			error = binaryError;
			return;
		}

		isSaving = true;
		try {
			const envVarsObject: Record<string, string> | null =
				envVars.length > 0
					? envVars.reduce(
							(acc, { key, value }) => {
								acc[key] = value;
								return acc;
							},
							{} as Record<string, string>
						)
					: null;

			const profileInput: AgentProfileInput = {
				name: name.trim(),
				binary: binary.trim(),
				flags: flags,
				custom_command: customCommand.trim() || null,
				env_vars: envVarsObject
			};

			await onSave(profileInput, isEditing ? profile?.name : undefined);
			open = false;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			isSaving = false;
		}
	}

	function handleFlagKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			addFlag();
		}
	}

	function handleEnvKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			addEnvVar();
		}
	}
</script>

<Dialog bind:open>
	<DialogContent class="max-h-[85vh] overflow-y-auto sm:max-w-lg">
		<DialogHeader>
			<DialogTitle>{title}</DialogTitle>
			<DialogDescription>
				Configure an agent profile with binary, flags, and environment variables.
			</DialogDescription>
		</DialogHeader>
		<div class="space-y-4 py-4">
			{#if error}
				<div
					class="rounded-md border border-red-200 bg-red-50 p-3 text-sm text-red-700 dark:border-red-800 dark:bg-red-950 dark:text-red-300"
					role="alert"
				>
					{error}
				</div>
			{/if}

			<div>
				<label for="profile-name" class="mb-1.5 block text-sm font-medium">Name</label>
				<Input
					id="profile-name"
					bind:value={name}
					placeholder="e.g., claude-code"
					disabled={isSaving}
				/>
			</div>

			<div>
				<label for="profile-binary" class="mb-1.5 block text-sm font-medium">Binary path</label>
				<Input
					id="profile-binary"
					bind:value={binary}
					placeholder="e.g., claude"
					disabled={isSaving}
				/>
				<p class="mt-1 text-xs text-muted-foreground">
					Path to the agent executable, or command name if in PATH.
				</p>
			</div>

			<div>
				<label class="mb-1.5 block text-sm font-medium">Flags</label>
				<div class="flex gap-2">
					<Input
						bind:value={newFlag}
						placeholder="e.g., --dangerously-skip-permissions"
						disabled={isSaving}
						onkeydown={handleFlagKeydown}
					/>
					<Button
						type="button"
						variant="outline"
						size="sm"
						onclick={addFlag}
						disabled={isSaving || !newFlag.trim()}
					>
						<PlusIcon class="size-4" />
					</Button>
				</div>
				{#if flags.length > 0}
					<div class="mt-2 flex flex-wrap gap-1.5">
						{#each flags as flag, index (index)}
							<span
								class="inline-flex items-center gap-1 rounded-md bg-muted px-2 py-0.5 text-xs font-medium"
							>
								<code>{flag}</code>
								<button
									type="button"
									class="text-muted-foreground hover:text-foreground"
									onclick={() => removeFlag(index)}
									disabled={isSaving}
								>
									<TrashIcon class="size-3" />
								</button>
							</span>
						{/each}
					</div>
				{/if}
			</div>

			<div>
				<label for="profile-custom" class="mb-1.5 block text-sm font-medium">
					Custom command <span class="text-muted-foreground">(optional)</span>
				</label>
				<Input
					id="profile-custom"
					bind:value={customCommand}
					placeholder="e.g., docker run agent-image"
					disabled={isSaving}
				/>
				<p class="mt-1 text-xs text-muted-foreground">
					If set, overrides binary and flags entirely.
				</p>
			</div>

			<div>
				<label class="mb-1.5 block text-sm font-medium">
					Environment variables <span class="text-muted-foreground">(optional)</span>
				</label>
				<div class="flex gap-2">
					<Input
						bind:value={newEnvKey}
						placeholder="KEY"
						disabled={isSaving}
						onkeydown={handleEnvKeydown}
						class="flex-1"
					/>
					<Input
						bind:value={newEnvValue}
						placeholder="value"
						disabled={isSaving}
						onkeydown={handleEnvKeydown}
						class="flex-1"
					/>
					<Button
						type="button"
						variant="outline"
						size="sm"
						onclick={addEnvVar}
						disabled={isSaving || !newEnvKey.trim()}
					>
						<PlusIcon class="size-4" />
					</Button>
				</div>
				{#if envVars.length > 0}
					<div class="mt-2 space-y-1">
						{#each envVars as envVar, index (index)}
							<div
								class="flex items-center justify-between rounded-md bg-muted px-2 py-1 text-xs font-medium"
							>
								<code>{envVar.key}={envVar.value}</code>
								<button
									type="button"
									class="text-muted-foreground hover:text-foreground"
									onclick={() => removeEnvVar(index)}
									disabled={isSaving}
								>
									<TrashIcon class="size-3" />
								</button>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		</div>
		<DialogFooter>
			<Button variant="outline" onclick={() => (open = false)} disabled={isSaving}>Cancel</Button>
			<Button onclick={handleSave} disabled={isSaving}>
				{isSaving ? 'Saving...' : isEditing ? 'Save changes' : 'Add profile'}
			</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
