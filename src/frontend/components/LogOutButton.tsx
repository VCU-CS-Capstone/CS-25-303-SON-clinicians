import React from 'react';

import { Button } from '~/components/Button';
import { useSession } from '~/contexts/SessionContext';

const LogOutButton = () => {
  const { logout } = useSession();

  return <Button title="Log Out" onPress={logout} />;
};

export default LogOutButton;
