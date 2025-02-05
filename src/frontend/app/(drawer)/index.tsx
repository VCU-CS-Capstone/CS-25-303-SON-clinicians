import { Stack } from 'expo-router';
import { Text, View } from 'react-native';

import { Container } from '~/components/Container';
import { ScreenContent } from '~/components/ScreenContent';
import { HamburgerMenu } from '~/components/menus/hamburger';

export default function Home() {
  return (
    <>
      <Stack.Screen options={{ title: 'Home' }} />
      <Container>
        <View className="mt-40 flex flex-row gap-24">
          <Text>Hello World</Text>
          <View className="border">
            <HamburgerMenu iconHeight={24} iconWidth={24}>
              <Text>Menu</Text>
            </HamburgerMenu>
          </View>
        </View>
        <ScreenContent path="app/(drawer)/index.tsx" title="Home" />
      </Container>
    </>
  );
}
