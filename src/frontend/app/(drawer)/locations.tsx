import { useEffect, useState } from 'react';
import { FlatList, Text, View } from 'react-native';
import ProtectedRoute from '~/components/ProtectedRoute';
import api from '~/lib/api';
import { Location, Program } from '~/lib/types/locations';

export default function ShowLocations() {
  const [locations, setLocations] = useState<Map<Program, Location[]>>(new Map());
  const [error, setError] = useState<string | undefined>(undefined);
  const [loading, setLoading] = useState(true);
  const fetchLocations = async () => {
    try {
      const locationsResponse = await api.locations.fetchAll();

      for (const location of locationsResponse) {
        console.log(location);

        if (!locations.has(location.program)) {
          locations.set(location.program, []);
        }
        locations.get(location.program)?.push(location);
      }
      setError(undefined);
      setLoading(false);
    } catch (e: any) {
      setError(e.message as string);
      setLoading(false);
    }
  };
  useEffect(() => {
    fetchLocations();
  }, []);

  return (
    <ProtectedRoute>
      <View>
        <Text>Mobile Health and Wellness Location </Text>
        <FlatList
          data={locations.get(Program.MHWP) || []}
          renderItem={({ item }) => <LocationSummary location={item} />}
          keyExtractor={(item) => item.id.toString()}
        />
      </View>
      <View>
        <Text>Mobile Health and Wellness Location </Text>

        <FlatList
          data={locations.get(Program.RHWP) || []}
          renderItem={({ item }) => <LocationSummary location={item} />}
          keyExtractor={(item) => item.id.toString()}
        />
      </View>
    </ProtectedRoute>
  );
}

function LocationSummary({ location }: { location: Location }) {
  return (
    <View className="mb-4 border-2 border-solid border-red-100">
      <Text>{location.id}</Text>
      <Text>{location.name}</Text>
      <Text>{location.parent_location}</Text>
    </View>
  );
}
