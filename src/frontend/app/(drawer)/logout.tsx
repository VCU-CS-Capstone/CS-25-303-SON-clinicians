import { Redirect, useRouter } from 'expo-router';
import { View, Text } from 'react-native';
import { useSession } from '~/contexts/SessionContext';

export default function Logout() {
  const session = useSession();
  session.logout();
  return <Redirect href={'/'} />;
}
