import { Tabs, useLocalSearchParams } from 'expo-router';

import { TabBarIcon } from '~/components/TabBarIcon';

export default function TabLayout() {
  const { participant_id } = useLocalSearchParams<{ participant_id: string }>();

  return (
    <Tabs
      screenOptions={{
        headerShown: false,
        tabBarActiveTintColor: 'black',
      }}
    >
      <Tabs.Screen
        initialParams={{ participant_id: participant_id }}
        name="blood-pressure-trends"
        options={{
          title: 'Blood Pressure',
          tabBarIcon: ({ color }) => <TabBarIcon name="code" color={color} />,
        }}
      />
      <Tabs.Screen
        initialParams={{ participant_id: participant_id }}
        name="glucose-trends"
        options={{
          title: 'Glucose',
          tabBarIcon: ({ color }) => <TabBarIcon name="code" color={color} />,
        }}
      />
      <Tabs.Screen
        initialParams={{ participant_id: participant_id }}
        name="weight-trends"
        options={{
          title: 'Weight',
          tabBarIcon: ({ color }) => <TabBarIcon name="code" color={color} />,
        }}
      />
    </Tabs>
  );
}
