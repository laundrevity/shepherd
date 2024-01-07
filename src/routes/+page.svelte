<script lang="ts">
	import Canvas from '../lib/Canvas.svelte';
	import { invoke } from '@tauri-apps/api/tauri';
	import { onMount } from 'svelte';

	let width: number;
	let height: number;

	onMount(() => {
		invoke('event_loop');

		(async () => {
			const size = (await invoke('get_window_size')) as number[];
			width = size[0];
			height = size[1];
		})();

		const handleKeyDown = (event: KeyboardEvent) => {
			if (['w', 'a', 's', 'd'].includes(event.key)) {
				invoke('key_down', { key: event.key });
			}
		};

		const handleKeyUp = (event: KeyboardEvent) => {
			if (['w', 'a', 's', 'd'].includes(event.key)) {
				invoke('key_up', { key: event.key });
			}
		};

		window.addEventListener('keydown', handleKeyDown);
		window.addEventListener('keyup', handleKeyUp);

		return () => {
			window.removeEventListener('keydown', handleKeyDown);
			window.removeEventListener('keyup', handleKeyUp);
		};
	});
</script>

<main>
	<Canvas {width} {height} />
</main>
