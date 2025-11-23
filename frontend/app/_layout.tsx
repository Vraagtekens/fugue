import '@/global.css';

import { ThemeProvider } from '@react-navigation/native';
import { PortalHost } from '@rn-primitives/portal';
import { Stack, Tabs } from 'expo-router';
import { StatusBar } from 'expo-status-bar';
import { useColorScheme } from 'nativewind';
import { NAV_THEME } from '@/lib/theme';
import { SQLiteProvider } from 'expo-sqlite';

export { ErrorBoundary } from 'expo-router';

export default function RootLayout() {
  const { colorScheme } = useColorScheme();

  return (
    <SQLiteProvider databaseName="db.db" assetSource={{ assetId: require('../assets/sql/db.db') }}>
      <ThemeProvider value={NAV_THEME[colorScheme ?? 'light']}>
        <StatusBar style={colorScheme === 'dark' ? 'light' : 'dark'} />

        <Stack screenOptions={{ headerShown: false }}>
          {/* This loads the tabs layout */}
          <Stack.Screen name="(tabs)" />

          {/* Settings is separate, not in tabs */}
          <Stack.Screen
            name="settings/index"
            options={() => {
              const isDark = colorScheme === 'dark';

              console.log(isDark);

              return {
                headerShown: true,
                title: 'Settings',
                headerTransparent: true,

                // 🔥 Header color styles
                headerTitleStyle: {
                  color: isDark ? '#fff' : '#000',
                  fontWeight: '600',
                  fontSize: 18,
                },
                headerTintColor: isDark ? '#fff' : '#000', // back button + icons
              };
            }}
          />
        </Stack>

        <PortalHost />
      </ThemeProvider>
    </SQLiteProvider>
  );
}
