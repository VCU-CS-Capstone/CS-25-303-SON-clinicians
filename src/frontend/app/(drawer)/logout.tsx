import { Redirect } from 'expo-router';
import { useSession } from '~/contexts/SessionContext';

export default function Logout() {
  const { logout } = useSession();
  logout();
  return (
    <>
      <Redirect href={'/login'} />;
    </>
  );
}
