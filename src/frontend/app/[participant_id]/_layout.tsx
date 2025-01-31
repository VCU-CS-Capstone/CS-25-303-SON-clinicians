import { Ionicons, MaterialIcons } from '@expo/vector-icons';
import { Link, useLocalSearchParams, useRouter } from 'expo-router';
import { Drawer } from 'expo-router/drawer';

import { Button } from 'react-native';
import { useNavigation } from '@react-navigation/native';

const DrawerLayout = () => {
  // TODO: Add a back button to go up the stack by one
  const { participant_id } = useLocalSearchParams<{ participant_id: string }>();

  return (
    <>
      <Drawer>
        <Drawer.Screen
          name="go-back"
          options={{
            headerTitle: 'Go Back',
            drawerLabel: 'Back',
            drawerIcon: ({ size, color }) => (
              <Ionicons name="home-outline" size={size} color={color} />
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
              <Ionicons name="home-outline" size={size} color={color} />
            ),
          }}
        />
        <Drawer.Screen
          name="blood-pressure-trends"
          initialParams={{ participant_id: participant_id }}
          options={{
            headerTitle: 'Blood Pressure Trends',
            drawerLabel: 'Blood Pressure Trends',
            drawerIcon: ({ size, color }) => (
              <Ionicons name="home-outline" size={size} color={color} />
            ),
          }}
        />
        <Drawer.Screen
          name="weight-trends"
          initialParams={{ participant_id: participant_id }}
          options={{
            headerTitle: 'Weight Trends',
            drawerLabel: 'Weight Trends',
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
