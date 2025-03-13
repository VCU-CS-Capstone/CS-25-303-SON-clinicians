import '~/global.css';

import { Stack } from 'expo-router';
import { GestureHandlerRootView } from 'react-native-gesture-handler';

import { SessionProvider, useSession } from '../contexts/SessionContext';

export const unstable_settings = {
  // Ensure that reloading on `/modal` keeps a back button present.
  initialRouteName: '(drawer)',
};

export default function RootLayout() {
  //To understand this.
  // Inside the Stack we have (drawer) and [participant_id] as the screen names.
  // Drawer is the default. But once you go to search-participants and select a participant,
  // the screen name changes to [participant_id] and it will be stacked on top of the drawer.
  // This is how the navigation works in the app.
  return (
    <SessionProvider>
      <GestureHandlerRootView style={{ flex: 1 }}>
        <Stack>
          <Stack.Screen name="(login)" options={{ headerShown: false }} />
          <Stack.Screen name="(drawer)" options={{ headerShown: false }} />
          <Stack.Screen name="[participant_id]" options={{ headerShown: false }} />
        </Stack>
      </GestureHandlerRootView>
    </SessionProvider>
  );
}

export function PrimaryView() {
  return (
    <GestureHandlerRootView style={{ flex: 1 }}>
      <Stack>
        <Stack.Screen name="(login)" options={{ headerShown: false }} />
        <Stack.Screen name="(drawer)" options={{ headerShown: false }} />
        <Stack.Screen name="[participant_id]" options={{ headerShown: false }} />
      </Stack>
    </GestureHandlerRootView>
  );
}
