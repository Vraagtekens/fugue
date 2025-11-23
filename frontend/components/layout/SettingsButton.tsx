import { useColorScheme } from 'nativewind';
import { Button } from '../ui/button';
import { Icon } from '../ui/icon';
import { MoonStarIcon, Settings, SunIcon } from 'lucide-react-native';
import { View } from 'react-native';
import { Link } from 'expo-router';

export function SettingsButton() {
  return (
    <View>
      <Link href="/settings" asChild>
        <Button variant="ghost">
          <Icon className="h-6" as={Settings} />
        </Button>
      </Link>
    </View>
  );
}
