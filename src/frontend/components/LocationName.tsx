import { useEffect, useState } from 'react';
import { Text } from 'react-native';
import api from '~/lib/api';
import { Location } from '~/lib/types/locations';

export interface LocationNameProps {
  locationId: number;
}
export const LocationName = ({ locationId }: LocationNameProps) => {
  const [location, setLocation] = useState<Location | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);
  const [loading, setLoading] = useState(true);
  const fetchLocation = async () => {
    try {
      const locationResponse = await api.locations.fetchById(locationId);
      setLocation(locationResponse);
      setError(undefined);
      setLoading(false);
    } catch (e: any) {
      setError(e.message as string);
      setLoading(false);
    }
  };
  useEffect(() => {
    fetchLocation();
  }, []);
  return (
    <>
      {loading ? (
        <Text>Loading...</Text>
      ) : error ? (
        <Text>Error: {error}</Text>
      ) : location ? (
        <Text>{location.name}</Text>
      ) : null}
    </>
  );
};
