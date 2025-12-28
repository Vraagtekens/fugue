// (tabs)/3d/index.tsx
import { View } from 'react-native';
import { Stack } from 'expo-router';
import { useColorScheme } from 'nativewind';
import { MetaballCanvas } from './MetaballCanvas';

const SCREEN_OPTIONS = {
  title: '3D Model',
  headerTransparent: true,
  headerBackButtonDisplayMode: 'minimal',
};

export default function Screen() {
  const { colorScheme } = useColorScheme();
  const backgroundColor = colorScheme === 'dark' ? '#111' : '#f8f8f8';

  return (
    <>
      <Stack.Screen options={SCREEN_OPTIONS} />

      <View style={{ flex: 1, backgroundColor }}>
        <View className="flex h-[500px] flex-col items-center justify-center gap-5 p-10">
          <MetaballCanvas />
          {/* <div className="bg-gradient-purple h-[250px] w-full rounded-xl"></div> */}
          {/* <div className="bg-gradient-green-blue h-[250px] w-full rounded-xl"></div> */}
        </View>

        <View className="h-[100vh]"></View>
      </View>
    </>
  );
}
