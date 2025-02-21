import { useEffect, useState } from 'react';
import { FlatList, Text, View } from 'react-native';
import ProtectedRoute from '~/components/ProtectedRoute';
import api from '~/lib/api';
import {
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
      <View style={styles.parent}>
        <View style={styles.header}>
          <Text
            style={{
              fontWeight: 'bold',
              fontSize: 18,
            }}
          >
            Lists all the locations in the system.
          </Text>
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
    <View style={styles.SummaryBox}>
      <Text style={{ fontWeight: 'bold' }}>
        {location.name} - ID# {location.id}
      </Text>
      <Text>Program: {Program.fullName(location.program)}</Text>
      {location.parent_location ? (
        <Text>Parent Location: {location.parent_location.name}</Text>
      ) : null}
    </View>
  );
}

const styles = {
  parent: {
    paddingLeft: 0.5,
    paddingRight: 0.5,
  },
  header: {
    marginBottom: 4,
    borderBottomWidth: 1,
    padding: 2,
  },

  SummaryBox: {
    padding: 4,
    borderWidth: 1,
    marginBottom: 4,
  },
};
