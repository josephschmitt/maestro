<script lang="ts">
	import StatusManager from '$lib/components/status-manager.svelte';
	import LinkedDirectories from '$lib/components/settings/linked-directories.svelte';
	import AuthToken from '$lib/components/settings/auth-token.svelte';
	import HttpServerSettings from '$lib/components/settings/http-server-settings.svelte';
	import ProjectOverrides from '$lib/components/settings/project-overrides.svelte';
	import { Separator } from '$lib/components/ui/separator/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { currentProject } from '$lib/stores/project.js';
	import { getGlobalConfig, getStatusGroupDefaults } from '$lib/services/config.js';
	import type { AgentProfileConfig, StatusGroupConfigInput } from '$lib/types/index.js';
	import { resolve } from '$app/paths';
	import { onMount } from 'svelte';
	import SettingsIcon from '@lucide/svelte/icons/settings';

	let agents: AgentProfileConfig[] = $state([]);
	let globalDefaults: Record<string, StatusGroupConfigInput> = $state({});
	let globalDefaultAgent = $state('claude-code');
	let isLoadingConfig = $state(true);

	onMount(async () => {
		try {
			const [config, defaults] = await Promise.all([getGlobalConfig(), getStatusGroupDefaults()]);
			agents = config.agents;
			globalDefaultAgent = config.default_agent;
			globalDefaults = defaults.status;
		} catch {
			// Ignore errors, will show empty state
		} finally {
			isLoadingConfig = false;
		}
	});
</script>

<div class="flex h-full flex-col overflow-hidden">
	<div class="flex h-14 items-center border-b border-border px-6">
		<h1 class="text-lg font-semibold">
			{#if $currentProject}
				{$currentProject.name} — Settings
			{:else}
				Settings
			{/if}
		</h1>
	</div>

	<div class="flex-1 overflow-y-auto p-6">
		<div class="mx-auto max-w-2xl">
			<section>
				<div class="mb-6 rounded-md border border-border bg-muted/30 p-4">
					<div class="flex items-center justify-between">
						<div>
							<h3 class="text-sm font-semibold">Global Settings</h3>
							<p class="text-xs text-muted-foreground">
								Configure agents, models, and defaults that apply to all projects.
							</p>
						</div>
						<a href={resolve('/settings/global')}>
							<Button variant="outline" size="sm">
								<SettingsIcon class="size-3.5" />
								Open Global Settings
							</Button>
						</a>
					</div>
				</div>
			</section>

			<section>
				<h2 class="mb-4 text-base font-semibold">Status Management</h2>
				<p class="mb-4 text-sm text-muted-foreground">
					Manage the statuses available in each group. Cards move through these statuses on the
					board.
				</p>
				<StatusManager />
			</section>

			<Separator class="my-8" />

			<section>
				<h2 class="mb-4 text-base font-semibold">Agent Configuration</h2>
				<p class="mb-4 text-sm text-muted-foreground">
					Override global agent settings for this project. Use the toggle to switch between using
					global defaults and project-specific overrides.
				</p>
				{#if isLoadingConfig}
					<p class="text-sm text-muted-foreground">Loading configuration...</p>
				{:else}
					<ProjectOverrides {agents} {globalDefaults} {globalDefaultAgent} />
				{/if}
			</section>

			<Separator class="my-8" />

			<section>
				<h2 class="mb-4 text-base font-semibold">Linked Directories</h2>
				<p class="mb-4 text-sm text-muted-foreground">
					Link external directories (git repos, doc folders) to this project. These are used
					as working directories for agents.
				</p>
				<LinkedDirectories />
			</section>

			<Separator class="my-8" />

			<section>
				<h2 class="mb-4 text-base font-semibold">Authentication</h2>
				<p class="mb-4 text-sm text-muted-foreground">
					Manage authentication for HTTP server connections.
				</p>
				<AuthToken />
			</section>

			<Separator class="my-8" />

			<section>
				<h2 class="mb-4 text-base font-semibold">HTTP Server</h2>
				<p class="mb-4 text-sm text-muted-foreground">
					Configure the HTTP server for browser-based access. This allows you to use Maestro from
					a web browser, either on this machine or from other devices on your network.
				</p>
				<HttpServerSettings />
			</section>
		</div>
	</div>
</div>
