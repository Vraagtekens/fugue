import { useEffect, useState } from "@lynx-js/react";
import { useNavigate } from "react-router";

export function Home(props: { onRender?: () => void }) {
	const nav = useNavigate();

	// 25 minutes = 1500 seconds
	const POMODORO_TIME = 0.2 * 60;

	const [seconds, setSeconds] = useState(POMODORO_TIME);
	const [running, setRunning] = useState(false);

	// Countdown logic
	useEffect(() => {
		if (!running) return;

		if (seconds === 0) {
			setRunning(false);
			return;
		}

		const id = setInterval(() => {
			setSeconds((s) => s - 1);
		}, 1000);

		return () => clearInterval(id);
	}, [running, seconds]);

	props.onRender?.();

	// Format 00:00
	const minutes = Math.floor(seconds / 60)
		.toString()
		.padStart(2, "0");
	const secs = (seconds % 60).toString().padStart(2, "0");

	return (
		<page>
			<view className="flex flex-col justify-center items-center min-h-screen gap-8">
				{/* Title */}
				<text className="text-5xl font-bold">Pomodoro Timer</text>

				{/* Timer Display */}
				<view className="bg-red-600 text-white px-12 py-8 rounded-3xl shadow-xl">
					<text className="text-7xl font-mono tracking-widest">
						{minutes}:{secs}
					</text>
				</view>

				{/* Controls */}
				<view className="flex flex-row gap-4 mt-4">
					{/* Start */}
					{!running && seconds > 0 && (
						<text
							bindtap={() => setRunning(true)}
							className="px-6 py-3 bg-green-600 text-white rounded-xl text-xl font-semibold shadow active:scale-95"
						>
							Start
						</text>
					)}

					{/* Pause */}
					{running && (
						<text
							bindtap={() => setRunning(false)}
							className="px-6 py-3 bg-yellow-500 text-white rounded-xl text-xl font-semibold shadow active:scale-95"
						>
							Pause
						</text>
					)}

					{/* Reset */}
					<text
						bindtap={() => {
							setSeconds(POMODORO_TIME);
							setRunning(false);
						}}
						className="px-6 py-3 bg-gray-800 text-white rounded-xl text-xl font-semibold shadow active:scale-95"
					>
						Reset
					</text>
				</view>

				{/* Navigation */}
				<view className="mt-10">
					<text
						bindtap={() => nav("/")}
						className="text-blue-600 underline text-lg"
					>
						Back to Index
					</text>
				</view>
			</view>
		</page>
	);
}
