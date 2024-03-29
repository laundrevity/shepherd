<script lang="ts">
	import { onMount } from 'svelte';
	import { listen } from '@tauri-apps/api/event';

	export let window_width: number;
	export let window_height: number;
	export let circle_radius: number;
	export let diamond_radius: number;
	export let triangle_radius: number;
	export let square_radius: number;

	export let score: number;
	export let multiplier: number;

	export let onCanvasMounted: Function;

	type Triangle = [number, number, number]; // x, y, rotation
	type Circle = [number, number]; // x, y
	type Diamond = [number, number]; // x, y
	type Square = [number, number]; // x, y
	type Sprite = { Triangle?: Triangle; Circle?: Circle; Diamond?: Diamond; Square?: Square };

	let canvas: HTMLCanvasElement;
	let ctx: CanvasRenderingContext2D | null;
	let sprites: Sprite[] = [];

	onMount(() => {
		ctx = canvas.getContext('2d');
		listenForSpriteUpdates();

		onCanvasMounted(canvas);
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

			// Sort sprites so that triangles are drawn first
			const sortedSprites = [...sprites].sort((a, b) => {
				if ('Triangle' in a && !('Triangle' in b)) return -1;
				if ('Triangle' in b && !('Triangle' in a)) return 1;
				return 0;
			});

			sortedSprites.forEach((sprite) => {
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
				if (sprite.Square) {
					const [x, y] = sprite.Square;
					drawSquare(x, y);
				}
			});

			// Draw HUD
			drawHUD();
		}
	}

	function drawHUD(): void {
		if (ctx !== null) {
			ctx.font = '16px Arial';
			ctx.textAlign = 'center'; // Align text to center
			ctx.fillStyle = 'rgba(255, 255, 255, 0.4)'; // Semi-translucent white

			// Calculate the center x position
			const centerX = window_width / 2;

			// Draw the score and multiplier
			ctx.fillText(`${score}`, centerX, 30); // Draw score at top-center
			ctx.fillText(`x${multiplier}`, centerX, 60); // Draw multiplier below score
		}
	}

	function drawTriangle(x: number, y: number, rotation: number): void {
		const rad = rotation * (Math.PI / 180);
		if (ctx !== null) {
			ctx.beginPath();

			ctx.shadowBlur = 30;
			ctx.shadowColor = 'white';

			ctx.moveTo(x + triangle_radius * Math.cos(rad), y + triangle_radius * Math.sin(rad));
			ctx.lineTo(
				x + triangle_radius * Math.cos(rad + (2 * Math.PI) / 3),
				y + triangle_radius * Math.sin(rad + (2 * Math.PI) / 3)
			);
			ctx.lineTo(
				x + triangle_radius * Math.cos(rad + (4 * Math.PI) / 3),
				y + triangle_radius * Math.sin(rad + (4 * Math.PI) / 3)
			);
			ctx.closePath();
			ctx.strokeStyle = 'white';
			ctx.stroke();

			ctx.fillStyle = 'white';
			ctx.fill();

			// Reset shadow settings before stroke
			ctx.shadowColor = 'transparent';
			ctx.shadowBlur = 0;
		}
	}

	function drawCircle(x: number, y: number): void {
		if (ctx !== null) {
			ctx.beginPath();

			ctx.shadowBlur = 30;
			ctx.shadowColor = 'rgba(125, 125, 255, 1.0)';

			ctx.arc(x, y, circle_radius, 0, 2 * Math.PI);
			ctx.closePath();

			ctx.strokeStyle = 'rgba(125, 125, 255, 1.0)';
			ctx.stroke();

			ctx.fillStyle = 'black';
			ctx.fill();

			// Reset shadow settings before stroke
			ctx.shadowColor = 'transparent';
			ctx.shadowBlur = 0;
		}
	}

	function drawDiamond(x: number, y: number): void {
		if (ctx !== null) {
			ctx.beginPath();
			ctx.moveTo(x, y - diamond_radius);
			ctx.lineTo(x + diamond_radius, y);
			ctx.lineTo(x, y + diamond_radius);
			ctx.lineTo(x - diamond_radius, y);
			ctx.closePath();

			ctx.strokeStyle = 'black';
			ctx.stroke();

			ctx.fillStyle = 'rgba(255, 0, 0, 0.9)';
			ctx.fill();
		}
	}

	function drawSquare(x: number, y: number): void {
		// Multiplier had radius 5 in backend so R^2 + R^2 = S^2 => S/2 = sqrt(2) * R / 2  = 7.07/2 = 3.53
		if (ctx !== null) {
			const s = square_radius / Math.sqrt(2);
			ctx.beginPath();
			ctx.moveTo(x - s, y - s);
			ctx.lineTo(x - s, y + s);
			ctx.lineTo(x + s, y + s);
			ctx.lineTo(x + s, y - s);
			ctx.closePath();

			ctx.fillStyle = 'green';
			ctx.fill();
		}
	}
</script>

<canvas bind:this={canvas} width={window_width} height={window_height}></canvas>

<style>
	canvas {
		display: block; /* Remove extra space below canvas */
		margin: auto; /* Center in the parent element */
		background: black; /* Optional, if you want the canvas background to be black always */
	}
</style>
