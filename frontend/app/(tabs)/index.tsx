import { Link, Stack, useRouter } from 'expo-router';
import { useColorScheme } from 'nativewind';
import { View, Text } from 'react-native';

export default function HomeScreen() {
  const { colorScheme } = useColorScheme();
  const backgroundColor = colorScheme === 'dark' ? '#111' : '#f8f8f8';

  return (
    <>
      <Stack.Screen />

      <View style={{ flex: 1, backgroundColor }}>
        <View className="flex flex-col items-center justify-center gap-5 p-10">
          <Text>test</Text>
        </View>
      </View>
    </>
  );
}
