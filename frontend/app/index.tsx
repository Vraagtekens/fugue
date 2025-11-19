import { Link, useRouter } from 'expo-router';
import { Settings } from 'lucide-react-native';
import { useEffect, useState } from 'react';
import { Text, TouchableOpacity, View } from 'react-native';
import { Button } from '@/components/ui/button';
import { Icon } from '@/components/ui/icon';
import { cn } from '@/lib/utils'; // reusables utility

export default function PomodoroScreen() {
  const router = useRouter();

  const POMODORO_TIME = 0.2 * 60;

  const [seconds, setSeconds] = useState(POMODORO_TIME);
  const [running, setRunning] = useState(false);

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

  const minutes = Math.floor(seconds / 60)
    .toString()
    .padStart(2, '0');

  const secs = (seconds % 60).toString().padStart(2, '0');

  return (
    <View className="flex-1 items-center justify-center gap-8 bg-background p-6">
      <Link href="/settings" asChild>
        <Button variant="ghost">
          <Icon className="h-6" as={Settings} />
        </Button>
      </Link>

      <Text className="text-4xl font-bold">Pomodoro Timer</Text>

      <View className="rounded-3xl bg-red-600 px-14 py-8">
        <Text className="font-mono text-7xl text-white">
          {minutes}:{secs}
        </Text>
      </View>

      <View className="mt-4 flex-row gap-3">
        {!running && seconds > 0 && (
          <TouchableOpacity
            onPress={() => setRunning(true)}
            className="rounded-xl bg-green-600 px-6 py-3">
            <Text className="text-xl font-semibold text-white">Start</Text>
          </TouchableOpacity>
        )}

        {running && (
          <TouchableOpacity
            onPress={() => setRunning(false)}
            className="rounded-xl bg-yellow-500 px-6 py-3">
            <Text className="text-xl font-semibold text-white">Pause</Text>
          </TouchableOpacity>
        )}

        <TouchableOpacity
          onPress={() => {
            setSeconds(POMODORO_TIME);
            setRunning(false);
          }}
          className="rounded-xl bg-gray-800 px-6 py-3">
          <Text className="text-xl font-semibold text-white">Reset</Text>
        </TouchableOpacity>
      </View>
    </View>
  );
}
