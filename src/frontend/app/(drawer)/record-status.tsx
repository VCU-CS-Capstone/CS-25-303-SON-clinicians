import { Stack } from 'expo-router';

import { Container } from '~/components/Container';
import { ScreenContent } from '~/components/ScreenContent';

export default function Records() {
  return (
    <>
      <Stack.Screen options={{ title: 'Record Status' }} />
      <Container>
        <ScreenContent path="app/(drawer)/record-status.tsx" title="Tab One" />
      </Container>
    </>
  );
}
