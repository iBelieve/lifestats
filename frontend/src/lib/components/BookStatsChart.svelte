<script lang="ts">
	import { Bar } from 'svelte5-chartjs';
	import {
		Chart as ChartJS,
		Title,
		Tooltip,
		Legend,
		BarElement,
		CategoryScale,
		LinearScale
	} from 'chart.js';
	import type { BibleStats, BookStats } from '$lib/api/client';
	import chartColors from '$lib/theme/chartColors';
	import SegmentedControl from './SegmentedControl.svelte';

	type ViewMode = 'verses' | 'passages';
	let viewMode: ViewMode = $state('verses');

	// Custom plugin to draw a vertical line between OT and NT
	const testamentDividerPlugin = {
		id: 'testamentDivider',
		afterDatasetsDraw: (chart: ChartJS, _args: unknown, options: { ntStartIndex?: number }) => {
			const ntIdx = options.ntStartIndex;
			if (ntIdx === undefined || ntIdx === -1 || ntIdx === 0) return;

			const { ctx, chartArea, scales } = chart;
			const xScale = scales.x;

			// Calculate position between last OT book and first NT book
			const lastOTPosition = xScale.getPixelForValue(ntIdx - 1);
			const firstNTPosition = xScale.getPixelForValue(ntIdx);
			const dividerX = (lastOTPosition + firstNTPosition) / 2;

			ctx.save();
			ctx.strokeStyle = chartColors.label.gray;
			ctx.lineWidth = 2;
			ctx.setLineDash([5, 5]);
			ctx.beginPath();
			ctx.moveTo(dividerX, chartArea.top);
			ctx.lineTo(dividerX, chartArea.bottom);
			ctx.stroke();
			ctx.restore();
		}
	};

	// Register Chart.js components
	ChartJS.register(
		Title,
		Tooltip,
		Legend,
		BarElement,
		CategoryScale,
		LinearScale,
		testamentDividerPlugin
	);

	interface Props {
		data: BibleStats;
	}

	const { data }: Props = $props();

	// Combine OT and NT books, marking each with its testament
	interface BookWithTestament extends BookStats {
		testament: 'OT' | 'NT';
	}

	const combinedBooks: BookWithTestament[] = [
		...data.old_testament.book_stats.map((book) => ({ ...book, testament: 'OT' as const })),
		...data.new_testament.book_stats.map((book) => ({ ...book, testament: 'NT' as const }))
	];

	// Filter out books with no data for the current view mode
	const hasData = (book: BookStats, mode: ViewMode) => {
		if (mode === 'verses') {
			return book.mature_verses > 0 || book.young_verses > 0 || book.learning_verses > 0;
		}
		return book.mature_passages > 0 || book.young_passages > 0 || book.learning_passages > 0;
	};

	const combinedBooksFiltered = $derived(combinedBooks.filter((book) => hasData(book, viewMode)));

	// Separate OT and NT books for mobile charts
	const otBooksFiltered = $derived(
		data.old_testament.book_stats.filter((book) => hasData(book, viewMode))
	);
	const ntBooksFiltered = $derived(
		data.new_testament.book_stats.filter((book) => hasData(book, viewMode))
	);

	// Find the index where NT books start (for the combined chart divider)
	const ntStartIndex = $derived(combinedBooksFiltered.findIndex((book) => book.testament === 'NT'));

	// Helper to get the appropriate value based on view mode
	const getValue = (book: BookStats, field: 'mature' | 'young' | 'learning') => {
		if (viewMode === 'verses') {
			return book[`${field}_verses`];
		}
		return book[`${field}_passages`];
	};

	// Label for the current view mode
	const unitLabel = $derived(viewMode === 'verses' ? 'verses' : 'passages');

	// Transform data for Chart.js format (stacked bar chart) - Combined chart for desktop
	const combinedChartData = $derived({
		labels: combinedBooksFiltered.map((book) => book.book),
		datasets: [
			// Bottom stack: Mature (darkest)
			{
				label: 'Mature',
				data: combinedBooksFiltered.map((book) => getValue(book, 'mature')),
				backgroundColor: combinedBooksFiltered.map((book) =>
					book.testament === 'OT'
						? chartColors.book.oldTestament.mature
						: chartColors.book.newTestament.mature
				),
				borderColor: combinedBooksFiltered.map((book) =>
					book.testament === 'OT'
						? chartColors.book.oldTestament.border
						: chartColors.book.newTestament.border
				),
				borderWidth: 1,
				borderRadius: 4,
				hoverBackgroundColor: combinedBooksFiltered.map((book) =>
					book.testament === 'OT'
						? chartColors.book.oldTestament.hover
						: chartColors.book.newTestament.hover
				),
				stack: 'stack0'
			},
			// Middle stack: Young (medium shade)
			{
				label: 'Young',
				data: combinedBooksFiltered.map((book) => getValue(book, 'young')),
				backgroundColor: combinedBooksFiltered.map((book) =>
					book.testament === 'OT'
						? chartColors.book.oldTestament.young
						: chartColors.book.newTestament.young
				),
				borderColor: combinedBooksFiltered.map((book) =>
					book.testament === 'OT'
						? chartColors.book.oldTestament.border
						: chartColors.book.newTestament.border
				),
				borderWidth: 1,
				borderRadius: 4,
				hoverBackgroundColor: combinedBooksFiltered.map((book) =>
					book.testament === 'OT'
						? chartColors.book.oldTestament.hover
						: chartColors.book.newTestament.hover
				),
				stack: 'stack0'
			},
			// Top stack: Learning (lightest shade)
			{
				label: 'Learning',
				data: combinedBooksFiltered.map((book) => getValue(book, 'learning')),
				backgroundColor: combinedBooksFiltered.map((book) =>
					book.testament === 'OT'
						? chartColors.book.oldTestament.learning
						: chartColors.book.newTestament.learning
				),
				borderColor: combinedBooksFiltered.map((book) =>
					book.testament === 'OT'
						? chartColors.book.oldTestament.border
						: chartColors.book.newTestament.border
				),
				borderWidth: 1,
				borderRadius: 4,
				hoverBackgroundColor: combinedBooksFiltered.map((book) =>
					book.testament === 'OT'
						? chartColors.book.oldTestament.hover
						: chartColors.book.newTestament.hover
				),
				stack: 'stack0'
			}
		]
	});

	const combinedOptions = $derived({
		responsive: true,
		maintainAspectRatio: false,
		plugins: {
			testamentDivider: {
				ntStartIndex
			},
			legend: {
				display: true,
				position: 'top' as const,
				labels: {
					generateLabels: () => [
						{
							text: 'Old Testament',
							fillStyle: chartColors.book.oldTestament.mature,
							strokeStyle: chartColors.book.oldTestament.border,
							lineWidth: 1
						},
						{
							text: 'New Testament',
							fillStyle: chartColors.book.newTestament.mature,
							strokeStyle: chartColors.book.newTestament.border,
							lineWidth: 1
						}
					]
				}
			},
			tooltip: {
				callbacks: {
					label: (context: { dataset: { label?: string }; parsed: { y: number | null } }) => {
						const value = context.parsed.y ?? 0;
						const label = context.dataset.label || '';
						return `${label}: ${value} ${unitLabel}`;
					},
					footer: (tooltipItems: Array<{ parsed: { y: number | null } }>) => {
						const total = tooltipItems.reduce((sum, item) => sum + (item.parsed.y ?? 0), 0);
						return `Total: ${total} ${unitLabel}`;
					}
				}
			}
		},
		scales: {
			x: {
				stacked: true,
				grid: {
					display: false
				},
				ticks: {
					maxRotation: 90,
					minRotation: 45,
					font: {
						size: 10
					}
				}
			},
			y: {
				stacked: true,
				beginAtZero: true,
				grid: {
					color: chartColors.grid.gray
				},
				title: {
					display: true,
					text: viewMode === 'verses' ? 'Verses' : 'Passages'
				},
				ticks: {
					precision: 0
				}
			}
		}
	});

	// Helper function to create chart data for a single testament
	const createTestamentChartData = (
		books: BookStats[],
		colors: { mature: string; young: string; learning: string; border: string; hover: string },
		mode: ViewMode
	) => ({
		labels: books.map((book) => book.book),
		datasets: [
			{
				label: 'Mature',
				data: books.map((book) => (mode === 'verses' ? book.mature_verses : book.mature_passages)),
				backgroundColor: colors.mature,
				borderColor: colors.border,
				borderWidth: 1,
				borderRadius: 4,
				hoverBackgroundColor: colors.hover,
				stack: 'stack0'
			},
			{
				label: 'Young',
				data: books.map((book) => (mode === 'verses' ? book.young_verses : book.young_passages)),
				backgroundColor: colors.young,
				borderColor: colors.border,
				borderWidth: 1,
				borderRadius: 4,
				hoverBackgroundColor: colors.hover,
				stack: 'stack0'
			},
			{
				label: 'Learning',
				data: books.map((book) =>
					mode === 'verses' ? book.learning_verses : book.learning_passages
				),
				backgroundColor: colors.learning,
				borderColor: colors.border,
				borderWidth: 1,
				borderRadius: 4,
				hoverBackgroundColor: colors.hover,
				stack: 'stack0'
			}
		]
	});

	// Chart data for mobile split charts
	const otChartData = $derived(
		createTestamentChartData(otBooksFiltered, chartColors.book.oldTestament, viewMode)
	);
	const ntChartData = $derived(
		createTestamentChartData(ntBooksFiltered, chartColors.book.newTestament, viewMode)
	);

	// Shared options for mobile split charts (no testament divider needed)
	const mobileOptions = $derived({
		responsive: true,
		maintainAspectRatio: false,
		plugins: {
			testamentDivider: false,
			legend: {
				display: true,
				position: 'top' as const
			},
			tooltip: {
				callbacks: {
					label: (context: { dataset: { label?: string }; parsed: { y: number | null } }) => {
						const value = context.parsed.y ?? 0;
						const label = context.dataset.label || '';
						return `${label}: ${value} ${unitLabel}`;
					},
					footer: (tooltipItems: Array<{ parsed: { y: number | null } }>) => {
						const total = tooltipItems.reduce((sum, item) => sum + (item.parsed.y ?? 0), 0);
						return `Total: ${total} ${unitLabel}`;
					}
				}
			}
		},
		scales: {
			x: {
				stacked: true,
				grid: {
					display: false
				},
				ticks: {
					maxRotation: 90,
					minRotation: 45,
					font: {
						size: 10
					}
				}
			},
			y: {
				stacked: true,
				beginAtZero: true,
				grid: {
					color: chartColors.grid.gray
				},
				title: {
					display: true,
					text: viewMode === 'verses' ? 'Verses' : 'Passages'
				},
				ticks: {
					precision: 0
				}
			}
		}
	});

	// Calculate statistics based on view mode
	const otLearning = $derived(
		viewMode === 'verses'
			? data.old_testament.learning_verses
			: data.old_testament.learning_passages
	);
	const otYoung = $derived(
		viewMode === 'verses' ? data.old_testament.young_verses : data.old_testament.young_passages
	);
	const otMature = $derived(
		viewMode === 'verses' ? data.old_testament.mature_verses : data.old_testament.mature_passages
	);
	const otTotal = $derived(otLearning + otYoung + otMature);

	const ntLearning = $derived(
		viewMode === 'verses'
			? data.new_testament.learning_verses
			: data.new_testament.learning_passages
	);
	const ntYoung = $derived(
		viewMode === 'verses' ? data.new_testament.young_verses : data.new_testament.young_passages
	);
	const ntMature = $derived(
		viewMode === 'verses' ? data.new_testament.mature_verses : data.new_testament.mature_passages
	);
	const ntTotal = $derived(ntLearning + ntYoung + ntMature);

	const totalLearning = $derived(otLearning + ntLearning);
	const totalYoung = $derived(otYoung + ntYoung);
	const totalMature = $derived(otMature + ntMature);
	const grandTotal = $derived(totalLearning + totalYoung + totalMature);

	// Options for the segmented control
	const viewModeOptions: Array<{ value: ViewMode; label: string }> = [
		{ value: 'verses', label: 'Verses' },
		{ value: 'passages', label: 'Passages' }
	];
