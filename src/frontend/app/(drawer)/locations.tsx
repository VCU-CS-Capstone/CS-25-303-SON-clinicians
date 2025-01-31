import { useEffect, useState } from 'react';
import { FlatList, Text, View } from 'react-native';
import ProtectedRoute from '~/components/ProtectedRoute';
import api from '~/lib/api';
import {
  Location,
  LocationWithParentItem,
  organizeLocationsToWithParents,
  Program,
} from '~/lib/types/locations';

export default function ShowLocations() {
  const [locations, setLocations] = useState<LocationWithParentItem[]>([]);
  const [error, setError] = useState<string | undefined>(undefined);
  const [loading, setLoading] = useState(true);
  const fetchLocations = async () => {
    try {
      const locationsResponse = await api.locations.fetchAll();

      setLocations(organizeLocationsToWithParents(locationsResponse));
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
      <View className="px-2">
        <View className="mb-4 border-b px-2">
          <Text className="text-lg">Lists all the locations in the system. </Text>
        </View>
        <FlatList
          data={locations || []}
          renderItem={({ item }) => <LocationSummary location={item} />}
          keyExtractor={(item) => item.id.toString()}
        />
      </View>
    </ProtectedRoute>
  );
}

function LocationSummary({ location }: { location: LocationWithParentItem }) {
  return (
    <View className="mb-4 border-2 border-solid border-red-100">
      <Text className="text-xl">
        {location.name} - ID# {location.id}
      </Text>
      <Text>Program: {Program.fullName(location.program)}</Text>
      {location.parent_location ? (
        <Text className="text-lg">Parent Location: {location.parent_location.name}</Text>
      ) : null}
    </View>
  );
}
