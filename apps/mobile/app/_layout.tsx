import { Stack } from 'expo-router';
import { StatusBar } from 'expo-status-bar';
import { SafeAreaProvider } from 'react-native-safe-area-context';

export default function RootLayout() {
  return (
    <SafeAreaProvider>
      <Stack>
        <Stack.Screen name="index" options={{ title: 'GroupWeave' }} />
        <Stack.Screen name="voting" options={{ title: 'Voting' }} />
        <Stack.Screen name="staking" options={{ title: 'Staking' }} />
        <Stack.Screen name="profile" options={{ title: 'Profile' }} />
      </Stack>
      <StatusBar style="auto" />
    </SafeAreaProvider>
  );
}