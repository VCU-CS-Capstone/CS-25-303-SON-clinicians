import { useRouter } from 'expo-router';
import * as SecureStore from 'expo-secure-store';
import { useState } from 'react';
import { View, Text, Alert, TextInput, Button } from 'react-native';

import { useSession } from '~/contexts/SessionContext';
import api from '~/lib/api';
import { StyleSheet } from 'react-native';

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
    <View style={styles.container}>
      <Text style={styles.title}>Login</Text>
      <View style={styles.inputContainer}>
        <TextInput
          style={styles.input}
          placeholder="Username"
          value={username}
          autoCorrect={false}
          autoFocus={true}
          autoComplete="username"
          onChangeText={setUsername}
          autoCapitalize="none"
        />
        <TextInput
          style={styles.input}
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

const styles = StyleSheet.create({
  container: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#f7fafc',
    paddingHorizontal: 16,
  },
  title: {
    marginBottom: 32,
    fontSize: 24,
    fontWeight: 'bold',
    color: '#2d3748',
  },
  inputContainer: {
    width: '100%',
    maxWidth: 400,
  },
  input: {
    marginBottom: 16,
    borderRadius: 4,
    borderWidth: 1,
    padding: 14,
    paddingTop: 12,
    paddingBottom: 12,
    borderColor: '#e2e8f0',
  },
});

export default LoginScreen;
