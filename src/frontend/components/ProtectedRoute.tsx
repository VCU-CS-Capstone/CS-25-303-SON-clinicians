import { useRouter } from 'expo-router';
import React, { useEffect } from 'react';
import { View, Text } from 'react-native';

import { useSession } from '~/contexts/SessionContext';

const ProtectedRoute = ({ children }: { children: React.ReactNode }) => {
  const { session } = useSession();
  const router = useRouter();

  useEffect(() => {
    if (!session) {
      router.replace('/(login)/LoginScreen');
    }
  }, [session]);

  if (!session) {
    return (
      <View className="flex-1 items-center justify-center">
        <Text>Loading...</Text>
      </View>
    );
  }

  return <>{children}</>;
};

export default ProtectedRoute;
