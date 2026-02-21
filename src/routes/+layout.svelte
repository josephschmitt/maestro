<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import ProjectSwitcher from '$lib/components/project-switcher.svelte';
	import CreateProjectDialog from '$lib/components/dialogs/create-project-dialog.svelte';
	import LinkDirectoryPrompt from '$lib/components/dialogs/link-directory-prompt.svelte';
	import RepoSelectorDialog from '$lib/components/dialogs/repo-selector-dialog.svelte';
	import BranchNameDialog from '$lib/components/dialogs/branch-name-dialog.svelte';
	import QuitDialog from '$lib/components/dialogs/quit-dialog.svelte';
	import AgentCrashedDialog from '$lib/components/dialogs/agent-crashed-dialog.svelte';
	import {
		repoSelectorState,
		branchNameState,
		resolveRepoSelection,
		resolveBranchName
	} from '$lib/stores/worktree-flow.js';
	import { initializeProject, hasProject } from '$lib/stores/project.js';
	import { listRunningWorkspaces, stopAllAgents } from '$lib/services/agent.js';
	import { resumeAgent } from '$lib/stores/agent.js';
	import { onMount, onDestroy } from 'svelte';
	import SettingsIcon from '@lucide/svelte/icons/settings';
	import LayoutDashboardIcon from '@lucide/svelte/icons/layout-dashboard';
	import { Button } from '$lib/components/ui/button/index.js';
	import { resolve } from '$app/paths';
	import FocusRegion from '$lib/focus/region.svelte';

	interface CrashedAgent {
		workspace_id: string;
		project_id: string;
		card_id: string;
		session_id: string | null;
	}

	let { children } = $props();
	let createDialogOpen = $state(false);
	let quitDialogOpen = $state(false);
	let quitRunningCount = $state(0);
	let crashDialogOpen = $state(false);
	let crashedAgents = $state<CrashedAgent[]>([]);
	let cleanupFns: (() => void)[] = [];

	onMount(() => {
		initializeProject();
		setupTauriListeners();
	});

	onDestroy(() => {
		for (const fn of cleanupFns) fn();
	});

	async function setupTauriListeners() {
		try {
			const { listen } = await import('@tauri-apps/api/event');

			const unlistenCrashed = await listen<{ workspace_id: string; project_id: string }>(
				'agent-crashed',
				(event) => {
					crashedAgents = [
						...crashedAgents,
						{
							workspace_id: event.payload.workspace_id,
							project_id: event.payload.project_id,
							card_id: '',
							session_id: null
						}
					];
					crashDialogOpen = true;
				}
			);

			const unlistenStartupCrash = await listen<CrashedAgent[]>(
				'agents-crashed-on-startup',
				(event) => {
					crashedAgents = event.payload;
					crashDialogOpen = true;
				}
			);

			cleanupFns.push(unlistenCrashed, unlistenStartupCrash);
		} catch {
			// Not in Tauri environment
		}
	}

	async function handleQuitCheck() {
		try {
			const running = await listRunningWorkspaces();
			if (running.length > 0) {
				quitRunningCount = running.length;
				quitDialogOpen = true;
			}
		} catch {
			// ignore
		}
	}

	async function handleStopAll() {
		await stopAllAgents();
		quitDialogOpen = false;
	}

	function handleKeepRunning() {
		quitDialogOpen = false;
	}

	function handleCancelQuit() {
		quitDialogOpen = false;
	}

	function handleResumeCrashed(workspaceId: string, _projectId: string) {
		const agent = crashedAgents.find((a) => a.workspace_id === workspaceId);
		if (agent?.card_id) {
			resumeAgent(workspaceId, agent.card_id);
		}
		crashedAgents = crashedAgents.filter((a) => a.workspace_id !== workspaceId);
		if (crashedAgents.length === 0) crashDialogOpen = false;
	}

	function handleDismissCrashed() {
		crashedAgents = [];
		crashDialogOpen = false;
	}
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<div class="flex h-screen w-screen overflow-hidden">
	<FocusRegion region="sidebar">
		<aside
			class="flex w-56 shrink-0 flex-col border-r border-border bg-sidebar text-sidebar-foreground"
		>
			<div
				data-tauri-drag-region
				class="flex h-14 items-center gap-2 border-b border-sidebar-border px-4 pt-5"
			>
				<span class="text-lg font-semibold">Maestro</span>
			</div>
			<nav class="flex-1 p-2">
				<ProjectSwitcher onCreateClick={() => (createDialogOpen = true)} />
			</nav>
			{#if $hasProject}
				<div class="space-y-0.5 border-t border-sidebar-border p-2">
					<a href={resolve('/board')}>
						<Button variant="ghost" size="sm" class="w-full justify-start gap-2">
							<LayoutDashboardIcon class="size-3.5" />
							Board
						</Button>
					</a>
					<a href={resolve('/settings')}>
						<Button variant="ghost" size="sm" class="w-full justify-start gap-2">
							<SettingsIcon class="size-3.5" />
							Settings
						</Button>
					</a>
				</div>
			{/if}
		</aside>
	</FocusRegion>

	<main class="flex flex-1 flex-col overflow-hidden">
		<div data-tauri-drag-region class="h-8 w-full shrink-0"></div>
		{@render children()}
	</main>
</div>

<CreateProjectDialog bind:open={createDialogOpen} />
<LinkDirectoryPrompt />
<RepoSelectorDialog
	bind:open={$repoSelectorState.open}
	repos={$repoSelectorState.repos}
	onselect={(repo) => resolveRepoSelection(repo)}
	onskip={() => resolveRepoSelection(null)}
/>
<BranchNameDialog
	bind:open={$branchNameState.open}
	defaultBranchName={$branchNameState.defaultBranchName}
	onconfirm={(name) => resolveBranchName(name)}
	oncancel={() => resolveBranchName(null)}
/>
<QuitDialog
	bind:open={quitDialogOpen}
	runningCount={quitRunningCount}
	onstopall={handleStopAll}
	onkeeprunning={handleKeepRunning}
	oncancel={handleCancelQuit}
/>
<AgentCrashedDialog
	bind:open={crashDialogOpen}
	{crashedAgents}
	onresume={handleResumeCrashed}
	ondismiss={handleDismissCrashed}
/>
