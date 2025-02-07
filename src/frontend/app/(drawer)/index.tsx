import { Stack } from 'expo-router';
import { Text, View } from 'react-native';

import { HamburgerMenu } from '~/components/menus/hamburger';
import { SafeAreaView } from 'react-native';

export default function Home() {
  return (
    <>
      <Stack.Screen options={{ title: 'Home' }} />
      <SafeAreaView className={styles.container}>
        <View className="mt-40 flex flex-row gap-24">
          <Text>Hello World</Text>
          <View className="border">
            <HamburgerMenu iconHeight={24} iconWidth={24}>
              <Text>Menu</Text>
            </HamburgerMenu>
          </View>
        </View>
      </SafeAreaView>
    </>
  );
}

const styles = {
  container: 'flex flex-1 m-6',
};
