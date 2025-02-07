import { Ionicons, MaterialIcons } from '@expo/vector-icons';
import { Link } from 'expo-router';
import { Drawer } from 'expo-router/drawer';

const DrawerLayout = () => (
  <Drawer>
    <Drawer.Screen
      name="index"
      options={{
        headerTitle: 'Home',
        drawerLabel: 'Home',
        drawerIcon: ({ size, color }) => <Ionicons name="home-outline" size={size} color={color} />,
      }}
    />
    <Drawer.Screen
      name="search-participant"
      options={{
        headerTitle: 'Search Participant',
        drawerLabel: 'Search Participant',
        drawerIcon: ({ size, color }) => <Ionicons name="home-outline" size={size} color={color} />,
      }}
    />
    <Drawer.Screen
      name="locations"
      options={{
        headerTitle: 'Locations',
        drawerLabel: 'Locations',
        drawerIcon: ({ size, color }) => <Ionicons name="map-outline" size={size} color={color} />,
      }}
    />

    <Drawer.Screen
      name="logout"
      options={{
        headerTitle: 'Logout',
        drawerLabel: 'Logout',
        drawerIcon: ({ size, color }) => <Ionicons name="home-outline" size={size} color={color} />,
      }}
    />
  </Drawer>
);

export default DrawerLayout;
