import { useSession } from '~/contexts/SessionContext';
import LoginScreen from './LoginScreen';
import { Redirect } from 'expo-router';

const DrawerLayout = () => {
  const context = useSession();
  if (!context) {
    return null;
  }
  if (!context.isValid()) {
    return <LoginScreen />;
  } else {
    return <Redirect href={'/(drawer)'} />;
  }
};
export default DrawerLayout;