</script>

<!-- Toggle between verses and passages -->
<div class="mb-4 flex justify-center">
	<SegmentedControl options={viewModeOptions} bind:value={viewMode} />
</div>

<!-- Mobile: Split charts for OT and NT -->
<div class="md:hidden">
	{#if otBooksFiltered.length > 0}
		<div class="mb-6">
			<h3 class="mb-2 text-center text-sm font-medium">Old Testament</h3>
			<div class="h-48 w-full">
				<Bar data={otChartData} options={mobileOptions} />
			</div>
		</div>
	{/if}
	{#if ntBooksFiltered.length > 0}
		<div>
			<h3 class="mb-2 text-center text-sm font-medium">New Testament</h3>
			<div class="h-48 w-full">
				<Bar data={ntChartData} options={mobileOptions} />
			</div>
		</div>
	{/if}
</div>

<!-- Desktop: Combined chart with testament divider -->
<div class="hidden h-96 w-full md:block">
	<Bar data={combinedChartData} options={combinedOptions} />
</div>

<!-- Mobile table: simplified with abbreviated labels -->
<div class="mt-6 flex justify-center md:hidden">
	<table class="w-full">
		<thead>
			<tr>
				<th class="px-2 text-left"></th>
				<th class="px-2 text-right">OT</th>
				<th class="px-2 text-right">NT</th>
				<th class="px-2 text-right">Total</th>
			</tr>
		</thead>
		<tbody>
			<tr>
				<td class="px-2 py-1">Learning</td>
				<td class="px-2 py-1 text-right">{otLearning}</td>
				<td class="px-2 py-1 text-right">{ntLearning}</td>
				<td class="px-2 py-1 text-right">{totalLearning}</td>
			</tr>
			<tr>
				<td class="px-2 py-1">Young</td>
				<td class="px-2 py-1 text-right">{otYoung}</td>
				<td class="px-2 py-1 text-right">{ntYoung}</td>
				<td class="px-2 py-1 text-right">{totalYoung}</td>
			</tr>
			<tr>
				<td class="px-2 py-1">Mature</td>
				<td class="px-2 py-1 text-right">{otMature}</td>
				<td class="px-2 py-1 text-right">{ntMature}</td>
				<td class="px-2 py-1 text-right">{totalMature}</td>
			</tr>
			<tr class="border-t border-gray-300">
				<td class="px-2 py-1">Total</td>
				<td class="px-2 py-1 text-right">{otTotal}</td>
				<td class="px-2 py-1 text-right">{ntTotal}</td>
				<td class="px-2 py-1 text-right">{grandTotal}</td>
			</tr>
		</tbody>
	</table>
</div>

<!-- Desktop table: full labels -->
<div class="mt-6 hidden justify-center md:flex">
	<table>
		<thead>
			<tr>
				<th class="px-4 text-left"></th>
				<th class="px-4 text-right">Learning</th>
				<th class="px-4 text-right">Young</th>
				<th class="px-4 text-right">Mature</th>
				<th class="px-4 text-right">Total</th>
			</tr>
		</thead>
		<tbody>
			<tr>
				<td class="px-4 py-1">Old Testament</td>
				<td class="px-4 py-1 text-right">{otLearning}</td>
				<td class="px-4 py-1 text-right">{otYoung}</td>
				<td class="px-4 py-1 text-right">{otMature}</td>
				<td class="px-4 py-1 text-right">{otTotal}</td>
			</tr>
			<tr>
				<td class="px-4 py-1">New Testament</td>
				<td class="px-4 py-1 text-right">{ntLearning}</td>
				<td class="px-4 py-1 text-right">{ntYoung}</td>
				<td class="px-4 py-1 text-right">{ntMature}</td>
				<td class="px-4 py-1 text-right">{ntTotal}</td>
			</tr>
			<tr class="border-t border-gray-300">
				<td class="px-4 py-1">Total</td>
				<td class="px-4 py-1 text-right">{totalLearning}</td>
				<td class="px-4 py-1 text-right">{totalYoung}</td>
				<td class="px-4 py-1 text-right">{totalMature}</td>
				<td class="px-4 py-1 text-right">{grandTotal}</td>
			</tr>
		</tbody>
	</table>
</div>
