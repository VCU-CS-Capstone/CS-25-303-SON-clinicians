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
    <View className={styles.container}>
      <Text className={styles.title}>Login</Text>
      <View className={styles.inputContainer}>
        <TextInput
          className={styles.input}
          placeholder="Username"
          value={username}
          autoCorrect={false}
          autoFocus={true}
          autoComplete="username"
          onChangeText={setUsername}
          autoCapitalize="none"
        />
        <TextInput
          className={styles.input}
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

const styles = {
  container: 'flex-1 justify-center items-center bg-gray-100 px-4',
  title: 'mb-8 text-2xl font-bold text-gray-800',
  inputContainer: 'w-full max-w-md',
  input: 'mb-4 rounded border p-2',
};

export default LoginScreen;
