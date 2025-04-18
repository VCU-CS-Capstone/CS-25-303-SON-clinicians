import { Ionicons, MaterialIcons } from '@expo/vector-icons';
import { Link, useLocalSearchParams, useRouter } from 'expo-router';
import { Drawer } from 'expo-router/drawer';

import { Button } from 'react-native';
import { useNavigation } from '@react-navigation/native';
import ProtectedRoute from '~/components/ProtectedRoute';

const DrawerLayout = () => {
  // TODO: Add a back button to go up the stack by one
  const { participant_id } = useLocalSearchParams<{ participant_id: string }>();

  return (
    <ProtectedRoute>
      <Drawer>
        <Drawer.Screen
          name="go-back"
          options={{
            headerTitle: 'Go Back',
            drawerLabel: 'Back',
            drawerIcon: ({ size, color }) => (
              <Ionicons name="return-up-back-outline" size={size} color={color} />
            ),
          }}
        />
        <Drawer.Screen
          name="overview"
          //This is required to make sure when the user navigates to the overview page, the participant_id is passed as a parameter
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
              <Ionicons name="medkit-outline" size={size} color={color} />
            ),
          }}
        />
        <Drawer.Screen
          name="(trends)"
          initialParams={{ participant_id: participant_id }}
          options={{
            headerTitle: 'Health Trends',
            drawerLabel: 'Health Trends',
            drawerIcon: ({ size, color }) => (
              <Ionicons name="analytics-outline" size={size} color={color} />
            ),
          }}
        />
        <Drawer.Screen
          name="medications"
          initialParams={{ participant_id: participant_id }}
          options={{
            headerTitle: 'Medications',
            drawerLabel: 'Medications',
            drawerIcon: ({ size, color }) => (
              <Ionicons name="medical-outline" size={size} color={color} />
            ),
          }}
        />
        <Drawer.Screen
          name="goals"
          initialParams={{ participant_id: participant_id }}
          options={{
            headerTitle: 'Goals',
            drawerLabel: 'Goals',
            drawerIcon: ({ size, color }) => (
              <Ionicons name="flag-outline" size={size} color={color} />
            ),
          }}
        />
      </Drawer>
    </ProtectedRoute>
  );
};

export default DrawerLayout;
