import { Link, Stack, useRouter } from 'expo-router';
import { View } from 'react-native';
import { MetaballCanvas } from '@/components/MetaballCanvas';
import { useColorScheme } from 'nativewind';

export default function MetaballScreen() {
  const { colorScheme } = useColorScheme();
  const backgroundColor = colorScheme === 'dark' ? '#111' : '#f8f8f8';

  return (
    <>
      <Stack.Screen />

      <MetaballCanvas />

      {/* <View style={{ flex: 1, backgroundColor }}>
        <View className="flex h-[500px] flex-col items-center justify-center gap-5 p-10">
   
        </View>

        <View className="h-[100vh]"></View>
      </View> */}
    </>
  );
}
