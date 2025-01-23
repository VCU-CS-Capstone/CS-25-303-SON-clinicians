import { Stack } from 'expo-router';

import { Container } from '~/components/Container';
import { ScreenContent } from '~/components/ScreenContent';

export default function Calendar() {
  return (
    <>
      <Stack.Screen options={{ title: 'Calendar' }} />
      <Container>
        <ScreenContent path="app/(drawer)/calendar.tsx" title="Home" />
      </Container>
    </>
  );
}
