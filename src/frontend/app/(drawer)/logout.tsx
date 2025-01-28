import { Stack } from 'expo-router';

import { Container } from '~/components/Container';
import LogOutButton from '~/components/LogOutButton';
import { ScreenContent } from '~/components/ScreenContent';

export default function Logout() {
  return (
    <>
      <LogOutButton />
    </>
  );
}
