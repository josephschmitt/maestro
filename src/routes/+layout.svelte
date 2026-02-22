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
	import AuthTokenDialog from '$lib/components/dialogs/auth-token-dialog.svelte';
	import ConnectionStatus from '$lib/components/connection-status.svelte';
	import ToastContainer from '$lib/components/ui/toast-container.svelte';
	import ErrorBoundary from '$lib/components/ui/error-boundary.svelte';
	import {
		repoSelectorState,
		branchNameState,
		resolveRepoSelection,
		resolveBranchName
	} from '$lib/stores/worktree-flow.js';
	import { initializeProject, hasProject, loadProjects } from '$lib/stores/project.js';
	import { loadStatuses } from '$lib/stores/statuses.js';
	import { loadCards } from '$lib/stores/cards.js';
	import { loadLinkedDirectories } from '$lib/stores/directories.js';
	import { listenEvent } from '$lib/services/events.js';
	import { listRunningWorkspaces, stopAllAgents } from '$lib/services/agent.js';
	import { resumeAgent } from '$lib/stores/agent.js';
	import { authStore, authPromptOpen } from '$lib/stores/auth.js';
	import { onMount, onDestroy } from 'svelte';
	import SettingsIcon from '@lucide/svelte/icons/settings';
	import LayoutDashboardIcon from '@lucide/svelte/icons/layout-dashboard';
	import { Button } from '$lib/components/ui/button/index.js';
	import { resolve } from '$app/paths';
	import FocusRegion from '$lib/focus/region.svelte';
	import { sidebarWidth } from '$lib/stores/sidebar.js';

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
	let authDialogOpen = $state(false);
	let cleanupFns: (() => void)[] = [];

	$effect(() => {
		authDialogOpen = $authPromptOpen;
	});

	let isResizing = $state(false);
	let startX = 0;
	let startWidth = 0;

	function handleResizeStart(e: MouseEvent) {
		e.preventDefault();
		isResizing = true;
		startX = e.clientX;
		startWidth = $sidebarWidth;
		document.body.style.cursor = 'col-resize';
		document.body.style.userSelect = 'none';
	}

	function handleResizeMove(e: MouseEvent) {
		if (!isResizing) return;
		const delta = e.clientX - startX;
		sidebarWidth.setWidth(startWidth + delta);
	}

	function handleResizeEnd() {
		if (!isResizing) return;
		isResizing = false;
		document.body.style.cursor = '';
		document.body.style.userSelect = '';
	}

	onMount(() => {
		authStore.initialize();
		initializeProject();
		setupTauriListeners();
		setupReconnectHandler();

		document.addEventListener('mousemove', handleResizeMove);
		document.addEventListener('mouseup', handleResizeEnd);

		return () => {
			document.removeEventListener('mousemove', handleResizeMove);
			document.removeEventListener('mouseup', handleResizeEnd);
		};
	});

	async function setupReconnectHandler() {
		const unlisten = await listenEvent('__ws_reconnected__', () => {
			loadProjects();
			loadStatuses();
			loadCards();
			loadLinkedDirectories();
		});
		cleanupFns.push(unlisten);
	}

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

			const { getCurrentWindow } = await import('@tauri-apps/api/window');
			const unlistenClose = await getCurrentWindow().onCloseRequested(async (event) => {
				const running = await listRunningWorkspaces();
				if (running.length > 0) {
					event.preventDefault();
					quitRunningCount = running.length;
					quitDialogOpen = true;
				}
			});

			cleanupFns.push(unlistenCrashed, unlistenStartupCrash, unlistenClose);
		} catch {
			// Not in Tauri environment
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

	function handleResumeCrashed(workspaceId: string, projectId: string) {
		const agent = crashedAgents.find(
			(a) => a.workspace_id === workspaceId && a.project_id === projectId
		);
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

	function handleAuthToken(token: string) {
		authStore.saveToken(token);
	}
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<ErrorBoundary>
<div class="flex h-screen w-screen overflow-hidden">
	<FocusRegion region="sidebar">
		<aside
			class="relative flex shrink-0 flex-col border-r border-border bg-transparent text-sidebar-foreground"
			style="width: {$sidebarWidth}px"
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
			<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
			<div
				role="separator"
				aria-label="Resize sidebar"
				aria-orientation="vertical"
				tabindex="0"
				class="group absolute -right-1 top-0 h-full w-3 cursor-col-resize"
				onmousedown={handleResizeStart}
				onkeydown={(e) => {
					if (e.key === 'ArrowLeft') sidebarWidth.setWidth($sidebarWidth - 10);
					if (e.key === 'ArrowRight') sidebarWidth.setWidth($sidebarWidth + 10);
				}}
			>
				<div
					class="absolute left-1 top-0 h-full w-1 group-hover:bg-primary/20 group-active:bg-primary/30 {isResizing ? 'bg-primary/30' : ''}"
				></div>
			</div>
		</aside>
	</FocusRegion>

	<main class="flex flex-1 flex-col overflow-hidden bg-background">
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
<AuthTokenDialog bind:open={authDialogOpen} onconfirm={handleAuthToken} />
<ConnectionStatus />
</ErrorBoundary>
<ToastContainer />
