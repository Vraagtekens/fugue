import { useCallback, useEffect, useState } from "@lynx-js/react";
import arrow from "./assets/arrow.png";
import lynxLogo from "./assets/lynx-logo.png";
import reactLynxLogo from "./assets/react-logo.png";
import { useNavigate } from "react-router";

export function Home(props: { onRender?: () => void }) {
	const [alterLogo, setAlterLogo] = useState(false);

	const nav = useNavigate();

	useEffect(() => {
		console.info("Hello, ReactLynx");
	}, []);
	props.onRender?.();

	const onTap = useCallback(() => {
		"background only";
		setAlterLogo((prevAlterLogo) => !prevAlterLogo);
	}, []);

	return (
		// <view>
		// 	<view className="Background" />
		// 	<view className="App">
		// 		<view className="Banner">
		// 			<view className="Logo" bindtap={onTap}>
		// 				{alterLogo ? (
		// 					<image src={reactLynxLogo} className="Logo--react" />
		// 				) : (
		// 					<image src={lynxLogo} className="Logo--lynx" />
		// 				)}
		// 			</view>
		// 			<text className="Title">React</text>
		// 			<text className="Subtitle">on Lynx</text>
		// 		</view>
		// 		<view className="Content">
		// 			<image src={arrow} className="Arrow" />
		// 			<text className="Description">Tap the logo and have fun!</text>
		// 			<text className="Hint">
		// 				Edit
		// 				<text
		// 					style={{
		// 						fontStyle: "italic",
		// 						color: "rgba(255, 255, 255, 0.85)",
		// 					}}
		// 				>
		// 					{" src/App.tsx "}
		// 				</text>
		// 				to see updates!
		// 			</text>
		// 		</view>
		// 		<view style={{ flex: 1 }} />
		// 	</view>
		// </view>
		<page>
			<view className="flex flex-col justify-center items-center min-h-screen text-center">
				<text className="text-6xl font-bold leading-normal underline">
					ReactLynx + TailwindCSS
				</text>
				<text className="text-lg font-normal text-gray-500 leading-normal">
					Start building amazing things with ReactLynx.
				</text>
				<view
					className="flex flex-row p-20 rounded-full"
					style={{
						backgroundColor: "rgba(255, 255, 255, 0.5)",
						padding: "40px",
						margin: "10px",
					}}
				>
					<image
						src="https://images.unsplash.com/photo-1554629947-334ff61d85dc?ixid=MnwxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8&ixlib=rb-1.2.1&auto=format&fit=crop&w=320&h=320&q=80"
						className="mt-8 rounded-full w-16 h-16 translate-x-6"
						style={{
							marginRight: "40px",
						}}
					/>
					{/* static should not be supported */}
					<image
						src="https://images.unsplash.com/photo-1554629947-334ff61d85dc?ixid=MnwxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8&ixlib=rb-1.2.1&auto=format&fit=crop&w=320&h=320&q=80"
						className="mt-8 rounded-full w-16 h-16 rotate-45 static"
						style={{
							marginRight: "40px",
						}}
					/>
					{/* static should be supported */}
					<image
						src="https://images.unsplash.com/photo-1554629947-334ff61d85dc?ixid=MnwxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8&ixlib=rb-1.2.1&auto=format&fit=crop&w=320&h=320&q=80"
						className="mt-8 rounded-full w-16 h-16 scale-150 sticky"
						style={{
							marginRight: "40px",
						}}
					/>
				</view>
				<view className="grid grid-cols-3 gap-4">
					<text>01</text>
					<text>02</text>
					<text>03</text>
					<text>04</text>
					<text>05</text>
					<text>06</text>
				</view>
				<view>
					<text bindtap={() => nav("/")}>Navigate to index</text>
				</view>
			</view>
		</page>
	);
}
