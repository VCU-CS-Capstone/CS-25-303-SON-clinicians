import { useRouter } from 'expo-router';
import * as SecureStore from 'expo-secure-store';
import { useState } from 'react';
import { View, Text, Alert, TextInput, Button } from 'react-native';

import { useSession } from '~/contexts/SessionContext';
import api from '~/lib/api';

const LoginScreen = () => {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const { setSession } = useSession();
  const router = useRouter();

  const handleLogin = async () => {
    try {
      const response = await api.login(username, password);
      const sessionId = response.session.session_key;
      if (!sessionId) throw new Error('Invalid login response');

      setSession(sessionId);
      await SecureStore.setItemAsync('session', sessionId);
      router.replace('/');
    } catch (error) {
      Alert.alert(`Failed to login ${error}`);
    }
  };

  return (
    <View className="justify-cetner flex-1 items-center bg-gray-100 px-4">
      <Text className="mb-8 text-2xl font-bold text-gray-800">Login</Text>
      <View className="w-full max-w-md">
        <TextInput
          className="mb-4 rounded border p-2"
          placeholder="Username"
          value={username}
          autoCorrect={false}
          autoFocus={true}
          autoComplete="username"
          onChangeText={setUsername}
          autoCapitalize="none"
        />
        <TextInput
          className="mb-4 rounded border p-2"
          placeholder="Password"
          value={password}
          autoCorrect={false}
          autoComplete="password"
          onChangeText={setPassword}
          secureTextEntry
          autoCapitalize="none"
        />
        <Button title="Login" onPress={handleLogin} />
      </View>
    </View>
  );
};

export default LoginScreen;
