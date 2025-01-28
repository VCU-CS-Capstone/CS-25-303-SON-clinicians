import { Ionicons, MaterialIcons } from '@expo/vector-icons';
import { Link, useLocalSearchParams } from 'expo-router';
import { Drawer } from 'expo-router/drawer';

import { Button } from 'react-native';
import { useNavigation } from '@react-navigation/native';

const DrawerLayout = () => {
  const navigation = useNavigation();
  // TODO: Add a back button to go up the stack by one
  const { participant_id } = useLocalSearchParams<{ participant_id: string }>();

  return (
    <>
      <Drawer>
        <Drawer.Screen
          name="overview"
          initialParams={{ participant_id: participant_id }}
          options={{
            headerTitle: 'Overview',
            drawerLabel: 'Overview',

            drawerIcon: ({ size, color }) => (
              <Ionicons name="home-outline" size={size} color={color} />
            ),
          }}
        />
        <Drawer.Screen
          name="visits"
          initialParams={{ participant_id: participant_id }}
          options={{
            headerTitle: 'Visits',
            drawerLabel: 'Visits',
            drawerIcon: ({ size, color }) => (
              <Ionicons name="home-outline" size={size} color={color} />
            ),
          }}
        />
      </Drawer>
    </>
  );
};

export default DrawerLayout;
