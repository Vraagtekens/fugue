import { SettingsButton } from '@/components/layout/SettingsButton';
import { ThemeToggle } from '@/components/layout/ThemeToggle';
import { Tabs } from 'expo-router';
import { useColorScheme } from 'nativewind';

const SCREEN_OPTIONS = {
  headerTransparent: true,
  headerRight: () => <SettingsButton />,
};

export default function TabsLayout() {
  const { colorScheme } = useColorScheme();
  const isDark = colorScheme === 'dark';

  return (
    <Tabs
      screenOptions={{
        headerTransparent: true,
        headerRight: () => <SettingsButton />,
        // headerShown: false,

        // 🔥 Tab colors
        tabBarActiveTintColor: isDark ? '#fff' : '#000',
        tabBarInactiveTintColor: isDark ? '#888' : '#777',
        tabBarStyle: {
          backgroundColor: isDark ? '#111' : '#f8f8f8',
          borderTopColor: isDark ? '#222' : '#ddd',
          paddingBottom: 6,
          paddingTop: 4,
          height: 60,
        },
      }}>
      <Tabs.Screen
        name="index" // Pomodoro
        options={{ title: 'Timer', headerTitle: '' }}
      />
      <Tabs.Screen
        name="sessions/index" // Sessions tab
        options={{ title: 'Sessions', headerTitle: '' }}
      />
      <Tabs.Screen
        name="3d/index" // Sessions tab
        options={{ title: '3D', headerTitle: '' }}
      />
    </Tabs>
  );
}

// (tabs)/_layout.tsx (or wherever you define native tabs)
// import { NativeTabs, Icon, Label } from 'expo-router/unstable-native-tabs';
// import { useColorScheme } from 'nativewind';
// import { SettingsButton } from '@/components/layout/SettingsButton'; // optional

// export default function TabLayout() {
//   const { colorScheme } = useColorScheme();
//   const isDark = colorScheme === 'dark';

//   return (
//     <NativeTabs
//       // you can set bar background color
//       backgroundColor={isDark ? '#111' : '#f8f8f8'}
//       // optionally control pop-to-root behavior
//       backBehavior="initialRoute">
//       <NativeTabs.Trigger name="index">
//         <Label>Timer</Label>
//         <Icon
//           sf={isDark ? 'house.fill' : 'house'}
//           // you could use a drawable for Android if needed
//         />
//       </NativeTabs.Trigger>

//       <NativeTabs.Trigger name="sessions/index">
//         <Label>Sessions</Label>
//         <Icon sf={isDark ? 'clock.fill' : 'clock'} />
//       </NativeTabs.Trigger>
//     </NativeTabs>
//   );
// }
