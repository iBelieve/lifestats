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
	import { Temporal } from '@js-temporal/polyfill';
	import type { FaithWeeklyStats } from '$lib/api/client';
	import chartColors from '$lib/theme/chartColors';
	import { formatMinutesToHoursMinutes } from '$lib/utils/timeFormat';
	import colors from 'tailwindcss/colors';

	// Register Chart.js components
	ChartJS.register(Title, Tooltip, Legend, BarElement, CategoryScale, LinearScale);

	interface Props {
		data: FaithWeeklyStats;
	}

	const { data }: Props = $props();

	// Day names for labels
	const dayNames = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];

	// Color scheme: darker at bottom (Sunday) to lighter at top (Saturday)
	const dayColors = [
		{ bg: colors.orange[800], border: colors.orange[900], hover: colors.orange[900] }, // Sunday
		{ bg: colors.orange[700], border: colors.orange[800], hover: colors.orange[800] }, // Monday
		{ bg: colors.orange[600], border: colors.orange[700], hover: colors.orange[700] }, // Tuesday
		{ bg: colors.orange[500], border: colors.orange[600], hover: colors.orange[600] }, // Wednesday
		{ bg: colors.orange[400], border: colors.orange[500], hover: colors.orange[500] }, // Thursday
		{ bg: colors.orange[300], border: colors.orange[400], hover: colors.orange[400] }, // Friday
		{ bg: colors.orange[200], border: colors.orange[300], hover: colors.orange[300] } // Saturday
	];

	// Format date to show month/day for week start
	const formatDate = (dateStr: string) => {
		const date = Temporal.PlainDate.from(dateStr);
		return `${date.month}/${date.day}`;
	};

	// Transform data for Chart.js stacked bar format
	// Only include days that have non-zero values across all weeks
	const chartData = $derived({
		labels: data.weeks.map((week) => formatDate(week.week_start)),
		datasets: dayNames
			.map((dayName, dayIndex) => ({
				label: dayName,
				data: data.weeks.map((week) => week.at_church_daily_minutes[dayIndex] || 0),
				backgroundColor: dayColors[dayIndex].bg,
				borderColor: dayColors[dayIndex].border,
				borderWidth: 1,
				borderRadius: 4,
				hoverBackgroundColor: dayColors[dayIndex].hover,
				stack: 'stack0'
			}))
			.filter((dataset) => dataset.data.some((value) => value > 0))
	});

	const options = {
		responsive: true,
		maintainAspectRatio: false,
		plugins: {
			legend: {
				display: false
			},
			tooltip: {
				callbacks: {
					label: (context: { dataset: { label?: string }; parsed: { y: number | null } }) => {
						const minutes = context.parsed.y ?? 0;
						const day = context.dataset.label || '';
						return `${day}: ${formatMinutesToHoursMinutes(minutes)}`;
					},
					footer: (tooltipItems: Array<{ parsed: { y: number | null } }>) => {
						// Show weekly total
						const total = tooltipItems.reduce((sum, item) => sum + (item.parsed.y ?? 0), 0);
						return `Total: ${formatMinutesToHoursMinutes(total)}`;
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
					maxRotation: 45,
					minRotation: 45,
					font: {
						size: 11
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
					text: 'Hours'
				},
				ticks: {
					stepSize: 60, // Force ticks at exact hour intervals
					callback: function (value: string | number) {
						const minutes = typeof value === 'number' ? value : parseFloat(value);
						return formatMinutesToHoursMinutes(minutes);
					}
				}
			}
		}
	};
</script>

<div class="h-64 w-full md:h-80">
	<Bar data={chartData} {options} />
</div>
