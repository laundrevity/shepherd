<script lang="ts">
	import { onMount } from 'svelte';
	import { listen } from '@tauri-apps/api/event';

	// Declare 'width' and 'height' as props
	export let width: number;
	export let height: number;

	type Triangle = [number, number, number]; // x, y, rotation
	type Circle = [number, number]; // x, y
	type Diamond = [number, number]; // x, y
	type Sprite = { Triangle?: Triangle; Circle?: Circle; Diamond?: Diamond };

	let canvas: HTMLCanvasElement;
	let ctx: CanvasRenderingContext2D | null;
	let sprites: Sprite[] = [];

	onMount(() => {
		ctx = canvas.getContext('2d');
		listenForSpriteUpdates();
	});

	function listenForSpriteUpdates(): void {
		listen('update_sprites', (event) => {
			sprites = event.payload as Sprite[];
			renderSprites();
		});
	}

	function renderSprites(): void {
		if (ctx !== null) {
			ctx.fillStyle = 'black';
			ctx.clearRect(0, 0, canvas.width, canvas.height);
			sprites.forEach((sprite) => {
				if (sprite.Triangle) {
					const [x, y, rotation] = sprite.Triangle;
					drawTriangle(x, y, rotation);
				}
				if (sprite.Circle) {
					const [x, y] = sprite.Circle;
					drawCircle(x, y);
				}
				if (sprite.Diamond) {
					const [x, y] = sprite.Diamond;
					drawDiamond(x, y);
				}
			});
		}
	}

	function drawTriangle(x: number, y: number, rotation: number): void {
		const rad = rotation * (Math.PI / 180);
		if (ctx !== null) {
			ctx.beginPath();
			ctx.moveTo(x + 50 * Math.cos(rad), y + 50 * Math.sin(rad));
			ctx.lineTo(
				x + 50 * Math.cos(rad + (2 * Math.PI) / 3),
				y + 50 * Math.sin(rad + (2 * Math.PI) / 3)
			);
			ctx.lineTo(
				x + 50 * Math.cos(rad + (4 * Math.PI) / 3),
				y + 50 * Math.sin(rad + (4 * Math.PI) / 3)
			);
			ctx.closePath();
			ctx.strokeStyle = 'white';
			ctx.stroke();
		}
	}

	function drawCircle(x: number, y: number): void {
		if (ctx !== null) {
			ctx.beginPath();
			ctx.arc(x, y, 20, 0, 2 * Math.PI);
			ctx.fillStyle = 'red';
			ctx.fill();
		}
	}

	function drawDiamond(x: number, y: number): void {
		if (ctx !== null) {
			ctx.beginPath();
			ctx.moveTo(x, y - 20);
			ctx.lineTo(x + 20, y);
			ctx.lineTo(x, y + 20);
			ctx.lineTo(x - 20, y);
			ctx.closePath();
			ctx.fillStyle = 'blue';
			ctx.fill();
		}
	}
</script>

<canvas bind:this={canvas} {width} {height}></canvas>

<style>
	canvas {
		display: block; /* Remove extra space below canvas */
		margin: auto; /* Center in the parent element */
		background: black; /* Optional, if you want the canvas background to be black always */
	}
</style>
